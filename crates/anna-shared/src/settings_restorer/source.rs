// v0.0.659: Settings Restorer - Restore Source
// Source for restore operations

use serde::{Deserialize, Serialize};

/// Restore source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreSource {
    /// Archive ID
    pub archive_id: String,
    /// Archive data
    pub data: String,
    /// Timestamp
    pub timestamp: u64,
    /// Priority
    pub priority: u32,
}

impl RestoreSource {
    /// Create new source
    pub fn new(archive_id: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            archive_id: archive_id.into(),
            data: data.into(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            priority: 0,
        }
    }

    /// With timestamp
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// With priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_new() {
        let s = RestoreSource::new("archive_1", "{\"key\":\"value\"}");
        assert_eq!(s.archive_id, "archive_1");
    }

    #[test]
    fn test_source_with_priority() {
        let s = RestoreSource::new("archive_1", "data").with_priority(10);
        assert_eq!(s.priority, 10);
    }
}
