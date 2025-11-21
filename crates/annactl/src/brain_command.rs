//! Brain Command - Full sysadmin brain diagnostic analysis
//!
//! Beta.217c: Standalone command for comprehensive system diagnostics
//!
//! Purpose: Run complete brain analysis and display all insights
//! Usage: annactl brain [--json] [--verbose]
//! Output: Formatted diagnostic report with all insights

use anna_common::ipc::{BrainAnalysisData, DiagnosticInsightData};
use anna_common::terminal_format as fmt;
use anyhow::Result;
use std::time::Instant;

use crate::logging::{ErrorDetails, LogEntry};
use crate::rpc_client::RpcClient;

/// Execute 'annactl brain' command - full diagnostic analysis
pub async fn execute_brain_command(
    json: bool,
    verbose: bool,
    req_id: &str,
    start_time: Instant,
) -> Result<()> {
    // Fetch brain analysis via RPC
    let analysis = match fetch_brain_analysis().await {
        Ok(data) => data,
        Err(e) => {
            eprintln!("{}", fmt::error(&format!("Failed to fetch brain analysis: {}", e)));

            let duration_ms = start_time.elapsed().as_millis() as u64;
            let log_entry = LogEntry {
                ts: LogEntry::now(),
                req_id: req_id.to_string(),
                state: "error".to_string(),
                command: "brain".to_string(),
                allowed: Some(true),
                args: vec![],
                exit_code: 1,
                citation: "[archwiki:System_maintenance]".to_string(),
                duration_ms,
                ok: false,
                error: Some(ErrorDetails {
                    code: "RPC_ERROR".to_string(),
                    message: e.to_string(),
                }),
            };
            let _ = log_entry.write();

            std::process::exit(1);
        }
    };

    // JSON output mode
    if json {
        let json_str = serde_json::to_string_pretty(&analysis)?;
        println!("{}", json_str);

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let log_entry = LogEntry {
            ts: LogEntry::now(),
            req_id: req_id.to_string(),
            state: "healthy".to_string(),
            command: "brain".to_string(),
            allowed: Some(true),
            args: vec!["--json".to_string()],
            exit_code: 0,
            citation: "[archwiki:System_maintenance]".to_string(),
            duration_ms,
            ok: true,
            error: None,
        };
        let _ = log_entry.write();

        return Ok(());
    }

    // Formatted output
    display_brain_analysis(&analysis, verbose);

    // Log command
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let exit_code = if analysis.critical_count > 0 { 1 } else { 0 };

    let log_entry = LogEntry {
        ts: LogEntry::now(),
        req_id: req_id.to_string(),
        state: if analysis.critical_count > 0 {
            "critical"
        } else if analysis.warning_count > 0 {
            "warning"
        } else {
            "healthy"
        }
        .to_string(),
        command: "brain".to_string(),
        allowed: Some(true),
        args: if verbose {
            vec!["--verbose".to_string()]
        } else {
            vec![]
        },
        exit_code,
        citation: "[archwiki:System_maintenance]".to_string(),
        duration_ms,
        ok: analysis.critical_count == 0,
        error: if analysis.critical_count > 0 {
            Some(ErrorDetails {
                code: "CRITICAL_ISSUES".to_string(),
                message: format!("{} critical issue(s) detected", analysis.critical_count),
            })
        } else {
            None
        },
    };
    let _ = log_entry.write();

    std::process::exit(exit_code);
}

/// Fetch brain analysis from daemon
async fn fetch_brain_analysis() -> Result<BrainAnalysisData> {
    use anna_common::ipc::{Method, ResponseData};

    let mut client = RpcClient::connect().await?;
    let response = client.call(Method::BrainAnalysis).await?;

    match response {
        ResponseData::BrainAnalysis(data) => Ok(data),
        _ => Err(anyhow::anyhow!("Unexpected response type")),
    }
}

