// v0.0.749: Settings County Ordinance (Phase 325)
// County ordinance management

use serde::{Deserialize, Serialize};

/// County ordinance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountyOrdinance {
    /// Ordinance ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Township number
    pub township: u32,
    /// Enacted
    pub enacted: bool,
}

impl CountyOrdinance {
    /// Create new ordinance
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            township: 0,
            enacted: true,
        }
    }

    /// Set township
    pub fn township(mut self, t: u32) -> Self {
        self.township = t;
        self
    }

    /// Make enacted
    pub fn make_enacted(&mut self) {
        self.enacted = true;
    }

    /// Make repealed
    pub fn make_repealed(&mut self) {
        self.enacted = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ordinance_new() {
        let o = CountyOrdinance::new("o1", "Title", "Content");
        assert_eq!(o.id, "o1");
    }

    #[test]
    fn test_ordinance_builder() {
        let o = CountyOrdinance::new("o1", "Title", "Content")
            .township(1);
        assert_eq!(o.township, 1);
    }

    #[test]
    fn test_ordinance_enacted() {
        let mut o = CountyOrdinance::new("o1", "Title", "Content");
        o.make_repealed();
        assert!(!o.enacted);
        o.make_enacted();
        assert!(o.enacted);
    }
}
