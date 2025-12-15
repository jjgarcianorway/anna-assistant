// v0.0.786: Settings Hideaway (Phase 362)
// Secret hideaway for settings seclusion

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Hideaway type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HideawayType {
    /// Secret hideaway
    #[default]
    Secret,
    /// Private hideaway
    Private,
    /// Remote hideaway
    Remote,
    /// Hidden hideaway
    Hidden,
}

impl std::fmt::Display for HideawayType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Secret => write!(f, "secret"),
            Self::Private => write!(f, "private"),
            Self::Remote => write!(f, "remote"),
            Self::Hidden => write!(f, "hidden"),
        }
    }
}

/// Hideaway status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HideawayStatus {
    /// Secluded status
    #[default]
    Secluded,
    /// Concealed status
    Concealed,
    /// Sheltered status
    Sheltered,
    /// Isolated status
    Isolated,
}

impl std::fmt::Display for HideawayStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Secluded => write!(f, "secluded"),
            Self::Concealed => write!(f, "concealed"),
            Self::Sheltered => write!(f, "sheltered"),
            Self::Isolated => write!(f, "isolated"),
        }
    }
}

/// Hideaway config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HideawayConfig {
    /// Name
    pub name: String,
    /// Hideaway type
    pub hideaway_type: HideawayType,
    /// Status
    pub status: HideawayStatus,
    /// Max occupants
    pub max_occupants: usize,
}

impl HideawayConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            hideaway_type: HideawayType::Secret,
            status: HideawayStatus::Secluded,
            max_occupants: 100,
        }
    }

    /// Set type
    pub fn hideaway_type(mut self, ht: HideawayType) -> Self {
        self.hideaway_type = ht;
        self
    }

    /// Set status
    pub fn status(mut self, s: HideawayStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max occupants
    pub fn max_occupants(mut self, max: usize) -> Self {
        self.max_occupants = max;
        self
    }
}

impl Default for HideawayConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Hideaway occupant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HideawayOccupant {
    /// Occupant ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Nook number
    pub nook: u32,
    /// Hidden
    pub hidden: bool,
}

impl HideawayOccupant {
    /// Create new occupant
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            nook: 0,
            hidden: true,
        }
    }

    /// Set nook
    pub fn nook(mut self, n: u32) -> Self {
        self.nook = n;
        self
    }

    /// Make hidden
    pub fn make_hidden(&mut self) {
        self.hidden = true;
    }

    /// Make visible
    pub fn make_visible(&mut self) {
        self.hidden = false;
    }
}

/// Hideaway guardian
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HideawayGuardian {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Occupant ID
    pub occupant_id: String,
}

impl HideawayGuardian {
    /// Create new guardian
    pub fn new(key: impl Into<String>, name: impl Into<String>, occupant_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            occupant_id: occupant_id.into(),
        }
    }
}

/// Hideaway stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HideawayStats {
    /// Total occupants
    pub total_occupants: usize,
    /// Hidden occupants
    pub hidden: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl HideawayStats {
    /// Update from occupants
    pub fn update(&mut self, occupants: &[HideawayOccupant], hideaway_type: HideawayType) {
        self.total_occupants = occupants.len();
        self.hidden = occupants.iter().filter(|o| o.hidden).count();
        *self.by_type.entry(hideaway_type.to_string()).or_insert(0) += 1;
    }

    /// Hidden rate
    pub fn hidden_rate(&self) -> f64 {
        if self.total_occupants == 0 { 0.0 } else { self.hidden as f64 / self.total_occupants as f64 * 100.0 }
    }
}

/// Settings hideaway
#[derive(Debug, Clone, Default)]
pub struct SettingsHideaway {
    /// Config
    config: HideawayConfig,
    /// Occupants
    occupants: Vec<HideawayOccupant>,
    /// Guardians
    guardians: Vec<HideawayGuardian>,
    /// Stats
    stats: HideawayStats,
}

impl SettingsHideaway {
    /// Create new hideaway system
    pub fn new(config: HideawayConfig) -> Self {
        Self {
            config,
            occupants: Vec::new(),
            guardians: Vec::new(),
            stats: HideawayStats::default(),
        }
    }

    /// Add occupant
    pub fn add_occupant(&mut self, occupant: HideawayOccupant) -> bool {
        if self.occupants.len() >= self.config.max_occupants {
            return false;
        }
        self.occupants.push(occupant);
        self.update_stats();
        true
    }

