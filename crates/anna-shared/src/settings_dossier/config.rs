// v0.0.697: Settings Dossier Config (Phase 273)
// Configuration for dossiers

use serde::{Deserialize, Serialize};
use super::types::{DossierType, DossierClassification};

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
