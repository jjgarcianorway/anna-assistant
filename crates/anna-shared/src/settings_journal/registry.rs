// v0.0.707: Settings Journal (Phase 283)
// Journal registry

use std::collections::HashMap;
use super::journal::SettingsJournal;

/// Journal registry
#[derive(Debug, Clone, Default)]
pub struct JournalRegistry {
    /// Journals by ID
    journals: HashMap<String, SettingsJournal>,
}

impl JournalRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register journal
    pub fn register(&mut self, id: impl Into<String>, journal: SettingsJournal) {
        self.journals.insert(id.into(), journal);
    }

    /// Unregister journal
    pub fn unregister(&mut self, id: &str) -> bool {
        self.journals.remove(id).is_some()
    }

    /// Get journal
    pub fn get(&self, id: &str) -> Option<&SettingsJournal> {
        self.journals.get(id)
    }

    /// Get journal mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsJournal> {
        self.journals.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.journals.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::config::JournalConfig;

    #[test]
    fn test_registry_new() {
        let r = JournalRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = JournalRegistry::new();
        r.register("j1", SettingsJournal::new(JournalConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
