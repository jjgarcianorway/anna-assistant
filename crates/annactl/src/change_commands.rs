//! Change management commands for annactl (v0.0.312).
//!
//! v0.0.97: Extracted from commands.rs for modularity.
//! v0.0.292: Added auto-confirm for low-risk operations.
//! v0.0.312: Added RunCommand support for executing system commands.

use anyhow::Result;
use std::io::{self, Write};

use anna_shared::change::{ChangeOperation, ChangeRisk};
use anna_shared::ui::{colors, symbols};
use anna_shared::user_profile::UserProfile;

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
        // v0.0.312: Handle RunCommand differently
        let op = match &plan.operation {
            ChangeOperation::EnsureLine { line } => {
                println!("  [{}] File: {}", idx + 1, plan.target_path.display());
                println!("      {}", plan.description);
                println!("      Backup: {}", plan.backup_path.display());
                format!("Ensure line exists: \"{}\"", line)
            }
            ChangeOperation::AppendLine { line } => {
                println!("  [{}] File: {}", idx + 1, plan.target_path.display());
                println!("      {}", plan.description);
                println!("      Backup: {}", plan.backup_path.display());
                format!("Append line: \"{}\"", line)
            }
            ChangeOperation::RunCommand { command, what_it_does, needs_sudo } => {
                println!("  [{}] {}Command Execution{}", idx + 1, colors::WARN, colors::RESET);
                println!("      {}", what_it_does);
                println!("      Command: {}{}{}", colors::BOLD, command, colors::RESET);
                if *needs_sudo {
                    println!("      {}Requires elevated privileges{}", colors::WARN, colors::RESET);
                }
                "Execute shell command".to_string()
            }
        };
        let risk_color = match plan.risk {
            ChangeRisk::Low => colors::OK,
            ChangeRisk::Medium => colors::WARN,
            ChangeRisk::High => colors::ERR,
        };
        println!("      Risk: {}{:?}{}", risk_color, plan.risk, colors::RESET);
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

    // v0.0.292: Check if we can auto-confirm low-risk changes
    let profile = UserProfile::load();
    let all_low_risk = plans.iter().all(|p| matches!(p.risk, ChangeRisk::Low));
    let auto_confirm = profile.preferences.auto_confirm_low_risk && all_low_risk;

    if auto_confirm {
        println!(
            "{}Auto-confirming low-risk change (per user settings){}",
            colors::DIM,
            colors::RESET
        );
    } else {
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
    }

    // Apply the change
    let mut applied_count = 0usize;
    let mut noop_count = 0usize;
    let mut failed = false;

    for (idx, plan) in plans.iter().enumerate() {
        // v0.0.312: Handle RunCommand via daemon RPC
        if let ChangeOperation::RunCommand { command, .. } = &plan.operation {
            println!();
            println!("{}Executing command...{}", colors::DIM, colors::RESET);

            let exec_result = execute_command_via_daemon(command, "change").await;
            match exec_result {
                Ok(result) => {
                    if result.success {
                        println!(
                            "{}{}{}  Command completed successfully in {}ms",
                            colors::OK,
                            symbols::OK,
                            colors::RESET,
                            result.duration_ms
                        );
                        if !result.stdout.is_empty() {
                            println!("\n{}", result.stdout.trim());
                        }
                        applied_count += 1;
                    } else {
                        println!(
                            "{}{}{}  Command failed (exit code {})",
                            colors::ERR,
                            symbols::ERR,
                            colors::RESET,
                            result.exit_code
                        );
                        if !result.stderr.is_empty() {
                            println!("{}{}{}", colors::ERR, result.stderr.trim(), colors::RESET);
                        }
                        failed = true;
                    }
                }
                Err(e) => {
                    println!(
                        "{}{}{}  Failed to execute: {}",
                        colors::ERR,
                        symbols::ERR,
                        colors::RESET,
                        e
                    );
                    failed = true;
                }
            }
            continue;
        }

        // File changes use apply_change
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

/// v0.0.312: Execute a command via daemon (with elevated privileges)
async fn execute_command_via_daemon(
    command: &str,
    request_id: &str,
) -> Result<anna_shared::rpc::CommandExecutionResult> {
    use crate::client::AnnadClient;
    let mut client = AnnadClient::connect().await?;
    client.execute_command(command, request_id).await
}
