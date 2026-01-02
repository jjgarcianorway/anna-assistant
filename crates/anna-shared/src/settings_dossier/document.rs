// v0.0.697: Settings Dossier Document (Phase 273)
// Dossier document structure

use serde::{Deserialize, Serialize};
use super::types::DossierClassification;

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
