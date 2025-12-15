// v0.0.742: Settings Zone (Phase 318)
// Designated zone for settings boundaries

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Zone type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ZoneType {
    /// Free trade zone
    #[default]
    FreeTrade,
    /// Economic zone
    Economic,
    /// Security zone
    Security,
    /// Buffer zone
    Buffer,
}

impl std::fmt::Display for ZoneType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FreeTrade => write!(f, "free-trade"),
            Self::Economic => write!(f, "economic"),
            Self::Security => write!(f, "security"),
            Self::Buffer => write!(f, "buffer"),
        }
    }
}

/// Zone status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ZoneStatus {
    /// Proposed status
    #[default]
    Proposed,
    /// Established status
    Established,
    /// Operational status
    Operational,
    /// Suspended status
    Suspended,
}

impl std::fmt::Display for ZoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Proposed => write!(f, "proposed"),
            Self::Established => write!(f, "established"),
            Self::Operational => write!(f, "operational"),
            Self::Suspended => write!(f, "suspended"),
        }
    }
}

/// Zone config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneConfig {
    /// Name
    pub name: String,
    /// Zone type
    pub zone_type: ZoneType,
    /// Status
    pub status: ZoneStatus,
    /// Max regulations
    pub max_regulations: usize,
}

impl ZoneConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            zone_type: ZoneType::FreeTrade,
            status: ZoneStatus::Proposed,
            max_regulations: 100,
        }
    }

    /// Set type
    pub fn zone_type(mut self, zt: ZoneType) -> Self {
        self.zone_type = zt;
        self
    }

    /// Set status
    pub fn status(mut self, s: ZoneStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max regulations
    pub fn max_regulations(mut self, max: usize) -> Self {
        self.max_regulations = max;
        self
    }
}

impl Default for ZoneConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Zone regulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneRegulation {
    /// Regulation ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Sector number
    pub sector: u32,
    /// Enforced
    pub enforced: bool,
}

impl ZoneRegulation {
    /// Create new regulation
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            sector: 0,
            enforced: true,
        }
    }

    /// Set sector
    pub fn sector(mut self, s: u32) -> Self {
        self.sector = s;
        self
    }

    /// Make enforced
    pub fn make_enforced(&mut self) {
        self.enforced = true;
    }

    /// Make advisory
    pub fn make_advisory(&mut self) {
        self.enforced = false;
    }
}

/// Zone participant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneParticipant {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Regulation ID
    pub regulation_id: String,
}

impl ZoneParticipant {
    /// Create new participant
    pub fn new(key: impl Into<String>, name: impl Into<String>, regulation_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            regulation_id: regulation_id.into(),
        }
    }
}

/// Zone stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ZoneStats {
    /// Total regulations
    pub total_regulations: usize,
    /// Enforced regulations
    pub enforced: usize,
    /// Operational count
    pub operational_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ZoneStats {
    /// Update from regulations
    pub fn update(&mut self, regulations: &[ZoneRegulation], zone_type: ZoneType) {
        self.total_regulations = regulations.len();
        self.enforced = regulations.iter().filter(|r| r.enforced).count();
        *self.by_type.entry(zone_type.to_string()).or_insert(0) += 1;
    }

    /// Enforcement rate
    pub fn enforcement_rate(&self) -> f64 {
        if self.total_regulations == 0 { 0.0 } else { self.enforced as f64 / self.total_regulations as f64 * 100.0 }
    }
}

/// Settings zone
#[derive(Debug, Clone, Default)]
pub struct SettingsZone {
    /// Config
    config: ZoneConfig,
    /// Regulations
    regulations: Vec<ZoneRegulation>,
    /// Participants
    participants: Vec<ZoneParticipant>,
    /// Stats
    stats: ZoneStats,
}

impl SettingsZone {
    /// Create new zone system
    pub fn new(config: ZoneConfig) -> Self {
        Self {
            config,
            regulations: Vec::new(),
            participants: Vec::new(),
            stats: ZoneStats::default(),
        }
    }

    /// Add regulation
    pub fn add_regulation(&mut self, regulation: ZoneRegulation) -> bool {
        if self.regulations.len() >= self.config.max_regulations {
            return false;
        }
        self.regulations.push(regulation);
        self.update_stats();
        true
    }

