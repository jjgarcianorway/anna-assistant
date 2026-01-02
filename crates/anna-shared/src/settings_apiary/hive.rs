// v0.0.779: Settings Apiary - Hive (Phase 355)
// Apiary hive management

use serde::{Deserialize, Serialize};

/// Apiary hive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiaryHive {
    /// Hive ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Stand number
    pub stand: u32,
    /// Productive
    pub productive: bool,
}

impl ApiaryHive {
    /// Create new hive
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            stand: 0,
            productive: true,
        }
    }

    /// Set stand
    pub fn stand(mut self, s: u32) -> Self {
        self.stand = s;
        self
    }

    /// Make productive
    pub fn make_productive(&mut self) {
        self.productive = true;
    }

    /// Make dormant
    pub fn make_dormant(&mut self) {
        self.productive = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hive_new() {
        let h = ApiaryHive::new("h1", "Title", "Content");
        assert_eq!(h.id, "h1");
    }

    #[test]
    fn test_hive_builder() {
        let h = ApiaryHive::new("h1", "Title", "Content")
            .stand(1);
        assert_eq!(h.stand, 1);
    }

    #[test]
    fn test_hive_productive() {
        let mut h = ApiaryHive::new("h1", "Title", "Content");
        h.make_dormant();
        assert!(!h.productive);
        h.make_productive();
        assert!(h.productive);
    }
}
