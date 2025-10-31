// Anna v0.10.1 - annactl alerts command

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

const ALERTS_PATH: &str = "/var/lib/anna/alerts.json";

#[derive(Debug, Deserialize, Serialize)]
struct AlertsFile {
    version: String,
    generated: i64,
    alerts: Vec<IntegrityAlert>,
}

#[derive(Debug, Deserialize, Serialize)]
struct IntegrityAlert {
    id: String,
    timestamp: i64,
    severity: String,
    component: String,
    message: String,
    fix_command: Option<String>,
    impact: String,
}

pub fn show_alerts() -> Result<()> {
    if !std::path::Path::new(ALERTS_PATH).exists() {
        println!("\n✓ No alerts found - system integrity OK\n");
        println!("  Last check: (alerts.json not yet generated)");
        println!("  Next check: when integrity watchdog runs (every 10 minutes)");
        println!();
        return Ok(());
    }

    let content = fs::read_to_string(ALERTS_PATH)
        .context("Failed to read alerts.json")?;

    let alerts_file: AlertsFile = serde_json::from_str(&content)
        .context("Failed to parse alerts.json")?;

    if alerts_file.alerts.is_empty() {
        let dt = chrono::DateTime::from_timestamp(alerts_file.generated, 0)
            .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "unknown".to_string());

        println!("\n✓ No alerts found - system integrity OK\n");
        println!("  Last check: {}", dt);
        println!();
        return Ok(());
    }

    // Count by severity
    let critical = alerts_file.alerts.iter().filter(|a| a.severity == "Critical").count();
    let errors = alerts_file.alerts.iter().filter(|a| a.severity == "Error").count();
    let warnings = alerts_file.alerts.iter().filter(|a| a.severity == "Warning").count();

    println!("\n╭─ System Integrity Alerts ────────────────────────────────────────");
    println!("│");
    println!("│  {} critical, {} errors, {} warnings", critical, errors, warnings);
    println!("│");

    for alert in &alerts_file.alerts {
        let icon = match alert.severity.as_str() {
            "Critical" => "✗",
            "Error" => "⚠",
            "Warning" => "⚠",
            _ => "ℹ",
        };

        println!("│  {} [{}] {}", icon, alert.component, alert.message);
        println!("│     Impact: {}", alert.impact);

        if let Some(fix) = &alert.fix_command {
            println!("│     Fix:    annactl fix {}", alert.id);
            println!("│             (runs: {})", fix);
        }

        println!("│");
    }

    println!("╰──────────────────────────────────────────────────────────────────");
    println!();
    println!("  To fix an issue:  annactl fix <issue-id>");
    println!("  To see capabilities: annactl capabilities");
    println!();

    Ok(())
}

pub fn fix_issue(issue_id: &str, skip_confirmation: bool) -> Result<()> {
    if !std::path::Path::new(ALERTS_PATH).exists() {
        anyhow::bail!("No alerts found. Run: annactl alerts");
    }

    let content = fs::read_to_string(ALERTS_PATH)?;
    let alerts_file: AlertsFile = serde_json::from_str(&content)?;

    // Find the alert
    let alert = alerts_file
        .alerts
        .iter()
        .find(|a| a.id == issue_id)
        .ok_or_else(|| anyhow::anyhow!("Alert '{}' not found", issue_id))?;

    let fix_command = alert
        .fix_command
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No fix command available for '{}'", issue_id))?;

    // Show plan
    println!("\n╭─ Fix Plan ───────────────────────────────────────────────────────");
    println!("│");
    println!("│  Issue:   {}", alert.message);
    println!("│  Impact:  {}", alert.impact);
    println!("│  Command: {}", fix_command);
    println!("│");
    println!("╰──────────────────────────────────────────────────────────────────");
    println!();

    if !skip_confirmation {
        print!("Proceed with fix? [y/N]: ");
        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let answer = input.trim().to_lowercase();
        if answer != "y" && answer != "yes" {
            println!("✗ Fix cancelled");
            return Ok(());
        }
    }

    // Execute fix command
    println!("\n⏳ Executing fix...\n");

    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(fix_command)
        .status()
        .context("Failed to execute fix command")?;

    if status.success() {
        println!("\n✓ Fix completed successfully");
        println!("\n  Run 'annactl alerts' to verify the issue is resolved.");
    } else {
        anyhow::bail!("Fix command failed with exit code: {}", status);
    }

    Ok(())
}
