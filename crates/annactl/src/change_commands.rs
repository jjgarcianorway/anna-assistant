//! Change management commands for annactl (v0.0.355).
//!
//! v0.0.97: Extracted from commands.rs for modularity.
//! v0.0.292: Added auto-confirm for low-risk operations.
//! v0.0.312: Added RunCommand support for executing system commands.
//! v0.0.342: Use centralized UI helpers for consistency.
//! v0.0.344: Use print_title() for header display.
//! v0.0.355: Use print_ok(), print_err() for status messages.

use anyhow::Result;
use std::io::{self, Write};

use anna_shared::change::{ChangeOperation, ChangeRisk};
use anna_shared::ui::{colors, kv, print_err, print_footer, print_hint, print_label, print_ok, print_section_header, print_title};
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
        print_label("change", "No changes proposed", colors::DIM);
        return Ok(ChangeSummary {
            applied: 0,
            noop: 0,
            failed: false,
        });
    }

    println!();
    print_title("Proposed Change");
    println!();

    for (idx, plan) in plans.iter().enumerate() {
        // v0.0.312: Handle RunCommand differently
        let op = match &plan.operation {
            ChangeOperation::EnsureLine { line } => {
                print_section_header(&format!("step {}", idx + 1));
                kv("file", &plan.target_path.display().to_string());
                kv("description", &plan.description);
                kv("backup", &plan.backup_path.display().to_string());
                format!("Ensure line exists: \"{}\"", line)
            }
            ChangeOperation::AppendLine { line } => {
                print_section_header(&format!("step {}", idx + 1));
                kv("file", &plan.target_path.display().to_string());
                kv("description", &plan.description);
                kv("backup", &plan.backup_path.display().to_string());
                format!("Append line: \"{}\"", line)
            }
            ChangeOperation::RunCommand { command, what_it_does, needs_sudo } => {
                print_section_header(&format!("step {} (command)", idx + 1));
                kv("description", what_it_does);
                kv("command", &format!("{}{}{}", colors::BOLD, command, colors::RESET));
                if *needs_sudo {
                    kv("privileges", &format!("{}elevated{}", colors::WARN, colors::RESET));
                }
                "Execute shell command".to_string()
            }
        };
        let risk_color = match plan.risk {
            ChangeRisk::Low => colors::OK,
            ChangeRisk::Medium => colors::WARN,
            ChangeRisk::High => colors::ERR,
        };
        kv("risk", &format!("{}{:?}{}", risk_color, plan.risk, colors::RESET));
        kv("action", &op);
        println!();
    }

    if plans.len() > 1 {
        print_hint(&format!("{} steps to apply (idempotent)", plans.len()));
    }

    // v0.0.292: Check if we can auto-confirm low-risk changes
    let profile = UserProfile::load();
    let all_low_risk = plans.iter().all(|p| matches!(p.risk, ChangeRisk::Low));
    let auto_confirm = profile.preferences.auto_confirm_low_risk && all_low_risk;

    if auto_confirm {
        print_hint("Auto-confirming low-risk change (per user settings)");
    } else {
        // Ask for confirmation
        print_section_header("confirmation");
        print!("  Apply this change? [y/N] ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!();
            print_label("cancelled", "Change aborted by user", colors::DIM);
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

    println!();
    print_section_header("execution");

    for (idx, plan) in plans.iter().enumerate() {
        // v0.0.312: Handle RunCommand via daemon RPC
        if let ChangeOperation::RunCommand { command, .. } = &plan.operation {
            print_hint("Executing command...");

            let exec_result = execute_command_via_daemon(command, "change").await;
            match exec_result {
                Ok(result) => {
                    if result.success {
                        print_ok(&format!("Command completed in {}ms", result.duration_ms));
                        if !result.stdout.is_empty() {
                            print_hint(result.stdout.trim());
                        }
                        applied_count += 1;
                    } else {
                        print_err(&format!("Command failed (exit code {})", result.exit_code));
                        if !result.stderr.is_empty() {
                            print_hint(result.stderr.trim());
                        }
                        failed = true;
                    }
                }
                Err(e) => {
                    print_err(&format!("Failed to execute: {}", e));
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
                print_ok(&format!("Step {} applied (ID: {})", idx + 1, id));
            } else {
                print_ok(&format!("Step {} applied", idx + 1));
            }
            if let Some(ref backup) = result.backup_path {
                print_hint(&format!("Backup: {}", backup.display()));
                print_hint("Ask me to undo this change if needed.");
            }
            applied_count += 1;
        } else if result.was_noop {
            print_ok(&format!("Step {}: Already configured", idx + 1));
            noop_count += 1;
        } else if let Some(ref err) = result.error {
            print_err(&format!("Step {} failed: {}", idx + 1, err));
            failed = true;
        }
    }

    println!();
    if failed {
        print_label("result", "One or more steps failed", colors::ERR);
    } else {
        print_section_header("summary");
        kv("applied", &format!("{}", applied_count));
        kv("noop", &format!("{}", noop_count));
    }
    print_footer();

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
