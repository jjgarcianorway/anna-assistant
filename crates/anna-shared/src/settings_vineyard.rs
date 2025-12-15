// v0.0.767: Settings Vineyard (Phase 343)
// Grape vineyard for settings viticulture

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Vineyard type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum VineyardType {
    /// Red wine vineyard
    #[default]
    RedWine,
    /// White wine vineyard
    WhiteWine,
    /// Table grape vineyard
    TableGrape,
    /// Raisin vineyard
    Raisin,
}

impl std::fmt::Display for VineyardType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RedWine => write!(f, "red-wine"),
            Self::WhiteWine => write!(f, "white-wine"),
            Self::TableGrape => write!(f, "table-grape"),
            Self::Raisin => write!(f, "raisin"),
        }
    }
}

/// Vineyard status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum VineyardStatus {
    /// Pruned status
    #[default]
    Pruned,
    /// Budding status
    Budding,
    /// Ripening status
    Ripening,
    /// Vintage status
    Vintage,
}

impl std::fmt::Display for VineyardStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pruned => write!(f, "pruned"),
            Self::Budding => write!(f, "budding"),
            Self::Ripening => write!(f, "ripening"),
            Self::Vintage => write!(f, "vintage"),
        }
    }
}

/// Vineyard config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VineyardConfig {
    /// Name
    pub name: String,
    /// Vineyard type
    pub vineyard_type: VineyardType,
    /// Status
    pub status: VineyardStatus,
    /// Max vines
    pub max_vines: usize,
}

impl VineyardConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            vineyard_type: VineyardType::RedWine,
            status: VineyardStatus::Pruned,
            max_vines: 100,
        }
    }

    /// Set type
    pub fn vineyard_type(mut self, vt: VineyardType) -> Self {
        self.vineyard_type = vt;
        self
    }

    /// Set status
    pub fn status(mut self, s: VineyardStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max vines
    pub fn max_vines(mut self, max: usize) -> Self {
        self.max_vines = max;
        self
    }
}

impl Default for VineyardConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Vineyard vine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VineyardVine {
    /// Vine ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Terrace number
    pub terrace: u32,
    /// Bearing
    pub bearing: bool,
}

impl VineyardVine {
    /// Create new vine
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            terrace: 0,
            bearing: true,
        }
    }

    /// Set terrace
    pub fn terrace(mut self, t: u32) -> Self {
        self.terrace = t;
        self
    }

    /// Make bearing
    pub fn make_bearing(&mut self) {
        self.bearing = true;
    }

    /// Make dormant
    pub fn make_dormant(&mut self) {
        self.bearing = false;
    }
}

/// Vineyard vintner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VineyardVintner {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Vine ID
    pub vine_id: String,
}

impl VineyardVintner {
    /// Create new vintner
    pub fn new(key: impl Into<String>, name: impl Into<String>, vine_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            vine_id: vine_id.into(),
        }
    }
}

/// Vineyard stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VineyardStats {
    /// Total vines
    pub total_vines: usize,
    /// Bearing vines
    pub bearing: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl VineyardStats {
    /// Update from vines
    pub fn update(&mut self, vines: &[VineyardVine], vineyard_type: VineyardType) {
        self.total_vines = vines.len();
        self.bearing = vines.iter().filter(|v| v.bearing).count();
        *self.by_type.entry(vineyard_type.to_string()).or_insert(0) += 1;
    }

    /// Bearing rate
    pub fn bearing_rate(&self) -> f64 {
        if self.total_vines == 0 { 0.0 } else { self.bearing as f64 / self.total_vines as f64 * 100.0 }
    }
}

/// Settings vineyard
#[derive(Debug, Clone, Default)]
pub struct SettingsVineyard {
    /// Config
    config: VineyardConfig,
    /// Vines
    vines: Vec<VineyardVine>,
    /// Vintners
    vintners: Vec<VineyardVintner>,
    /// Stats
    stats: VineyardStats,
}

impl SettingsVineyard {
    /// Create new vineyard system
    pub fn new(config: VineyardConfig) -> Self {
        Self {
            config,
            vines: Vec::new(),
            vintners: Vec::new(),
            stats: VineyardStats::default(),
        }
    }

    /// Add vine
    pub fn add_vine(&mut self, vine: VineyardVine) -> bool {
        if self.vines.len() >= self.config.max_vines {
            return false;
        }
        self.vines.push(vine);
        self.update_stats();
        true
    }

