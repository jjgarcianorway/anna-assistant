//! Rolling evidence cache implementation.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use super::evidence_types::{EvidenceEntry, EvidenceType};
use super::utils::now_epoch;

/// Rolling evidence cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceCache {
    /// Evidence entries (newest first)
    entries: VecDeque<EvidenceEntry>,
    /// Maximum entries to keep
    max_entries: usize,
    /// Index by domain for faster lookup
    #[serde(skip)]
    domain_index: HashMap<String, Vec<usize>>,
    /// Last update timestamp
    pub last_updated: u64,
}

impl Default for EvidenceCache {
    fn default() -> Self {
        Self::new(super::super::MAX_EVIDENCE_CACHE)
    }
}

impl EvidenceCache {
    /// Create a new cache with max entries
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries,
            domain_index: HashMap::new(),
            last_updated: now_epoch(),
        }
    }

    /// Add an entry to the cache
    pub fn add(&mut self, entry: EvidenceEntry) {
        // Add to front (newest first)
        self.entries.push_front(entry);

        // Prune if over limit
        while self.entries.len() > self.max_entries {
            self.entries.pop_back();
        }

        self.last_updated = now_epoch();
        self.rebuild_index();
    }

    /// Get entries by domain
    pub fn by_domain(&self, domain: &str) -> Vec<&EvidenceEntry> {
        self.entries.iter().filter(|e| e.domain == domain).collect()
    }

    /// Get entries by intent
    pub fn by_intent(&self, intent: &str) -> Vec<&EvidenceEntry> {
        self.entries
            .iter()
            .filter(|e| e.intent.as_deref() == Some(intent))
            .collect()
    }

    /// Get entries by evidence type
    pub fn by_type(&self, evidence_type: EvidenceType) -> Vec<&EvidenceEntry> {
        self.entries
            .iter()
            .filter(|e| e.evidence_type == evidence_type)
            .collect()
    }

    /// Search entries by keywords
    pub fn search(&self, keywords: &[&str]) -> Vec<&EvidenceEntry> {
        self.entries
            .iter()
            .filter(|e| {
                keywords.iter().any(|kw| {
                    e.keywords.iter().any(|ek| ek.contains(kw))
                        || e.content.to_lowercase().contains(&kw.to_lowercase())
                })
            })
            .collect()
    }

    /// Get recent entries (last N)
    pub fn recent(&self, count: usize) -> Vec<&EvidenceEntry> {
        self.entries.iter().take(count).collect()
    }

    /// Get similar entries to a ticket
    pub fn similar_to_ticket(
        &self,
        domain: &str,
        intent: &str,
        keywords: &[&str],
    ) -> Vec<&EvidenceEntry> {
        self.entries
            .iter()
            .filter(|e| {
                e.domain == domain
                    || e.intent.as_deref() == Some(intent)
                    || keywords
                        .iter()
                        .any(|kw| e.keywords.contains(&kw.to_string()))
            })
            .take(10)
            .collect()
    }

    /// Get man page evidence for a command
    pub fn get_man_page(&self, command: &str) -> Option<&EvidenceEntry> {
        self.entries.iter().find(|e| {
            e.evidence_type == EvidenceType::ManPage
                && e.citation.as_deref() == Some(&format!("man:{}", command))
        })
    }

    /// Get help evidence for a command
    pub fn get_help(&self, command: &str) -> Option<&EvidenceEntry> {
        self.entries.iter().find(|e| {
            e.evidence_type == EvidenceType::HelpOutput
                && e.citation.as_deref() == Some(&format!("help:{}", command))
        })
    }

    /// Prune old entries (older than days)
    pub fn prune_old(&mut self, max_age_days: u32) {
        let cutoff = now_epoch() - (max_age_days as u64 * 24 * 60 * 60);
        self.entries.retain(|e| e.timestamp >= cutoff);
        self.rebuild_index();
    }

    /// Get total entry count
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Rebuild the domain index
    fn rebuild_index(&mut self) {
        self.domain_index.clear();
        for (idx, entry) in self.entries.iter().enumerate() {
            self.domain_index
                .entry(entry.domain.clone())
                .or_default()
                .push(idx);
        }
    }

    /// Get entries that haven't been used for learning
    pub fn unused_for_learning(&self) -> Vec<&EvidenceEntry> {
        self.entries
            .iter()
            .filter(|e| !e.used_for_learning)
            .collect()
    }

    /// Mark entries as used for a recipe
    pub fn mark_used_for_recipe(&mut self, entry_ids: &[&str]) {
        for entry in self.entries.iter_mut() {
            if entry_ids.contains(&entry.id.as_str()) {
                entry.used_for_learning = true;
            }
        }
    }
}
