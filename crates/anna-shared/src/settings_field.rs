// v0.0.762: Settings Field (Phase 338)
// Agricultural field for settings cultivation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Field type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FieldType {
    /// Arable field
    #[default]
    Arable,
    /// Pastoral field
    Pastoral,
    /// Fallow field
    Fallow,
    /// Orchard field
    Orchard,
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Arable => write!(f, "arable"),
            Self::Pastoral => write!(f, "pastoral"),
            Self::Fallow => write!(f, "fallow"),
            Self::Orchard => write!(f, "orchard"),
        }
    }
}

/// Field status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FieldStatus {
    /// Prepared status
    #[default]
    Prepared,
    /// Planted status
    Planted,
    /// Growing status
    Growing,
    /// Harvested status
    Harvested,
}

impl std::fmt::Display for FieldStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Prepared => write!(f, "prepared"),
            Self::Planted => write!(f, "planted"),
            Self::Growing => write!(f, "growing"),
            Self::Harvested => write!(f, "harvested"),
        }
    }
}

/// Field config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldConfig {
    /// Name
    pub name: String,
    /// Field type
    pub field_type: FieldType,
    /// Status
    pub status: FieldStatus,
    /// Max crops
    pub max_crops: usize,
}

impl FieldConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            field_type: FieldType::Arable,
            status: FieldStatus::Prepared,
            max_crops: 100,
        }
    }

    /// Set type
    pub fn field_type(mut self, ft: FieldType) -> Self {
        self.field_type = ft;
        self
    }

    /// Set status
    pub fn status(mut self, s: FieldStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max crops
    pub fn max_crops(mut self, max: usize) -> Self {
        self.max_crops = max;
        self
    }
}

impl Default for FieldConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Field crop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldCrop {
    /// Crop ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Furrow number
    pub furrow: u32,
    /// Yielded
    pub yielded: bool,
}

impl FieldCrop {
    /// Create new crop
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            furrow: 0,
            yielded: true,
        }
    }

    /// Set furrow
    pub fn furrow(mut self, f: u32) -> Self {
        self.furrow = f;
        self
    }

    /// Make yielded
    pub fn make_yielded(&mut self) {
        self.yielded = true;
    }

    /// Make barren
    pub fn make_barren(&mut self) {
        self.yielded = false;
    }
}

/// Field farmer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldFarmer {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Crop ID
    pub crop_id: String,
}

impl FieldFarmer {
    /// Create new farmer
    pub fn new(key: impl Into<String>, name: impl Into<String>, crop_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            crop_id: crop_id.into(),
        }
    }
}

/// Field stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldStats {
    /// Total crops
    pub total_crops: usize,
    /// Yielded crops
    pub yielded: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl FieldStats {
    /// Update from crops
    pub fn update(&mut self, crops: &[FieldCrop], field_type: FieldType) {
        self.total_crops = crops.len();
        self.yielded = crops.iter().filter(|c| c.yielded).count();
        *self.by_type.entry(field_type.to_string()).or_insert(0) += 1;
    }

    /// Yield rate
    pub fn yield_rate(&self) -> f64 {
        if self.total_crops == 0 { 0.0 } else { self.yielded as f64 / self.total_crops as f64 * 100.0 }
    }
}

/// Settings field
#[derive(Debug, Clone, Default)]
pub struct SettingsField {
    /// Config
    config: FieldConfig,
    /// Crops
    crops: Vec<FieldCrop>,
    /// Farmers
    farmers: Vec<FieldFarmer>,
    /// Stats
    stats: FieldStats,
}

impl SettingsField {
    /// Create new field system
    pub fn new(config: FieldConfig) -> Self {
        Self {
            config,
            crops: Vec::new(),
            farmers: Vec::new(),
            stats: FieldStats::default(),
        }
    }

    /// Add crop
    pub fn add_crop(&mut self, crop: FieldCrop) -> bool {
        if self.crops.len() >= self.config.max_crops {
            return false;
        }
        self.crops.push(crop);
        self.update_stats();
        true
    }