    /// Get vine
    pub fn get_vine(&self, id: &str) -> Option<&VineyardVine> {
        self.vines.iter().find(|v| v.id == id)
    }

    /// Get vine mut
    pub fn get_vine_mut(&mut self, id: &str) -> Option<&mut VineyardVine> {
        self.vines.iter_mut().find(|v| v.id == id)
    }

    /// Add vintner
    pub fn add_vintner(&mut self, vintner: VineyardVintner) {
        self.vintners.push(vintner);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.vines, self.config.vineyard_type);
    }

    /// Get stats
    pub fn stats(&self) -> &VineyardStats {
        &self.stats
    }

    /// Vine count
    pub fn vine_count(&self) -> usize {
        self.vines.len()
    }
}

/// Vineyard registry
#[derive(Debug, Clone, Default)]
pub struct VineyardRegistry {
    /// Vineyards by ID
    vineyards: HashMap<String, SettingsVineyard>,
}

impl VineyardRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register vineyard
    pub fn register(&mut self, id: impl Into<String>, vineyard: SettingsVineyard) {
        self.vineyards.insert(id.into(), vineyard);
    }

    /// Unregister vineyard
    pub fn unregister(&mut self, id: &str) -> bool {
        self.vineyards.remove(id).is_some()
    }

    /// Get vineyard
    pub fn get(&self, id: &str) -> Option<&SettingsVineyard> {
        self.vineyards.get(id)
    }

    /// Get vineyard mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsVineyard> {
        self.vineyards.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.vineyards.len()
    }
}

/// Format vineyard registry
pub fn format_vineyard_registry(registry: &VineyardRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Vineyard Registry:\n");
    output.push_str(&format!("  Vineyards: {}\n", registry.count()));
    output
}

/// Check if query is about vineyard
pub fn is_vineyard_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings vineyard") || lower.contains("vineyard settings") || lower.contains("grape vineyard")
}

/// Fun fact about vineyard
pub fn vineyard_fun_fact() -> &'static str {
    "Anna's settings vineyard establishes viticulture boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vineyard_type_display() {
        assert_eq!(format!("{}", VineyardType::RedWine), "red-wine");
        assert_eq!(format!("{}", VineyardType::WhiteWine), "white-wine");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", VineyardStatus::Pruned), "pruned");
        assert_eq!(format!("{}", VineyardStatus::Vintage), "vintage");
    }

    #[test]
    fn test_config_new() {
        let c = VineyardConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = VineyardConfig::new("test")
            .vineyard_type(VineyardType::TableGrape)
            .status(VineyardStatus::Ripening);
        assert_eq!(c.vineyard_type, VineyardType::TableGrape);
        assert_eq!(c.status, VineyardStatus::Ripening);
    }

    #[test]
    fn test_vine_new() {
        let v = VineyardVine::new("v1", "Title", "Content");
        assert_eq!(v.id, "v1");
    }

    #[test]
    fn test_vine_builder() {
        let v = VineyardVine::new("v1", "Title", "Content")
            .terrace(1);
        assert_eq!(v.terrace, 1);
    }

    #[test]
    fn test_vine_bearing() {
        let mut v = VineyardVine::new("v1", "Title", "Content");
        v.make_dormant();
        assert!(!v.bearing);
        v.make_bearing();
        assert!(v.bearing);
    }

    #[test]
    fn test_vintner_new() {
        let v = VineyardVintner::new("key", "name", "v1");
        assert_eq!(v.vine_id, "v1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = VineyardStats::default();
        let vine = VineyardVine::new("v1", "Title", "Content");
        s.update(&[vine], VineyardType::RedWine);
        assert_eq!(s.total_vines, 1);
        assert_eq!(s.bearing, 1);
    }

    #[test]
    fn test_vineyard_new() {
        let v = SettingsVineyard::new(VineyardConfig::default());
        assert_eq!(v.vine_count(), 0);
    }

    #[test]
    fn test_vineyard_add_vine() {
        let mut v = SettingsVineyard::new(VineyardConfig::default());
        v.add_vine(VineyardVine::new("v1", "Title", "Content"));
        assert_eq!(v.vine_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = VineyardRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = VineyardRegistry::new();
        r.register("v1", SettingsVineyard::new(VineyardConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_vineyard_query() {
        assert!(is_vineyard_query("settings vineyard"));
        assert!(!is_vineyard_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = vineyard_fun_fact();
        assert!(fact.contains("vineyard"));
    }
}
