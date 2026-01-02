// v0.0.695: Settings Folio (Phase 271)
// Folio configuration

use serde::{Deserialize, Serialize};
use super::types::FolioType;

/// Folio config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolioConfig {
    /// Name
    pub name: String,
    /// Folio type
    pub folio_type: FolioType,
    /// Description
    pub description: String,
    /// Max sections
    pub max_sections: usize,
}

impl FolioConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            folio_type: FolioType::Active,
            description: String::new(),
            max_sections: 100,
        }
    }

    /// Set type
    pub fn folio_type(mut self, ft: FolioType) -> Self {
        self.folio_type = ft;
        self
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set max sections
    pub fn max_sections(mut self, max: usize) -> Self {
        self.max_sections = max;
        self
    }
}

impl Default for FolioConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = FolioConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = FolioConfig::new("test")
            .folio_type(FolioType::Template)
            .max_sections(50);
        assert_eq!(c.folio_type, FolioType::Template);
        assert_eq!(c.max_sections, 50);
    }
}
