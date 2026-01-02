// v0.0.757: Settings Parcel - Examiner (Phase 333)

use serde::{Deserialize, Serialize};

/// Parcel examiner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParcelExaminer {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Title ID
    pub title_id: String,
}

impl ParcelExaminer {
    /// Create new examiner
    pub fn new(key: impl Into<String>, name: impl Into<String>, title_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            title_id: title_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_examiner_new() {
        let e = ParcelExaminer::new("key", "name", "t1");
        assert_eq!(e.title_id, "t1");
    }
}
