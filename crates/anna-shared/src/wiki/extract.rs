//! Extract commands and instructions from wiki text.

use super::ExtractedCommand;
use regex::Regex;

/// Extract commands from wiki article text
pub fn extract_commands(text: &str, source_article: &str) -> Vec<ExtractedCommand> {
    let mut commands = Vec::new();

    // Pattern 1: Lines starting with $ or #
    let shell_pattern = Regex::new(r"(?m)^[\$#]\s+(.+)$").unwrap();
    for cap in shell_pattern.captures_iter(text) {
        let cmd = cap.get(1).unwrap().as_str().trim();
        if is_valid_command(cmd) {
            commands.push(ExtractedCommand {
                command: cmd.to_string(),
                description: find_description_for_command(text, cmd),
                requires_root: cap.get(0).unwrap().as_str().starts_with('#'),
                source_article: source_article.to_string(),
            });
        }
    }

    // Pattern 2: Code blocks (```bash or ``` followed by commands)
    let code_block_pattern = Regex::new(r"```(?:bash|sh|shell)?\n([\s\S]*?)```").unwrap();
    for cap in code_block_pattern.captures_iter(text) {
        let block = cap.get(1).unwrap().as_str();
        for line in block.lines() {
            let line = line.trim();
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') && !line.starts_with("# ") {
                continue;
            }
            // Handle lines with $ or # prefix
            let cmd = line.trim_start_matches(['$', '#', ' ']);
            if is_valid_command(cmd) && !commands.iter().any(|c| c.command == cmd) {
                commands.push(ExtractedCommand {
                    command: cmd.to_string(),
                    description: find_description_for_command(text, cmd),
                    requires_root: line.starts_with('#'),
                    source_article: source_article.to_string(),
                });
            }
        }
    }

    // Pattern 3: pacman commands
    let pacman_pattern = Regex::new(r"(?:sudo\s+)?pacman\s+-[A-Za-z]+\s+\S+").unwrap();
    for mat in pacman_pattern.find_iter(text) {
        let cmd = mat.as_str();
        if !commands.iter().any(|c| c.command.contains(cmd)) {
            commands.push(ExtractedCommand {
                command: cmd.to_string(),
                description: find_description_for_command(text, cmd),
                requires_root: cmd.contains("sudo") || cmd.contains("-S") || cmd.contains("-R"),
                source_article: source_article.to_string(),
            });
        }
    }

    // Pattern 4: systemctl commands
    let systemctl_pattern = Regex::new(r"(?:sudo\s+)?systemctl\s+\S+\s+\S+").unwrap();
    for mat in systemctl_pattern.find_iter(text) {
        let cmd = mat.as_str();
        if !commands.iter().any(|c| c.command.contains(cmd)) {
            commands.push(ExtractedCommand {
                command: cmd.to_string(),
                description: find_description_for_command(text, cmd),
                requires_root: cmd.contains("sudo") || cmd.contains("enable") || cmd.contains("start"),
                source_article: source_article.to_string(),
            });
        }
    }

    // Deduplicate and limit
    let mut seen = std::collections::HashSet::new();
    commands.retain(|c| seen.insert(c.command.clone()));
    commands.truncate(20); // Limit to 20 commands per article

    commands
}

/// Check if a string looks like a valid command
fn is_valid_command(cmd: &str) -> bool {
    // Must not be empty and not too long
    if cmd.is_empty() || cmd.len() > 200 {
        return false;
    }

    // Must start with a command-like word
    let first_word = cmd.split_whitespace().next().unwrap_or("");

    // Skip if it looks like prose
    if first_word.chars().next().map_or(false, |c| c.is_uppercase()) {
        return false;
    }

    // Skip common false positives
    let skip_words = ["the", "a", "an", "to", "and", "or", "if", "then", "else", "for", "while"];
    if skip_words.contains(&first_word.to_lowercase().as_str()) {
        return false;
    }

    // Should contain at least one command-like character
    cmd.contains('/') || cmd.contains('-') || cmd.contains('=') ||
        cmd.chars().any(|c| c.is_alphanumeric())
}

