// v0.0.697: Settings Dossier Main (Phase 273)
// Main settings dossier implementation

use super::config::DossierConfig;
use super::document::DossierDocument;
use super::entry::DossierEntry;
use super::stats::DossierStats;

/// Settings dossier
#[derive(Debug, Clone, Default)]
pub struct SettingsDossier {
    /// Config
    config: DossierConfig,
    /// Documents
    documents: Vec<DossierDocument>,
    /// Entries
    entries: Vec<DossierEntry>,
    /// Stats
    stats: DossierStats,
}

impl SettingsDossier {
    /// Create new dossier
    pub fn new(config: DossierConfig) -> Self {
        Self {
            config,
            documents: Vec::new(),
            entries: Vec::new(),
            stats: DossierStats::default(),
        }
    }

    /// Add document
    pub fn add_document(&mut self, document: DossierDocument) -> bool {
        if self.documents.len() >= self.config.max_documents {
            return false;
        }
        self.documents.push(document);
        self.update_stats();
        true
    }

    /// Get document
    pub fn get_document(&self, id: &str) -> Option<&DossierDocument> {
        self.documents.iter().find(|d| d.id == id)
    }

    /// Add entry
    pub fn add_entry(&mut self, entry: DossierEntry) {
        self.entries.push(entry);
        self.stats.record_entry();
    }

    /// Get entries for document
    pub fn get_entries(&self, document_id: &str) -> Vec<&DossierEntry> {
        self.entries.iter().filter(|e| e.document_id == document_id).collect()
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.documents);
    }

    /// Get stats
    pub fn stats(&self) -> &DossierStats {
        &self.stats
    }

    /// Document count
    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    /// Entry count
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}
