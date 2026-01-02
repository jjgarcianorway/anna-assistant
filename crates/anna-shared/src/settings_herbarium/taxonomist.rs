// v0.0.774: Settings Herbarium - Taxonomist
// Herbarium taxonomist management

use serde::{Deserialize, Serialize};

/// Herbarium taxonomist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HerbariumTaxonomist {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Specimen ID
    pub specimen_id: String,
}

impl HerbariumTaxonomist {
    /// Create new taxonomist
    pub fn new(key: impl Into<String>, name: impl Into<String>, specimen_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            specimen_id: specimen_id.into(),
        }
    }
}