/// Display brain analysis with formatting
fn display_brain_analysis(analysis: &BrainAnalysisData, verbose: bool) {
    println!("{}", fmt::bold("Anna Sysadmin Brain - Diagnostic Analysis"));
    println!("{}", "=".repeat(60));
    println!();

    // Summary section
    println!("{}", fmt::section_title(&fmt::emojis::INFO, "Summary"));
    println!();

    if analysis.critical_count == 0 && analysis.warning_count == 0 {
        println!(
            "  {} {}",
            fmt::emojis::SUCCESS,
            fmt::bold("All systems nominal")
        );
        println!("    {}", fmt::dimmed("No critical or warning issues detected"));
        println!();
        println!(
            "  {}",
            fmt::dimmed(&format!("Analysis completed at {}", analysis.timestamp))
        );
        return;
    }

    // Issue counts
    if analysis.critical_count > 0 {
        println!(
            "  {} {} {}",
            fmt::emojis::WARNING,
            fmt::bold(&format!("{}", analysis.critical_count)),
            fmt::bold("critical issue(s) requiring immediate attention")
        );
    }

    if analysis.warning_count > 0 {
        println!(
            "  {} {} {}",
            "⚠️",
            fmt::bold(&format!("{}", analysis.warning_count)),
            fmt::bold("warning(s) that should be investigated")
        );
    }

    println!();

    // Detailed insights
    println!("{}", fmt::section_title(&fmt::emojis::INFO, "Diagnostic Insights"));
    println!();

    for (idx, insight) in analysis.insights.iter().enumerate() {
        display_insight(insight, idx + 1, verbose);
        println!();
    }

    // Metadata
    let separator = "─".repeat(60);
    println!("{}", fmt::dimmed(&separator));
    println!(
        "  {} Analysis completed at {}",
        fmt::emojis::TIME,
        fmt::dimmed(&analysis.timestamp)
    );
    println!(
        "  {} {} total diagnostic rules evaluated",
        fmt::emojis::INFO,
        fmt::dimmed("9")
    );
    println!();
}

/// Display a single diagnostic insight
fn display_insight(insight: &DiagnosticInsightData, number: usize, verbose: bool) {
    // Severity indicator
    let severity_emoji = match insight.severity.as_str() {
        "critical" => fmt::emojis::WARNING,
        "warning" => "⚠️",
        _ => fmt::emojis::INFO,
    };

    let severity_text = match insight.severity.as_str() {
        "critical" => fmt::error(&insight.summary),
        "warning" => fmt::warning(&insight.summary),
        _ => fmt::dimmed(&insight.summary),
    };

    // Header
    println!(
        "{} {} {}",
        severity_emoji,
        fmt::bold(&format!("{}.", number)),
        fmt::bold(&severity_text)
    );
    println!();

    // Details (always show in brain command, unlike status)
    if verbose || insight.severity == "critical" || insight.severity == "warning" {
        // Wrap details at 80 characters for readability
        let wrapped_details = wrap_text(&insight.details, 76);
        for line in wrapped_details.lines() {
            println!("    {}", line);
        }
        println!();
    }

    // Evidence
    if !insight.evidence.is_empty() && verbose {
        println!("    {} {}", fmt::bold("Evidence:"), fmt::dimmed(&insight.evidence));
        println!();
    }

    // Commands to run
    if !insight.commands.is_empty() {
        println!("    {}", fmt::bold("Diagnostic commands:"));
        for cmd in &insight.commands {
            // Format command in green
            println!("      {} \x1b[32m{}\x1b[0m", fmt::symbols::ARROW, cmd);
        }
        println!();
    }

    // Citations (verbose mode only)
    if verbose && !insight.citations.is_empty() {
        println!("    {}", fmt::bold("References:"));
        for citation in &insight.citations {
            println!("      {} {}", fmt::symbols::ARROW, fmt::dimmed(citation));
        }
        println!();
    }
}

/// Wrap text at specified width
fn wrap_text(text: &str, width: usize) -> String {
    let mut result = String::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.len() + word.len() + 1 > width {
            if !current_line.is_empty() {
                result.push_str(&current_line);
                result.push('\n');
                current_line.clear();
            }
        }

        if !current_line.is_empty() {
            current_line.push(' ');
        }
        current_line.push_str(word);
    }

    if !current_line.is_empty() {
        result.push_str(&current_line);
    }

    result
}
