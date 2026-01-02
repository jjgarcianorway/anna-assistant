// v0.0.697: Settings Dossier Stats (Phase 273)
// Statistics tracking for dossiers

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::document::DossierDocument;

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
