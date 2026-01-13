//! Command retry logic with auto-installation and recovery.

use anna_shared::config::AnnaConfig;
use anna_shared::deps::{command_exists, install_package};
use anna_shared::memory::Memory;
use tracing::{debug, info, warn};

use super::errors::{classify_command_error, get_recovery_prompt};
use super::execute::execute_command;
use super::package_map::COMMAND_TO_PACKAGE;
use super::types::CommandErrorType;
use crate::core_loop::cache::{clear_failure_cache, get_perf_config, record_command_failure_cache};
use crate::core_loop::safety::is_dangerous_command;
use crate::ollama;

/// Try to auto-install a missing command
/// v0.0.919: Returns true if installation succeeded and command now exists
pub fn try_auto_install(cmd: &str) -> bool {
    // Extract the base command (first word)
    let base_cmd = cmd.split_whitespace().next().unwrap_or(cmd);

    // Check if auto-install is enabled
    let config = AnnaConfig::load().unwrap_or_default();
    if !config.auto_install_helpers {
        debug!("Auto-install disabled in config");
        return false;
    }

    // Already installed?
    if command_exists(base_cmd) {
        return true;
    }

    // Find the package for this command
    let package = COMMAND_TO_PACKAGE
        .iter()
        .find(|(c, _)| *c == base_cmd)
        .map(|(_, p)| *p);

    let package = match package {
        Some(p) => p,
        None => {
            debug!("No package mapping for command: {}", base_cmd);
            // Try using the command name as package name (works for many tools)
            base_cmd
        }
    };

    info!(
        "Auto-installing package '{}' for command '{}'",
        package, base_cmd
    );

    match install_package(package) {
        Ok(true) => {
            info!("Successfully installed package: {}", package);
            // v0.0.921: Clear failure cache since new command is available
            clear_failure_cache();
            true
        }
        Ok(false) => {
            // Already installed (shouldn't happen, but handle gracefully)
            true
        }
        Err(e) => {
            warn!("Failed to install package '{}': {}", package, e);
            false
        }
    }
}

/// Record a command failure in memory for future avoidance
/// v0.0.921: Also records to session-level failure cache
pub fn record_command_failure(cmd: &str, error_type: &CommandErrorType) {
    // Record to session-level cache for immediate effect
    record_command_failure_cache(cmd, &format!("{:?}", error_type));

    // Also record to long-term memory
    if let Ok(mut memory) = Memory::load() {
        for exp in memory.experiences.iter_mut() {
            if exp.successful_commands.contains(&cmd.to_string()) {
                exp.context.record_failure(cmd, &format!("{:?}", error_type));
            }
        }
        let _ = memory.save();
        debug!("Recorded command failure: {} ({:?})", cmd, error_type);
    }
}

/// Get alternative commands when the first one fails
pub async fn get_alternative_commands_smart(
    model: &str,
    original_cmd: &str,
    error_output: &str,
    question: &str,
    recovery_hint: &str,
) -> Option<Vec<String>> {
    let fast_timeout = get_perf_config().fast_llm_timeout_secs;

    let prompt = format!(
        r#"Command failed: `{}`
Error: {}
Question: "{}"

DIAGNOSIS: {}

Suggest 1-2 alternative commands for Arch Linux.
Reply with ONLY the commands, one per line. No explanation.
If no alternative exists, reply with "NONE"."#,
        original_cmd,
        if error_output.len() > 200 {
            &error_output[..200]
        } else {
            error_output
        },
        question,
        recovery_hint
    );

    match ollama::chat_with_timeout(model, &prompt, fast_timeout).await {
        Ok(response) => {
            let response = response.trim();
            if response == "NONE" || response.is_empty() {
                return None;
            }
            let alternatives: Vec<String> = response
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
                .map(|l| l.trim().to_string())
                .take(2)
                .collect();
            if alternatives.is_empty() {
                None
            } else {
                Some(alternatives)
            }
        }
        Err(e) => {
            debug!("Failed to get alternative commands: {}", e);
            None
        }
    }
}

/// Execute a command with retry logic
/// v0.0.919: Added auto-installation for missing commands
pub async fn execute_command_with_retry(
    model: &str,
    cmd: &str,
    question: &str,
    alternatives_budget: &mut u32,
) -> (String, Vec<String>) {
    let mut all_commands = vec![cmd.to_string()];

    info!("Executing command: {}", cmd);
    match execute_command(cmd) {
        Ok(output)
            if !output.trim().is_empty()
                && !output.contains("command not found")
                && !output.contains("No such file") =>
        {
            debug!("Command succeeded with {} bytes output", output.len());
            return (output, all_commands);
        }
        Ok(output) => {
            let (error_type, hint) = classify_command_error(&output, None);
            if error_type == CommandErrorType::Unknown && !output.trim().is_empty() {
                return (output, all_commands);
            }

            // v0.0.919: Try auto-installing missing command before asking LLM
            if error_type == CommandErrorType::CommandNotFound {
                if try_auto_install(cmd) {
                    // Retry the command after installation
                    if let Ok(retry_output) = execute_command(cmd) {
                        if !retry_output.trim().is_empty()
                            && !retry_output.contains("command not found")
                        {
                            info!("Command succeeded after auto-install");
                            return (retry_output, all_commands);
                        }
                    }
                }
            }
            record_command_failure(cmd, &error_type);

            if *alternatives_budget == 0 {
                return (output, all_commands);
            }
            *alternatives_budget = alternatives_budget.saturating_sub(1);

            let recovery_hint = get_recovery_prompt(&error_type, cmd);
            warn!("Command '{}' failed ({:?}): {}", cmd, error_type, hint);

            if let Some(alternatives) =
                get_alternative_commands_smart(model, cmd, &output, question, &recovery_hint).await
            {
                for alt_cmd in alternatives.iter() {
                    if is_dangerous_command(alt_cmd) {
                        continue;
                    }
                    all_commands.push(alt_cmd.clone());
                    if let Ok(alt_output) = execute_command(alt_cmd) {
                        if !alt_output.trim().is_empty()
                            && !alt_output.contains("command not found")
                        {
                            return (alt_output, all_commands);
                        }
                    }
                }
            }
            (output, all_commands)
        }
        Err(e) => {
            let error_msg = format!("Error: {}", e);
            let (error_type, _) = classify_command_error("", Some(&error_msg));
            record_command_failure(cmd, &error_type);

            if *alternatives_budget == 0 {
                return (error_msg, all_commands);
            }
            *alternatives_budget = alternatives_budget.saturating_sub(1);

            let recovery_hint = get_recovery_prompt(&error_type, cmd);
            if let Some(alternatives) =
                get_alternative_commands_smart(model, cmd, &error_msg, question, &recovery_hint)
                    .await
            {
                for alt_cmd in alternatives.iter() {
                    if is_dangerous_command(alt_cmd) {
                        continue;
                    }
                    all_commands.push(alt_cmd.clone());
                    if let Ok(alt_output) = execute_command(alt_cmd) {
                        if !alt_output.trim().is_empty() {
                            return (alt_output, all_commands);
                        }
                    }
                }
            }
            (error_msg, all_commands)
        }
    }
}
