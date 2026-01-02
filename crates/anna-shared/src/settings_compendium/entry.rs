// v0.0.700: Settings Compendium (Phase 276) - Milestone!
// Compendium entry

use serde::{Deserialize, Serialize};

/// Compendium entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompendiumEntry {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Article ID
    pub article_id: String,
    /// Definition
    pub definition: Option<String>,
}

impl CompendiumEntry {
    /// Create new entry
    pub fn new(key: impl Into<String>, value: impl Into<String>, article_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            article_id: article_id.into(),
            definition: None,
        }
    }

    /// Set definition
    pub fn definition(mut self, def: impl Into<String>) -> Self {
        self.definition = Some(def.into());
        self
    }
}
