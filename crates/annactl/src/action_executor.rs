//! Action Executor - Execute approved suggestions with change logging
//!
//! Implements the approval flow: show → confirm → execute → log

use anna_common::change_log::*;
use anna_common::change_log_db::ChangeLogDb;
use anna_common::context::db::DbLocation;
use anna_common::suggestions::Suggestion;
use anyhow::{Result, Context};
use std::io::{self, Write};
use std::process::Command;

/// Execute a suggestion after user approval
pub async fn execute_suggestion(suggestion: &Suggestion) -> Result<()> {
    // Show what will be executed
    println!("\n");
    println!("══════════════════════════════════════════════════");
    println!("ACTION PLAN: {}", suggestion.title);
    println!("══════════════════════════════════════════════════\n");

    if let Some(ref desc) = suggestion.fix_description {
        println!("What I will do:");
        println!("  {}\n", desc);
    }

    println!("Commands to execute:");
    for cmd in &suggestion.fix_commands {
        let needs_sudo = cmd.starts_with("sudo");
        let indicator = if needs_sudo { "🔐" } else { "  " };
        println!("  {} {}", indicator, cmd);
    }
    println!();

    if suggestion.fix_commands.iter().any(|cmd| cmd.starts_with("sudo")) {
        println!("⚠️  Some commands require sudo (root privileges)");
        println!();
    }

    // Ask for confirmation
    print!("Do you want me to proceed? [y/N]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") && !input.trim().eq_ignore_ascii_case("yes") {
        println!("\nAction cancelled. No changes were made.\n");
        return Ok(());
    }

    println!();
    println!("Executing actions...\n");

    // Create change unit
    let mut change_unit = ChangeUnit::new(
        format!("fix-{}", suggestion.key),
        format!("User requested: fix {}", suggestion.title),
    );

    // TODO: Capture metrics before
    // change_unit.set_metrics_before(capture_current_metrics());

    // Execute commands
    let mut all_success = true;

    for cmd in &suggestion.fix_commands {
        println!("Running: {}", cmd);

        let (program, args) = parse_command(cmd);

        let mut action = ChangeAction::command(
            program.clone(),
            args.clone(),
            format!("Execute: {}", cmd),
        );

        let result = Command::new(&program)
            .args(&args)
            .status()
            .context(format!("Failed to execute: {}", cmd))?;

        action.success = result.success();

        if let ActionType::Command { ref mut exit_code, .. } = action.action_type {
            *exit_code = result.code();
        }

        // Set rollback info if applicable
        if suggestion.key == "pacman-cache-cleanup" {
            action.rollback_info = Some(RollbackInfo {
                can_rollback: false,
                cannot_rollback_reason: Some(
                    "Pacman cache cleanup cannot be rolled back. Packages were only removed from cache, not from the system.".to_string()
                ),
                rollback_commands: Vec::new(),
                files_to_restore: Vec::new(),
            });
        } else if cmd.starts_with("sudo pacman -S") {
            // Package install can be rolled back
            let packages: Vec<String> = args.iter()
                .skip_while(|a| a.starts_with("-"))
                .map(|s| s.to_string())
                .collect();

            action.rollback_info = Some(RollbackInfo {
                can_rollback: true,
                cannot_rollback_reason: None,
                rollback_commands: vec![format!("sudo pacman -Rns {}", packages.join(" "))],
                files_to_restore: Vec::new(),
            });
        }

        change_unit.add_action(action);

        if !result.success() {
            eprintln!("❌ Command failed with exit code: {:?}", result.code());
            all_success = false;
            break;
        } else {
            println!("✓ Success\n");
        }
    }

    // Complete change unit
    let status = if all_success {
        ChangeStatus::Success
    } else {
        ChangeStatus::Partial
    };

    change_unit.complete(status);

    // TODO: Capture metrics after
    // change_unit.set_metrics_after(capture_current_metrics());

    // Persist to SQLite
    let db_location = DbLocation::auto_detect();
    match ChangeLogDb::open(db_location).await {
        Ok(db) => {
            if let Err(e) = db.save_change_unit(&change_unit).await {
                eprintln!("Warning: Failed to persist change log: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Warning: Failed to open change log database: {}", e);
        }
    }

    println!("══════════════════════════════════════════════════");
    if all_success {
        println!("✓ All actions completed successfully!");
    } else {
        println!("⚠️  Some actions failed. Check the output above.");
    }
    println!("══════════════════════════════════════════════════\n");

    println!("Change logged: {}", change_unit.id);
    println!("Can rollback: {}", if change_unit.can_rollback() { "Yes" } else { "No" });

    if !change_unit.can_rollback() {
        let limitations = change_unit.rollback_limitations();
        if !limitations.is_empty() {
            println!("\nRollback limitations:");
            for limitation in limitations {
                println!("  • {}", limitation);
            }
        }
    }

    println!();

    Ok(())
}

/// Parse a command string into program and args
fn parse_command(cmd: &str) -> (String, Vec<String>) {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return (String::new(), Vec::new());
    }

    let program = parts[0].to_string();
    let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

    (program, args)
}

/// Ask user which suggestion they want to apply
pub fn select_suggestion_to_apply(suggestions: &[&Suggestion]) -> Option<usize> {
    println!("\n");
    println!("Which suggestion would you like me to apply?");
    println!();

    let auto_fixable: Vec<(usize, &&Suggestion)> = suggestions
        .iter()
        .enumerate()
        .filter(|&(_, s)| s.auto_fixable)
        .collect();

    if auto_fixable.is_empty() {
        println!("None of the current suggestions can be automatically fixed.");
        println!("Please review the suggestions and apply fixes manually if needed.\n");
        return None;
    }

    for (display_num, (orig_idx, suggestion)) in auto_fixable.iter().enumerate() {
        println!("  {}. {}", display_num + 1, suggestion.title);
    }

    println!("  0. Cancel");
    println!();

    print!("Enter number: ");
    io::stdout().flush().ok();

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;

    let choice = input.trim().parse::<usize>().ok()?;

    if choice == 0 || choice > auto_fixable.len() {
        return None;
    }

    Some(auto_fixable[choice - 1].0)
}
