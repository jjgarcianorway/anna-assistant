// v0.0.762: Settings Field Farmer (Phase 338)
// Field farmer management

use serde::{Deserialize, Serialize};

/// Field farmer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldFarmer {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Crop ID
    pub crop_id: String,
}

impl FieldFarmer {
    /// Create new farmer
    pub fn new(key: impl Into<String>, name: impl Into<String>, crop_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            crop_id: crop_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_farmer_new() {
        let f = FieldFarmer::new("key", "name", "c1");
        assert_eq!(f.crop_id, "c1");
    }
}
