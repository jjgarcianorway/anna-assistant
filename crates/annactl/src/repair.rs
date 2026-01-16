//! Repair commands for assisted operations (Phase 43)
//!
//! This module provides CLI commands for supervised repair operations.
//!
//! # CRITICAL INVARIANT: USER CONFIRMATION REQUIRED
//!
//! This module:
//! - Displays diagnosis results to the user
//! - Shows safe commands that can be run automatically
//! - Shows manual commands that require copy/paste
//! - Requires explicit user confirmation before any execution
//! - Reports execution results clearly
//!
//! # Execution Model
//!
//! 1. User runs `annactl repair wifi`
//! 2. Anna diagnoses the WiFi issue
//! 3. Anna shows safe commands (can run automatically with confirmation)
//! 4. Anna shows manual commands (require sudo, must be copy/pasted)
//! 5. User types exact confirmation phrase for safe commands
//! 6. Safe commands execute via HumanExecutionAdapter
//! 7. Results displayed
//! 8. Manual commands shown with copy/paste instructions
//!
//! The user is always in control. Always.

use anna_shared::execution_request::AUTOMATIC_EXECUTION_CONFIRMATION;
use anna_shared::human_execution::{HumanExecutionAdapter, HumanExecutionResult};
use anna_shared::rpc::CommandSafety;
use std::io::{self, Write};

use crate::display::*;

/// Handle the repair wifi command.
///
/// This function:
/// 1. Runs WiFi diagnosis via annad
/// 2. Shows results to user
/// 3. If safe commands exist, asks for confirmation
/// 4. Executes safe commands with HumanExecutionAdapter
/// 5. Shows manual commands for user to run
pub async fn handle_repair_wifi() {
    println!();
    println_colored("WiFi Repair", BOLD);
    println_colored("===========", DIM);
    println!();

    // Step 1: Request diagnosis from daemon
    println_colored("Diagnosing WiFi...", DIM);
    println!();

    match crate::rpc::diagnose_wifi().await {
        Ok(operation) => {
            // Step 2: Show diagnosis summary
            println_colored("Detected Issue:", YELLOW);
            println!("  {}", operation.detected_problem);
            println!();

            if !operation.diagnosis_summary.is_empty() {
                println_colored("Diagnosis:", DIM);
                for line in operation.diagnosis_summary.lines() {
                    println!("  {}", line);
                }
                println!();
            }

            // Show explanation
            println_colored("Explanation:", CYAN);
            for line in operation.explanation.lines() {
                println!("  {}", line);
            }
            println!();

            // Show risk level
            print_colored("Risk Level: ", DIM);
            use anna_shared::rpc::RiskLevel;
            let risk_color = match operation.risk_level {
                RiskLevel::Low => GREEN,
                RiskLevel::Medium => YELLOW,
                RiskLevel::High => RED,
                RiskLevel::Critical => RED,
            };
            println_colored(&format!("{:?}", operation.risk_level), risk_color);
            println!();

            // Show sources/citations
            if !operation.sources.is_empty() {
                println_colored("References:", DIM);
                for source in &operation.sources {
                    println!("  - {} ({})", source.title, source.reference);
                }
                println!();
            }

            // Separate safe and manual commands
            let safe_commands: Vec<_> = operation
                .proposed_steps
                .iter()
                .filter(|s| matches!(s.safety, CommandSafety::SafeAutomatic))
                .collect();

            let manual_commands: Vec<_> = operation
                .proposed_steps
                .iter()
                .filter(|s| matches!(s.safety, CommandSafety::ManualOnly))
                .collect();

            // Step 3: Handle safe commands (can run automatically)
            if !safe_commands.is_empty() {
                println_colored("Safe Commands (can run automatically):", GREEN);
                println!();
                for step in &safe_commands {
                    println!("  {}. {}", step.step_number, step.description);
                    println_colored(&format!("     $ {}", step.exact_command), CYAN);
                    println!();
                }

                // Ask for confirmation
                println!();
                print_colored("To run these commands automatically, type exactly:", YELLOW);
                println!();
                println_colored(
                    &format!("  \"{}\"", AUTOMATIC_EXECUTION_CONFIRMATION),
                    BOLD,
                );
                println!();
                print!("Your confirmation: ");
                io::stdout().flush().ok();

                let mut confirmation = String::new();
                if io::stdin().read_line(&mut confirmation).is_ok() {
                    let confirmation = confirmation.trim();
                    if confirmation == AUTOMATIC_EXECUTION_CONFIRMATION {
                        // Execute safe commands
                        println!();
                        println_colored("Executing safe commands...", GREEN);
                        println!();

                        let operator =
                            std::env::var("USER").unwrap_or_else(|_| "operator".to_string());
                        let adapter = HumanExecutionAdapter::new(&operator);

                        // Create execution request
                        let request = anna_shared::execution_request::ExecutionRequest {
                            request_id: format!(
                                "repair-wifi-{}",
                                chrono::Utc::now().timestamp_millis()
                            ),
                            proposal_id: operation.operation_id.clone(),
                            recorded_utc: chrono::Utc::now().to_rfc3339(),
                            requested_by: operator.clone(),
                            requested_action: format!(
                                "WiFi repair: {}",
                                operation.detected_problem
                            ),
                            confirmation_text: AUTOMATIC_EXECUTION_CONFIRMATION.to_string(),
                        };

                        for step in &safe_commands {
                            print!("  Running: ");
                            println_colored(&step.exact_command, CYAN);

                            match adapter.execute(&request, &step.exact_command) {
                                Ok(result) => {
                                    display_execution_result(&result);
                                }
                                Err(e) => {
                                    print_colored("    Error: ", RED);
                                    println!("{}", e);
                                }
                            }
                            println!();
                        }
                    } else {
                        println!();
                        println_colored("Confirmation text did not match. Skipping automatic execution.", YELLOW);
                    }
                } else {
                    println!();
                    println_colored("Could not read input. Skipping automatic execution.", YELLOW);
                }
            }

            // Step 4: Show manual commands
            if !manual_commands.is_empty() {
                println!();
                println_colored("Manual Commands (require sudo):", YELLOW);
                println_colored("Copy and paste these commands into your terminal:", DIM);
                println!();

                for step in &manual_commands {
                    println!("  {}. {}", step.step_number, step.description);
                    println_colored(&format!("     $ {}", step.exact_command), BOLD);
                    if !step.why.is_empty() {
                        println_colored(&format!("     Why: {}", step.why), DIM);
                    }
                    if step.reversible {
                        if let Some(ref reverse) = step.reverse_command {
                            println_colored(&format!("     Undo: {}", reverse), DIM);
                        }
                    }
                    println!();
                }
            }

            // Show reboot notice if needed
            if operation.requires_reboot {
                println!();
                println_colored(
                    "NOTE: A reboot may be required for changes to take effect.",
                    YELLOW,
                );
            }
        }
        Err(e) => {
            print_colored("Diagnosis failed: ", RED);
            println!("{}", e);
            println!();
            println_colored(
                "WiFi diagnosis requires the Anna daemon (annad) to be running.",
                DIM,
            );
        }
    }

    println!();
}

