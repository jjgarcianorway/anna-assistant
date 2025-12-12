//! Man page parser and indexer (v0.0.429).
//!
//! Parses man pages into indexed snippets by section.

use super::{DocSnippet, DocSourceKind, MAX_SNIPPET_SIZE};
use std::collections::HashMap;
use std::process::Command;

/// Parse a man page into snippets
pub fn parse_man_page(name: &str, section: Option<&str>) -> Result<Vec<DocSnippet>, ManParseError> {
    let raw = fetch_man_raw(name, section)?;
    let sections = split_into_sections(&raw);

    let man_section = section.unwrap_or("1");
    let mut snippets = Vec::new();

    // Create main snippet with NAME and DESCRIPTION
    if let Some(desc) = sections.get("NAME").or(sections.get("DESCRIPTION")) {
        let summary = extract_first_line(desc);
        snippets.push(DocSnippet::new(
            DocSourceKind::ManPage,
            name,
            Some(man_section),
            &summary,
            desc,
        ));
    }

    // Create snippets for key sections
    let key_sections = [
        "SYNOPSIS",
        "OPTIONS",
        "COMMANDS",
        "EXAMPLES",
        "EXIT STATUS",
        "SEE ALSO",
    ];

    for sec_name in &key_sections {
        if let Some(content) = sections.get(*sec_name) {
            if !content.trim().is_empty() && content.len() > 10 {
                let section_id = format!(
                    "{}:{}",
                    man_section,
                    sec_name.to_lowercase().replace(' ', "_")
                );
                let summary = format!("{} - {}", name, sec_name);

                let mut snippet = DocSnippet::new(
                    DocSourceKind::ManPage,
                    name,
                    Some(&section_id),
                    &summary,
                    content,
                );
                snippet.truncate_content(MAX_SNIPPET_SIZE);
                snippets.push(snippet);
            }
        }
    }

    if snippets.is_empty() {
        return Err(ManParseError::NoContent(name.to_string()));
    }

    Ok(snippets)
}

/// Fetch raw man page content
fn fetch_man_raw(name: &str, section: Option<&str>) -> Result<String, ManParseError> {
    // Validate name (prevent command injection)
    if !is_safe_name(name) {
        return Err(ManParseError::InvalidName(name.to_string()));
    }

    let mut cmd = Command::new("man");

    if let Some(sec) = section {
        cmd.arg(sec);
    }

    cmd.arg(name);

    // Pipe through col to remove formatting
    let output = cmd
        .output()
        .map_err(|e| ManParseError::CommandFailed(e.to_string()))?;

    if !output.status.success() {
        return Err(ManParseError::NotFound(name.to_string()));
    }

    let raw = String::from_utf8_lossy(&output.stdout).to_string();

    // Clean up formatting with col -b
    let col_output = Command::new("col")
        .arg("-b")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(raw.as_bytes());
            }
            child.wait_with_output()
        });

    match col_output {
        Ok(output) if output.status.success() => {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }
        _ => Ok(raw), // Fall back to raw if col fails
    }
}

