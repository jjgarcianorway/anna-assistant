// v0.0.658: Settings Archiver Metadata (Phase 234)
// Metadata and result types for archiver

use serde::{Deserialize, Serialize};

use super::types::{ArchiveFormat, ArchiveType};

/// Archive metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveMetadata {
    /// Archive ID
    pub id: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Archive type
    pub archive_type: ArchiveType,
    /// Format
    pub format: ArchiveFormat,
    /// Key count
    pub key_count: usize,
    /// Description
    pub description: Option<String>,
}

impl ArchiveMetadata {
    /// Create new metadata
    pub fn new(id: impl Into<String>, archive_type: ArchiveType, format: ArchiveFormat) -> Self {
        Self {
            id: id.into(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            archive_type,
            format,
            key_count: 0,
            description: None,
        }
    }

    /// With description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set key count
    pub fn with_key_count(mut self, count: usize) -> Self {
        self.key_count = count;
        self
    }
}

/// Archive result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveResult {
    /// Archive data (serialized)
    pub data: String,
    /// Metadata
    pub metadata: ArchiveMetadata,
    /// Keys archived
    pub keys_archived: Vec<String>,
    /// Keys skipped
    pub keys_skipped: Vec<String>,
}

impl ArchiveResult {
    /// Create new result
    pub fn new(metadata: ArchiveMetadata) -> Self {
        Self {
            data: String::new(),
            metadata,
            keys_archived: Vec::new(),
            keys_skipped: Vec::new(),
        }
    }

    /// Set data
    pub fn with_data(mut self, data: String) -> Self {
        self.data = data;
        self
    }

    /// Add archived key
    pub fn add_archived(&mut self, key: String) {
        self.keys_archived.push(key);
    }

    /// Add skipped key
    pub fn add_skipped(&mut self, key: String) {
        self.keys_skipped.push(key);
    }

    /// Total archived
    pub fn total_archived(&self) -> usize {
        self.keys_archived.len()
    }

    /// Has skipped
    pub fn has_skipped(&self) -> bool {
        !self.keys_skipped.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_new() {
        let m = ArchiveMetadata::new("test", ArchiveType::Full, ArchiveFormat::Json);
        assert_eq!(m.id, "test");
    }

    #[test]
    fn test_metadata_with_description() {
        let m = ArchiveMetadata::new("test", ArchiveType::Full, ArchiveFormat::Json)
            .with_description("My backup");
        assert_eq!(m.description, Some("My backup".to_string()));
    }

    #[test]
    fn test_result_new() {
        let m = ArchiveMetadata::new("test", ArchiveType::Full, ArchiveFormat::Json);
        let r = ArchiveResult::new(m);
        assert_eq!(r.total_archived(), 0);
    }

    #[test]
    fn test_result_add_archived() {
        let m = ArchiveMetadata::new("test", ArchiveType::Full, ArchiveFormat::Json);
        let mut r = ArchiveResult::new(m);
        r.add_archived("key1".to_string());
        assert_eq!(r.total_archived(), 1);
    }
}