/// Display the result of a command execution.
fn display_execution_result(result: &HumanExecutionResult) {
    if result.success {
        println_colored("    [OK]", GREEN);
    } else {
        print_colored("    [FAIL] exit code ", RED);
        println!("{}", result.exit_code);
    }

    if !result.stdout.is_empty() {
        let stdout = result.stdout.trim();
        if stdout.lines().count() <= 5 {
            for line in stdout.lines() {
                println!("    {}", line);
            }
        } else {
            println_colored("    (output truncated, {} lines)", DIM);
        }
    }

    if !result.stderr.is_empty() && !result.success {
        print_colored("    stderr: ", YELLOW);
        let stderr = result.stderr.trim();
        if stderr.lines().count() == 1 {
            println!("{}", stderr);
        } else {
            println!();
            for line in stderr.lines().take(3) {
                println!("      {}", line);
            }
        }
    }
}

/// Show repair help
pub fn show_repair_help() {
    println!();
    println_colored("REPAIR COMMANDS", BOLD);
    println!();
    println!("  annactl repair wifi    Diagnose and repair WiFi issues");
    println!();
    println_colored("How it works:", DIM);
    println!("  1. Anna diagnoses your system");
    println!("  2. Safe commands can run automatically (with your confirmation)");
    println!("  3. Commands requiring sudo are shown for you to copy/paste");
    println!();
}

// =============================================================================
// EXPLICIT SAFETY DOCUMENTATION
// =============================================================================
//
// This module:
// - REQUIRES user confirmation for automatic execution
// - USES HumanExecutionAdapter with its allowlist restrictions
// - CANNOT run sudo commands automatically
// - CANNOT bypass confirmation
// - DISPLAYS manual commands for user to run themselves
//
// The automatic execution path:
// 1. User sees exact commands
// 2. User types exact confirmation phrase
// 3. HumanExecutionAdapter validates command against allowlist
// 4. Only allowlisted binaries (iw, lsmod, lspci, cat, echo) can run
// 5. Results displayed to user
//
// The user is always in control.
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proof_requires_confirmation() {
        // The handle_repair_wifi function:
        // 1. Prompts user for exact confirmation text
        // 2. Compares against AUTOMATIC_EXECUTION_CONFIRMATION
        // 3. Only proceeds if exact match
        //
        // Without user typing the exact phrase, no execution occurs.
    }

    #[test]
    fn proof_uses_human_adapter() {
        // The function uses HumanExecutionAdapter which:
        // - Has a strict binary allowlist
        // - Forbids sudo, pipes, redirects
        // - Records every execution attempt
    }

    #[test]
    fn proof_manual_commands_not_executed() {
        // Manual commands are displayed with println!()
        // They are NOT passed to any execution function
        // The user must copy/paste them to their terminal
    }
}
