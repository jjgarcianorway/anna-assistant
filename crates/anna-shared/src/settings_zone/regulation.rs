// v0.0.742: Zone Regulation (Phase 318)

use serde::{Deserialize, Serialize};

/// Zone regulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneRegulation {
    /// Regulation ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Sector number
    pub sector: u32,
    /// Enforced
    pub enforced: bool,
}

impl ZoneRegulation {
    /// Create new regulation
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            sector: 0,
            enforced: true,
        }
    }

    /// Set sector
    pub fn sector(mut self, s: u32) -> Self {
        self.sector = s;
        self
    }

    /// Make enforced
    pub fn make_enforced(&mut self) {
        self.enforced = true;
    }

    /// Make advisory
    pub fn make_advisory(&mut self) {
        self.enforced = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regulation_new() {
        let r = ZoneRegulation::new("r1", "Title", "Content");
        assert_eq!(r.id, "r1");
    }

    #[test]
    fn test_regulation_builder() {
        let r = ZoneRegulation::new("r1", "Title", "Content")
            .sector(1);
        assert_eq!(r.sector, 1);
    }

    #[test]
    fn test_regulation_enforced() {
        let mut r = ZoneRegulation::new("r1", "Title", "Content");
        r.make_advisory();
        assert!(!r.enforced);
        r.make_enforced();
        assert!(r.enforced);
    }
}
