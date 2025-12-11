//! Knowledge source fetchers (v0.0.422).
//!
//! Fetches content from:
//! - Man pages (local)
//! - Help output (--help, -h)
//! - Arch Wiki (cache or online)
//! - Local documentation

use std::process::Command;
use std::time::Duration;

use super::FETCH_TIMEOUT_MS;

/// Result of a fetch operation
#[derive(Debug, Clone)]
pub struct SourceFetchResult {
    /// Fetched content
    pub content: String,
    /// Whether content was from cache
    pub from_cache: bool,
    /// Source path or URL
    pub source_path: String,
}

impl SourceFetchResult {
    /// Create new result
    pub fn new(content: String, source_path: &str) -> Self {
        Self {
            content,
            from_cache: false,
            source_path: source_path.to_string(),
        }
    }

    /// Mark as from cache
    pub fn cached(mut self) -> Self {
        self.from_cache = true;
        self
    }

    /// Check if fetch was successful
    pub fn is_ok(&self) -> bool {
        !self.content.is_empty()
    }
}

/// Fetch man page for a command
pub fn fetch_man_page(command: &str) -> Option<SourceFetchResult> {
    // Validate command name (alphanumeric, dash, underscore only)
    if !is_safe_name(command) {
        return None;
    }

    // Check if man page exists
    let check = Command::new("man")
        .args(["-w", command])
        .output()
        .ok()?;

    if !check.status.success() {
        return None;
    }

    let man_path = String::from_utf8_lossy(&check.stdout).trim().to_string();

    // Fetch man page content
    let output = Command::new("man")
        .args(["-P", "cat", command])
        .env("MANWIDTH", "80")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let content = String::from_utf8_lossy(&output.stdout);

    // Extract relevant sections (NAME, SYNOPSIS, DESCRIPTION, up to 100 lines)
    let extracted = extract_man_sections(&content);

    if extracted.is_empty() {
        return None;
    }

    Some(SourceFetchResult::new(extracted, &man_path))
}

/// Extract key sections from man page
fn extract_man_sections(content: &str) -> String {
    let mut result = String::new();
    let mut in_section = false;
    let mut current_section = String::new();
    let mut lines_collected = 0;
    const MAX_LINES: usize = 100;

    let important_sections = ["NAME", "SYNOPSIS", "DESCRIPTION", "OPTIONS", "COMMANDS"];

    for line in content.lines() {
        if lines_collected >= MAX_LINES {
            break;
        }

        // Check for section header (all caps, at start of line)
        let trimmed = line.trim();
        if !trimmed.is_empty()
            && trimmed.chars().all(|c| c.is_ascii_uppercase() || c.is_whitespace())
            && trimmed.len() < 30
        {
            current_section = trimmed.to_string();
            in_section = important_sections.contains(&current_section.as_str());

            if in_section {
                result.push_str(&format!("\n{}\n", current_section));
                lines_collected += 1;
            }
            continue;
        }

        if in_section {
            result.push_str(line);
            result.push('\n');
            lines_collected += 1;
        }
    }

    result.trim().to_string()
}

/// Fetch help output for a command
pub fn fetch_help_output(command: &str) -> Option<SourceFetchResult> {
    // Validate command name
    if !is_safe_name(command) {
        return None;
    }

    // Check against dangerous commands
    if is_dangerous_command(command) {
        return None;
    }

    // Try --help first
    let output = Command::new(command)
        .arg("--help")
        .output();

    if let Ok(out) = output {
        let content = if out.status.success() {
            String::from_utf8_lossy(&out.stdout).to_string()
        } else {
            // Some commands output help to stderr
            String::from_utf8_lossy(&out.stderr).to_string()
        };

        if looks_like_help(&content) {
            let truncated = truncate_help(&content, 50);
            return Some(SourceFetchResult::new(truncated, &format!("{} --help", command)));
        }
    }

    // Try -h as fallback
    let output = Command::new(command)
        .arg("-h")
        .output();

    if let Ok(out) = output {
        let content = if out.status.success() {
            String::from_utf8_lossy(&out.stdout).to_string()
        } else {
            String::from_utf8_lossy(&out.stderr).to_string()
        };

        if looks_like_help(&content) {
            let truncated = truncate_help(&content, 50);
            return Some(SourceFetchResult::new(truncated, &format!("{} -h", command)));
        }
    }

    None
}

/// Check if output looks like help text
fn looks_like_help(content: &str) -> bool {
    let lower = content.to_lowercase();
    let has_help_indicators = lower.contains("usage")
        || lower.contains("options")
        || lower.contains("--help")
        || lower.contains("commands")
        || lower.contains("arguments");

    let not_error = !lower.contains("command not found")
        && !lower.contains("no such file")
        && !lower.contains("permission denied");

    has_help_indicators && not_error && content.len() >= 50
}

