// v0.0.697: Settings Dossier Entry (Phase 273)
// Individual dossier entry structure

use serde::{Deserialize, Serialize};

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
