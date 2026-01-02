// v0.0.530: Knowledge Citation Tracker - Main Tracker
// Manages collection of citations with search and filtering capabilities

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::record::CitationRecord;
use super::types::{CitationReliability, CitationSource};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
