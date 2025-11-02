//! Actions command for Anna v0.14.0 "Orion III"
//!
//! Manage autonomous actions

use anyhow::Result;
use anna_common::{header, section, TermCaps};

use crate::action_engine::{ActionManager, ExecutionPolicy};
use crate::audit_log::{AuditEntry, AuditLog};

/// Actions command mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionsMode {
    List,
    ListAutoRunnable,
    Run { dry_run: bool },
    Revert,
}

/// Run actions command
pub fn run_actions(mode: ActionsMode, action_id: Option<String>, json: bool) -> Result<()> {
    let manager = ActionManager::new()?;
    let audit = AuditLog::new()?;

    match mode {
        ActionsMode::List => {
            let actions = manager.load_actions()?;

            if json {
                let json_str = serde_json::to_string_pretty(&actions)?;
                println!("{}", json_str);
                return Ok(());
            }

            print_actions_list(&actions);
        }
        ActionsMode::ListAutoRunnable => {
            let actions = manager.get_auto_runnable()?;

            if json {
                let json_str = serde_json::to_string_pretty(&actions)?;
                println!("{}", json_str);
                return Ok(());
            }

            print_actions_list(&actions);
        }
        ActionsMode::Run { dry_run } => {
            let action_id = action_id.expect("Action ID required for --run");
            let action = manager
                .find_action(&action_id)?
                .ok_or_else(|| anyhow::anyhow!("Action not found: {}", action_id))?;

            if dry_run {
                let preview = manager.execute_dry_run(&action)?;
                println!("{}", preview);
            } else {
                // Check execution policy
                let policy = action.execution_policy();
                if policy == ExecutionPolicy::LogOnly {
                    eprintln!("❌ Critical actions cannot be executed automatically");
                    eprintln!("This action is for logging/advisory purposes only");
                    std::process::exit(1);
                }

                if policy == ExecutionPolicy::Confirm && !confirm_execution(&action)? {
                    println!("Execution cancelled");
                    return Ok(());
                }

                // Execute
                println!("Executing: {}", action.description);
                let result = manager.execute(&action, "user")?;

                // Log to audit
                let entry = AuditEntry::new("user", &action.description, "execute")
                    .with_action_id(action.id.clone())
                    .with_result(if result.success { "success".to_string() } else { "fail".to_string() })
                    .with_details(format!("Exit code: {:?}, Duration: {}ms", result.exit_code, result.duration_ms));

                audit.log(entry)?;

                // Print result
                print_execution_result(&result);
            }
        }
        ActionsMode::Revert => {
            let action_id = action_id.expect("Action ID required for --revert");
            let action = manager
                .find_action(&action_id)?
                .ok_or_else(|| anyhow::anyhow!("Action not found: {}", action_id))?;

            if !action.reversible {
                eprintln!("❌ Action is not reversible: {}", action.id);
                std::process::exit(1);
            }

            if !confirm_revert(&action)? {
                println!("Revert cancelled");
                return Ok(());
            }

            println!("Reverting: {}", action.description);
            let result = manager.revert(&action, "user")?;

            // Log to audit
            let entry = AuditEntry::new("user", &format!("Revert: {}", action.description), "revert")
                .with_action_id(format!("{}_revert", action.id))
                .with_result(if result.success { "success".to_string() } else { "fail".to_string() })
                .with_details(format!("Exit code: {:?}, Duration: {}ms", result.exit_code, result.duration_ms));

            audit.log(entry)?;

            print_execution_result(&result);
        }
    }

    Ok(())
}

/// Print actions list
fn print_actions_list(actions: &[crate::action_engine::Action]) {
    let caps = TermCaps::detect();

    println!("{}", header(&caps, "Registered Actions"));
    println!();

    if actions.is_empty() {
        println!("No actions registered");
        println!();
        println!("Use 'annactl actions --init' to register safe built-in actions");
        return;
    }

    for action in actions {
        let policy = action.execution_policy();
        let policy_badge = match policy {
            ExecutionPolicy::AutoRun => "🤖 AUTO",
            ExecutionPolicy::Confirm => "⚠️  CONFIRM",
            ExecutionPolicy::LogOnly => "📋 LOG-ONLY",
            ExecutionPolicy::DryRun => "🔍 DRY-RUN",
        };

        let reversible_badge = if action.reversible { "↩️  Reversible" } else { "⚠️  Irreversible" };

        println!("{}", section(&caps, &format!("{} {}", action.emoji(), action.id)));
        println!();
        println!("  Description: {}", action.description);
        println!("  Priority:    {}", action.priority);
        println!("  Policy:      {}", policy_badge);
        println!("  Reversible:  {}", reversible_badge);
        println!("  Command:     {}", action.command);

        if let Some(ref revert_cmd) = action.revert_command {
            println!("  Revert Cmd:  {}", revert_cmd);
        }

        println!("  Tags:        {}", action.tags.join(", "));
        println!();
    }
}

/// Print execution result
fn print_execution_result(result: &crate::action_engine::ActionResult) {
    let caps = TermCaps::detect();

    let status_emoji = if result.success { "✅" } else { "❌" };
    let status_text = if result.success { "SUCCESS" } else { "FAILED" };

    println!();
    println!("{}", section(&caps, &format!("{} {}", status_emoji, status_text)));
    println!();

    if result.success {
        println!("  Action completed successfully");
    } else {
        println!("  Action failed with exit code: {:?}", result.exit_code);
    }

    println!("  Duration: {}ms", result.duration_ms);
    println!();

    if !result.output.trim().is_empty() {
        println!("{}", section(&caps, "Output"));
        println!();
        for line in result.output.lines().take(20) {
            println!("  {}", line);
        }
        println!();
    }
}

/// Confirm execution with user
fn confirm_execution(action: &crate::action_engine::Action) -> Result<bool> {
    println!();
    println!("⚠️  This action requires confirmation:");
    println!();
    println!("  Description: {}", action.description);
    println!("  Command:     {}", action.command);
    println!("  Reversible:  {}", if action.reversible { "yes" } else { "no" });
    println!();

    print!("Execute this action? [y/N]: ");
    use std::io::Write;
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_lowercase() == "y")
}

/// Confirm revert with user
fn confirm_revert(action: &crate::action_engine::Action) -> Result<bool> {
    println!();
    println!("⚠️  Revert confirmation:");
    println!();
    println!("  Original action: {}", action.description);
    if let Some(ref revert_cmd) = action.revert_command {
        println!("  Revert command:  {}", revert_cmd);
    }
    println!();

    print!("Revert this action? [y/N]: ");
    use std::io::Write;
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_lowercase() == "y")
}

/// Initialize safe built-in actions
pub fn initialize_actions() -> Result<()> {
    let manager = ActionManager::new()?;
    let actions = crate::action_engine::create_safe_actions();

    println!("Registering {} safe built-in actions...", actions.len());
    println!();

    for action in actions {
        println!("  {} {}: {}", action.emoji(), action.id, action.description);
        manager.register(action)?;
    }

    println!();
    println!("✅ Actions registered successfully");
    println!();
    println!("Use 'annactl actions --list' to view all actions");

    Ok(())
}
