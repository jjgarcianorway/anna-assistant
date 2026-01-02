// v0.0.695: Settings Folio (Phase 271)
// Folio section

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Folio section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolioSection {
    /// Section ID
    pub id: String,
    /// Name
    pub name: String,
    /// Settings
    pub settings: HashMap<String, String>,
    /// Order
    pub order: usize,
}

impl FolioSection {
    /// Create new section
    pub fn new(id: impl Into<String>, name: impl Into<String>, order: usize) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            settings: HashMap::new(),
            order,
        }
    }

    /// Add setting
    pub fn add(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.settings.insert(key.into(), value.into());
    }

    /// Get setting
    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    /// Setting count
    pub fn count(&self) -> usize {
        self.settings.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_new() {
        let s = FolioSection::new("s1", "Section 1", 0);
        assert_eq!(s.count(), 0);
    }

    #[test]
    fn test_section_add() {
        let mut s = FolioSection::new("s1", "Section 1", 0);
        s.add("key", "value");
        assert_eq!(s.count(), 1);
    }
}