    /// Get regulation
    pub fn get_regulation(&self, id: &str) -> Option<&ZoneRegulation> {
        self.regulations.iter().find(|r| r.id == id)
    }

    /// Get regulation mut
    pub fn get_regulation_mut(&mut self, id: &str) -> Option<&mut ZoneRegulation> {
        self.regulations.iter_mut().find(|r| r.id == id)
    }

    /// Add participant
    pub fn add_participant(&mut self, participant: ZoneParticipant) {
        self.participants.push(participant);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.regulations, self.config.zone_type);
    }

    /// Get stats
    pub fn stats(&self) -> &ZoneStats {
        &self.stats
    }

    /// Regulation count
    pub fn regulation_count(&self) -> usize {
        self.regulations.len()
    }
}

/// Zone registry
#[derive(Debug, Clone, Default)]
pub struct ZoneRegistry {
    /// Zones by ID
    zones: HashMap<String, SettingsZone>,
}

impl ZoneRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register zone
    pub fn register(&mut self, id: impl Into<String>, zone: SettingsZone) {
        self.zones.insert(id.into(), zone);
    }

    /// Unregister zone
    pub fn unregister(&mut self, id: &str) -> bool {
        self.zones.remove(id).is_some()
    }

    /// Get zone
    pub fn get(&self, id: &str) -> Option<&SettingsZone> {
        self.zones.get(id)
    }

    /// Get zone mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsZone> {
        self.zones.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.zones.len()
    }
}

/// Format zone registry
pub fn format_zone_registry(registry: &ZoneRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Zone Registry:\n");
    output.push_str(&format!("  Zones: {}\n", registry.count()));
    output
}

/// Check if query is about zone
pub fn is_zone_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings zone") || lower.contains("zone settings") || lower.contains("free trade zone")
}

/// Fun fact about zone
pub fn zone_fun_fact() -> &'static str {
    "Anna's settings zone establishes designated boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_type_display() {
        assert_eq!(format!("{}", ZoneType::FreeTrade), "free-trade");
        assert_eq!(format!("{}", ZoneType::Economic), "economic");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ZoneStatus::Proposed), "proposed");
        assert_eq!(format!("{}", ZoneStatus::Operational), "operational");
    }

    #[test]
    fn test_config_new() {
        let c = ZoneConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ZoneConfig::new("test")
            .zone_type(ZoneType::Security)
            .status(ZoneStatus::Established);
        assert_eq!(c.zone_type, ZoneType::Security);
        assert_eq!(c.status, ZoneStatus::Established);
    }

    #[test]
    fn test_regulation_new() {
        let r = ZoneRegulation::new("r1", "Title", "Content");
        assert_eq!(r.id, "r1");
    }

    #[test]
    fn test_regulation_builder() {
        let r = ZoneRegulation::new("r1", "Title", "Content")
            .sector(1);
        assert_eq!(r.sector, 1);
    }

    #[test]
    fn test_regulation_enforced() {
        let mut r = ZoneRegulation::new("r1", "Title", "Content");
        r.make_advisory();
        assert!(!r.enforced);
        r.make_enforced();
        assert!(r.enforced);
    }

    #[test]
    fn test_participant_new() {
        let p = ZoneParticipant::new("key", "name", "r1");
        assert_eq!(p.regulation_id, "r1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = ZoneStats::default();
        let regulation = ZoneRegulation::new("r1", "Title", "Content");
        s.update(&[regulation], ZoneType::FreeTrade);
        assert_eq!(s.total_regulations, 1);
        assert_eq!(s.enforced, 1);
    }

    #[test]
    fn test_zone_new() {
        let z = SettingsZone::new(ZoneConfig::default());
        assert_eq!(z.regulation_count(), 0);
    }

    #[test]
    fn test_zone_add_regulation() {
        let mut z = SettingsZone::new(ZoneConfig::default());
        z.add_regulation(ZoneRegulation::new("r1", "Title", "Content"));
        assert_eq!(z.regulation_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = ZoneRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ZoneRegistry::new();
        r.register("z1", SettingsZone::new(ZoneConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_zone_query() {
        assert!(is_zone_query("settings zone"));
        assert!(!is_zone_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = zone_fun_fact();
        assert!(fact.contains("zone"));
    }
}
