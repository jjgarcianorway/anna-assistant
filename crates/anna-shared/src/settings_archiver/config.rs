// v0.0.658: Settings Archiver Configuration (Phase 234)
// Configuration types for archiver

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::types::{ArchiveFormat, ArchiveType};

/// Archiver config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiverConfig {
    /// Archive format
    pub format: ArchiveFormat,
    /// Archive type
    pub archive_type: ArchiveType,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Include metadata
    pub include_metadata: bool,
    /// Compress output
    pub compress: bool,
}

impl ArchiverConfig {
    /// Create new config
    pub fn new(format: ArchiveFormat) -> Self {
        Self {
            format,
            archive_type: ArchiveType::Full,
            category: None,
            include_metadata: true,
            compress: false,
        }
    }

    /// Set archive type
    pub fn archive_type(mut self, archive_type: ArchiveType) -> Self {
        self.archive_type = archive_type;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set include metadata
    pub fn include_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    /// Set compress
    pub fn compress(mut self, compress: bool) -> Self {
        self.compress = compress;
        self
    }
}

impl Default for ArchiverConfig {
    fn default() -> Self {
        Self::new(ArchiveFormat::Json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = ArchiverConfig::new(ArchiveFormat::Json);
        assert!(c.include_metadata);
    }

    #[test]
    fn test_config_builder() {
        let c = ArchiverConfig::new(ArchiveFormat::Toml)
            .archive_type(ArchiveType::Snapshot)
            .compress(true);
        assert_eq!(c.archive_type, ArchiveType::Snapshot);
        assert!(c.compress);
    }
}
