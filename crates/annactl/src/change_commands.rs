//! Change management commands for annactl.
//!
//! v0.0.97: Extracted from commands.rs for modularity.

use anyhow::Result;
use std::io::{self, Write};

use anna_shared::ui::{colors, symbols};

/// Outcome summary for applying proposed changes
pub struct ChangeSummary {
    pub applied: usize,
    pub noop: usize,
    pub failed: bool,
}

/// Handle proposed config change with user confirmation
pub async fn handle_proposed_change(
    plans: &[anna_shared::change::ChangePlan],
) -> Result<ChangeSummary> {
    use anna_shared::change::apply_change;

    if plans.is_empty() {
        println!("{}No changes proposed.{}", colors::WARN, colors::RESET);
        return Ok(ChangeSummary {
            applied: 0,
            noop: 0,
            failed: false,
        });
    }

    println!();
    println!("{}Proposed Change{}", colors::BOLD, colors::RESET);
    for (idx, plan) in plans.iter().enumerate() {
        println!("  [{}] File: {}", idx + 1, plan.target_path.display());
        println!("      {}", plan.description);
        println!("      Risk: {:?}", plan.risk);
        println!("      Backup: {}", plan.backup_path.display());
        let op = match &plan.operation {
            anna_shared::change::ChangeOperation::EnsureLine { line } => {
                format!("Ensure line exists: \"{}\"", line)
            }
            anna_shared::change::ChangeOperation::AppendLine { line } => {
                format!("Append line: \"{}\"", line)
            }
        };
        println!("      Action: {}", op);
    }
    println!();
    if plans.len() > 1 {
        println!(
            "{}{} steps to apply (idempotent).{}",
            colors::DIM,
            plans.len(),
            colors::RESET
        );
    }

    // Ask for confirmation
    print!("Apply this change? [y/N] ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Change cancelled.");
        return Ok(ChangeSummary {
            applied: 0,
            noop: 0,
            failed: false,
        });
    }

    // Apply the change
    let mut applied_count = 0usize;
    let mut noop_count = 0usize;
    let mut failed = false;

    for (idx, plan) in plans.iter().enumerate() {
        let result = apply_change(plan);

        if result.applied {
            // Record to history
            if let Ok(Some(id)) = anna_shared::change_history::record_change(plan, &result) {
                println!();
                println!(
                    "{}{}{}  Step {} applied. (ID: {})",
                    colors::OK,
                    symbols::OK,
                    colors::RESET,
                    idx + 1,
                    id
                );
            } else {
                println!();
                println!(
                    "{}{}{}  Step {} applied.",
                    colors::OK,
                    symbols::OK,
                    colors::RESET,
                    idx + 1
                );
            }
            if let Some(ref backup) = result.backup_path {
                println!("    Backup: {}", backup.display());
                println!("    To undo: annactl undo <id>");
            }
            applied_count += 1;
        } else if result.was_noop {
            println!();
            println!(
                "{}{}{}  Step {}: No changes needed - configuration already present.",
                colors::OK,
                symbols::OK,
                colors::RESET,
                idx + 1
            );
            noop_count += 1;
        } else if let Some(ref err) = result.error {
            println!();
            println!(
                "{}{}{}  Step {} failed: {}",
                colors::ERR,
                symbols::ERR,
                colors::RESET,
                idx + 1,
                err
            );
            failed = true;
        }
    }

    if failed {
        println!(
            "{}One or more steps failed.{} See above for details.",
            colors::ERR,
            colors::RESET
        );
    } else {
        println!();
        println!(
            "{}Change summary:{} applied={}, noop={}",
            colors::BOLD,
            colors::RESET,
            applied_count,
            noop_count
        );
    }

    Ok(ChangeSummary {
        applied: applied_count,
        noop: noop_count,
        failed,
    })
}

// v0.0.144: handle_history and handle_undo removed - use natural language instead
