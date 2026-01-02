use serde::{Deserialize, Serialize};

/// Vivarium keeper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VivariumKeeper {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Creature ID
    pub creature_id: String,
}

impl VivariumKeeper {
    /// Create new keeper
    pub fn new(key: impl Into<String>, name: impl Into<String>, creature_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            creature_id: creature_id.into(),
        }
    }
}
