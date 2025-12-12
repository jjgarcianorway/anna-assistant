//! Tool help extractor and indexer (v0.0.429).
//!
//! Extracts --help output from commands and indexes it.

use super::{DocSnippet, DocSourceKind, MAX_SNIPPET_SIZE};
use std::collections::HashSet;
use std::process::Command;
use std::time::Duration;

/// Extract help output from a command
pub fn extract_help(command: &str) -> Result<DocSnippet, HelpExtractError> {
    // Validate command
    if !is_safe_command(command) {
        return Err(HelpExtractError::UnsafeCommand(command.to_string()));
    }

    // Try --help first, then -h
    let output = try_help_flags(command)?;

    // Parse and create snippet
    let summary = extract_summary(&output);
    let content = clean_help_output(&output);

    if content.trim().is_empty() {
        return Err(HelpExtractError::NoContent(command.to_string()));
    }

    let mut snippet = DocSnippet::new(DocSourceKind::ToolHelp, command, None, &summary, &content);
    snippet.truncate_content(MAX_SNIPPET_SIZE);

    Ok(snippet)
}

/// Try help flags in order
fn try_help_flags(command: &str) -> Result<String, HelpExtractError> {
    let flags = ["--help", "-h", "-?", "help"];

    for flag in &flags {
        if let Ok(output) = run_with_timeout(command, flag, Duration::from_secs(5)) {
            if !output.trim().is_empty() && looks_like_help(&output) {
                return Ok(output);
            }
        }
    }

    Err(HelpExtractError::NoHelpAvailable(command.to_string()))
}

/// Run command with timeout
fn run_with_timeout(
    command: &str,
    flag: &str,
    timeout: Duration,
) -> Result<String, HelpExtractError> {
    let mut cmd = Command::new(command);
    cmd.arg(flag);

    // Set timeout via spawn + wait_timeout (simplified: just use normal wait)
    let output = cmd
        .output()
        .map_err(|e| HelpExtractError::CommandFailed(e.to_string()))?;

    // Help output often goes to stderr
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Return whichever has more content (help often goes to stderr)
    if stderr.len() > stdout.len() && looks_like_help(&stderr) {
        Ok(stderr.to_string())
    } else {
        Ok(stdout.to_string())
    }
}

/// Check if output looks like help text
fn looks_like_help(output: &str) -> bool {
    let lower = output.to_lowercase();

    // Must have some common help patterns
    let help_patterns = [
        "usage:",
        "options:",
        "commands:",
        "-h",
        "--help",
        "arguments:",
        "description:",
        "synopsis:",
    ];

    help_patterns.iter().any(|p| lower.contains(p))
}

/// Extract a summary from help output
fn extract_summary(output: &str) -> String {
    // Try to find a description line
    for line in output.lines().take(10) {
        let trimmed = line.trim();

        // Skip empty lines and usage lines
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.to_lowercase().starts_with("usage:") {
            continue;
        }

        // First non-usage line is likely the description
        if trimmed.len() > 10 && trimmed.len() < 200 {
            return if trimmed.len() > 100 {
                format!("{}...", &trimmed[..97])
            } else {
                trimmed.to_string()
            };
        }
    }

    // Fall back to command name
    output
        .lines()
        .next()
        .map(|l| l.trim().to_string())
        .unwrap_or_else(|| "Command help".to_string())
}

/// Clean up help output for storage
fn clean_help_output(output: &str) -> String {
    let mut result = String::new();
    let mut prev_empty = false;

    for line in output.lines() {
        let trimmed = line.trim_end();

        // Skip multiple empty lines
        if trimmed.is_empty() {
            if !prev_empty {
                result.push('\n');
                prev_empty = true;
            }
            continue;
        }

        prev_empty = false;
        result.push_str(trimmed);
        result.push('\n');
    }

    result.trim().to_string()
}

/// Check if command is safe to run
fn is_safe_command(command: &str) -> bool {
    // Must be in whitelist
    SAFE_COMMANDS.contains(&command)
}

