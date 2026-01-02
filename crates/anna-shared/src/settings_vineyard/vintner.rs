// v0.0.767: Settings Vineyard Vintner
// Vintner managing vines

use serde::{Deserialize, Serialize};

/// Vineyard vintner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VineyardVintner {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Vine ID
    pub vine_id: String,
}

impl VineyardVintner {
    /// Create new vintner
    pub fn new(key: impl Into<String>, name: impl Into<String>, vine_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            vine_id: vine_id.into(),
        }
    }
}
