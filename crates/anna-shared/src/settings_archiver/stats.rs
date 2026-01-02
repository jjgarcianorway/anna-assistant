// v0.0.658: Settings Archiver Statistics (Phase 234)
// Statistics tracking for archiver operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::ArchiveFormat;

/// Archiver stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArchiverStats {
    /// Total archives created
    pub total_archives: usize,
    /// Total keys archived
    pub total_keys_archived: usize,
    /// Total data size (bytes)
    pub total_data_size: usize,
    /// By format
    pub by_format: HashMap<String, usize>,
}

impl ArchiverStats {
    /// Record archive
    pub fn record(&mut self, format: ArchiveFormat, keys_archived: usize, data_size: usize) {
        self.total_archives += 1;
        self.total_keys_archived += keys_archived;
        self.total_data_size += data_size;
        *self.by_format.entry(format.to_string()).or_insert(0) += 1;
    }

    /// Average archive size
    pub fn average_archive_size(&self) -> f64 {
        if self.total_archives == 0 {
            0.0
        } else {
            self.total_data_size as f64 / self.total_archives as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_record() {
        let mut s = ArchiverStats::default();
        s.record(ArchiveFormat::Json, 10, 500);
        assert_eq!(s.total_archives, 1);
        assert_eq!(s.total_keys_archived, 10);
    }
}
