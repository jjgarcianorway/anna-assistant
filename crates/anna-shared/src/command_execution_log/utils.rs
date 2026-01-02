//! Utility functions for command execution logging

use super::types::CommandRisk;

/// Extract command pattern (first word/command name)
pub fn extract_command_pattern(command: &str) -> String {
    let cmd = command.trim();
    // Handle sudo prefix
    let cmd = if cmd.starts_with("sudo ") {
        &cmd[5..]
    } else {
        cmd
    };

    // Get first word as pattern
    cmd.split_whitespace().next().unwrap_or("unknown").to_string()
}

/// Classify command risk level
pub fn classify_risk(command: &str) -> CommandRisk {
    let cmd = command.to_lowercase();

    // Critical commands - only exact root paths
    if (cmd.contains("rm -rf /") && !cmd.contains("rm -rf /tmp") && !cmd.contains("rm -rf /var") && !cmd.contains("rm -rf /home"))
        || cmd.contains("mkfs")
        || cmd.contains("dd if=")
        || cmd.contains("> /dev/")
    {
        return CommandRisk::Critical;
    }

    // High risk
    if cmd.starts_with("rm ")
        || cmd.contains("chmod")
        || cmd.contains("chown")
        || cmd.contains("systemctl stop")
        || cmd.contains("systemctl disable")
        || cmd.contains("kill ")
        || cmd.contains("pkill")
    {
        return CommandRisk::HighRisk;
    }

    // Medium risk
    if cmd.contains("pacman -S")
        || cmd.contains("apt install")
        || cmd.contains("dnf install")
        || cmd.contains("systemctl start")
        || cmd.contains("systemctl restart")
        || cmd.contains("pip install")
        || cmd.contains("npm install")
    {
        return CommandRisk::MediumRisk;
    }

    // Low risk
    if cmd.contains("echo ")
        || cmd.contains("printf")
        || cmd.contains("touch")
        || cmd.contains("mkdir")
    {
        return CommandRisk::LowRisk;
    }

    // Read-only
    if cmd.starts_with("cat ")
        || cmd.starts_with("ls")
        || cmd.starts_with("ps")
        || cmd.starts_with("df")
        || cmd.starts_with("du")
        || cmd.starts_with("free")
        || cmd.starts_with("top")
        || cmd.starts_with("htop")
        || cmd.starts_with("systemctl status")
        || cmd.starts_with("journalctl")
        || cmd.starts_with("uname")
        || cmd.starts_with("hostname")
        || cmd.starts_with("whoami")
        || cmd.starts_with("which")
        || cmd.starts_with("whereis")
        || cmd.starts_with("file ")
        || cmd.starts_with("head")
        || cmd.starts_with("tail")
        || cmd.starts_with("grep")
        || cmd.starts_with("find")
        || cmd.starts_with("locate")
    {
        return CommandRisk::ReadOnly;
    }

    CommandRisk::LowRisk
}
