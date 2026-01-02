// v0.0.746: Settings Province - Edict (Phase 322)
// Province edicts

use serde::{Deserialize, Serialize};

/// Province edict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvinceEdict {
    /// Edict ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Prefecture number
    pub prefecture: u32,
    /// Provincial
    pub provincial: bool,
}

impl ProvinceEdict {
    /// Create new edict
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            prefecture: 0,
            provincial: true,
        }
    }

    /// Set prefecture
    pub fn prefecture(mut self, p: u32) -> Self {
        self.prefecture = p;
        self
    }

    /// Make provincial
    pub fn make_provincial(&mut self) {
        self.provincial = true;
    }

    /// Make local
    pub fn make_local(&mut self) {
        self.provincial = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edict_new() {
        let e = ProvinceEdict::new("e1", "Title", "Content");
        assert_eq!(e.id, "e1");
    }

    #[test]
    fn test_edict_builder() {
        let e = ProvinceEdict::new("e1", "Title", "Content")
            .prefecture(1);
        assert_eq!(e.prefecture, 1);
    }

    #[test]
    fn test_edict_provincial() {
        let mut e = ProvinceEdict::new("e1", "Title", "Content");
        e.make_local();
        assert!(!e.provincial);
        e.make_provincial();
        assert!(e.provincial);
    }
}
