// v0.0.750: Settings Municipality Councilor (Phase 326)
// Municipality councilor structure

use serde::{Deserialize, Serialize};

/// Municipality councilor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MunicipalityCouncilor {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Code ID
    pub code_id: String,
}

impl MunicipalityCouncilor {
    /// Create new councilor
    pub fn new(key: impl Into<String>, name: impl Into<String>, code_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            code_id: code_id.into(),
        }
    }
}
