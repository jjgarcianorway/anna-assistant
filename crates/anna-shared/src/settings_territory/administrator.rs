// v0.0.745: Settings Territory - Administrator
// Territory administrator management

use serde::{Deserialize, Serialize};

/// Territory administrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryAdministrator {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Ordinance ID
    pub ordinance_id: String,
}

impl TerritoryAdministrator {
    /// Create new administrator
    pub fn new(key: impl Into<String>, name: impl Into<String>, ordinance_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            ordinance_id: ordinance_id.into(),
        }
    }
}
