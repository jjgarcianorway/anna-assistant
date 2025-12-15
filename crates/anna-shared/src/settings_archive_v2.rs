// v0.0.702: Settings Archive V2 (Phase 278)
// Long-term archive of settings history

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Archive type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ArchiveTypeV2 {
    /// Cold archive
    #[default]
    Cold,
    /// Warm archive
    Warm,
    /// Deep archive
    Deep,
    /// Glacier archive
    Glacier,
}

impl std::fmt::Display for ArchiveTypeV2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cold => write!(f, "cold"),
            Self::Warm => write!(f, "warm"),
            Self::Deep => write!(f, "deep"),
            Self::Glacier => write!(f, "glacier"),
        }
    }
}

/// Archive retention
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ArchiveRetention {
    /// 30 days
    #[default]
    Days30,
    /// 90 days
    Days90,
    /// 1 year
    Year1,
    /// Indefinite
    Indefinite,
}

impl std::fmt::Display for ArchiveRetention {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Days30 => write!(f, "30d"),
            Self::Days90 => write!(f, "90d"),
            Self::Year1 => write!(f, "1y"),
            Self::Indefinite => write!(f, "indefinite"),
        }
    }
}

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

/// Archive stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArchiveStatsV2 {
    /// Total boxes
    pub total_boxes: usize,
    /// Total records
    pub total_records: usize,
    /// Sealed boxes
    pub sealed_boxes: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ArchiveStatsV2 {
    /// Update from archive
    pub fn update(&mut self, boxes: &[ArchiveBox], archive_type: ArchiveTypeV2) {
        self.total_boxes = boxes.len();
        self.total_records = boxes.iter().map(|b| b.record_count()).sum();
        self.sealed_boxes = boxes.iter().filter(|b| b.sealed).count();
        *self.by_type.entry(archive_type.to_string()).or_insert(0) += 1;
    }

    /// Sealed rate
    pub fn sealed_rate(&self) -> f64 {
        if self.total_boxes == 0 { 0.0 } else { self.sealed_boxes as f64 / self.total_boxes as f64 * 100.0 }
    }
}

/// Settings archive v2
#[derive(Debug, Clone, Default)]
pub struct SettingsArchiveV2 {
    /// Config
    config: ArchiveConfigV2,
    /// Boxes
    boxes: Vec<ArchiveBox>,
    /// Stats
    stats: ArchiveStatsV2,
}

impl SettingsArchiveV2 {
    /// Create new archive
    pub fn new(config: ArchiveConfigV2) -> Self {
        Self {
            config,
            boxes: Vec::new(),
            stats: ArchiveStatsV2::default(),
        }
    }

    /// Add box
    pub fn add_box(&mut self, archive_box: ArchiveBox) {
        self.boxes.push(archive_box);
        self.update_stats();
    }

    /// Get box
    pub fn get_box(&self, id: &str) -> Option<&ArchiveBox> {
        self.boxes.iter().find(|b| b.id == id)
    }

    /// Get box mut
    pub fn get_box_mut(&mut self, id: &str) -> Option<&mut ArchiveBox> {
        self.boxes.iter_mut().find(|b| b.id == id)
    }

    /// Archive record to box
    pub fn archive_to_box(&mut self, box_id: &str, record: ArchiveRecord) -> bool {
        if let Some(archive_box) = self.get_box_mut(box_id) {
            archive_box.add(record);
            self.update_stats();
            true
        } else {
            false
        }
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.boxes, self.config.archive_type);
    }

    /// Get stats
    pub fn stats(&self) -> &ArchiveStatsV2 {
        &self.stats
    }

    /// Box count
    pub fn box_count(&self) -> usize {
        self.boxes.len()
    }
}

/// Archive registry v2
#[derive(Debug, Clone, Default)]
pub struct ArchiveRegistryV2 {
    /// Archives by ID
    archives: HashMap<String, SettingsArchiveV2>,
}

impl ArchiveRegistryV2 {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register archive
    pub fn register(&mut self, id: impl Into<String>, archive: SettingsArchiveV2) {
        self.archives.insert(id.into(), archive);
    }

    /// Unregister archive
    pub fn unregister(&mut self, id: &str) -> bool {
        self.archives.remove(id).is_some()
    }

    /// Get archive
    pub fn get(&self, id: &str) -> Option<&SettingsArchiveV2> {
        self.archives.get(id)
    }

    /// Get archive mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsArchiveV2> {
        self.archives.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.archives.len()
    }
}

/// Format archive registry v2
pub fn format_archive_registry_v2(registry: &ArchiveRegistryV2) -> String {
    let mut output = String::new();
    output.push_str("Settings Archive V2 Registry:\n");
    output.push_str(&format!("  Archives: {}\n", registry.count()));
    output
}

/// Check if query is about archive v2
pub fn is_archive_v2_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings archive") || lower.contains("archive settings") || lower.contains("long-term storage")
}

/// Fun fact about archive v2
pub fn archive_v2_fun_fact() -> &'static str {
    "Anna's settings archive v2 preserves your configurations for long-term storage!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_type_display() {
        assert_eq!(format!("{}", ArchiveTypeV2::Cold), "cold");
        assert_eq!(format!("{}", ArchiveTypeV2::Glacier), "glacier");
    }

    #[test]
    fn test_retention_display() {
        assert_eq!(format!("{}", ArchiveRetention::Days30), "30d");
        assert_eq!(format!("{}", ArchiveRetention::Indefinite), "indefinite");
    }

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

    #[test]
    fn test_stats_update() {
        let mut s = ArchiveStatsV2::default();
        let boxes = vec![ArchiveBox::new("b1", "Box")];
        s.update(&boxes, ArchiveTypeV2::Cold);
        assert_eq!(s.total_boxes, 1);
    }

    #[test]
    fn test_archive_new() {
        let a = SettingsArchiveV2::new(ArchiveConfigV2::default());
        assert_eq!(a.box_count(), 0);
    }

    #[test]
    fn test_archive_add_box() {
        let mut a = SettingsArchiveV2::new(ArchiveConfigV2::default());
        a.add_box(ArchiveBox::new("b1", "Box 1"));
        assert_eq!(a.box_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = ArchiveRegistryV2::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ArchiveRegistryV2::new();
        r.register("a1", SettingsArchiveV2::new(ArchiveConfigV2::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_archive_v2_query() {
        assert!(is_archive_v2_query("settings archive"));
        assert!(!is_archive_v2_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = archive_v2_fun_fact();
        assert!(fact.contains("archive"));
    }
}
