//! Tool Manager - Auto-install missing diagnostic tools.
//!
//! Anna can install missing tools to improve diagnostic capabilities.
//! Tracks installed tools in /var/lib/anna/installed_deps.txt for cleanup.

use anyhow::{Context, Result};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info, warn};

/// Path to track Anna-installed tools
fn installed_tools_path() -> PathBuf {
    PathBuf::from("/var/lib/anna/installed_deps.txt")
}

/// Common diagnostic tools Anna might need
pub const DIAGNOSTIC_TOOLS: &[(&str, &str)] = &[
    ("bc", "Calculator for math operations"),
    ("jq", "JSON parsing and manipulation"),
    ("htop", "Interactive process viewer"),
    ("lsof", "List open files"),
    ("nethogs", "Network bandwidth per process"),
    ("iotop", "Disk I/O per process"),
    ("ncdu", "Disk usage analyzer (better than du)"),
    ("tree", "Directory tree viewer"),
    ("strace", "System call tracer"),
    ("tcpdump", "Network packet capture"),
    ("bandwhich", "Network utilization by process"),
    ("duf", "Better disk usage (df replacement)"),
    ("btop", "Better htop alternative"),
];

/// Check if a tool is available in PATH
pub fn is_tool_available(tool_name: &str) -> bool {
    Command::new("which")
        .arg(tool_name)
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

/// Get tool description if it's in the diagnostic tools list
pub fn get_tool_description(tool_name: &str) -> Option<&'static str> {
    DIAGNOSTIC_TOOLS
        .iter()
        .find(|(name, _)| *name == tool_name)
        .map(|(_, desc)| *desc)
}

/// Install a tool using pacman (Arch Linux)
/// Returns Ok(true) if installed, Ok(false) if cancelled, Err on failure
pub fn install_tool(tool_name: &str) -> Result<bool> {
    info!("Installing tool: {}", tool_name);

    // Try pacman first (official repos)
    let install_cmd = format!("pkexec pacman -S --noconfirm {}", tool_name);

    debug!("Running: {}", install_cmd);
    let output = Command::new("sh")
        .arg("-c")
        .arg(&install_cmd)
        .output()
        .context("Failed to execute install command")?;

    if output.status.success() {
        info!("Successfully installed {}", tool_name);
        track_installed_tool(tool_name)?;
        Ok(true)
    } else {
        // If pacman fails, try yay (AUR)
        if is_tool_available("yay") {
            warn!("pacman failed, trying yay for {}", tool_name);
            let yay_cmd = format!("yay -S --noconfirm {}", tool_name);
            let yay_output = Command::new("sh")
                .arg("-c")
                .arg(&yay_cmd)
                .output()
                .context("Failed to execute yay")?;

            if yay_output.status.success() {
                info!("Successfully installed {} from AUR", tool_name);
                track_installed_tool(tool_name)?;
                Ok(true)
            } else {
                let stderr = String::from_utf8_lossy(&yay_output.stderr);
                warn!("Failed to install {}: {}", tool_name, stderr);
                Ok(false)
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to install {}: {}", tool_name, stderr);
            Ok(false)
        }
    }
}

/// Track an installed tool in /var/lib/anna/installed_deps.txt
fn track_installed_tool(tool_name: &str) -> Result<()> {
    let path = installed_tools_path();

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Append tool with timestamp
    let timestamp = chrono::Utc::now().to_rfc3339();
    let entry = format!("{} ({})\n", tool_name, timestamp);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .context("Failed to open installed_deps.txt")?;

    file.write_all(entry.as_bytes())
        .context("Failed to write to installed_deps.txt")?;

    debug!("Tracked installed tool: {}", tool_name);
    Ok(())
}

/// List all tools Anna has installed
pub fn list_installed_tools() -> Result<Vec<String>> {
    let path = installed_tools_path();

    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(&path).context("Failed to open installed_deps.txt")?;
    let reader = BufReader::new(file);

    let mut tools = Vec::new();
    for line in reader.lines() {
        let line = line?;
        // Parse "tool_name (timestamp)"
        if let Some(tool_name) = line.split_whitespace().next() {
            tools.push(tool_name.to_string());
        }
    }

    Ok(tools)
}

/// Check if a tool was installed by Anna
pub fn is_anna_installed(tool_name: &str) -> Result<bool> {
    let tools = list_installed_tools()?;
    Ok(tools.contains(&tool_name.to_string()))
}

/// Remove all Anna-installed tools (used during uninstall)
pub fn remove_all_anna_tools() -> Result<Vec<String>> {
    let tools = list_installed_tools()?;

    if tools.is_empty() {
        info!("No Anna-installed tools to remove");
        return Ok(Vec::new());
    }

    info!("Removing {} Anna-installed tools", tools.len());
    let mut removed = Vec::new();

    for tool in &tools {
        info!("Removing tool: {}", tool);

        let remove_cmd = format!("pkexec pacman -Rns --noconfirm {}", tool);
        match Command::new("sh").arg("-c").arg(&remove_cmd).output() {
            Ok(output) if output.status.success() => {
                info!("Removed {}", tool);
                removed.push(tool.clone());
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to remove {}: {}", tool, stderr);
            }
            Err(e) => {
                warn!("Error removing {}: {}", tool, e);
            }
        }
    }

    // Clear the tracking file
    if !removed.is_empty() {
        let path = installed_tools_path();
        if let Err(e) = std::fs::remove_file(&path) {
            warn!("Failed to remove installed_deps.txt: {}", e);
        } else {
            info!("Cleared installed_deps.txt");
        }
    }

    Ok(removed)
}

/// Check if a tool is needed and prompt to install if missing
pub fn ensure_tool_available(tool_name: &str) -> Result<bool> {
    if is_tool_available(tool_name) {
        debug!("Tool {} already available", tool_name);
        return Ok(true);
    }

    info!("Tool {} not found", tool_name);

    // For now, auto-install without prompting (can add prompt later)
    // In production, this should ask via Telegram/CLI
    install_tool(tool_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_tool_available_sh() {
        // sh should always be available
        assert!(is_tool_available("sh"));
    }

    #[test]
    fn test_is_tool_available_nonexistent() {
        assert!(!is_tool_available("this_tool_definitely_does_not_exist_12345"));
    }

    #[test]
    fn test_get_tool_description() {
        assert_eq!(get_tool_description("jq"), Some("JSON parsing and manipulation"));
        assert_eq!(get_tool_description("nonexistent"), None);
    }
}
