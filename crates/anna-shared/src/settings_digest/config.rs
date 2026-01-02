// v0.0.709: Digest Configuration (Phase 285)
// Configuration for digest generation

use serde::{Deserialize, Serialize};
use super::types::{DigestType, DigestFormat};

/// Digest config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestConfig {
    /// Name
    pub name: String,
    /// Digest type
    pub digest_type: DigestType,
    /// Format
    pub format: DigestFormat,
    /// Max sections
    pub max_sections: usize,
}

impl DigestConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            digest_type: DigestType::Daily,
            format: DigestFormat::Summary,
            max_sections: 20,
        }
    }

    /// Set type
    pub fn digest_type(mut self, dt: DigestType) -> Self {
        self.digest_type = dt;
        self
    }

    /// Set format
    pub fn format(mut self, f: DigestFormat) -> Self {
        self.format = f;
        self
    }

    /// Set max sections
    pub fn max_sections(mut self, max: usize) -> Self {
        self.max_sections = max;
        self
    }
}

impl Default for DigestConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = DigestConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = DigestConfig::new("test")
            .digest_type(DigestType::Weekly)
            .format(DigestFormat::Highlights);
        assert_eq!(c.digest_type, DigestType::Weekly);
        assert_eq!(c.format, DigestFormat::Highlights);
    }
}
