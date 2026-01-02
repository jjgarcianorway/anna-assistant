// v0.0.708: Settings Memo (Phase 284)
// Memo configuration

use serde::{Deserialize, Serialize};
use super::types::MemoType;

/// Memo config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoConfig {
    /// Name
    pub name: String,
    /// Memo type
    pub memo_type: MemoType,
    /// Max memos
    pub max_memos: usize,
    /// Require acknowledgment
    pub require_ack: bool,
}

impl MemoConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            memo_type: MemoType::Internal,
            max_memos: 500,
            require_ack: false,
        }
    }

    /// Set type
    pub fn memo_type(mut self, mt: MemoType) -> Self {
        self.memo_type = mt;
        self
    }

    /// Set max memos
    pub fn max_memos(mut self, max: usize) -> Self {
        self.max_memos = max;
        self
    }

    /// Set require acknowledgment
    pub fn require_ack(mut self, req: bool) -> Self {
        self.require_ack = req;
        self
    }
}

impl Default for MemoConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = MemoConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = MemoConfig::new("test")
            .memo_type(MemoType::Confidential)
            .require_ack(true);
        assert_eq!(c.memo_type, MemoType::Confidential);
        assert!(c.require_ack);
    }
}