/// Truncate help output to max lines
fn truncate_help(content: &str, max_lines: usize) -> String {
    content
        .lines()
        .take(max_lines)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Fetch Arch Wiki page content
pub fn fetch_arch_wiki(topic: &str) -> Option<SourceFetchResult> {
    // Try local cache first
    if let Some(result) = fetch_wiki_from_cache(topic) {
        return Some(result.cached());
    }

    // Arch Wiki online fetch would go here, but for now we only support cache
    // This keeps Anna local-first
    None
}

/// Fetch from local wiki cache
fn fetch_wiki_from_cache(topic: &str) -> Option<SourceFetchResult> {
    let home_cache = dirs::home_dir()
        .map(|h| h.join(".anna/wiki-cache"))
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp/anna-wiki-cache"));

    let cache_paths = [
        std::path::PathBuf::from("/var/lib/anna/wiki-cache"),
        home_cache,
    ];

    // Normalize topic to filename
    let filename = topic
        .replace(' ', "_")
        .replace('/', "_")
        .to_lowercase();

    for cache_path in &cache_paths {

        // Try exact match
        let exact_path = cache_path.join(format!("{}.txt", filename));
        if exact_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&exact_path) {
                return Some(SourceFetchResult::new(
                    content,
                    exact_path.to_string_lossy().as_ref(),
                ));
            }
        }

        // Try partial match
        if let Ok(entries) = std::fs::read_dir(&cache_path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains(&filename) && name.ends_with(".txt") {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        return Some(SourceFetchResult::new(
                            content,
                            entry.path().to_string_lossy().as_ref(),
                        ));
                    }
                }
            }
        }
    }

    None
}

/// Fetch local documentation
pub fn fetch_local_doc(topic: &str) -> Option<SourceFetchResult> {
    let doc_paths = [
        "/usr/share/doc",
        "/usr/share/help",
        "/usr/local/share/doc",
    ];

    let topic_lower = topic.to_lowercase();

    for doc_root in &doc_paths {
        let root_path = std::path::Path::new(doc_root);
        if !root_path.exists() {
            continue;
        }

        // Search for matching directory or file
        if let Ok(entries) = std::fs::read_dir(root_path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains(&topic_lower) {
                    // If directory, look for README
                    if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        let readme_names = ["README", "README.md", "README.txt", "readme"];
                        for readme in &readme_names {
                            let readme_path = entry.path().join(readme);
                            if readme_path.exists() {
                                if let Ok(content) = std::fs::read_to_string(&readme_path) {
                                    let truncated = truncate_doc(&content, 100);
                                    return Some(SourceFetchResult::new(
                                        truncated,
                                        readme_path.to_string_lossy().as_ref(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// Fetch pacman package info
pub fn fetch_pacman_info(package: &str) -> Option<SourceFetchResult> {
    if !is_safe_name(package) {
        return None;
    }

    // Get package info
    let output = Command::new("pacman")
        .args(["-Qi", package])
        .output()
        .ok()?;

    if !output.status.success() {
        // Try searching for package
        let search = Command::new("pacman")
            .args(["-Ss", package])
            .output()
            .ok()?;

        if search.status.success() {
            let content = String::from_utf8_lossy(&search.stdout);
            let truncated = truncate_doc(&content, 20);
            return Some(SourceFetchResult::new(truncated, &format!("pacman -Ss {}", package)));
        }
        return None;
    }

    let content = String::from_utf8_lossy(&output.stdout).to_string();
    Some(SourceFetchResult::new(content, &format!("pacman -Qi {}", package)))
}

/// Truncate document to max lines
fn truncate_doc(content: &str, max_lines: usize) -> String {
    content
        .lines()
        .take(max_lines)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Check if name is safe (no shell injection)
fn is_safe_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() < 100
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
}

/// Check if command is dangerous
fn is_dangerous_command(cmd: &str) -> bool {
    const DANGEROUS: &[&str] = &[
        "rm", "dd", "mkfs", "fdisk", "parted", "sudo", "su", "chmod", "chown",
        "kill", "pkill", "killall", "reboot", "shutdown", "halt", "poweroff",
        "mv", "cp", "shred", "wipefs",
    ];
    DANGEROUS.contains(&cmd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_name() {
        assert!(is_safe_name("systemctl"));
        assert!(is_safe_name("vim-enhanced"));
        assert!(is_safe_name("python3.11"));
        assert!(!is_safe_name("rm -rf /"));
        assert!(!is_safe_name("; rm -rf /"));
        assert!(!is_safe_name(""));
    }

    #[test]
    fn test_dangerous_commands() {
        assert!(is_dangerous_command("rm"));
        assert!(is_dangerous_command("dd"));
        assert!(is_dangerous_command("sudo"));
        assert!(!is_dangerous_command("systemctl"));
        assert!(!is_dangerous_command("pacman"));
    }

    #[test]
    fn test_looks_like_help() {
        assert!(looks_like_help("Usage: command [options]\n\nOptions:\n  --help\n  -v, --version"));
        assert!(!looks_like_help("command not found"));
        assert!(!looks_like_help("short"));
    }

    #[test]
    fn test_extract_man_sections() {
        let man_content = r#"
NAME
       systemctl - Control the systemd system and service manager

SYNOPSIS
       systemctl [OPTIONS...] COMMAND [UNIT...]

DESCRIPTION
       systemctl may be used to introspect and control the state of the
       systemd system and service manager.

SEE ALSO
       systemd(1)
"#;
        let extracted = extract_man_sections(man_content);
        assert!(extracted.contains("NAME"));
        assert!(extracted.contains("SYNOPSIS"));
        assert!(extracted.contains("DESCRIPTION"));
    }
}
