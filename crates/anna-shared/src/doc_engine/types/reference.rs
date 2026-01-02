//! Documentation reference types (for recipes)

use super::snippet::DocSnippet;
use super::source::DocSourceKind;
use serde::{Deserialize, Serialize};

/// Reference to documentation (for recipes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocReference {
    /// Source type
    pub source: DocSourceKind,
    /// Document/topic name
    pub name: String,
    /// Optional section
    pub section: Option<String>,
    /// Why this doc is referenced
    pub reason: Option<String>,
}

impl DocReference {
    /// Create a new doc reference
    pub fn new(source: DocSourceKind, name: &str) -> Self {
        Self {
            source,
            name: name.to_string(),
            section: None,
            reason: None,
        }
    }

    /// Set section
    pub fn with_section(mut self, section: &str) -> Self {
        self.section = Some(section.to_string());
        self
    }

    /// Set reason for reference
    pub fn with_reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_string());
        self
    }

    /// Generate snippet ID for lookup
    pub fn snippet_id(&self) -> String {
        DocSnippet::generate_id(self.source, &self.name, self.section.as_deref())
    }

    /// Get citation string
    pub fn citation(&self) -> String {
        self.source
            .citation_format(&self.name, self.section.as_deref())
    }

    /// Arch Wiki reference
    pub fn arch_wiki(page: &str) -> Self {
        Self::new(DocSourceKind::ArchWiki, page)
    }

    /// Man page reference
    pub fn man_page(command: &str, section: &str) -> Self {
        Self::new(DocSourceKind::ManPage, command).with_section(section)
    }

    /// Help output reference
    pub fn tool_help(command: &str) -> Self {
        Self::new(DocSourceKind::ToolHelp, command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_reference() {
        let r = DocReference::arch_wiki("pacman").with_section("Tips and tricks");
        assert_eq!(r.citation(), "Arch Wiki: pacman#Tips and tricks");

        let r = DocReference::man_page("journalctl", "1");
        assert_eq!(r.citation(), "journalctl(1)");
    }
}
