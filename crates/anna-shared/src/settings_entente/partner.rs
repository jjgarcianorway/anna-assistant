// v0.0.734: Settings Entente (Phase 310)
// Entente partner

use serde::{Deserialize, Serialize};

/// Entente partner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntentePartner {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Understanding ID
    pub understanding_id: String,
}

impl EntentePartner {
    /// Create new partner
    pub fn new(key: impl Into<String>, name: impl Into<String>, understanding_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            understanding_id: understanding_id.into(),
        }
    }
}