/// Split man page into sections
fn split_into_sections(content: &str) -> HashMap<String, String> {
    let mut sections = HashMap::new();
    let mut current_section = String::new();
    let mut current_content = String::new();

    for line in content.lines() {
        // Section headers are all caps, start at column 0
        if is_section_header(line) {
            // Save previous section
            if !current_section.is_empty() && !current_content.trim().is_empty() {
                sections.insert(current_section.clone(), current_content.trim().to_string());
            }
            current_section = line.trim().to_string();
            current_content = String::new();
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    // Save last section
    if !current_section.is_empty() && !current_content.trim().is_empty() {
        sections.insert(current_section, current_content.trim().to_string());
    }

    sections
}

/// Check if a line is a section header
fn is_section_header(line: &str) -> bool {
    let trimmed = line.trim();

    // Must not be empty and not be indented
    if trimmed.is_empty() || line.starts_with(' ') || line.starts_with('\t') {
        return false;
    }

    // Must be all uppercase (with spaces allowed)
    trimmed
        .chars()
        .all(|c| c.is_uppercase() || c.is_whitespace() || c == '-')
        && trimmed.chars().any(|c| c.is_alphabetic())
}

/// Extract first meaningful line
fn extract_first_line(content: &str) -> String {
    content
        .lines()
        .find(|l| !l.trim().is_empty())
        .map(|l| {
            let s = l.trim();
            if s.len() > 150 {
                format!("{}...", &s[..147])
            } else {
                s.to_string()
            }
        })
        .unwrap_or_default()
}

/// Check if name is safe (no shell injection)
fn is_safe_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() < 64
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
}

/// Search for man pages by keyword
pub fn search_man_pages(keyword: &str, limit: usize) -> Vec<ManPageInfo> {
    if !is_safe_name(keyword) {
        return Vec::new();
    }

    let output = Command::new("man").arg("-k").arg(keyword).output();

    match output {
        Ok(out) if out.status.success() => {
            parse_apropos_output(&String::from_utf8_lossy(&out.stdout), limit)
        }
        _ => Vec::new(),
    }
}

/// Info about a man page from apropos
#[derive(Debug, Clone)]
pub struct ManPageInfo {
    pub name: String,
    pub section: String,
    pub description: String,
}

/// Parse apropos (man -k) output
fn parse_apropos_output(output: &str, limit: usize) -> Vec<ManPageInfo> {
    let mut results = Vec::new();

    for line in output.lines().take(limit * 2) {
        // Format: "name (section) - description"
        if let Some((name_section, desc)) = line.split_once(" - ") {
            let name_section = name_section.trim();
            if let Some((name, section)) = parse_name_section(name_section) {
                results.push(ManPageInfo {
                    name,
                    section,
                    description: desc.trim().to_string(),
                });

                if results.len() >= limit {
                    break;
                }
            }
        }
    }

    results
}

/// Parse "name (section)" format
fn parse_name_section(s: &str) -> Option<(String, String)> {
    // Handle formats like "systemctl (1)" or "systemctl(1)"
    let s = s.trim();

    if let Some(paren_start) = s.rfind('(') {
        if let Some(paren_end) = s.rfind(')') {
            if paren_end > paren_start {
                let name = s[..paren_start].trim().to_string();
                let section = s[paren_start + 1..paren_end].trim().to_string();

                if !name.is_empty() && !section.is_empty() {
                    return Some((name, section));
                }
            }
        }
    }

    None
}

/// Get list of common man pages to index
pub fn get_essential_man_pages() -> Vec<(&'static str, &'static str)> {
    vec![
        // Systemd
        ("systemctl", "1"),
        ("journalctl", "1"),
        ("systemd", "1"),
        ("systemd.unit", "5"),
        ("systemd.service", "5"),
        ("systemd.timer", "5"),
        // Package management
        ("pacman", "8"),
        ("pacman.conf", "5"),
        // Filesystem
        ("mount", "8"),
        ("umount", "8"),
        ("fstab", "5"),
        ("df", "1"),
        ("du", "1"),
        ("fdisk", "8"),
        ("lsblk", "8"),
        ("blkid", "8"),
        // Network
        ("ip", "8"),
        ("ss", "8"),
        ("networkctl", "1"),
        // System
        ("free", "1"),
        ("top", "1"),
        ("ps", "1"),
        ("kill", "1"),
        ("grep", "1"),
        ("find", "1"),
        // Boot
        ("bootctl", "1"),
        ("mkinitcpio", "8"),
        // Users
        ("useradd", "8"),
        ("passwd", "1"),
        ("sudo", "8"),
    ]
}

/// Man page parsing errors
#[derive(Debug, Clone)]
pub enum ManParseError {
    InvalidName(String),
    NotFound(String),
    CommandFailed(String),
    NoContent(String),
}

impl std::fmt::Display for ManParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidName(n) => write!(f, "Invalid man page name: {}", n),
            Self::NotFound(n) => write!(f, "Man page not found: {}", n),
            Self::CommandFailed(e) => write!(f, "Man command failed: {}", e),
            Self::NoContent(n) => write!(f, "No content in man page: {}", n),
        }
    }
}

impl std::error::Error for ManParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_section_header() {
        assert!(is_section_header("NAME"));
        assert!(is_section_header("DESCRIPTION"));
        assert!(is_section_header("SEE ALSO"));
        assert!(is_section_header("EXIT STATUS"));

        assert!(!is_section_header("  NAME")); // indented
        assert!(!is_section_header("Name")); // not all caps
        assert!(!is_section_header("")); // empty
        assert!(!is_section_header("   ")); // whitespace only
    }

    #[test]
    fn test_is_safe_name() {
        assert!(is_safe_name("systemctl"));
        assert!(is_safe_name("systemd.unit"));
        assert!(is_safe_name("ip-link"));

        assert!(!is_safe_name("")); // empty
        assert!(!is_safe_name("foo;rm -rf /")); // injection
        assert!(!is_safe_name("$(whoami)")); // injection
    }

    #[test]
    fn test_parse_name_section() {
        assert_eq!(
            parse_name_section("systemctl (1)"),
            Some(("systemctl".to_string(), "1".to_string()))
        );
        assert_eq!(
            parse_name_section("pacman(8)"),
            Some(("pacman".to_string(), "8".to_string()))
        );
        assert_eq!(
            parse_name_section("fstab (5)"),
            Some(("fstab".to_string(), "5".to_string()))
        );
    }

    #[test]
    fn test_split_into_sections() {
        let content = "NAME\n       test - a test command\n\nDESCRIPTION\n       This is the description.\n\nOPTIONS\n       -h  help\n";
        let sections = split_into_sections(content);

        assert!(sections.contains_key("NAME"));
        assert!(sections.contains_key("DESCRIPTION"));
        assert!(sections.contains_key("OPTIONS"));
    }

    #[test]
    fn test_extract_first_line() {
        let content = "\n   first line here\n   second line\n";
        assert_eq!(extract_first_line(content), "first line here");

        let content = "";
        assert_eq!(extract_first_line(content), "");
    }
}
