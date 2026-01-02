//! Man page source (v0.0.435).

use serde::{Deserialize, Serialize};
use std::process::Command;

use super::error::SourceError;

/// Man page source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManPageSource {
    /// Command name.
    pub command: String,
    /// Optional section number (1-8).
    pub section: Option<u8>,
    /// Retrieved content.
    pub content: Option<String>,
    /// Relevant sections extracted.
    pub sections: Vec<ManSection>,
}

impl ManPageSource {
    /// Create a new man page source.
    pub fn new(command: &str) -> Self {
        Self {
            command: command.to_string(),
            section: None,
            content: None,
            sections: Vec::new(),
        }
    }

    /// With specific section.
    pub fn with_section(mut self, section: u8) -> Self {
        self.section = Some(section);
        self
    }

    /// Retrieve the man page content.
    pub fn retrieve(&mut self) -> Result<(), SourceError> {
        let mut cmd = Command::new("man");
        cmd.arg("-P").arg("cat"); // Use cat as pager for plain text

        if let Some(sec) = self.section {
            cmd.arg(sec.to_string());
        }
        cmd.arg(&self.command);

        let output = cmd
            .output()
            .map_err(|e| SourceError::CommandFailed(format!("man: {}", e)))?;

        if !output.status.success() {
            return Err(SourceError::NotFound(format!(
                "man page for {}",
                self.command
            )));
        }

        self.content = Some(String::from_utf8_lossy(&output.stdout).to_string());
        self.extract_sections();
        Ok(())
    }

    /// Extract relevant sections from content.
    fn extract_sections(&mut self) {
        let Some(content) = &self.content else { return };

        let mut current_section = String::new();
        let mut current_content = String::new();
        let mut in_section = false;

        for line in content.lines() {
            // Section headers are typically ALL CAPS at start of line
            if line
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false)
                && !line.contains("  ")
                && line.len() < 30
            {
                if in_section && !current_content.is_empty() {
                    self.sections.push(ManSection {
                        name: current_section.clone(),
                        content: current_content.trim().to_string(),
                    });
                }
                current_section = line.trim().to_string();
                current_content.clear();
                in_section = true;
            } else if in_section {
                current_content.push_str(line);
                current_content.push('\n');
            }
        }

        // Add last section
        if in_section && !current_content.is_empty() {
            self.sections.push(ManSection {
                name: current_section,
                content: current_content.trim().to_string(),
            });
        }
    }

    /// Search for text and return snippets with context.
    pub fn search(&self, query: &str) -> Vec<String> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        for section in &self.sections {
            let content_lower = section.content.to_lowercase();
            if content_lower.contains(&query_lower) {
                // Find the relevant lines
                for line in section.content.lines() {
                    if line.to_lowercase().contains(&query_lower) {
                        let snippet = format!("({}) → {}", section.name, line.trim());
                        if snippet.len() <= super::super::MAX_CITATION_EXCERPT_LEN {
                            results.push(snippet);
                        } else {
                            results.push(format!(
                                "{}...",
                                &snippet[..super::super::MAX_CITATION_EXCERPT_LEN]
                            ));
                        }
                    }
                }
            }
        }

        results.truncate(5); // Max 5 snippets
        results
    }
}

/// A section within a man page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManSection {
    /// Section name (e.g., "OPTIONS", "DESCRIPTION").
    pub name: String,
    /// Section content.
    pub content: String,
}
