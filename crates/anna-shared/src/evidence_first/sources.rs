//! Knowledge sources (v0.0.435).
//!
//! Abstractions for retrieving evidence from local sources.

use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::Duration;

/// A knowledge source that can provide evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgeSource {
    /// Man page documentation.
    ManPage(ManPageSource),
    /// Command help text.
    HelpText(HelpTextSource),
    /// Local documentation files.
    LocalDocs(LocalDocsSource),
    /// Cached Arch Wiki page.
    ArchWiki(String),
    /// System probe output.
    ProbeOutput(String),
}

impl KnowledgeSource {
    /// Human-readable label.
    pub fn label(&self) -> String {
        match self {
            Self::ManPage(m) => format!(
                "man {}{}",
                m.section.map(|s| format!("{} ", s)).unwrap_or_default(),
                m.command
            ),
            Self::HelpText(h) => format!("{} {}", h.command, h.variant.flag()),
            Self::LocalDocs(d) => format!("doc:{}", d.package.as_deref().unwrap_or(&d.path)),
            Self::ArchWiki(page) => format!("archwiki:{}", page),
            Self::ProbeOutput(id) => format!("probe:{}", id),
        }
    }
}

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
                        if snippet.len() <= super::MAX_CITATION_EXCERPT_LEN {
                            results.push(snippet);
                        } else {
                            results.push(format!(
                                "{}...",
                                &snippet[..super::MAX_CITATION_EXCERPT_LEN]
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

/// Help text source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpTextSource {
    /// Command name.
    pub command: String,
    /// Which variant worked.
    pub variant: HelpVariant,
    /// Retrieved content.
    pub content: Option<String>,
}

/// Help text retrieval variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HelpVariant {
    /// --help flag.
    LongHelp,
    /// -h flag.
    ShortHelp,
    /// help subcommand.
    HelpSubcommand,
}

impl HelpVariant {
    /// Get the flag/argument.
    pub fn flag(&self) -> &'static str {
        match self {
            Self::LongHelp => "--help",
            Self::ShortHelp => "-h",
            Self::HelpSubcommand => "help",
        }
    }
}

impl HelpTextSource {
    /// Create a new help text source.
    pub fn new(command: &str) -> Self {
        Self {
            command: command.to_string(),
            variant: HelpVariant::LongHelp,
            content: None,
        }
    }

    /// Retrieve help text, trying multiple variants.
    pub fn retrieve(&mut self) -> Result<(), SourceError> {
        // Try variants in order
        for variant in [
            HelpVariant::LongHelp,
            HelpVariant::ShortHelp,
            HelpVariant::HelpSubcommand,
        ] {
            if let Ok(content) = self.try_variant(variant) {
                self.variant = variant;
                self.content = Some(content);
                return Ok(());
            }
        }

        Err(SourceError::NotFound(format!("help for {}", self.command)))
    }

    /// Try a specific help variant.
    fn try_variant(&self, variant: HelpVariant) -> Result<String, SourceError> {
        let output = match variant {
            HelpVariant::LongHelp => Command::new(&self.command).arg("--help").output(),
            HelpVariant::ShortHelp => Command::new(&self.command).arg("-h").output(),
            HelpVariant::HelpSubcommand => Command::new(&self.command).arg("help").output(),
        };

        let output = output.map_err(|e| SourceError::CommandFailed(e.to_string()))?;

        // Help might be on stdout or stderr
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let content = if stdout.len() > stderr.len() {
            stdout.to_string()
        } else {
            stderr.to_string()
        };

        if content.len() > 50 {
            Ok(content)
        } else {
            Err(SourceError::NotFound("empty help".to_string()))
        }
    }

