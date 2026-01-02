// v0.0.713: Settings Notice Config (Phase 289)
// Notice configuration

use serde::{Deserialize, Serialize};
use super::types::{NoticeType, NoticePriority};

/// Notice config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoticeConfig {
    /// Name
    pub name: String,
    /// Notice type
    pub notice_type: NoticeType,
    /// Default priority
    pub default_priority: NoticePriority,
    /// Max notices
    pub max_notices: usize,
}

impl NoticeConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            notice_type: NoticeType::Information,
            default_priority: NoticePriority::Normal,
            max_notices: 200,
        }
    }

    /// Set type
    pub fn notice_type(mut self, nt: NoticeType) -> Self {
        self.notice_type = nt;
        self
    }

    /// Set default priority
    pub fn default_priority(mut self, dp: NoticePriority) -> Self {
        self.default_priority = dp;
        self
    }

    /// Set max notices
    pub fn max_notices(mut self, max: usize) -> Self {
        self.max_notices = max;
        self
    }
}

impl Default for NoticeConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = NoticeConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = NoticeConfig::new("test")
            .notice_type(NoticeType::Warning)
            .default_priority(NoticePriority::High);
        assert_eq!(c.notice_type, NoticeType::Warning);
        assert_eq!(c.default_priority, NoticePriority::High);
    }
}
