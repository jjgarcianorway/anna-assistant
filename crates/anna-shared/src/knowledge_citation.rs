// v0.0.530: Knowledge Citation Tracker (Phase 106)
// Tracks citations from authoritative sources (Arch Wiki, man pages, --help) per VISION.md

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of knowledge source
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CitationSource {
    ArchWiki,
    ManPage,
    HelpCommand,
    InfoPage,
    OfficialDocs,
    LocalWiki,
    ConfigFile,
}

impl Default for CitationSource {
    fn default() -> Self {
        Self::ArchWiki
    }
}

impl std::fmt::Display for CitationSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArchWiki => write!(f, "Arch Wiki"),
            Self::ManPage => write!(f, "Man Page"),
            Self::HelpCommand => write!(f, "--help"),
            Self::InfoPage => write!(f, "Info Page"),
            Self::OfficialDocs => write!(f, "Official Docs"),
            Self::LocalWiki => write!(f, "Local Wiki"),
            Self::ConfigFile => write!(f, "Config File"),
        }
    }
}

/// Reliability of citation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub enum CitationReliability {
    Unverified,
    #[default]
    Trusted,
    Verified,
    Authoritative,
}

impl std::fmt::Display for CitationReliability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unverified => write!(f, "Unverified"),
            Self::Trusted => write!(f, "Trusted"),
            Self::Verified => write!(f, "Verified"),
            Self::Authoritative => write!(f, "Authoritative"),
        }
    }
}

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

/// Knowledge citation tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeCitationTracker {
    citations: HashMap<String, CitationRecord>,
    next_id: u32,
}

impl KnowledgeCitationTracker {
    /// Create new tracker
    pub fn new() -> Self {
        Self {
            citations: HashMap::new(),
            next_id: 1,
        }
    }

    /// Add a citation
    pub fn add(
        &mut self,
        source: CitationSource,
        title: &str,
        location: &str,
        snippet: &str,
        timestamp: &str,
    ) -> String {
        let id = format!("CIT-{:05}", self.next_id);
        self.next_id += 1;

        let citation = CitationRecord::new(&id, source, title, location, snippet, timestamp);
        self.citations.insert(id.clone(), citation);
        id
    }

    /// Get citation by ID
    pub fn get(&self, id: &str) -> Option<&CitationRecord> {
        self.citations.get(id)
    }

    /// Get mutable citation
    pub fn get_mut(&mut self, id: &str) -> Option<&mut CitationRecord> {
        self.citations.get_mut(id)
    }

    /// Record usage
    pub fn record_use(&mut self, id: &str, timestamp: &str) {
        if let Some(c) = self.citations.get_mut(id) {
            c.record_use(timestamp);
        }
    }

    /// Get citations by source
    pub fn by_source(&self, source: &CitationSource) -> Vec<&CitationRecord> {
        self.citations
            .values()
            .filter(|c| &c.source == source)
            .collect()
    }

    /// Get most used citations
    pub fn most_used(&self, n: usize) -> Vec<&CitationRecord> {
        let mut list: Vec<_> = self.citations.values().collect();
        list.sort_by(|a, b| b.used_count.cmp(&a.used_count));
        list.into_iter().take(n).collect()
    }

    /// Get authoritative citations
    pub fn authoritative(&self) -> Vec<&CitationRecord> {
        self.citations
            .values()
            .filter(|c| c.reliability == CitationReliability::Authoritative)
            .collect()
    }

    /// Search citations by title
    pub fn search(&self, query: &str) -> Vec<&CitationRecord> {
        let lower = query.to_lowercase();
        self.citations
            .values()
            .filter(|c| c.title.to_lowercase().contains(&lower))
            .collect()
    }

    /// Get citations for ticket
    pub fn for_ticket(&self, ticket_id: &str) -> Vec<&CitationRecord> {
        self.citations
            .values()
            .filter(|c| c.ticket_id.as_deref() == Some(ticket_id))
            .collect()
    }

    /// Source statistics
    pub fn source_stats(&self) -> HashMap<CitationSource, usize> {
        let mut stats = HashMap::new();
        for c in self.citations.values() {
            *stats.entry(c.source.clone()).or_insert(0) += 1;
        }
        stats
    }

    /// Total citations
    pub fn total(&self) -> usize {
        self.citations.len()
    }

    /// All citations
    pub fn all(&self) -> Vec<&CitationRecord> {
        self.citations.values().collect()
    }
}

