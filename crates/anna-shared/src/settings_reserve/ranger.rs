// v0.0.782: Settings Reserve - Ranger
// Reserve ranger management

use serde::{Deserialize, Serialize};

/// Reserve ranger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReserveRanger {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Species ID
    pub species_id: String,
}

impl ReserveRanger {
    /// Create new ranger
    pub fn new(key: impl Into<String>, name: impl Into<String>, species_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            species_id: species_id.into(),
        }
    }
}
