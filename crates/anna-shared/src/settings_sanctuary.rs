// v0.0.781: Settings Sanctuary (Phase 357)
// Wildlife sanctuary for settings conservation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Sanctuary type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SanctuaryType {
    /// Wildlife sanctuary
    #[default]
    Wildlife,
    /// Marine sanctuary
    Marine,
    /// Bird sanctuary
    Bird,
    /// Forest sanctuary
    Forest,
}

impl std::fmt::Display for SanctuaryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wildlife => write!(f, "wildlife"),
            Self::Marine => write!(f, "marine"),
            Self::Bird => write!(f, "bird"),
            Self::Forest => write!(f, "forest"),
        }
    }
}

/// Sanctuary status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SanctuaryStatus {
    /// Protected status
    #[default]
    Protected,
    /// Monitored status
    Monitored,
    /// Rehabilitating status
    Rehabilitating,
    /// Expanding status
    Expanding,
}

impl std::fmt::Display for SanctuaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Protected => write!(f, "protected"),
            Self::Monitored => write!(f, "monitored"),
            Self::Rehabilitating => write!(f, "rehabilitating"),
            Self::Expanding => write!(f, "expanding"),
        }
    }
}

/// Sanctuary config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctuaryConfig {
    /// Name
    pub name: String,
    /// Sanctuary type
    pub sanctuary_type: SanctuaryType,
    /// Status
    pub status: SanctuaryStatus,
    /// Max residents
    pub max_residents: usize,
}

impl SanctuaryConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            sanctuary_type: SanctuaryType::Wildlife,
            status: SanctuaryStatus::Protected,
            max_residents: 100,
        }
    }

    /// Set type
    pub fn sanctuary_type(mut self, st: SanctuaryType) -> Self {
        self.sanctuary_type = st;
        self
    }

    /// Set status
    pub fn status(mut self, s: SanctuaryStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max residents
    pub fn max_residents(mut self, max: usize) -> Self {
        self.max_residents = max;
        self
    }
}

impl Default for SanctuaryConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Sanctuary resident
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctuaryResident {
    /// Resident ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Habitat number
    pub habitat: u32,
    /// Thriving
    pub thriving: bool,
}

impl SanctuaryResident {
    /// Create new resident
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            habitat: 0,
            thriving: true,
        }
    }

    /// Set habitat
    pub fn habitat(mut self, h: u32) -> Self {
        self.habitat = h;
        self
    }

    /// Make thriving
    pub fn make_thriving(&mut self) {
        self.thriving = true;
    }

    /// Make recovering
    pub fn make_recovering(&mut self) {
        self.thriving = false;
    }
}

/// Sanctuary warden
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctuaryWarden {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Resident ID
    pub resident_id: String,
}

impl SanctuaryWarden {
    /// Create new warden
    pub fn new(key: impl Into<String>, name: impl Into<String>, resident_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            resident_id: resident_id.into(),
        }
    }
}

/// Sanctuary stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SanctuaryStats {
    /// Total residents
    pub total_residents: usize,
    /// Thriving residents
    pub thriving: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl SanctuaryStats {
    /// Update from residents
    pub fn update(&mut self, residents: &[SanctuaryResident], sanctuary_type: SanctuaryType) {
        self.total_residents = residents.len();
        self.thriving = residents.iter().filter(|r| r.thriving).count();
        *self.by_type.entry(sanctuary_type.to_string()).or_insert(0) += 1;
    }

    /// Thriving rate
    pub fn thriving_rate(&self) -> f64 {
        if self.total_residents == 0 { 0.0 } else { self.thriving as f64 / self.total_residents as f64 * 100.0 }
    }
}

/// Settings sanctuary
#[derive(Debug, Clone, Default)]
pub struct SettingsSanctuary {
    /// Config
    config: SanctuaryConfig,
    /// Residents
    residents: Vec<SanctuaryResident>,
    /// Wardens
    wardens: Vec<SanctuaryWarden>,
    /// Stats
    stats: SanctuaryStats,
}

impl SettingsSanctuary {
    /// Create new sanctuary system
    pub fn new(config: SanctuaryConfig) -> Self {
        Self {
            config,
            residents: Vec::new(),
            wardens: Vec::new(),
            stats: SanctuaryStats::default(),
        }
    }

    /// Add resident
    pub fn add_resident(&mut self, resident: SanctuaryResident) -> bool {
        if self.residents.len() >= self.config.max_residents {
            return false;
        }
        self.residents.push(resident);
        self.update_stats();
        true
    }

    /// Get resident
    pub fn get_resident(&self, id: &str) -> Option<&SanctuaryResident> {
        self.residents.iter().find(|r| r.id == id)
    }

