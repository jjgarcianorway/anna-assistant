// v0.0.756: Settings Lot (Phase 332)
// Land lot for settings property

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Lot type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum LotType {
    /// Residential lot
    #[default]
    Residential,
    /// Commercial lot
    Commercial,
    /// Industrial lot
    Industrial,
    /// Agricultural lot
    Agricultural,
}

impl std::fmt::Display for LotType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Residential => write!(f, "residential"),
            Self::Commercial => write!(f, "commercial"),
            Self::Industrial => write!(f, "industrial"),
            Self::Agricultural => write!(f, "agricultural"),
        }
    }
}

/// Lot status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum LotStatus {
    /// Vacant status
    #[default]
    Vacant,
    /// Improved status
    Improved,
    /// Subdivided status
    Subdivided,
    /// Consolidated status
    Consolidated,
}

impl std::fmt::Display for LotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vacant => write!(f, "vacant"),
            Self::Improved => write!(f, "improved"),
            Self::Subdivided => write!(f, "subdivided"),
            Self::Consolidated => write!(f, "consolidated"),
        }
    }
}

/// Lot config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotConfig {
    /// Name
    pub name: String,
    /// Lot type
    pub lot_type: LotType,
    /// Status
    pub status: LotStatus,
    /// Max deeds
    pub max_deeds: usize,
}

impl LotConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            lot_type: LotType::Residential,
            status: LotStatus::Vacant,
            max_deeds: 100,
        }
    }

    /// Set type
    pub fn lot_type(mut self, lt: LotType) -> Self {
        self.lot_type = lt;
        self
    }

    /// Set status
    pub fn status(mut self, s: LotStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max deeds
    pub fn max_deeds(mut self, max: usize) -> Self {
        self.max_deeds = max;
        self
    }
}

impl Default for LotConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Lot deed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotDeed {
    /// Deed ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Parcel number
    pub parcel: u32,
    /// Registered
    pub registered: bool,
}

impl LotDeed {
    /// Create new deed
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            parcel: 0,
            registered: true,
        }
    }

    /// Set parcel
    pub fn parcel(mut self, p: u32) -> Self {
        self.parcel = p;
        self
    }

    /// Make registered
    pub fn make_registered(&mut self) {
        self.registered = true;
    }

    /// Make unregistered
    pub fn make_unregistered(&mut self) {
        self.registered = false;
    }
}

/// Lot assessor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotAssessor {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Deed ID
    pub deed_id: String,
}

impl LotAssessor {
    /// Create new assessor
    pub fn new(key: impl Into<String>, name: impl Into<String>, deed_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            deed_id: deed_id.into(),
        }
    }
}

/// Lot stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LotStats {
    /// Total deeds
    pub total_deeds: usize,
    /// Registered deeds
    pub registered: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl LotStats {
    /// Update from deeds
    pub fn update(&mut self, deeds: &[LotDeed], lot_type: LotType) {
        self.total_deeds = deeds.len();
        self.registered = deeds.iter().filter(|d| d.registered).count();
        *self.by_type.entry(lot_type.to_string()).or_insert(0) += 1;
    }

    /// Registered rate
    pub fn registered_rate(&self) -> f64 {
        if self.total_deeds == 0 { 0.0 } else { self.registered as f64 / self.total_deeds as f64 * 100.0 }
    }
}

/// Settings lot
#[derive(Debug, Clone, Default)]
pub struct SettingsLot {
    /// Config
    config: LotConfig,
    /// Deeds
    deeds: Vec<LotDeed>,
    /// Assessors
    assessors: Vec<LotAssessor>,
    /// Stats
    stats: LotStats,
}

impl SettingsLot {
    /// Create new lot system
    pub fn new(config: LotConfig) -> Self {
        Self {
            config,
            deeds: Vec::new(),
            assessors: Vec::new(),
            stats: LotStats::default(),
        }
    }

    /// Add deed
    pub fn add_deed(&mut self, deed: LotDeed) -> bool {
        if self.deeds.len() >= self.config.max_deeds {
            return false;
        }
        self.deeds.push(deed);
        self.update_stats();
        true
    }