    /// Get occupant
    pub fn get_occupant(&self, id: &str) -> Option<&HideawayOccupant> {
        self.occupants.iter().find(|o| o.id == id)
    }

    /// Get occupant mut
    pub fn get_occupant_mut(&mut self, id: &str) -> Option<&mut HideawayOccupant> {
        self.occupants.iter_mut().find(|o| o.id == id)
    }

    /// Add guardian
    pub fn add_guardian(&mut self, guardian: HideawayGuardian) {
        self.guardians.push(guardian);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.occupants, self.config.hideaway_type);
    }

    /// Get stats
    pub fn stats(&self) -> &HideawayStats {
        &self.stats
    }

    /// Occupant count
    pub fn occupant_count(&self) -> usize {
        self.occupants.len()
    }
}

/// Hideaway registry
#[derive(Debug, Clone, Default)]
pub struct HideawayRegistry {
    /// Hideaways by ID
    hideaways: HashMap<String, SettingsHideaway>,
}

impl HideawayRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register hideaway
    pub fn register(&mut self, id: impl Into<String>, hideaway: SettingsHideaway) {
        self.hideaways.insert(id.into(), hideaway);
    }

    /// Unregister hideaway
    pub fn unregister(&mut self, id: &str) -> bool {
        self.hideaways.remove(id).is_some()
    }

    /// Get hideaway
    pub fn get(&self, id: &str) -> Option<&SettingsHideaway> {
        self.hideaways.get(id)
    }

    /// Get hideaway mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsHideaway> {
        self.hideaways.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.hideaways.len()
    }
}

/// Format hideaway registry
pub fn format_hideaway_registry(registry: &HideawayRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Hideaway Registry:\n");
    output.push_str(&format!("  Hideaways: {}\n", registry.count()));
    output
}

/// Check if query is about hideaway
pub fn is_hideaway_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings hideaway") || lower.contains("hideaway settings") || lower.contains("secret hideaway")
}

/// Fun fact about hideaway
pub fn hideaway_fun_fact() -> &'static str {
    "Anna's settings hideaway keeps configurations safely hidden!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hideaway_type_display() {
        assert_eq!(format!("{}", HideawayType::Secret), "secret");
        assert_eq!(format!("{}", HideawayType::Private), "private");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", HideawayStatus::Secluded), "secluded");
        assert_eq!(format!("{}", HideawayStatus::Isolated), "isolated");
    }

    #[test]
    fn test_config_new() {
        let c = HideawayConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = HideawayConfig::new("test")
            .hideaway_type(HideawayType::Private)
            .status(HideawayStatus::Concealed);
        assert_eq!(c.hideaway_type, HideawayType::Private);
        assert_eq!(c.status, HideawayStatus::Concealed);
    }

    #[test]
    fn test_occupant_new() {
        let o = HideawayOccupant::new("o1", "Title", "Content");
        assert_eq!(o.id, "o1");
    }

    #[test]
    fn test_occupant_builder() {
        let o = HideawayOccupant::new("o1", "Title", "Content")
            .nook(1);
        assert_eq!(o.nook, 1);
    }

    #[test]
    fn test_occupant_visibility() {
        let mut o = HideawayOccupant::new("o1", "Title", "Content");
        o.make_visible();
        assert!(!o.hidden);
        o.make_hidden();
        assert!(o.hidden);
    }

    #[test]
    fn test_guardian_new() {
        let g = HideawayGuardian::new("key", "name", "o1");
        assert_eq!(g.occupant_id, "o1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = HideawayStats::default();
        let occupant = HideawayOccupant::new("o1", "Title", "Content");
        s.update(&[occupant], HideawayType::Secret);
        assert_eq!(s.total_occupants, 1);
        assert_eq!(s.hidden, 1);
    }

    #[test]
    fn test_hideaway_new() {
        let h = SettingsHideaway::new(HideawayConfig::default());
        assert_eq!(h.occupant_count(), 0);
    }

    #[test]
    fn test_hideaway_add_occupant() {
        let mut h = SettingsHideaway::new(HideawayConfig::default());
        h.add_occupant(HideawayOccupant::new("o1", "Title", "Content"));
        assert_eq!(h.occupant_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = HideawayRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = HideawayRegistry::new();
        r.register("h1", SettingsHideaway::new(HideawayConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_hideaway_query() {
        assert!(is_hideaway_query("settings hideaway"));
        assert!(!is_hideaway_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = hideaway_fun_fact();
        assert!(fact.contains("hideaway"));
    }
}