    /// Get crop
    pub fn get_crop(&self, id: &str) -> Option<&FieldCrop> {
        self.crops.iter().find(|c| c.id == id)
    }

    /// Get crop mut
    pub fn get_crop_mut(&mut self, id: &str) -> Option<&mut FieldCrop> {
        self.crops.iter_mut().find(|c| c.id == id)
    }

    /// Add farmer
    pub fn add_farmer(&mut self, farmer: FieldFarmer) {
        self.farmers.push(farmer);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.crops, self.config.field_type);
    }

    /// Get stats
    pub fn stats(&self) -> &FieldStats {
        &self.stats
    }

    /// Crop count
    pub fn crop_count(&self) -> usize {
        self.crops.len()
    }
}

/// Field registry
#[derive(Debug, Clone, Default)]
pub struct FieldRegistry {
    /// Fields by ID
    fields: HashMap<String, SettingsField>,
}

impl FieldRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register field
    pub fn register(&mut self, id: impl Into<String>, field: SettingsField) {
        self.fields.insert(id.into(), field);
    }

    /// Unregister field
    pub fn unregister(&mut self, id: &str) -> bool {
        self.fields.remove(id).is_some()
    }

    /// Get field
    pub fn get(&self, id: &str) -> Option<&SettingsField> {
        self.fields.get(id)
    }

    /// Get field mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsField> {
        self.fields.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.fields.len()
    }
}

/// Format field registry
pub fn format_field_registry(registry: &FieldRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Field Registry:\n");
    output.push_str(&format!("  Fields: {}\n", registry.count()));
    output
}

/// Check if query is about field
pub fn is_field_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings field") || lower.contains("field settings") || lower.contains("agricultural field")
}

/// Fun fact about field
pub fn field_fun_fact() -> &'static str {
    "Anna's settings field establishes cultivation boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_type_display() {
        assert_eq!(format!("{}", FieldType::Arable), "arable");
        assert_eq!(format!("{}", FieldType::Pastoral), "pastoral");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", FieldStatus::Prepared), "prepared");
        assert_eq!(format!("{}", FieldStatus::Harvested), "harvested");
    }

    #[test]
    fn test_config_new() {
        let c = FieldConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = FieldConfig::new("test")
            .field_type(FieldType::Orchard)
            .status(FieldStatus::Growing);
        assert_eq!(c.field_type, FieldType::Orchard);
        assert_eq!(c.status, FieldStatus::Growing);
    }

    #[test]
    fn test_crop_new() {
        let c = FieldCrop::new("c1", "Title", "Content");
        assert_eq!(c.id, "c1");
    }

    #[test]
    fn test_crop_builder() {
        let c = FieldCrop::new("c1", "Title", "Content")
            .furrow(1);
        assert_eq!(c.furrow, 1);
    }

    #[test]
    fn test_crop_yielded() {
        let mut c = FieldCrop::new("c1", "Title", "Content");
        c.make_barren();
        assert!(!c.yielded);
        c.make_yielded();
        assert!(c.yielded);
    }

    #[test]
    fn test_farmer_new() {
        let f = FieldFarmer::new("key", "name", "c1");
        assert_eq!(f.crop_id, "c1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = FieldStats::default();
        let crop = FieldCrop::new("c1", "Title", "Content");
        s.update(&[crop], FieldType::Arable);
        assert_eq!(s.total_crops, 1);
        assert_eq!(s.yielded, 1);
    }

    #[test]
    fn test_field_new() {
        let f = SettingsField::new(FieldConfig::default());
        assert_eq!(f.crop_count(), 0);
    }

    #[test]
    fn test_field_add_crop() {
        let mut f = SettingsField::new(FieldConfig::default());
        f.add_crop(FieldCrop::new("c1", "Title", "Content"));
        assert_eq!(f.crop_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = FieldRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = FieldRegistry::new();
        r.register("f1", SettingsField::new(FieldConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_field_query() {
        assert!(is_field_query("settings field"));
        assert!(!is_field_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = field_fun_fact();
        assert!(fact.contains("field"));
    }
}
