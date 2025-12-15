// v0.0.697: Settings Dossier (Phase 273)
// Comprehensive dossier of settings information

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Dossier type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DossierType {
    /// Standard dossier
    #[default]
    Standard,
    /// Confidential dossier
    Confidential,
    /// Summary dossier
    Summary,
    /// Full dossier
    Full,
}

impl std::fmt::Display for DossierType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Standard => write!(f, "standard"),
            Self::Confidential => write!(f, "confidential"),
            Self::Summary => write!(f, "summary"),
            Self::Full => write!(f, "full"),
        }
    }
}

/// Dossier classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DossierClassification {
    /// Public
    #[default]
    Public,
    /// Internal
    Internal,
    /// Restricted
    Restricted,
    /// Secret
    Secret,
}

impl std::fmt::Display for DossierClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public => write!(f, "public"),
            Self::Internal => write!(f, "internal"),
            Self::Restricted => write!(f, "restricted"),
            Self::Secret => write!(f, "secret"),
        }
    }
}

/// Dossier config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DossierConfig {
    /// Name
    pub name: String,
    /// Dossier type
    pub dossier_type: DossierType,
    /// Classification
    pub classification: DossierClassification,
    /// Max documents
    pub max_documents: usize,
}

impl DossierConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            dossier_type: DossierType::Standard,
            classification: DossierClassification::Public,
            max_documents: 100,
        }
    }

    /// Set type
    pub fn dossier_type(mut self, dt: DossierType) -> Self {
        self.dossier_type = dt;
        self
    }

    /// Set classification
    pub fn classification(mut self, c: DossierClassification) -> Self {
        self.classification = c;
        self
    }

    /// Set max documents
    pub fn max_documents(mut self, max: usize) -> Self {
        self.max_documents = max;
        self
    }
}

impl Default for DossierConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Dossier document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DossierDocument {
    /// Document ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Classification
    pub classification: DossierClassification,
    /// Created date
    pub created: String,
}

impl DossierDocument {
    /// Create new document
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            classification: DossierClassification::Public,
            created: String::new(),
        }
    }

    /// Set classification
    pub fn classification(mut self, c: DossierClassification) -> Self {
        self.classification = c;
        self
    }

    /// Set created date
    pub fn created(mut self, date: impl Into<String>) -> Self {
        self.created = date.into();
        self
    }

    /// Is restricted
    pub fn is_restricted(&self) -> bool {
        matches!(self.classification, DossierClassification::Restricted | DossierClassification::Secret)
    }
}

/// Dossier entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DossierEntry {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Document ID
    pub document_id: String,
    /// Notes
    pub notes: Option<String>,
}

impl DossierEntry {
    /// Create new entry
    pub fn new(key: impl Into<String>, value: impl Into<String>, document_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            document_id: document_id.into(),
            notes: None,
        }
    }

    /// Set notes
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

/// Dossier stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DossierStats {
    /// Total documents
    pub total_documents: usize,
    /// Total entries
    pub total_entries: usize,
    /// By classification
    pub by_classification: HashMap<String, usize>,
}

impl DossierStats {
    /// Update from dossier
    pub fn update(&mut self, documents: &[DossierDocument]) {
        self.total_documents = documents.len();
        self.by_classification.clear();
        for doc in documents {
            *self.by_classification.entry(doc.classification.to_string()).or_insert(0) += 1;
        }
    }

    /// Record entry
    pub fn record_entry(&mut self) {
        self.total_entries += 1;
    }
}

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

/// Dossier registry
#[derive(Debug, Clone, Default)]
pub struct DossierRegistry {
    /// Dossiers by ID
    dossiers: HashMap<String, SettingsDossier>,
}

impl DossierRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register dossier
    pub fn register(&mut self, id: impl Into<String>, dossier: SettingsDossier) {
        self.dossiers.insert(id.into(), dossier);
    }

    /// Unregister dossier
    pub fn unregister(&mut self, id: &str) -> bool {
        self.dossiers.remove(id).is_some()
    }

    /// Get dossier
    pub fn get(&self, id: &str) -> Option<&SettingsDossier> {
        self.dossiers.get(id)
    }

    /// Get dossier mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsDossier> {
        self.dossiers.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.dossiers.len()
    }
}

/// Format dossier registry
pub fn format_dossier_registry(registry: &DossierRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Dossier Registry:\n");
    output.push_str(&format!("  Dossiers: {}\n", registry.count()));
    output
}

/// Check if query is about dossier
pub fn is_dossier_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings dossier") || lower.contains("dossier settings") || lower.contains("settings file")
}

/// Fun fact about dossier
pub fn dossier_fun_fact() -> &'static str {
    "Anna's settings dossier keeps comprehensive documentation of your configurations!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dossier_type_display() {
        assert_eq!(format!("{}", DossierType::Standard), "standard");
        assert_eq!(format!("{}", DossierType::Confidential), "confidential");
    }

    #[test]
    fn test_classification_display() {
        assert_eq!(format!("{}", DossierClassification::Public), "public");
        assert_eq!(format!("{}", DossierClassification::Secret), "secret");
    }

    #[test]
    fn test_config_new() {
        let c = DossierConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = DossierConfig::new("test")
            .dossier_type(DossierType::Full)
            .classification(DossierClassification::Internal);
        assert_eq!(c.dossier_type, DossierType::Full);
        assert_eq!(c.classification, DossierClassification::Internal);
    }

    #[test]
    fn test_document_new() {
        let d = DossierDocument::new("d1", "Doc 1", "Content");
        assert_eq!(d.id, "d1");
    }

    #[test]
    fn test_document_restricted() {
        let d = DossierDocument::new("d1", "Doc 1", "Content")
            .classification(DossierClassification::Secret);
        assert!(d.is_restricted());
    }

    #[test]
    fn test_entry_new() {
        let e = DossierEntry::new("key", "value", "d1");
        assert_eq!(e.document_id, "d1");
    }

    #[test]
    fn test_entry_notes() {
        let e = DossierEntry::new("key", "value", "d1").notes("important");
        assert!(e.notes.is_some());
    }

    #[test]
    fn test_stats_update() {
        let mut s = DossierStats::default();
        let docs = vec![DossierDocument::new("d1", "Doc", "Content")];
        s.update(&docs);
        assert_eq!(s.total_documents, 1);
    }

    #[test]
    fn test_dossier_new() {
        let d = SettingsDossier::new(DossierConfig::default());
        assert_eq!(d.document_count(), 0);
    }

    #[test]
    fn test_dossier_add_document() {
        let mut d = SettingsDossier::new(DossierConfig::default());
        d.add_document(DossierDocument::new("d1", "Doc 1", "Content"));
        assert_eq!(d.document_count(), 1);
    }

    #[test]
    fn test_dossier_add_entry() {
        let mut d = SettingsDossier::new(DossierConfig::default());
        d.add_entry(DossierEntry::new("key", "value", "d1"));
        assert_eq!(d.entry_count(), 1);
    }

    #[test]
    fn test_dossier_get_entries() {
        let mut d = SettingsDossier::new(DossierConfig::default());
        d.add_entry(DossierEntry::new("key", "value", "d1"));
        let entries = d.get_entries("d1");
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = DossierRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = DossierRegistry::new();
        r.register("d1", SettingsDossier::new(DossierConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_dossier_query() {
        assert!(is_dossier_query("settings dossier"));
        assert!(!is_dossier_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = dossier_fun_fact();
        assert!(fact.contains("dossier"));
    }
}
