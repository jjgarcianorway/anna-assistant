// v0.0.784: Settings Haven (Phase 360)
// Safe haven for settings protection

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Haven type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HavenType {
    /// Safe haven
    #[default]
    Safe,
    /// Secure haven
    Secure,
    /// Protected haven
    Protected,
    /// Peaceful haven
    Peaceful,
}

impl std::fmt::Display for HavenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Safe => write!(f, "safe"),
            Self::Secure => write!(f, "secure"),
            Self::Protected => write!(f, "protected"),
            Self::Peaceful => write!(f, "peaceful"),
        }
    }
}

/// Haven status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HavenStatus {
    /// Open status
    #[default]
    Open,
    /// Sheltering status
    Sheltering,
    /// Guarding status
    Guarding,
    /// Welcoming status
    Welcoming,
}

impl std::fmt::Display for HavenStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Sheltering => write!(f, "sheltering"),
            Self::Guarding => write!(f, "guarding"),
            Self::Welcoming => write!(f, "welcoming"),
        }
    }
}

/// Haven config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HavenConfig {
    /// Name
    pub name: String,
    /// Haven type
    pub haven_type: HavenType,
    /// Status
    pub status: HavenStatus,
    /// Max guests
    pub max_guests: usize,
}

impl HavenConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            haven_type: HavenType::Safe,
            status: HavenStatus::Open,
            max_guests: 100,
        }
    }

    /// Set type
    pub fn haven_type(mut self, ht: HavenType) -> Self {
        self.haven_type = ht;
        self
    }

    /// Set status
    pub fn status(mut self, s: HavenStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max guests
    pub fn max_guests(mut self, max: usize) -> Self {
        self.max_guests = max;
        self
    }
}

impl Default for HavenConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Haven guest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HavenGuest {
    /// Guest ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Room number
    pub room: u32,
    /// Comfortable
    pub comfortable: bool,
}

impl HavenGuest {
    /// Create new guest
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            room: 0,
            comfortable: true,
        }
    }

    /// Set room
    pub fn room(mut self, r: u32) -> Self {
        self.room = r;
        self
    }

    /// Make comfortable
    pub fn make_comfortable(&mut self) {
        self.comfortable = true;
    }

    /// Make restless
    pub fn make_restless(&mut self) {
        self.comfortable = false;
    }
}

/// Haven keeper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HavenKeeper {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Guest ID
    pub guest_id: String,
}

impl HavenKeeper {
    /// Create new keeper
    pub fn new(key: impl Into<String>, name: impl Into<String>, guest_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            guest_id: guest_id.into(),
        }
    }
}

/// Haven stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HavenStats {
    /// Total guests
    pub total_guests: usize,
    /// Comfortable guests
    pub comfortable: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl HavenStats {
    /// Update from guests
    pub fn update(&mut self, guests: &[HavenGuest], haven_type: HavenType) {
        self.total_guests = guests.len();
        self.comfortable = guests.iter().filter(|g| g.comfortable).count();
        *self.by_type.entry(haven_type.to_string()).or_insert(0) += 1;
    }

    /// Comfort rate
    pub fn comfort_rate(&self) -> f64 {
        if self.total_guests == 0 { 0.0 } else { self.comfortable as f64 / self.total_guests as f64 * 100.0 }
    }
}

/// Settings haven
#[derive(Debug, Clone, Default)]
pub struct SettingsHaven {
    /// Config
    config: HavenConfig,
    /// Guests
    guests: Vec<HavenGuest>,
    /// Keepers
    keepers: Vec<HavenKeeper>,
    /// Stats
    stats: HavenStats,
}

impl SettingsHaven {
    /// Create new haven system
    pub fn new(config: HavenConfig) -> Self {
        Self {
            config,
            guests: Vec::new(),
            keepers: Vec::new(),
            stats: HavenStats::default(),
        }
    }

    /// Add guest
    pub fn add_guest(&mut self, guest: HavenGuest) -> bool {
        if self.guests.len() >= self.config.max_guests {
            return false;
        }
        self.guests.push(guest);
        self.update_stats();
        true
    }

