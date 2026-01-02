// v0.0.778: Settings Aviary (Phase 354)
// Aviary keeper

use serde::{Deserialize, Serialize};

/// Aviary keeper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AviaryKeeper {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Bird ID
    pub bird_id: String,
}

impl AviaryKeeper {
    /// Create new keeper
    pub fn new(key: impl Into<String>, name: impl Into<String>, bird_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            bird_id: bird_id.into(),
        }
    }
}
