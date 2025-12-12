//! Evidence cache for learning engine (v0.0.427).
//!
//! Rolling cache of:
//! - Probe outputs
//! - Documentation links (Arch Wiki, man pages)
//! - Prior tickets with similar patterns
//!
//! Not directly used to answer users - used as context
//! when generating or refining recipes.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// An evidence entry in the cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceEntry {
    /// Unique entry ID
    pub id: String,
    /// Type of evidence
    pub evidence_type: EvidenceType,
    /// Associated ticket ID (if any)
    #[serde(default)]
    pub ticket_id: Option<String>,
    /// Domain/category
    pub domain: String,
    /// Intent (if known)
    #[serde(default)]
    pub intent: Option<String>,
    /// Raw content (probe output, help text, etc.)
    pub content: String,
    /// Citation (e.g., "man:systemctl", "help:pacman", "wiki:systemd")
    #[serde(default)]
    pub citation: Option<String>,
    /// Keywords extracted from this evidence
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Timestamp (Unix epoch seconds)
    pub timestamp: u64,
    /// Whether this contributed to a learned recipe
    #[serde(default)]
    pub used_for_learning: bool,
}

/// Type of evidence
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    /// Output from a probe
    ProbeOutput,
    /// Man page excerpt
    ManPage,
    /// Help output from command
    HelpOutput,
    /// Arch Wiki excerpt
    ArchWiki,
    /// Prior successful ticket summary
    TicketSummary,
    /// Documentation from /usr/share/doc
    LocalDoc,
}

impl EvidenceEntry {
    /// Create a new probe output entry
    pub fn probe_output(probe_id: &str, output: &str, domain: &str) -> Self {
        Self {
            id: format!("probe:{}:{}", probe_id, now_epoch()),
            evidence_type: EvidenceType::ProbeOutput,
            ticket_id: None,
            domain: domain.to_string(),
            intent: None,
            content: truncate(output, 2000),
            citation: Some(format!("probe:{}", probe_id)),
            keywords: extract_keywords(output),
            timestamp: now_epoch(),
            used_for_learning: false,
        }
    }

    /// Create a man page entry
    pub fn man_page(command: &str, excerpt: &str) -> Self {
        Self {
            id: format!("man:{}:{}", command, now_epoch()),
            evidence_type: EvidenceType::ManPage,
            ticket_id: None,
            domain: infer_domain(command),
            intent: None,
            content: truncate(excerpt, 2000),
            citation: Some(format!("man:{}", command)),
            keywords: extract_keywords(excerpt),
            timestamp: now_epoch(),
            used_for_learning: false,
        }
    }

    /// Create a help output entry
    pub fn help_output(command: &str, output: &str) -> Self {
        Self {
            id: format!("help:{}:{}", command, now_epoch()),
            evidence_type: EvidenceType::HelpOutput,
            ticket_id: None,
            domain: infer_domain(command),
            intent: None,
            content: truncate(output, 2000),
            citation: Some(format!("help:{}", command)),
            keywords: extract_keywords(output),
            timestamp: now_epoch(),
            used_for_learning: false,
        }
    }

    /// Create a ticket summary entry
    pub fn ticket_summary(ticket_id: &str, domain: &str, intent: &str, summary: &str) -> Self {
        Self {
            id: format!("ticket:{}:{}", ticket_id, now_epoch()),
            evidence_type: EvidenceType::TicketSummary,
            ticket_id: Some(ticket_id.to_string()),
            domain: domain.to_string(),
            intent: Some(intent.to_string()),
            content: truncate(summary, 1000),
            citation: None,
            keywords: extract_keywords(summary),
            timestamp: now_epoch(),
            used_for_learning: false,
        }
    }

    /// Mark as used for learning
    pub fn mark_used(&mut self) {
        self.used_for_learning = true;
    }
}

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
        Self::new(super::MAX_EVIDENCE_CACHE)
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

/// Extract keywords from text
fn extract_keywords(text: &str) -> Vec<String> {
    let stopwords = [
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
        "do", "does", "did", "will", "would", "could", "should", "may", "might", "must", "shall",
        "can", "to", "of", "in", "for", "on", "with", "at", "by", "from", "or", "and", "not", "no",
        "but", "if", "then", "else", "this", "that", "these", "those", "it", "its",
    ];

    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
        .filter(|w| w.len() > 2 && !stopwords.contains(w))
        .take(20)
        .map(|s| s.to_string())
        .collect()
}

/// Infer domain from command name
fn infer_domain(command: &str) -> String {
    match command {
        "systemctl" | "journalctl" | "systemd-analyze" => "services.systemd".to_string(),
        "pacman" | "yay" | "paru" | "pamac" => "packages".to_string(),
        "free" | "vmstat" | "top" | "htop" => "performance.memory".to_string(),
        "df" | "du" | "lsblk" | "fdisk" | "mount" => "storage.disk".to_string(),
        "ip" | "ss" | "ping" | "netstat" | "nmcli" => "network".to_string(),
        "docker" | "podman" => "containers".to_string(),
        "git" => "development.git".to_string(),
        _ => "general".to_string(),
    }
}

/// Truncate text to max length
fn truncate(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len - 3])
    }
}

/// Get current Unix epoch seconds
fn now_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_entry_creation() {
        let entry =
            EvidenceEntry::probe_output("free", "Mem: 16000 8000 8000", "performance.memory");
        assert!(entry.id.starts_with("probe:free:"));
        assert_eq!(entry.evidence_type, EvidenceType::ProbeOutput);
        assert!(entry.citation.unwrap().contains("probe:free"));
    }

    #[test]
    fn test_cache_add_and_prune() {
        let mut cache = EvidenceCache::new(5);

        for i in 0..10 {
            cache.add(EvidenceEntry::probe_output(
                &format!("probe{}", i),
                "output",
                "test",
            ));
        }

        // Should have pruned to max 5
        assert_eq!(cache.len(), 5);
    }

    #[test]
    fn test_cache_search() {
        let mut cache = EvidenceCache::new(100);
        cache.add(EvidenceEntry::probe_output(
            "free",
            "memory available 8000",
            "memory",
        ));
        cache.add(EvidenceEntry::probe_output("df", "disk usage 50%", "disk"));

        let results = cache.search(&["memory"]);
        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("memory"));
    }

    #[test]
    fn test_keyword_extraction() {
        let keywords = extract_keywords("The systemd service failed to start");
        assert!(keywords.contains(&"systemd".to_string()));
        assert!(keywords.contains(&"service".to_string()));
        assert!(keywords.contains(&"failed".to_string()));
        assert!(!keywords.contains(&"the".to_string())); // Stopword
    }

    #[test]
    fn test_domain_inference() {
        assert_eq!(infer_domain("systemctl"), "services.systemd");
        assert_eq!(infer_domain("pacman"), "packages");
        assert_eq!(infer_domain("free"), "performance.memory");
        assert_eq!(infer_domain("unknown"), "general");
    }
}
