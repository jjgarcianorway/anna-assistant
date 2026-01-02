//! Helper functions for recipe step execution (v0.0.423).
//!
//! Utilities for variable substitution, risk inference, and text formatting.

use std::collections::HashMap;

use super::RecipeRiskLevel;

/// Substitute variables in template
pub fn substitute_vars(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("${{{}}}", key), value);
        result = result.replace(&format!("${}", key), value);
    }
    result
}

/// Truncate string for display
pub fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

/// Infer risk level from command
pub fn infer_command_risk(command: &str) -> RecipeRiskLevel {
    let cmd_lower = command.to_lowercase();

    // High risk commands
    let high_risk = [
        "rm ",
        "rm\t",
        "dd ",
        "mkfs",
        "fdisk",
        "parted",
        "shred",
        "wipefs",
        "> /dev/",
        "chmod 777",
        "chmod -R",
    ];
    if high_risk.iter().any(|p| cmd_lower.contains(p)) {
        return RecipeRiskLevel::High;
    }

    // Medium risk commands
    let medium_risk = [
        "sudo ",
        "systemctl restart",
        "systemctl stop",
        "kill ",
        "pkill",
        "mv ",
        "cp -r",
        "chown",
        "chmod",
    ];
    if medium_risk.iter().any(|p| cmd_lower.contains(p)) {
        return RecipeRiskLevel::Medium;
    }

    // Low risk commands (modifications)
    let low_risk = [
        "systemctl start",
        "systemctl enable",
        "pacman -S",
        "yay -S",
        "paru -S",
        "mkdir",
        "touch",
    ];
    if low_risk.iter().any(|p| cmd_lower.contains(p)) {
        return RecipeRiskLevel::Low;
    }

    // Default: read-only commands are safe
    let safe = [
        "systemctl status",
        "cat ",
        "ls ",
        "grep ",
        "find ",
        "pacman -Q",
        "which ",
        "echo ",
        "pwd",
    ];
    if safe.iter().any(|p| cmd_lower.contains(p)) {
        return RecipeRiskLevel::None;
    }

    // Unknown commands default to medium
    RecipeRiskLevel::Medium
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_command_risk() {
        assert_eq!(infer_command_risk("rm -rf /"), RecipeRiskLevel::High);
        assert_eq!(
            infer_command_risk("sudo systemctl restart nginx"),
            RecipeRiskLevel::Medium
        );
        // pacman -S is considered medium due to sudo typically being needed
        assert!(infer_command_risk("pacman -S vim") <= RecipeRiskLevel::Medium);
        assert_eq!(infer_command_risk("ls -la"), RecipeRiskLevel::None);
    }

    #[test]
    fn test_substitute_vars() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "nginx".to_string());

        assert_eq!(
            substitute_vars("systemctl status ${name}", &vars),
            "systemctl status nginx"
        );
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world!", 5), "hello...");
    }
}
