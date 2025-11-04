//! Rollback command for Anna v1.2 "Rollback & Foresight"
//!
//! Safely undo autonomous actions with state restoration

use anyhow::{Context, Result};
use anna_common::{header, section, status, Level, TermCaps};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::apply_cmd::{RollbackToken, StateSnapshot};
use crate::audit_log::{AuditEntry, AuditLog};

/// Rollback mode
pub enum RollbackMode {
    Last,              // Undo last action
    Specific(String),  // Undo specific action by ID
    List,              // Show rollback history
}

/// Run rollback command
pub fn run_rollback(mode: RollbackMode, dry_run: bool) -> Result<()> {
    let caps = TermCaps::detect();

    println!("{}", header(&caps, "Rollback Actions"));
    println!();

    match mode {
        RollbackMode::Last => rollback_last(dry_run)?,
        RollbackMode::Specific(id) => rollback_specific(&id, dry_run)?,
        RollbackMode::List => show_rollback_history()?,
    }

    Ok(())
}

/// Rollback the last applied action
fn rollback_last(dry_run: bool) -> Result<()> {
    let caps = TermCaps::detect();
    let tokens = load_rollback_tokens()?;

    if tokens.is_empty() {
        println!("{}", status(&caps, Level::Warn, "No actions to rollback"));
        println!();
        println!("  No rollback tokens found. Try running 'annactl apply' first.");
        return Ok(());
    }

    // Get the most recent successful action
    let last_token = tokens.iter()
        .filter(|t| t.success)
        .last()
        .context("No successful actions found in rollback history")?;

    println!("{}", section(&caps, "Last Applied Action"));
    println!();
    print_token_details(last_token);

    if dry_run {
        println!("  [DRY RUN] Would rollback this action");
        return Ok(());
    }

    // Confirm with user
    if !confirm_rollback(last_token)? {
        println!("Rollback cancelled");
        return Ok(());
    }

    // Execute rollback
    perform_rollback(last_token)?;

    Ok(())
}

/// Rollback specific action by ID
fn rollback_specific(advice_id: &str, dry_run: bool) -> Result<()> {
    let caps = TermCaps::detect();
    let tokens = load_rollback_tokens()?;

    // Find the token for this advice ID
    let token = tokens.iter()
        .filter(|t| t.advice_id == advice_id && t.success)
        .last()
        .ok_or_else(|| anyhow::anyhow!("No rollback token found for: {}", advice_id))?;

    println!("{}", section(&caps, "Target Action"));
    println!();
    print_token_details(token);

    if dry_run {
        println!("  [DRY RUN] Would rollback this action");
        return Ok(());
    }

    // Confirm with user
    if !confirm_rollback(token)? {
        println!("Rollback cancelled");
        return Ok(());
    }

    // Execute rollback
    perform_rollback(token)?;

    Ok(())
}

/// Show rollback history
fn show_rollback_history() -> Result<()> {
    let caps = TermCaps::detect();
    let tokens = load_rollback_tokens()?;

    if tokens.is_empty() {
        println!("{}", status(&caps, Level::Warn, "No rollback history"));
        println!();
        println!("  No rollback tokens found. Try running 'annactl apply' first.");
        return Ok(());
    }

    println!("{}", section(&caps, "Rollback History"));
    println!();

    // Group by advice ID and show only the most recent for each
    let mut seen_ids = std::collections::HashSet::new();
    let mut unique_tokens = Vec::new();

    for token in tokens.iter().rev() {
        if seen_ids.insert(&token.advice_id) {
            unique_tokens.push(token);
        }
    }

    println!("Found {} unique actions:", unique_tokens.len());
    println!();

    for (i, token) in unique_tokens.iter().enumerate() {
        let timestamp = format_timestamp(token.executed_at);
        let status_icon = if token.success { "✓" } else { "✗" };

        println!("{}. {} {} — {}",
            i + 1,
            status_icon,
            token.advice_id,
            timestamp
        );

        if !token.success {
            println!("   (Failed - cannot rollback)");
        }
    }

    println!();
    println!("Use 'annactl rollback --last' to undo the most recent action");
    println!("Use 'annactl rollback --id <advice_id>' to undo a specific action");

    Ok(())
}

/// Print token details
fn print_token_details(token: &RollbackToken) {
    let timestamp = format_timestamp(token.executed_at);

    println!("  Advice ID:  {}", token.advice_id);
    println!("  Executed:   {}", timestamp);
    println!("  Command:    {}", token.command);
    println!("  Status:     {}", if token.success { "Success" } else { "Failed" });

    if let Some(snapshot) = &token.state_snapshot {
        println!("  Snapshot:   {} packages tracked", snapshot.packages_before.len());
    }

    println!();
}

/// Confirm rollback with user
fn confirm_rollback(token: &RollbackToken) -> Result<bool> {
    println!("⚠️  Rollback Confirmation");
    println!();
    println!("  This will attempt to undo the following action:");
    println!("  • {}", token.advice_id);
    println!("  • Original command: {}", token.command);
    println!();
    println!("  Rollback strategy:");

    // Determine rollback strategy based on command
    let strategy = determine_rollback_strategy(&token.command);
    println!("  • {}", strategy);
    println!();

    print!("Proceed with rollback? [y/N]: ");
    use std::io::Write;
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_lowercase() == "y")
}