/// Format citation for display
pub fn format_citation(cit: &CitationRecord) -> String {
    format!(
        "{} [{}]\n  Source: {} | Reliability: {}\n  Location: {}\n  Used: {} times\n  Snippet: {}...",
        cit.id,
        cit.title,
        cit.source,
        cit.reliability,
        cit.location,
        cit.used_count,
        if cit.snippet.len() > 50 {
            &cit.snippet[..50]
        } else {
            &cit.snippet
        }
    )
}

/// Format citation compact
pub fn format_citation_compact(cit: &CitationRecord) -> String {
    format!(
        "{}: {} [{}] (used {} times)",
        cit.id, cit.title, cit.source, cit.used_count
    )
}

/// Format citation oneline
pub fn format_citation_oneline(cit: &CitationRecord) -> String {
    format!("[{}] {}", cit.source, cit.title)
}

/// Format tracker summary
pub fn format_tracker_summary(tracker: &KnowledgeCitationTracker) -> String {
    let mut output = String::new();
    output.push_str("=== Knowledge Citations ===\n\n");

    output.push_str(&format!("Total Citations: {}\n\n", tracker.total()));

    output.push_str("--- By Source ---\n");
    for (source, count) in tracker.source_stats() {
        output.push_str(&format!("  {}: {}\n", source, count));
    }

    output.push_str("\n--- Most Used ---\n");
    for cit in tracker.most_used(5) {
        output.push_str(&format!("  {}\n", format_citation_compact(cit)));
    }

    output
}

/// Check if query is citation-related
pub fn is_citation_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("citation")
        || lower.contains("source")
        || lower.contains("reference")
        || lower.contains("wiki")
        || lower.contains("man page")
        || lower.contains("documentation")
}

/// Fun fact about citations
pub fn citation_fun_fact() -> &'static str {
    "The Arch Wiki is one of the most comprehensive Linux documentation sources, with over 13,000 articles - Anna considers it her bible!"
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

    #[test]
    fn test_tracker_add() {
        let mut tracker = KnowledgeCitationTracker::new();
        let id = tracker.add(
            CitationSource::ArchWiki,
            "Systemd",
            "wiki",
            "snippet",
            "ts",
        );
        assert_eq!(tracker.total(), 1);
        assert!(tracker.get(&id).is_some());
    }

    #[test]
    fn test_by_source() {
        let mut tracker = KnowledgeCitationTracker::new();
        tracker.add(CitationSource::ArchWiki, "A", "l", "s", "ts");
        tracker.add(CitationSource::ArchWiki, "B", "l", "s", "ts");
        tracker.add(CitationSource::ManPage, "C", "l", "s", "ts");
        assert_eq!(tracker.by_source(&CitationSource::ArchWiki).len(), 2);
    }

    #[test]
    fn test_most_used() {
        let mut tracker = KnowledgeCitationTracker::new();
        let id1 = tracker.add(CitationSource::ArchWiki, "Low", "l", "s", "ts");
        let id2 = tracker.add(CitationSource::ManPage, "High", "l", "s", "ts");
        tracker.record_use(&id2, "ts");
        tracker.record_use(&id2, "ts");
        tracker.record_use(&id1, "ts");
        let top = tracker.most_used(1);
        assert_eq!(top[0].title, "High");
    }

    #[test]
    fn test_search() {
        let mut tracker = KnowledgeCitationTracker::new();
        tracker.add(CitationSource::ArchWiki, "Vim Editor", "l", "s", "ts");
        tracker.add(CitationSource::ManPage, "Nano", "l", "s", "ts");
        let results = tracker.search("vim");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_source_stats() {
        let mut tracker = KnowledgeCitationTracker::new();
        tracker.add(CitationSource::ArchWiki, "A", "l", "s", "ts");
        tracker.add(CitationSource::ManPage, "B", "l", "s", "ts");
        tracker.add(CitationSource::ManPage, "C", "l", "s", "ts");
        let stats = tracker.source_stats();
        assert_eq!(stats.get(&CitationSource::ManPage), Some(&2));
    }

    #[test]
    fn test_is_citation_query() {
        assert!(is_citation_query("What's the source for this?"));
        assert!(is_citation_query("Show me the wiki page"));
        assert!(is_citation_query("Check man page for ls"));
        assert!(!is_citation_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = citation_fun_fact();
        assert!(fact.contains("Arch Wiki") || fact.contains("13,000"));
    }
}
