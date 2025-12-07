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
        println!("{}{} steps to apply (idempotent).{}", colors::DIM, plans.len(), colors::RESET);
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

/// Handle history command - show recent config changes
pub async fn handle_history() -> Result<()> {
    use anna_shared::change_history::read_history;

    println!();
    println!("{}Change History{}", colors::BOLD, colors::RESET);
    println!();

    match read_history(20) {
        Ok(entries) if entries.is_empty() => {
            println!("No config changes recorded yet.");
        }
        Ok(entries) => {
            for entry in entries {
                let status_color = if entry.undone {
                    colors::DIM
                } else if entry.can_undo {
                    colors::OK
                } else {
                    colors::WARN
                };

                println!(
                    "  {}{}{} {} {}",
                    status_color,
                    entry.id,
                    colors::RESET,
                    entry.timestamp,
                    if entry.undone {
                        "[undone]"
                    } else if entry.can_undo {
                        ""
                    } else {
                        "[no backup]"
                    }
                );
                println!("      {}", entry.description);
                println!("      File: {}", entry.target_path.display());
                println!();
            }
            println!("To undo a change: annactl undo <id>");
        }
        Err(e) => {
            eprintln!(
                "{}Error:{} Failed to read history: {}",
                colors::ERR,
                colors::RESET,
                e
            );
        }
    }

    Ok(())
}

/// Handle undo command - restore from backup
pub async fn handle_undo(id: &str) -> Result<()> {
    use anna_shared::change_history::{find_change, undo_change, UndoResult};

    println!();

    // First show what we're about to undo
    match find_change(id)? {
        None => {
            println!(
                "{}Error:{} Change '{}' not found in history.",
                colors::ERR,
                colors::RESET,
                id
            );
            println!("Use 'annactl history' to see available changes.");
            return Ok(());
        }
        Some(entry) if entry.undone => {
            println!(
                "{}Error:{} Change '{}' has already been undone.",
                colors::WARN,
                colors::RESET,
                id
            );
            return Ok(());
        }
        Some(entry) if !entry.can_undo => {
            println!(
                "{}Error:{} Cannot undo '{}' - backup file not found.",
                colors::ERR,
                colors::RESET,
                id
            );
            println!("Backup was: {}", entry.backup_path.display());
            return Ok(());
        }
        Some(entry) => {
            println!("{}Undo Change{}", colors::BOLD, colors::RESET);
            println!("  ID: {}", entry.id);
            println!("  Description: {}", entry.description);
            println!("  File: {}", entry.target_path.display());
            println!("  Restore from: {}", entry.backup_path.display());
            println!();

            print!("Restore this file? [y/N] ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Undo cancelled.");
                return Ok(());
            }
        }
    }

    // Perform the undo
    match undo_change(id)? {
        UndoResult::Success {
            restored_from,
            restored_to,
        } => {
            println!();
            println!(
                "{}{}{}  File restored successfully.",
                colors::OK,
                symbols::OK,
                colors::RESET
            );
            println!("    From: {}", restored_from.display());
            println!("    To: {}", restored_to.display());
        }
        UndoResult::NotFound => {
            println!("{}Error:{} Change not found.", colors::ERR, colors::RESET);
        }
        UndoResult::AlreadyUndone => {
            println!("{}Error:{} Already undone.", colors::WARN, colors::RESET);
        }
        UndoResult::NoBackup => {
            println!(
                "{}Error:{} Backup file not found.",
                colors::ERR,
                colors::RESET
            );
        }
    }

    Ok(())
}