/// Find description text near a command
fn find_description_for_command(text: &str, cmd: &str) -> Option<String> {
    // Find the command in text
    if let Some(pos) = text.find(cmd) {
        // Look backwards for description (line before)
        let before = &text[..pos];
        if let Some(line_start) = before.rfind('\n') {
            let prev_line = before[..line_start].lines().last().unwrap_or("").trim();
            if !prev_line.is_empty() && !prev_line.starts_with(['$', '#', '`']) {
                return Some(prev_line.to_string());
            }
        }

        // Look forwards for description (after command on same line or next line)
        let after = &text[pos + cmd.len()..];
        if let Some(desc) = after.lines().next() {
            let desc = desc.trim().trim_start_matches(['-', ':', '#']);
            if !desc.is_empty() && desc.len() < 100 {
                return Some(desc.trim().to_string());
            }
        }
    }

    None
}

/// Extract commands relevant to a specific query
pub fn extract_relevant_commands(
    text: &str,
    query: &str,
    source_article: &str,
) -> Vec<ExtractedCommand> {
    let all_commands = extract_commands(text, source_article);

    // Filter commands relevant to query
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();

    let mut relevant: Vec<_> = all_commands
        .into_iter()
        .filter(|cmd| {
            let cmd_lower = cmd.command.to_lowercase();
            let desc_lower = cmd.description.as_ref().map(|d| d.to_lowercase()).unwrap_or_default();

            // Check if command or description matches query words
            query_words.iter().any(|word| {
                cmd_lower.contains(word) || desc_lower.contains(word)
            })
        })
        .collect();

    // If no relevant commands found, return top commands from article
    if relevant.is_empty() {
        relevant = extract_commands(text, source_article);
        relevant.truncate(5);
    }

    relevant
}

/// Common command patterns for specific query types
pub fn suggest_commands_for_query(query: &str) -> Vec<&'static str> {
    let query_lower = query.to_lowercase();

    // Package queries
    if query_lower.contains("install") {
        return vec!["pacman -S <package>", "pacman -Ss <search>"];
    }
    if query_lower.contains("remove") || query_lower.contains("uninstall") {
        return vec!["pacman -R <package>", "pacman -Rs <package>"];
    }
    if query_lower.contains("update") || query_lower.contains("upgrade") {
        return vec!["pacman -Syu"];
    }
    if query_lower.contains("search") && query_lower.contains("package") {
        return vec!["pacman -Ss <search>", "pacman -Qs <search>"];
    }

    // Service queries
    if query_lower.contains("service") || query_lower.contains("systemd") {
        return vec!["systemctl status <service>", "systemctl enable <service>", "systemctl start <service>"];
    }

    // System info queries
    if query_lower.contains("disk") || query_lower.contains("storage") {
        return vec!["df -h", "lsblk", "du -h --max-depth=1"];
    }
    if query_lower.contains("memory") || query_lower.contains("ram") {
        return vec!["free -h", "cat /proc/meminfo"];
    }
    if query_lower.contains("cpu") || query_lower.contains("processor") {
        return vec!["lscpu", "cat /proc/cpuinfo"];
    }
    if query_lower.contains("network") {
        return vec!["ip addr", "ip route", "ss -tuln"];
    }

    // Default - no suggestions
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_shell_commands() {
        let text = r#"
To install a package:
$ pacman -S vim

To remove a package:
# pacman -R vim
"#;
        let commands = extract_commands(text, "Test");
        assert!(commands.iter().any(|c| c.command.contains("pacman -S vim")));
        assert!(commands.iter().any(|c| c.command.contains("pacman -R vim")));
    }

    #[test]
    fn test_extract_code_blocks() {
        let text = r#"
```bash
sudo pacman -Syu
systemctl enable sshd
```
"#;
        let commands = extract_commands(text, "Test");
        assert!(commands.iter().any(|c| c.command.contains("pacman -Syu")));
    }
}