/// Perform the actual rollback
fn perform_rollback(token: &RollbackToken) -> Result<()> {
    let caps = TermCaps::detect();

    println!("{}", section(&caps, "Executing Rollback"));
    println!();

    // Determine rollback command based on original action
    let rollback_cmd = create_rollback_command(&token.command, &token.state_snapshot)?;

    println!("  Rollback command: {}", rollback_cmd);
    println!();

    // Execute rollback
    let output = Command::new("sh")
        .arg("-c")
        .arg(&rollback_cmd)
        .output()
        .context("Failed to execute rollback command")?;

    let success = output.status.success();
    let output_str = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr).to_string();

    if success {
        println!("{}", status(&caps, Level::Ok, "Rollback successful"));

        // Log to audit
        let audit = AuditLog::new()?;
        let entry = AuditEntry::new("user", &format!("Rollback: {}", token.advice_id), "rollback")
            .with_action_id(format!("{}_rollback", token.advice_id))
            .with_result("success".to_string())
            .with_details(format!("Original: {} | Rollback: {}", token.command, rollback_cmd));

        audit.log(entry)?;

        // Remove the rollback token
        remove_rollback_token(&token.advice_id)?;
    } else {
        println!("{}", status(&caps, Level::Err, "Rollback failed"));
        println!();
        println!("  Output:");
        for line in output_str.lines().take(10) {
            println!("    {}", line);
        }
        println!();
        println!("  The original action may need manual rollback.");
        println!("  Consult the Arch Wiki for guidance.");
    }

    Ok(())
}

/// Create rollback command based on original command
fn create_rollback_command(original: &str, snapshot: &Option<StateSnapshot>) -> Result<String> {
    // Heuristic-based rollback command generation

    // Package installations
    if original.contains("pacman -S") && !original.contains("-Sy") {
        // Extract package names
        let packages = extract_packages_from_pacman(original);
        if !packages.is_empty() {
            return Ok(format!("sudo pacman -Rns {}", packages.join(" ")));
        }
    }

    // Package database updates (cannot be rolled back)
    if original.contains("pacman -Sy") {
        return Err(anyhow::anyhow!(
            "Package database updates cannot be rolled back automatically"
        ));
    }

    // Bootloader updates
    if original.contains("grub-mkconfig") {
        return Err(anyhow::anyhow!(
            "Bootloader updates require manual rollback (restore previous grub.cfg)"
        ));
    }

    // Cache cleanups (cannot be rolled back)
    if original.contains("pacman -Sc") || original.contains("rm -rf") {
        return Err(anyhow::anyhow!(
            "File deletions cannot be rolled back (data was removed)"
        ));
    }

    // Service restarts
    if original.contains("systemctl restart") {
        let service = original.split_whitespace().last().unwrap_or("");
        return Ok(format!("systemctl restart {}", service));
    }

    // Generic: cannot determine rollback
    Err(anyhow::anyhow!(
        "Cannot automatically determine rollback command for: {}",
        original
    ))
}

/// Determine rollback strategy description
fn determine_rollback_strategy(command: &str) -> String {
    if command.contains("pacman -S") && !command.contains("-Sy") {
        "Remove installed packages".to_string()
    } else if command.contains("pacman -Sy") {
        "⚠️  Database updates cannot be rolled back".to_string()
    } else if command.contains("grub-mkconfig") {
        "⚠️  Manual bootloader rollback required".to_string()
    } else if command.contains("pacman -Sc") || command.contains("rm -rf") {
        "⚠️  File deletions cannot be undone".to_string()
    } else if command.contains("systemctl") {
        "Restart service to previous state".to_string()
    } else {
        "⚠️  Manual rollback may be required".to_string()
    }
}

/// Extract package names from pacman command
fn extract_packages_from_pacman(command: &str) -> Vec<String> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    let mut packages = Vec::new();
    let mut found_s = false;

    for part in parts {
        if part == "-S" {
            found_s = true;
            continue;
        }
        if found_s && !part.starts_with('-') && part != "sudo" && part != "pacman" {
            packages.push(part.to_string());
        }
    }

    packages
}

/// Load rollback tokens from disk
fn load_rollback_tokens() -> Result<Vec<RollbackToken>> {
    let state_dir = get_state_dir()?;
    let token_file = state_dir.join("rollback_tokens.jsonl");

    if !token_file.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&token_file)?;
    let mut tokens = Vec::new();

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<RollbackToken>(line) {
            Ok(token) => tokens.push(token),
            Err(e) => {
                eprintln!("Warning: Failed to parse rollback token: {}", e);
                continue;
            }
        }
    }

    Ok(tokens)
}

/// Remove rollback token after successful rollback
fn remove_rollback_token(advice_id: &str) -> Result<()> {
    let state_dir = get_state_dir()?;
    let token_file = state_dir.join("rollback_tokens.jsonl");

    if !token_file.exists() {
        return Ok(());
    }

    let tokens = load_rollback_tokens()?;
    let filtered: Vec<RollbackToken> = tokens.into_iter()
        .filter(|t| t.advice_id != advice_id)
        .collect();

    // Rewrite file without the removed token
    let mut content = String::new();
    for token in filtered {
        let json = serde_json::to_string(&token)?;
        content.push_str(&json);
        content.push('\n');
    }

    fs::write(&token_file, content)?;

    Ok(())
}

/// Format Unix timestamp to human-readable date
fn format_timestamp(unix_time: u64) -> String {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    let duration = Duration::from_secs(unix_time);
    let datetime = UNIX_EPOCH + duration;
    let elapsed = SystemTime::now()
        .duration_since(datetime)
        .unwrap_or_default();

    let secs = elapsed.as_secs();
    if secs < 60 {
        format!("{} seconds ago", secs)
    } else if secs < 3600 {
        format!("{} minutes ago", secs / 60)
    } else if secs < 86400 {
        format!("{} hours ago", secs / 3600)
    } else {
        format!("{} days ago", secs / 86400)
    }
}

/// Get state directory
fn get_state_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME not set")?;
    Ok(PathBuf::from(home).join(".local/state/anna"))
}
