// Anna v0.12.3 - Advisor CLI Command

use anyhow::{Context, Result};
use anna_common::{header, section, status, Level, TermCaps};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

const SOCKET_PATH: &str = "/run/anna/annad.sock";

/// Advice from daemon (matches advisor_v13::Advice)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Advice {
    pub id: String,
    pub level: String, // "info", "warn", "error"
    pub category: String,
    pub title: String,
    pub reason: String,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_cmd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_risk: Option<String>,
    pub refs: Vec<String>,
}

/// Run Arch advisor and show results
pub async fn run_advisor(json: bool, explain_id: Option<String>) -> Result<()> {
    let advice_list = fetch_advisor_results().await?;

    if json {
        // JSON output
        let json_str = serde_json::to_string_pretty(&advice_list)?;
        println!("{}", json_str);
        return Ok(());
    }

    // Explain specific advice
    if let Some(id) = explain_id {
        return explain_advice(&advice_list, &id);
    }

    // Pretty TUI output
    print_advisor_tui(&advice_list);

    Ok(())
}

/// Fetch advisor results from daemon
async fn fetch_advisor_results() -> Result<Vec<Advice>> {
    let mut stream = UnixStream::connect(SOCKET_PATH)
        .await
        .context("Failed to connect to annad - is it running?")?;

    // Send RPC request
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "advisor_run",
        "params": {},
        "id": 1
    });

    let request_str = serde_json::to_string(&request)?;
    stream.write_all(request_str.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;

    // Read response
    let (reader, _writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    let response_line = lines
        .next_line()
        .await?
        .context("No response from daemon")?;

    // Parse JSON-RPC response
    let response: JsonValue = serde_json::from_str(&response_line)?;

    if let Some(error) = response.get("error") {
        anyhow::bail!("RPC error: {}", error);
    }

    let result = response
        .get("result")
        .context("No result in RPC response")?;

    let advice_list: Vec<Advice> = serde_json::from_value(result.clone())?;

    Ok(advice_list)
}

/// Print advisor results with TUI
fn print_advisor_tui(advice_list: &[Advice]) {
    let caps = TermCaps::detect();

    println!("{}", header(&caps, "Arch Linux Advisor"));
    println!();

    if advice_list.is_empty() {
        println!("{}", status(&caps, Level::Ok, "No issues found - system looks good!"));
        return;
    }

    // Group by category
    let mut categories: std::collections::HashMap<String, Vec<&Advice>> =
        std::collections::HashMap::new();

    for advice in advice_list {
        categories
            .entry(advice.category.clone())
            .or_insert_with(Vec::new)
            .push(advice);
    }

    // Print by category
    for (category, items) in categories.iter() {
        println!("{}", section(&caps, &format!("{} Issues", capitalize(category))));
        println!();

        for advice in items {
            let level = match advice.level.as_str() {
                "info" => Level::Info,
                "warn" => Level::Warn,
                "error" => Level::Err,
                _ => Level::Info,
            };

            println!("{}", status(&caps, level, &advice.title));
            println!("  Reason: {}", advice.reason);
            println!("  Action: {}", advice.action);
            println!("  ID: {}", advice.id);
            println!();
        }
    }

    // Summary
    println!("{}", section(&caps, ""));
    println!("{} issues found", advice_list.len());
    println!();
    println!("For details: annactl advisor arch --explain <id>");
}

/// Explain specific advice
fn explain_advice(advice_list: &[Advice], id: &str) -> Result<()> {
    let caps = TermCaps::detect();

    let advice = advice_list
        .iter()
        .find(|a| a.id == id)
        .context(format!("Advice ID '{}' not found", id))?;

    println!("{}", header(&caps, &format!("Advice: {}", id)));
    println!();
    println!("Title:    {}", advice.title);
    println!("Level:    {}", advice.level);
    println!("Category: {}", advice.category);
    println!();
    println!("Reason:");
    println!("  {}", advice.reason);
    println!();

    if let Some(explain) = &advice.explain {
        println!("Explanation:");
        println!("  {}", explain);
        println!();
    }

    println!("Action:");
    println!("  {}", advice.action);
    println!();

    if let Some(fix_cmd) = &advice.fix_cmd {
        println!("Fix Command:");
        for line in fix_cmd.lines() {
            println!("  {}", line);
        }
        println!();
    }

    if let Some(fix_risk) = &advice.fix_risk {
        println!("Risk Assessment:");
        println!("  {}", fix_risk);
        println!();
    }

    if !advice.refs.is_empty() {
        println!("References:");
        for r in &advice.refs {
            println!("  - {}", r);
        }
    }

    Ok(())
}

/// Capitalize first letter
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
