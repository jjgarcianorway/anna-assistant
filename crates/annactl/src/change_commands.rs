//! Change management commands for annactl.
//!
//! v0.0.97: Extracted from commands.rs for modularity.

use anyhow::Result;
use std::io::{self, Write};

use anna_shared::ui::{colors, symbols};

/// Handle proposed config change with user confirmation
pub async fn handle_proposed_change(plan: &anna_shared::change::ChangePlan) -> Result<()> {
    use anna_shared::change::apply_change;

    println!();
    println!("{}Proposed Change{}", colors::BOLD, colors::RESET);
    println!("  File: {}", plan.target_path.display());
    println!("  Risk: {:?}", plan.risk);
    println!("  Backup: {}", plan.backup_path.display());
    println!();

    // Ask for confirmation
    print!("Apply this change? [y/N] ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Change cancelled.");
        return Ok(());
    }

    // Apply the change
    let result = apply_change(plan);

    if result.applied {
        // Record to history
        if let Ok(Some(id)) = anna_shared::change_history::record_change(plan, &result) {
            println!();
            println!(
                "{}{}{}  Change applied successfully. (ID: {})",
                colors::OK,
                symbols::OK,
                colors::RESET,
                id
            );
        } else {
            println!();
            println!(
                "{}{}{}  Change applied successfully.",
                colors::OK,
                symbols::OK,
                colors::RESET
            );
        }
        if let Some(ref backup) = result.backup_path {
            println!("    Backup: {}", backup.display());
            println!("    To undo: annactl undo <id>");
        }
    } else if result.was_noop {
        println!();
        println!(
            "{}{}{}  No changes needed - configuration already present.",
            colors::OK,
            symbols::OK,
            colors::RESET
        );
    } else if let Some(ref err) = result.error {
        println!();
        println!(
            "{}{}{}  Failed to apply change: {}",
            colors::ERR,
            symbols::ERR,
            colors::RESET,
            err
        );
    }

    Ok(())
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
                    if entry.undone { "[undone]" } else if entry.can_undo { "" } else { "[no backup]" }
                );
                println!("      {}", entry.description);
                println!("      File: {}", entry.target_path.display());
                println!();
            }
            println!("To undo a change: annactl undo <id>");
        }
        Err(e) => {
            eprintln!("{}Error:{} Failed to read history: {}", colors::ERR, colors::RESET, e);
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
            println!("{}Error:{} Change '{}' not found in history.", colors::ERR, colors::RESET, id);
            println!("Use 'annactl history' to see available changes.");
            return Ok(());
        }
        Some(entry) if entry.undone => {
            println!("{}Error:{} Change '{}' has already been undone.", colors::WARN, colors::RESET, id);
            return Ok(());
        }
        Some(entry) if !entry.can_undo => {
            println!("{}Error:{} Cannot undo '{}' - backup file not found.", colors::ERR, colors::RESET, id);
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
        UndoResult::Success { restored_from, restored_to } => {
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
            println!("{}Error:{} Backup file not found.", colors::ERR, colors::RESET);
        }
    }

    Ok(())
}
