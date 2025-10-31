//! Autonomy System - Privilege and Risk Management
//!
//! Controls Anna's autonomy level, determining what actions she can take
//! without explicit user approval.

use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

const AUTONOMY_CONF: &str = "/etc/anna/autonomy.conf";
const AUTONOMY_LOG: &str = "/var/log/anna/autonomy.log";

#[derive(Debug, Clone, PartialEq)]
pub enum AutonomyLevel {
    Low,
    High,
}

impl AutonomyLevel {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "low" => Some(AutonomyLevel::Low),
            "high" => Some(AutonomyLevel::High),
            _ => None,
        }
    }

    fn as_str(&self) -> &str {
        match self {
            AutonomyLevel::Low => "low",
            AutonomyLevel::High => "high",
        }
    }

    fn description(&self) -> &str {
        match self {
            AutonomyLevel::Low => "Low-risk autonomy: self-repair, permission fixes, service restarts",
            AutonomyLevel::High => "High-risk autonomy: package installation, config updates, policy changes",
        }
    }

    fn capabilities(&self) -> Vec<&str> {
        match self {
            AutonomyLevel::Low => vec![
                "✓ Fix directory permissions",
                "✓ Restart annad service",
                "✓ Repair socket ownership",
                "✓ Reload policy files",
                "✓ Clear event history",
                "✓ Create/restore backups",
                "✗ Install packages (requires High)",
                "✗ Modify system configs (requires High)",
                "✗ Update policies automatically (requires High)",
            ],
            AutonomyLevel::High => vec![
                "✓ All Low-risk capabilities",
                "✓ Install missing dependencies",
                "✓ Modify system configuration files",
                "✓ Update polkit policies",
                "✓ Change autonomy level",
                "⚠ All actions logged to audit.log",
            ],
        }
    }
}

/// Get current autonomy level
pub async fn autonomy_get() -> Result<()> {
    let level = read_autonomy_level()?;
    let config = read_autonomy_config()?;

    println!("\n🔐 Anna Autonomy Status\n");
    println!("Current Level: {}", level.as_str().to_uppercase());
    println!("Description:   {}", level.description());
    println!();
    println!("Capabilities:");
    for cap in level.capabilities() {
        println!("  {}", cap);
    }
    println!();

    if let Some(changed_by) = config.get("changed_by") {
        if let Some(last_changed) = config.get("last_changed") {
            println!("Last changed:  {} by {}", last_changed, changed_by);
        }
    }

    println!();
    Ok(())
}

/// Set autonomy level with confirmation
pub async fn autonomy_set(level: &str, skip_confirm: bool) -> Result<()> {
    let new_level = AutonomyLevel::from_str(level)
        .ok_or_else(|| anyhow::anyhow!("Invalid autonomy level: {}", level))?;

    let current_level = read_autonomy_level()?;

    if current_level == new_level {
        println!("Autonomy level already set to: {}", level);
        return Ok(());
    }

    // Show what's changing
    println!("\n⚠️  Changing Autonomy Level\n");
    println!("Current: {} → New: {}", current_level.as_str(), new_level.as_str());
    println!();
    println!("{}", new_level.description());
    println!();

    if new_level == AutonomyLevel::High {
        println!("⚠️  HIGH-RISK AUTONOMY WARNING ⚠️\n");
        println!("This allows Anna to:");
        println!("  • Install system packages automatically");
        println!("  • Modify configuration files");
        println!("  • Update security policies");
        println!();
    }

    // Confirmation
    if !skip_confirm {
        print!("Do you want to continue? [y/N]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Autonomy level change cancelled.");
            return Ok(());
        }
    }

    // Write new level
    write_autonomy_level(&new_level)?;

    println!("\n✓ Autonomy level changed to: {}", new_level.as_str());
    println!();

    Ok(())
}

// Helper functions

fn read_autonomy_level() -> Result<AutonomyLevel> {
    if !Path::new(AUTONOMY_CONF).exists() {
        // Default to low if file doesn't exist
        return Ok(AutonomyLevel::Low);
    }

    let contents = fs::read_to_string(AUTONOMY_CONF)
        .context("Failed to read autonomy config")?;

    for line in contents.lines() {
        if line.starts_with("autonomy_level=") {
            let level_str = line.trim_start_matches("autonomy_level=").trim();
            return AutonomyLevel::from_str(level_str)
                .ok_or_else(|| anyhow::anyhow!("Invalid autonomy level in config: {}", level_str));
        }
    }

    // Default to low if not specified
    Ok(AutonomyLevel::Low)
}

fn read_autonomy_config() -> Result<std::collections::HashMap<String, String>> {
    use std::collections::HashMap;

    let mut config = HashMap::new();

    if !Path::new(AUTONOMY_CONF).exists() {
        return Ok(config);
    }

    let contents = fs::read_to_string(AUTONOMY_CONF)
        .context("Failed to read autonomy config")?;

    for line in contents.lines() {
        if let Some(pos) = line.find('=') {
            let key = line[..pos].trim().to_string();
            let value = line[pos + 1..].trim().to_string();
            config.insert(key, value);
        }
    }

    Ok(config)
}

fn write_autonomy_level(level: &AutonomyLevel) -> Result<()> {
    let username = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
    let timestamp = chrono::Local::now().to_rfc3339();

    let content = format!(
        "autonomy_level={}\nlast_changed={}\nchanged_by={}\n",
        level.as_str(),
        timestamp,
        username
    );

    // Write config file (requires sudo)
    let temp_file = "/tmp/anna-autonomy.conf";
    fs::write(temp_file, &content)?;

    let status = Command::new("sudo")
        .args(&["cp", temp_file, AUTONOMY_CONF])
        .status()
        .context("Failed to write autonomy config (need sudo)")?;

    if !status.success() {
        anyhow::bail!("Failed to write autonomy config");
    }

    // Set permissions
    Command::new("sudo")
        .args(&["chown", "root:anna", AUTONOMY_CONF])
        .status()?;

    Command::new("sudo")
        .args(&["chmod", "0644", AUTONOMY_CONF])
        .status()?;

    // Log the change
    log_autonomy_change(level, &username)?;

    // Clean up temp file
    let _ = fs::remove_file(temp_file);

    Ok(())
}

fn log_autonomy_change(level: &AutonomyLevel, username: &str) -> Result<()> {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let log_entry = format!(
        "[{}] [ESCALATED] Autonomy level changed to '{}' by user {}\n",
        timestamp,
        level.as_str(),
        username
    );

    // Append to log file (requires sudo)
    let temp_file = "/tmp/anna-autonomy-log.txt";
    fs::write(temp_file, &log_entry)?;

    let _ = Command::new("sudo")
        .args(&["sh", "-c", &format!("cat {} >> {}", temp_file, AUTONOMY_LOG)])
        .status();

    let _ = fs::remove_file(temp_file);

    Ok(())
}
