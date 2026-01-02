// v0.0.530: Knowledge Citation Tracker - Citation Record
// Individual citation record with methods for usage tracking

use serde::{Deserialize, Serialize};

use super::types::{CitationReliability, CitationSource};

/// Individual citation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationRecord {
    pub id: String,
    pub source: CitationSource,
    pub title: String,
    pub location: String,
    pub snippet: String,
    pub reliability: CitationReliability,
    pub ticket_id: Option<String>,
    pub used_count: u32,
    pub created_at: String,
    pub last_used: Option<String>,
}

impl CitationRecord {
    /// Create a new citation
    pub fn new(
        id: &str,
        source: CitationSource,
        title: &str,
        location: &str,
        snippet: &str,
        timestamp: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            source,
            title: title.to_string(),
            location: location.to_string(),
            snippet: snippet.to_string(),
            reliability: CitationReliability::Trusted,
            ticket_id: None,
            used_count: 0,
            created_at: timestamp.to_string(),
            last_used: None,
        }
    }

    /// Record usage of this citation
    pub fn record_use(&mut self, timestamp: &str) {
        self.used_count += 1;
        self.last_used = Some(timestamp.to_string());
    }

    /// Link to ticket
    pub fn link_ticket(&mut self, ticket_id: &str) {
        self.ticket_id = Some(ticket_id.to_string());
    }

    /// Set reliability
    pub fn set_reliability(&mut self, reliability: CitationReliability) {
        self.reliability = reliability;
    }

    /// Format as reference string
    pub fn as_reference(&self) -> String {
        format!("[{}] {} - {}", self.source, self.title, self.location)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_citation_creation() {
        let cit = CitationRecord::new(
            "CIT-001",
            CitationSource::ArchWiki,
            "Vim",
            "https://wiki.archlinux.org/title/Vim",
            "Vim is a terminal text editor.",
            "2024-01-01",
        );
        assert_eq!(cit.title, "Vim");
        assert_eq!(cit.source, CitationSource::ArchWiki);
    }

    #[test]
    fn test_record_use() {
        let mut cit = CitationRecord::new(
            "CIT-001",
            CitationSource::ManPage,
            "ls(1)",
            "man ls",
            "list directory contents",
            "2024-01-01",
        );
        cit.record_use("2024-01-02");
        assert_eq!(cit.used_count, 1);
        assert!(cit.last_used.is_some());
    }

    #[test]
    fn test_as_reference() {
        let cit = CitationRecord::new(
            "CIT-001",
            CitationSource::HelpCommand,
            "pacman --help",
            "pacman -h",
            "package manager",
            "2024-01-01",
        );
        let ref_str = cit.as_reference();
        assert!(ref_str.contains("--help"));
    }
}