/// Whitelist of safe commands
const SAFE_COMMANDS: &[&str] = &[
    // Systemd
    "systemctl",
    "journalctl",
    "loginctl",
    "timedatectl",
    "hostnamectl",
    "localectl",
    "networkctl",
    "bootctl",
    "coredumpctl",
    "busctl",
    "resolvectl",
    // Package management
    "pacman",
    "paru",
    "yay",
    "pikaur",
    "makepkg",
    "pkgfile",
    // Filesystem
    "df",
    "du",
    "lsblk",
    "blkid",
    "findmnt",
    "mount",
    "umount",
    "fstrim",
    "btrfs",
    "lvcreate",
    "vgdisplay",
    "pvdisplay",
    // Network
    "ip",
    "ss",
    "ping",
    "traceroute",
    "dig",
    "nslookup",
    "curl",
    "wget",
    "nmcli",
    "iw",
    "iwctl",
    // System info
    "free",
    "top",
    "htop",
    "ps",
    "pstree",
    "uname",
    "uptime",
    "lscpu",
    "lsmem",
    "lspci",
    "lsusb",
    "lsmod",
    "dmesg",
    "inxi",
    "neofetch",
    "fastfetch",
    // Files
    "ls",
    "find",
    "grep",
    "rg",
    "fd",
    "stat",
    "file",
    "tree",
    // Process
    "kill",
    "pkill",
    "killall",
    "nice",
    "renice",
    // Users
    "id",
    "who",
    "whoami",
    "groups",
    "passwd",
    "chsh",
    // Git
    "git",
    // Archive
    "tar",
    "gzip",
    "bzip2",
    "xz",
    "zip",
    "unzip",
    // Text
    "cat",
    "head",
    "tail",
    "less",
    "more",
    "wc",
    "sort",
    "uniq",
    "cut",
    "awk",
    "sed",
    // Performance
    "fio",
    "hdparm",
    "smartctl",
    "sensors",
    // Boot
    "mkinitcpio",
    "grub-mkconfig",
    "efibootmgr",
];

/// Get list of essential commands to index
pub fn get_essential_commands() -> Vec<&'static str> {
    vec![
        "systemctl",
        "journalctl",
        "pacman",
        "df",
        "du",
        "lsblk",
        "ip",
        "ss",
        "free",
        "ps",
        "fstrim",
        "mkinitcpio",
        "bootctl",
        "findmnt",
        "timedatectl",
    ]
}

/// Help extraction errors
#[derive(Debug, Clone)]
pub enum HelpExtractError {
    UnsafeCommand(String),
    CommandFailed(String),
    NoHelpAvailable(String),
    NoContent(String),
}

impl std::fmt::Display for HelpExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsafeCommand(c) => write!(f, "Unsafe command: {}", c),
            Self::CommandFailed(e) => write!(f, "Command failed: {}", e),
            Self::NoHelpAvailable(c) => write!(f, "No help available for: {}", c),
            Self::NoContent(c) => write!(f, "No content in help for: {}", c),
        }
    }
}

impl std::error::Error for HelpExtractError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_safe_command() {
        assert!(is_safe_command("systemctl"));
        assert!(is_safe_command("pacman"));
        assert!(is_safe_command("df"));

        assert!(!is_safe_command("rm"));
        assert!(!is_safe_command("dd"));
        assert!(!is_safe_command("mkfs"));
        assert!(!is_safe_command("unknown"));
    }

    #[test]
    fn test_looks_like_help() {
        assert!(looks_like_help("Usage: cmd [options]"));
        assert!(looks_like_help("Options:\n  -h  help"));
        assert!(looks_like_help("Commands:\n  start  Start service"));

        assert!(!looks_like_help("Hello world"));
        assert!(!looks_like_help("Error: file not found"));
    }

    #[test]
    fn test_extract_summary() {
        let help = "mycmd - A useful command\n\nUsage: mycmd [options]\n\nOptions:";
        assert_eq!(extract_summary(help), "mycmd - A useful command");

        let help = "Usage: foo\nThis is the description line\nMore stuff";
        assert_eq!(extract_summary(help), "This is the description line");
    }

    #[test]
    fn test_clean_help_output() {
        let input = "line 1\n\n\n\nline 2\n  trailing  \n";
        let output = clean_help_output(input);
        assert!(!output.contains("\n\n\n"));
        assert!(!output.ends_with(' '));
    }
}
