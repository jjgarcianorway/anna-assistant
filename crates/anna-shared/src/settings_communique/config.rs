// v0.0.715: Settings Communique - Config (Phase 291)
// Communique configuration

use serde::{Deserialize, Serialize};
use super::types::{CommuniqueType, CommuniqueClassification};

/// Communique config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommuniqueConfig {
    /// Name
    pub name: String,
    /// Communique type
    pub communique_type: CommuniqueType,
    /// Classification
    pub classification: CommuniqueClassification,
    /// Max messages
    pub max_messages: usize,
}

impl CommuniqueConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            communique_type: CommuniqueType::Official,
            classification: CommuniqueClassification::Public,
            max_messages: 300,
        }
    }

    /// Set type
    pub fn communique_type(mut self, ct: CommuniqueType) -> Self {
        self.communique_type = ct;
        self
    }

    /// Set classification
    pub fn classification(mut self, c: CommuniqueClassification) -> Self {
        self.classification = c;
        self
    }

    /// Set max messages
    pub fn max_messages(mut self, max: usize) -> Self {
        self.max_messages = max;
        self
    }
}

impl Default for CommuniqueConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = CommuniqueConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = CommuniqueConfig::new("test")
            .communique_type(CommuniqueType::Diplomatic)
            .classification(CommuniqueClassification::Restricted);
        assert_eq!(c.communique_type, CommuniqueType::Diplomatic);
        assert_eq!(c.classification, CommuniqueClassification::Restricted);
    }
}
