// v0.0.702: Settings Archive V2 (Phase 278)
// Archive records and boxes

use serde::{Deserialize, Serialize};

/// Archive record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveRecord {
    /// Record ID
    pub id: String,
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Archived date
    pub archived_date: String,
    /// Expiry date
    pub expiry_date: Option<String>,
}

impl ArchiveRecord {
    /// Create new record
    pub fn new(id: impl Into<String>, key: impl Into<String>, value: impl Into<String>, archived_date: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            key: key.into(),
            value: value.into(),
            archived_date: archived_date.into(),
            expiry_date: None,
        }
    }

    /// Set expiry
    pub fn expiry(mut self, date: impl Into<String>) -> Self {
        self.expiry_date = Some(date.into());
        self
    }
}

/// Archive box
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveBox {
    /// Box ID
    pub id: String,
    /// Label
    pub label: String,
    /// Records
    pub records: Vec<ArchiveRecord>,
    /// Sealed
    pub sealed: bool,
}

impl ArchiveBox {
    /// Create new box
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            records: Vec::new(),
            sealed: false,
        }
    }

    /// Add record
    pub fn add(&mut self, record: ArchiveRecord) {
        if !self.sealed {
            self.records.push(record);
        }
    }

    /// Seal box
    pub fn seal(&mut self) {
        self.sealed = true;
    }

    /// Record count
    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_new() {
        let r = ArchiveRecord::new("r1", "key", "value", "2025-12-15");
        assert_eq!(r.id, "r1");
    }

    #[test]
    fn test_record_expiry() {
        let r = ArchiveRecord::new("r1", "key", "value", "2025-12-15").expiry("2026-12-15");
        assert!(r.expiry_date.is_some());
    }

    #[test]
    fn test_box_new() {
        let b = ArchiveBox::new("b1", "Box 1");
        assert_eq!(b.record_count(), 0);
    }

    #[test]
    fn test_box_add() {
        let mut b = ArchiveBox::new("b1", "Box 1");
        b.add(ArchiveRecord::new("r1", "key", "value", "2025-12-15"));
        assert_eq!(b.record_count(), 1);
    }

    #[test]
    fn test_box_seal() {
        let mut b = ArchiveBox::new("b1", "Box 1");
        b.seal();
        assert!(b.sealed);
        b.add(ArchiveRecord::new("r1", "key", "value", "2025-12-15"));
        assert_eq!(b.record_count(), 0); // Can't add to sealed box
    }
}
