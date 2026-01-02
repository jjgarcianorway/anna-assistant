// v0.0.702: Settings Archive V2 (Phase 278)
// Archive configuration

use serde::{Deserialize, Serialize};
use super::types::{ArchiveTypeV2, ArchiveRetention};

/// Archive config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveConfigV2 {
    /// Name
    pub name: String,
    /// Archive type
    pub archive_type: ArchiveTypeV2,
    /// Retention
    pub retention: ArchiveRetention,
    /// Max records
    pub max_records: usize,
}

impl ArchiveConfigV2 {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            archive_type: ArchiveTypeV2::Cold,
            retention: ArchiveRetention::Days30,
            max_records: 10000,
        }
    }

    /// Set type
    pub fn archive_type(mut self, at: ArchiveTypeV2) -> Self {
        self.archive_type = at;
        self
    }

    /// Set retention
    pub fn retention(mut self, ret: ArchiveRetention) -> Self {
        self.retention = ret;
        self
    }

    /// Set max records
    pub fn max_records(mut self, max: usize) -> Self {
        self.max_records = max;
        self
    }
}

impl Default for ArchiveConfigV2 {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = ArchiveConfigV2::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ArchiveConfigV2::new("test")
            .archive_type(ArchiveTypeV2::Glacier)
            .retention(ArchiveRetention::Year1);
        assert_eq!(c.archive_type, ArchiveTypeV2::Glacier);
        assert_eq!(c.retention, ArchiveRetention::Year1);
    }
}
