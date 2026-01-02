//! Documentation provider implementations.

use super::types::{DocSnippet, DocSourceKind};

/// Documentation provider interface
pub trait DocProvider {
    /// Fetch documentation for a topic
    fn fetch(&self, topic: &str) -> Option<DocSnippet>;
    /// Get provider kind
    fn kind(&self) -> DocSourceKind;
}

/// Man page provider
pub struct ManPageProvider;

impl DocProvider for ManPageProvider {
    fn fetch(&self, topic: &str) -> Option<DocSnippet> {
        // Try to get man page content
        let output = std::process::Command::new("man")
            .args(["-P", "cat", topic])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let content = String::from_utf8_lossy(&output.stdout);
        let excerpt = extract_man_synopsis(&content);

        Some(DocSnippet::from_man(topic, &excerpt))
    }

    fn kind(&self) -> DocSourceKind {
        DocSourceKind::ManPage
    }
}

/// Help flag provider
pub struct HelpProvider;

impl DocProvider for HelpProvider {
    fn fetch(&self, command: &str) -> Option<DocSnippet> {
        let output = std::process::Command::new(command)
            .arg("--help")
            .output()
            .ok()?;

        let content = if output.status.success() {
            String::from_utf8_lossy(&output.stdout)
        } else {
            String::from_utf8_lossy(&output.stderr)
        };

        if content.is_empty() {
            return None;
        }

        // Take first ~500 chars as excerpt
        let excerpt: String = content.chars().take(500).collect();
        Some(DocSnippet::from_help(command, &excerpt))
    }

    fn kind(&self) -> DocSourceKind {
        DocSourceKind::HelpFlag
    }
}

/// Extract synopsis section from man page
fn extract_man_synopsis(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_synopsis = false;
    let mut synopsis = String::new();

    for line in lines {
        if line.contains("SYNOPSIS") || line.contains("Synopsis") {
            in_synopsis = true;
            continue;
        }
        if in_synopsis {
            if line.chars().all(|c| c.is_uppercase() || c.is_whitespace()) && !line.is_empty() {
                break; // Next section
            }
            synopsis.push_str(line);
            synopsis.push('\n');
            if synopsis.len() > 500 {
                break;
            }
        }
    }

    if synopsis.is_empty() {
        // Fallback: first 500 chars
        content.chars().take(500).collect()
    } else {
        synopsis.trim().to_string()
    }
}