    /// Get resident mut
    pub fn get_resident_mut(&mut self, id: &str) -> Option<&mut SanctuaryResident> {
        self.residents.iter_mut().find(|r| r.id == id)
    }

    /// Add warden
    pub fn add_warden(&mut self, warden: SanctuaryWarden) {
        self.wardens.push(warden);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.residents, self.config.sanctuary_type);
    }

    /// Get stats
    pub fn stats(&self) -> &SanctuaryStats {
        &self.stats
    }

    /// Resident count
    pub fn resident_count(&self) -> usize {
        self.residents.len()
    }
}

/// Sanctuary registry
#[derive(Debug, Clone, Default)]
pub struct SanctuaryRegistry {
    /// Sanctuaries by ID
    sanctuaries: HashMap<String, SettingsSanctuary>,
}

impl SanctuaryRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register sanctuary
    pub fn register(&mut self, id: impl Into<String>, sanctuary: SettingsSanctuary) {
        self.sanctuaries.insert(id.into(), sanctuary);
    }

    /// Unregister sanctuary
    pub fn unregister(&mut self, id: &str) -> bool {
        self.sanctuaries.remove(id).is_some()
    }

    /// Get sanctuary
    pub fn get(&self, id: &str) -> Option<&SettingsSanctuary> {
        self.sanctuaries.get(id)
    }

    /// Get sanctuary mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsSanctuary> {
        self.sanctuaries.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.sanctuaries.len()
    }
}

/// Format sanctuary registry
pub fn format_sanctuary_registry(registry: &SanctuaryRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Sanctuary Registry:\n");
    output.push_str(&format!("  Sanctuaries: {}\n", registry.count()));
    output
}

/// Check if query is about sanctuary
pub fn is_sanctuary_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings sanctuary") || lower.contains("sanctuary settings") || lower.contains("wildlife sanctuary")
}

/// Fun fact about sanctuary
pub fn sanctuary_fun_fact() -> &'static str {
    "Anna's settings sanctuary protects conservation boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanctuary_type_display() {
        assert_eq!(format!("{}", SanctuaryType::Wildlife), "wildlife");
        assert_eq!(format!("{}", SanctuaryType::Marine), "marine");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", SanctuaryStatus::Protected), "protected");
        assert_eq!(format!("{}", SanctuaryStatus::Expanding), "expanding");
    }

    #[test]
    fn test_config_new() {
        let c = SanctuaryConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = SanctuaryConfig::new("test")
            .sanctuary_type(SanctuaryType::Bird)
            .status(SanctuaryStatus::Monitored);
        assert_eq!(c.sanctuary_type, SanctuaryType::Bird);
        assert_eq!(c.status, SanctuaryStatus::Monitored);
    }

    #[test]
    fn test_resident_new() {
        let r = SanctuaryResident::new("r1", "Title", "Content");
        assert_eq!(r.id, "r1");
    }

    #[test]
    fn test_resident_builder() {
        let r = SanctuaryResident::new("r1", "Title", "Content")
            .habitat(1);
        assert_eq!(r.habitat, 1);
    }

    #[test]
    fn test_resident_thriving() {
        let mut r = SanctuaryResident::new("r1", "Title", "Content");
        r.make_recovering();
        assert!(!r.thriving);
        r.make_thriving();
        assert!(r.thriving);
    }

    #[test]
    fn test_warden_new() {
        let w = SanctuaryWarden::new("key", "name", "r1");
        assert_eq!(w.resident_id, "r1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = SanctuaryStats::default();
        let resident = SanctuaryResident::new("r1", "Title", "Content");
        s.update(&[resident], SanctuaryType::Wildlife);
        assert_eq!(s.total_residents, 1);
        assert_eq!(s.thriving, 1);
    }

    #[test]
    fn test_sanctuary_new() {
        let s = SettingsSanctuary::new(SanctuaryConfig::default());
        assert_eq!(s.resident_count(), 0);
    }

    #[test]
    fn test_sanctuary_add_resident() {
        let mut s = SettingsSanctuary::new(SanctuaryConfig::default());
        s.add_resident(SanctuaryResident::new("r1", "Title", "Content"));
        assert_eq!(s.resident_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = SanctuaryRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SanctuaryRegistry::new();
        r.register("s1", SettingsSanctuary::new(SanctuaryConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_sanctuary_query() {
        assert!(is_sanctuary_query("settings sanctuary"));
        assert!(!is_sanctuary_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = sanctuary_fun_fact();
        assert!(fact.contains("sanctuary"));
    }
}
