// v0.0.707: Settings Journal (Phase 283)
// Journal configuration

use serde::{Deserialize, Serialize};
use super::enums::JournalType;

/// Journal config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalConfig {
    /// Name
    pub name: String,
    /// Journal type
    pub journal_type: JournalType,
    /// Max entries
    pub max_entries: usize,
    /// Private
    pub private: bool,
}

impl JournalConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            journal_type: JournalType::Personal,
            max_entries: 1000,
            private: true,
        }
    }

    /// Set type
    pub fn journal_type(mut self, jt: JournalType) -> Self {
        self.journal_type = jt;
        self
    }

    /// Set max entries
    pub fn max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// Set private
    pub fn private(mut self, p: bool) -> Self {
        self.private = p;
        self
    }
}

impl Default for JournalConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = JournalConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = JournalConfig::new("test")
            .journal_type(JournalType::Research)
            .private(false);
        assert_eq!(c.journal_type, JournalType::Research);
        assert!(!c.private);
    }
}