    /// Search for text in help content.
    pub fn search(&self, query: &str) -> Vec<String> {
        let Some(content) = &self.content else {
            return Vec::new();
        };

        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        for line in content.lines() {
            if line.to_lowercase().contains(&query_lower) {
                let snippet = line.trim().to_string();
                if !snippet.is_empty() {
                    if snippet.len() <= super::MAX_CITATION_EXCERPT_LEN {
                        results.push(snippet);
                    } else {
                        results.push(format!(
                            "{}...",
                            &snippet[..super::MAX_CITATION_EXCERPT_LEN]
                        ));
                    }
                }
            }
        }

        results.truncate(5);
        results
    }
}

/// Local documentation source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalDocsSource {
    /// Path to documentation.
    pub path: String,
    /// Package name if known.
    pub package: Option<String>,
    /// Content (if small enough to load).
    pub content: Option<String>,
}

impl LocalDocsSource {
    /// Search /usr/share/doc for a package.
    pub fn find_for_package(package: &str) -> Option<Self> {
        let doc_path = format!("/usr/share/doc/{}", package);
        if std::path::Path::new(&doc_path).exists() {
            return Some(Self {
                path: doc_path,
                package: Some(package.to_string()),
                content: None,
            });
        }
        None
    }

    /// Search /usr/share/doc for files matching a query.
    pub fn search_docs(query: &str) -> Vec<Self> {
        let mut results = Vec::new();

        // Search with ripgrep if available
        let output = Command::new("rg")
            .args(["-l", "-i", query, "/usr/share/doc"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                for line in String::from_utf8_lossy(&output.stdout).lines().take(10) {
                    results.push(Self {
                        path: line.to_string(),
                        package: extract_package_from_path(line),
                        content: None,
                    });
                }
            }
        }

        results
    }

    /// Load content from file.
    pub fn load_content(&mut self) -> Result<(), SourceError> {
        let path = std::path::Path::new(&self.path);

        if path.is_file() {
            let content = std::fs::read_to_string(path)
                .map_err(|e| SourceError::ReadFailed(e.to_string()))?;

            // Limit content size
            if content.len() > 100_000 {
                self.content = Some(content[..100_000].to_string());
            } else {
                self.content = Some(content);
            }
        }

        Ok(())
    }
}

/// Extract package name from doc path.
fn extract_package_from_path(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() > 4 && parts[1] == "usr" && parts[2] == "share" && parts[3] == "doc" {
        return Some(parts[4].to_string());
    }
    None
}

/// Source retrieval error.
#[derive(Debug, Clone)]
pub enum SourceError {
    /// Command failed to run.
    CommandFailed(String),
    /// Source not found.
    NotFound(String),
    /// Failed to read content.
    ReadFailed(String),
    /// Timeout.
    Timeout,
}

impl std::fmt::Display for SourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CommandFailed(msg) => write!(f, "Command failed: {}", msg),
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
            Self::ReadFailed(msg) => write!(f, "Read failed: {}", msg),
            Self::Timeout => write!(f, "Timeout"),
        }
    }
}

impl std::error::Error for SourceError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_source_labels() {
        let man = KnowledgeSource::ManPage(ManPageSource::new("ls"));
        assert_eq!(man.label(), "man ls");

        let man_section = KnowledgeSource::ManPage(ManPageSource::new("ls").with_section(1));
        assert_eq!(man_section.label(), "man 1 ls");

        let help = KnowledgeSource::HelpText(HelpTextSource::new("git"));
        assert_eq!(help.label(), "git --help");

        let wiki = KnowledgeSource::ArchWiki("Systemd".to_string());
        assert_eq!(wiki.label(), "archwiki:Systemd");
    }

    #[test]
    fn test_help_variant_flags() {
        assert_eq!(HelpVariant::LongHelp.flag(), "--help");
        assert_eq!(HelpVariant::ShortHelp.flag(), "-h");
        assert_eq!(HelpVariant::HelpSubcommand.flag(), "help");
    }

    #[test]
    fn test_extract_package_from_path() {
        let path = "/usr/share/doc/systemd/README";
        assert_eq!(extract_package_from_path(path), Some("systemd".to_string()));

        let path = "/some/other/path";
        assert_eq!(extract_package_from_path(path), None);
    }
}
