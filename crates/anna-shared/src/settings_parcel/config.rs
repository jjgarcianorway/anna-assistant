// v0.0.757: Settings Parcel - Config (Phase 333)

use serde::{Deserialize, Serialize};
use super::types::{ParcelType, ParcelStatus};

/// Parcel config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParcelConfig {
    /// Name
    pub name: String,
    /// Parcel type
    pub parcel_type: ParcelType,
    /// Status
    pub status: ParcelStatus,
    /// Max titles
    pub max_titles: usize,
}

impl ParcelConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            parcel_type: ParcelType::FeeSimple,
            status: ParcelStatus::Platted,
            max_titles: 100,
        }
    }

    /// Set type
    pub fn parcel_type(mut self, pt: ParcelType) -> Self {
        self.parcel_type = pt;
        self
    }

    /// Set status
    pub fn status(mut self, s: ParcelStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max titles
    pub fn max_titles(mut self, max: usize) -> Self {
        self.max_titles = max;
        self
    }
}

impl Default for ParcelConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = ParcelConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ParcelConfig::new("test")
            .parcel_type(ParcelType::Easement)
            .status(ParcelStatus::Encumbered);
        assert_eq!(c.parcel_type, ParcelType::Easement);
        assert_eq!(c.status, ParcelStatus::Encumbered);
    }
}
