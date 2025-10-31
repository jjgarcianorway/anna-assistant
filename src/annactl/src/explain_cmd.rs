//! Explain Command - Show recent autonomic actions with reasoning

use anyhow::Result;
use anna_common::{anna_box, anna_info, MessageType};
use std::fs;
use std::path::Path;

const ADAPTIVE_LOG: &str = "/var/log/anna/adaptive.log";

/// Show last N autonomic actions with full explainability
pub async fn explain_last(count: usize) -> Result<()> {
    anna_box(&["Recent Autonomic Actions"], MessageType::Info);
    println!();

    if !Path::new(ADAPTIVE_LOG).exists() {
        anna_info("No autonomic actions logged yet.");
        anna_info("Anna will log actions once thermal/power management activates.");
        return Ok(());
    }

    // Read log file
    let content = fs::read_to_string(ADAPTIVE_LOG)?;

    // Split into action blocks (separated by blank lines)
    let blocks: Vec<&str> = content
        .split("\n\n")
        .filter(|s| !s.trim().is_empty())
        .collect();

    if blocks.is_empty() {
        anna_info("No actions logged yet.");
        return Ok(());
    }

    // Show last N blocks
    let recent: Vec<&str> = blocks
        .iter()
        .rev()
        .take(count)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    println!("Showing {} most recent action(s):\n", recent.len());

    for (i, block) in recent.iter().enumerate() {
        println!("─────────────────────────────────────────");
        println!("Action #{}", i + 1);
        println!("─────────────────────────────────────────");
        println!("{}", block);
        println!();
    }

    Ok(())
}

/// Show summary statistics
pub async fn explain_stats() -> Result<()> {
    anna_box(&["Autonomic Action Statistics"], MessageType::Info);
    println!();

    if !Path::new(ADAPTIVE_LOG).exists() {
        anna_info("No log file yet.");
        return Ok(());
    }

    let content = fs::read_to_string(ADAPTIVE_LOG)?;

    // Count total actions
    let total_actions = content.matches("[").filter(|s| s.contains("ACTION")).count();

    // Count successes and failures
    let successes = content.matches("RESULT: SUCCESS").count();
    let failures = content.matches("RESULT: FAILED").count();
    let skipped = content.matches("RESULT: SKIPPED").count();

    println!("Total Actions:  {}", total_actions);
    println!("  Success:      {}", successes);
    println!("  Failed:       {}", failures);
    println!("  Skipped:      {}", skipped);
    println!();

    // Show most common actions
    let actions: Vec<&str> = content
        .lines()
        .filter(|line| line.contains("ACTION"))
        .collect();

    if !actions.is_empty() {
        println!("Recent action types:");
        for action in actions.iter().rev().take(5) {
            if let Some(action_type) = action.split("ACTION ").nth(1) {
                if let Some(action_name) = action_type.split(" →").next() {
                    println!("  • {}", action_name);
                }
            }
        }
    }

    println!();

    Ok(())
}