    /// Get guest
    pub fn get_guest(&self, id: &str) -> Option<&HavenGuest> {
        self.guests.iter().find(|g| g.id == id)
    }

    /// Get guest mut
    pub fn get_guest_mut(&mut self, id: &str) -> Option<&mut HavenGuest> {
        self.guests.iter_mut().find(|g| g.id == id)
    }

    /// Add keeper
    pub fn add_keeper(&mut self, keeper: HavenKeeper) {
        self.keepers.push(keeper);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.guests, self.config.haven_type);
    }

    /// Get stats
    pub fn stats(&self) -> &HavenStats {
        &self.stats
    }

    /// Guest count
    pub fn guest_count(&self) -> usize {
        self.guests.len()
    }
}

/// Haven registry
#[derive(Debug, Clone, Default)]
pub struct HavenRegistry {
    /// Havens by ID
    havens: HashMap<String, SettingsHaven>,
}

impl HavenRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register haven
    pub fn register(&mut self, id: impl Into<String>, haven: SettingsHaven) {
        self.havens.insert(id.into(), haven);
    }

    /// Unregister haven
    pub fn unregister(&mut self, id: &str) -> bool {
        self.havens.remove(id).is_some()
    }

    /// Get haven
    pub fn get(&self, id: &str) -> Option<&SettingsHaven> {
        self.havens.get(id)
    }

    /// Get haven mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsHaven> {
        self.havens.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.havens.len()
    }
}

/// Format haven registry
pub fn format_haven_registry(registry: &HavenRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Haven Registry:\n");
    output.push_str(&format!("  Havens: {}\n", registry.count()));
    output
}

/// Check if query is about haven
pub fn is_haven_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings haven") || lower.contains("haven settings") || lower.contains("safe haven")
}

/// Fun fact about haven
pub fn haven_fun_fact() -> &'static str {
    "Anna's settings haven provides a safe place for configurations!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haven_type_display() {
        assert_eq!(format!("{}", HavenType::Safe), "safe");
        assert_eq!(format!("{}", HavenType::Secure), "secure");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", HavenStatus::Open), "open");
        assert_eq!(format!("{}", HavenStatus::Welcoming), "welcoming");
    }

    #[test]
    fn test_config_new() {
        let c = HavenConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = HavenConfig::new("test")
            .haven_type(HavenType::Secure)
            .status(HavenStatus::Guarding);
        assert_eq!(c.haven_type, HavenType::Secure);
        assert_eq!(c.status, HavenStatus::Guarding);
    }

    #[test]
    fn test_guest_new() {
        let g = HavenGuest::new("g1", "Title", "Content");
        assert_eq!(g.id, "g1");
    }

    #[test]
    fn test_guest_builder() {
        let g = HavenGuest::new("g1", "Title", "Content")
            .room(1);
        assert_eq!(g.room, 1);
    }

    #[test]
    fn test_guest_comfort() {
        let mut g = HavenGuest::new("g1", "Title", "Content");
        g.make_restless();
        assert!(!g.comfortable);
        g.make_comfortable();
        assert!(g.comfortable);
    }

    #[test]
    fn test_keeper_new() {
        let k = HavenKeeper::new("key", "name", "g1");
        assert_eq!(k.guest_id, "g1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = HavenStats::default();
        let guest = HavenGuest::new("g1", "Title", "Content");
        s.update(&[guest], HavenType::Safe);
        assert_eq!(s.total_guests, 1);
        assert_eq!(s.comfortable, 1);
    }

    #[test]
    fn test_haven_new() {
        let h = SettingsHaven::new(HavenConfig::default());
        assert_eq!(h.guest_count(), 0);
    }

    #[test]
    fn test_haven_add_guest() {
        let mut h = SettingsHaven::new(HavenConfig::default());
        h.add_guest(HavenGuest::new("g1", "Title", "Content"));
        assert_eq!(h.guest_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = HavenRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = HavenRegistry::new();
        r.register("h1", SettingsHaven::new(HavenConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_haven_query() {
        assert!(is_haven_query("settings haven"));
        assert!(!is_haven_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = haven_fun_fact();
        assert!(fact.contains("haven"));
    }
}
