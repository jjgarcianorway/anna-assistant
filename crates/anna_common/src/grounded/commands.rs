//! Command Module v6.0 - Grounded in PATH
//!
//! Source of truth: $PATH directories + which + man
//! No invented data. No hallucinations.

use std::collections::HashSet;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

/// A command (binary) on the system
#[derive(Debug, Clone)]
pub struct SystemCommand {
    pub name: String,
    pub path: String,
    pub description: String,
    pub owning_package: Option<String>,
}

/// Command counts - all from real PATH scanning
#[derive(Debug, Clone, Default)]
pub struct CommandCounts {
    pub total: usize,
    pub with_description: usize,
    pub with_package: usize,
}

/// Get the total count of unique executables in PATH
/// Source: ls each directory in $PATH
pub fn count_path_executables() -> usize {
    let path_var = std::env::var("PATH").unwrap_or_default();
    let mut seen: HashSet<String> = HashSet::new();

    for dir in path_var.split(':') {
        if dir.is_empty() {
            continue;
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if is_executable(&path) {
                    if let Some(name) = path.file_name() {
                        seen.insert(name.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    seen.len()
}

/// Check if a file is executable
fn is_executable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    match fs::metadata(path) {
        Ok(meta) => {
            let mode = meta.permissions().mode();
            // Check if any execute bit is set
            mode & 0o111 != 0
        }
        Err(_) => false,
    }
}

/// Get command info
/// Sources: which, man -f, --help
pub fn get_command_info(name: &str) -> Option<SystemCommand> {
    // Get path from which
    let path = get_command_path(name)?;

    // Get description from man or --help
    let description = get_command_description(name);

    // Get owning package
    let owning_package = super::packages::get_owning_package(&path);

    Some(SystemCommand {
        name: name.to_string(),
        path,
        description,
        owning_package,
    })
}

/// Get command path using which
/// Source: which <cmd>
pub fn get_command_path(name: &str) -> Option<String> {
    let output = Command::new("which").arg(name).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if path.is_empty() {
        None
    } else {
        Some(path)
    }
}

/// Get command description
/// Sources (in order): man -f, --help first line
pub fn get_command_description(name: &str) -> String {
    // Try man -f first (whatis)
    if let Some(desc) = get_man_description(name) {
        return desc;
    }

    // Try --help
    if let Some(desc) = get_help_description(name) {
        return desc;
    }

    // No description available
    String::new()
}

/// Get description from man -f (whatis)
/// Source: man -f <cmd>
fn get_man_description(name: &str) -> Option<String> {
    let output = Command::new("man").args(["-f", name]).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Format: "vim (1)              - Vi IMproved, a programmer's text editor"
    for line in stdout.lines() {
        if let Some(pos) = line.find(" - ") {
            let desc = line[pos + 3..].trim();
            if !desc.is_empty() {
                return Some(format!("{} (source: man -f)", desc));
            }
        }
    }

    None
}

/// Get description from --help
/// Source: <cmd> --help (first non-empty line or usage line)
fn get_help_description(name: &str) -> Option<String> {
    // Run with timeout to avoid hanging
    let output = Command::new("timeout")
        .args(["2", name, "--help"])
        .output()
        .ok()?;

    // Some commands return non-zero for --help but still give output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let text = if stdout.len() > stderr.len() {
        stdout
    } else {
        stderr
    };

    // Get first meaningful line
    for line in text.lines().take(5) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Skip usage lines, get description
        if line.to_lowercase().starts_with("usage:") {
            continue;
        }
        // Found something
        let truncated = if line.len() > 80 {
            format!("{}...", &line[..77])
        } else {
            line.to_string()
        };
        return Some(format!("{} (source: --help)", truncated));
    }

    None
}

/// List all commands in PATH
/// Source: ls each directory in $PATH
pub fn list_path_commands() -> Vec<String> {
    let path_var = std::env::var("PATH").unwrap_or_default();
    let mut commands: HashSet<String> = HashSet::new();

    for dir in path_var.split(':') {
        if dir.is_empty() {
            continue;
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if is_executable(&path) {
                    if let Some(name) = path.file_name() {
                        commands.insert(name.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    let mut result: Vec<String> = commands.into_iter().collect();
    result.sort();
    result
}

/// Check if a command exists
/// Source: which
pub fn command_exists(name: &str) -> bool {
    get_command_path(name).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_path_executables() {
        let count = count_path_executables();
        // Any Linux system should have many commands
        assert!(count > 100);
    }

    #[test]
    fn test_command_exists() {
        // These should exist on any Linux system
        assert!(command_exists("ls"));
        assert!(command_exists("cat"));
        // This shouldn't exist
        assert!(!command_exists("nonexistent_command_xyz"));
    }

    #[test]
    fn test_get_command_path() {
        let path = get_command_path("ls");
        assert!(path.is_some());
        assert!(path.unwrap().contains("ls"));
    }
}
