// v0.0.747: Settings Region Policy (Phase 323)
// Region policy management

use serde::{Deserialize, Serialize};

/// Region policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionPolicy {
    /// Policy ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Sector number
    pub sector: u32,
    /// Regional
    pub regional: bool,
}

impl RegionPolicy {
    /// Create new policy
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            sector: 0,
            regional: true,
        }
    }

    /// Set sector
    pub fn sector(mut self, s: u32) -> Self {
        self.sector = s;
        self
    }

    /// Make regional
    pub fn make_regional(&mut self) {
        self.regional = true;
    }

    /// Make local
    pub fn make_local(&mut self) {
        self.regional = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_new() {
        let p = RegionPolicy::new("p1", "Title", "Content");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_policy_builder() {
        let p = RegionPolicy::new("p1", "Title", "Content")
            .sector(1);
        assert_eq!(p.sector, 1);
    }

    #[test]
    fn test_policy_regional() {
        let mut p = RegionPolicy::new("p1", "Title", "Content");
        p.make_local();
        assert!(!p.regional);
        p.make_regional();
        assert!(p.regional);
    }
}
