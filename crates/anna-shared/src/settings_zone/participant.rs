// v0.0.742: Zone Participant (Phase 318)

use serde::{Deserialize, Serialize};

/// Zone participant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneParticipant {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Regulation ID
    pub regulation_id: String,
}

impl ZoneParticipant {
    /// Create new participant
    pub fn new(key: impl Into<String>, name: impl Into<String>, regulation_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            regulation_id: regulation_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_participant_new() {
        let p = ZoneParticipant::new("key", "name", "r1");
        assert_eq!(p.regulation_id, "r1");
    }
}