    /// Get deed
    pub fn get_deed(&self, id: &str) -> Option<&LotDeed> {
        self.deeds.iter().find(|d| d.id == id)
    }

    /// Get deed mut
    pub fn get_deed_mut(&mut self, id: &str) -> Option<&mut LotDeed> {
        self.deeds.iter_mut().find(|d| d.id == id)
    }

    /// Add assessor
    pub fn add_assessor(&mut self, assessor: LotAssessor) {
        self.assessors.push(assessor);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.deeds, self.config.lot_type);
    }

    /// Get stats
    pub fn stats(&self) -> &LotStats {
        &self.stats
    }

    /// Deed count
    pub fn deed_count(&self) -> usize {
        self.deeds.len()
    }
}

/// Lot registry
#[derive(Debug, Clone, Default)]
pub struct LotRegistry {
    /// Lots by ID
    lots: HashMap<String, SettingsLot>,
}

impl LotRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register lot
    pub fn register(&mut self, id: impl Into<String>, lot: SettingsLot) {
        self.lots.insert(id.into(), lot);
    }

    /// Unregister lot
    pub fn unregister(&mut self, id: &str) -> bool {
        self.lots.remove(id).is_some()
    }

    /// Get lot
    pub fn get(&self, id: &str) -> Option<&SettingsLot> {
        self.lots.get(id)
    }

    /// Get lot mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsLot> {
        self.lots.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.lots.len()
    }
}

/// Format lot registry
pub fn format_lot_registry(registry: &LotRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Lot Registry:\n");
    output.push_str(&format!("  Lots: {}\n", registry.count()));
    output
}

/// Check if query is about lot
pub fn is_lot_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings lot") || lower.contains("lot settings") || lower.contains("land lot")
}

/// Fun fact about lot
pub fn lot_fun_fact() -> &'static str {
    "Anna's settings lot establishes property boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lot_type_display() {
        assert_eq!(format!("{}", LotType::Residential), "residential");
        assert_eq!(format!("{}", LotType::Commercial), "commercial");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", LotStatus::Vacant), "vacant");
        assert_eq!(format!("{}", LotStatus::Improved), "improved");
    }

    #[test]
    fn test_config_new() {
        let c = LotConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = LotConfig::new("test")
            .lot_type(LotType::Commercial)
            .status(LotStatus::Subdivided);
        assert_eq!(c.lot_type, LotType::Commercial);
        assert_eq!(c.status, LotStatus::Subdivided);
    }

    #[test]
    fn test_deed_new() {
        let d = LotDeed::new("d1", "Title", "Content");
        assert_eq!(d.id, "d1");
    }

    #[test]
    fn test_deed_builder() {
        let d = LotDeed::new("d1", "Title", "Content")
            .parcel(1);
        assert_eq!(d.parcel, 1);
    }

    #[test]
    fn test_deed_registered() {
        let mut d = LotDeed::new("d1", "Title", "Content");
        d.make_unregistered();
        assert!(!d.registered);
        d.make_registered();
        assert!(d.registered);
    }

    #[test]
    fn test_assessor_new() {
        let a = LotAssessor::new("key", "name", "d1");
        assert_eq!(a.deed_id, "d1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = LotStats::default();
        let deed = LotDeed::new("d1", "Title", "Content");
        s.update(&[deed], LotType::Residential);
        assert_eq!(s.total_deeds, 1);
        assert_eq!(s.registered, 1);
    }

    #[test]
    fn test_lot_new() {
        let l = SettingsLot::new(LotConfig::default());
        assert_eq!(l.deed_count(), 0);
    }

    #[test]
    fn test_lot_add_deed() {
        let mut l = SettingsLot::new(LotConfig::default());
        l.add_deed(LotDeed::new("d1", "Title", "Content"));
        assert_eq!(l.deed_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = LotRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = LotRegistry::new();
        r.register("l1", SettingsLot::new(LotConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_lot_query() {
        assert!(is_lot_query("settings lot"));
        assert!(!is_lot_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = lot_fun_fact();
        assert!(fact.contains("lot"));
    }
}
