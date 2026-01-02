// v0.0.713: Settings Notice Registry (Phase 289)
// Registry for managing multiple notice systems

use std::collections::HashMap;
use super::notice::SettingsNotice;

/// Notice registry
#[derive(Debug, Clone, Default)]
pub struct NoticeRegistry {
    /// Notices by ID
    notices: HashMap<String, SettingsNotice>,
}

impl NoticeRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register notice
    pub fn register(&mut self, id: impl Into<String>, notice: SettingsNotice) {
        self.notices.insert(id.into(), notice);
    }

    /// Unregister notice
    pub fn unregister(&mut self, id: &str) -> bool {
        self.notices.remove(id).is_some()
    }

    /// Get notice
    pub fn get(&self, id: &str) -> Option<&SettingsNotice> {
        self.notices.get(id)
    }

    /// Get notice mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsNotice> {
        self.notices.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.notices.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_notice::config::NoticeConfig;

    #[test]
    fn test_registry_new() {
        let r = NoticeRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = NoticeRegistry::new();
        r.register("n1", SettingsNotice::new(NoticeConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
