//! Inventory helper functions (v0.0.188).

use std::process::Command;

/// Check if a tool is installed using `command -v`
pub fn check_tool_installed(name: &str) -> Option<String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {}", name))
        .output()
        .ok()?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Some(path);
        }
    }
    None
}

/// Get current Unix timestamp
pub fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Run a command and return stdout (trimmed)
pub fn run_command(cmd: &str) -> Option<String> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }
    let output = Command::new(parts[0]).args(&parts[1..]).output().ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}
