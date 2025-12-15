// v0.0.778: Settings Aviary (Phase 354)
// Bird aviary for settings ornithology

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Aviary type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AviaryType {
    /// Flight aviary
    #[default]
    Flight,
    /// Breeding aviary
    Breeding,
    /// Display aviary
    Display,
    /// Rescue aviary
    Rescue,
}

impl std::fmt::Display for AviaryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Flight => write!(f, "flight"),
            Self::Breeding => write!(f, "breeding"),
            Self::Display => write!(f, "display"),
            Self::Rescue => write!(f, "rescue"),
        }
    }
}

/// Aviary status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AviaryStatus {
    /// Active status
    #[default]
    Active,
    /// Nesting status
    Nesting,
    /// Molting status
    Molting,
    /// Quarantine status
    Quarantine,
}

impl std::fmt::Display for AviaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Nesting => write!(f, "nesting"),
            Self::Molting => write!(f, "molting"),
            Self::Quarantine => write!(f, "quarantine"),
        }
    }
}

/// Aviary config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AviaryConfig {
    /// Name
    pub name: String,
    /// Aviary type
    pub aviary_type: AviaryType,
    /// Status
    pub status: AviaryStatus,
    /// Max birds
    pub max_birds: usize,
}

impl AviaryConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            aviary_type: AviaryType::Flight,
            status: AviaryStatus::Active,
            max_birds: 100,
        }
    }

    /// Set type
    pub fn aviary_type(mut self, at: AviaryType) -> Self {
        self.aviary_type = at;
        self
    }

    /// Set status
    pub fn status(mut self, s: AviaryStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max birds
    pub fn max_birds(mut self, max: usize) -> Self {
        self.max_birds = max;
        self
    }
}

impl Default for AviaryConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Aviary bird
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AviaryBird {
    /// Bird ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Perch number
    pub perch: u32,
    /// Flying
    pub flying: bool,
}

impl AviaryBird {
    /// Create new bird
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            perch: 0,
            flying: true,
        }
    }

    /// Set perch
    pub fn perch(mut self, p: u32) -> Self {
        self.perch = p;
        self
    }

    /// Make flying
    pub fn make_flying(&mut self) {
        self.flying = true;
    }

    /// Make grounded
    pub fn make_grounded(&mut self) {
        self.flying = false;
    }
}

/// Aviary keeper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AviaryKeeper {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Bird ID
    pub bird_id: String,
}

impl AviaryKeeper {
    /// Create new keeper
    pub fn new(key: impl Into<String>, name: impl Into<String>, bird_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            bird_id: bird_id.into(),
        }
    }
}

/// Aviary stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AviaryStats {
    /// Total birds
    pub total_birds: usize,
    /// Flying birds
    pub flying: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl AviaryStats {
    /// Update from birds
    pub fn update(&mut self, birds: &[AviaryBird], aviary_type: AviaryType) {
        self.total_birds = birds.len();
        self.flying = birds.iter().filter(|b| b.flying).count();
        *self.by_type.entry(aviary_type.to_string()).or_insert(0) += 1;
    }

    /// Flight rate
    pub fn flight_rate(&self) -> f64 {
        if self.total_birds == 0 { 0.0 } else { self.flying as f64 / self.total_birds as f64 * 100.0 }
    }
}

/// Settings aviary
#[derive(Debug, Clone, Default)]
pub struct SettingsAviary {
    /// Config
    config: AviaryConfig,
    /// Birds
    birds: Vec<AviaryBird>,
    /// Keepers
    keepers: Vec<AviaryKeeper>,
    /// Stats
    stats: AviaryStats,
}

impl SettingsAviary {
    /// Create new aviary system
    pub fn new(config: AviaryConfig) -> Self {
        Self {
            config,
            birds: Vec::new(),
            keepers: Vec::new(),
            stats: AviaryStats::default(),
        }
    }

    /// Add bird
    pub fn add_bird(&mut self, bird: AviaryBird) -> bool {
        if self.birds.len() >= self.config.max_birds {
            return false;
        }
        self.birds.push(bird);
        self.update_stats();
        true
    }

    /// Get bird
    pub fn get_bird(&self, id: &str) -> Option<&AviaryBird> {
        self.birds.iter().find(|b| b.id == id)
    }

    /// Get bird mut
    pub fn get_bird_mut(&mut self, id: &str) -> Option<&mut AviaryBird> {
        self.birds.iter_mut().find(|b| b.id == id)
    }

    /// Add keeper
    pub fn add_keeper(&mut self, keeper: AviaryKeeper) {
        self.keepers.push(keeper);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.birds, self.config.aviary_type);
    }

    /// Get stats
    pub fn stats(&self) -> &AviaryStats {
        &self.stats
    }

    /// Bird count
    pub fn bird_count(&self) -> usize {
        self.birds.len()
    }
}

/// Aviary registry
#[derive(Debug, Clone, Default)]
pub struct AviaryRegistry {
    /// Aviaries by ID
    aviaries: HashMap<String, SettingsAviary>,
}

impl AviaryRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register aviary
    pub fn register(&mut self, id: impl Into<String>, aviary: SettingsAviary) {
        self.aviaries.insert(id.into(), aviary);
    }

    /// Unregister aviary
    pub fn unregister(&mut self, id: &str) -> bool {
        self.aviaries.remove(id).is_some()
    }

    /// Get aviary
    pub fn get(&self, id: &str) -> Option<&SettingsAviary> {
        self.aviaries.get(id)
    }

    /// Get aviary mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsAviary> {
        self.aviaries.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.aviaries.len()
    }
}

/// Format aviary registry
pub fn format_aviary_registry(registry: &AviaryRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Aviary Registry:\n");
    output.push_str(&format!("  Aviaries: {}\n", registry.count()));
    output
}

/// Check if query is about aviary
pub fn is_aviary_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings aviary") || lower.contains("aviary settings") || lower.contains("bird aviary")
}

/// Fun fact about aviary
pub fn aviary_fun_fact() -> &'static str {
    "Anna's settings aviary houses ornithology boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aviary_type_display() {
        assert_eq!(format!("{}", AviaryType::Flight), "flight");
        assert_eq!(format!("{}", AviaryType::Breeding), "breeding");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", AviaryStatus::Active), "active");
        assert_eq!(format!("{}", AviaryStatus::Nesting), "nesting");
    }

    #[test]
    fn test_config_new() {
        let c = AviaryConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = AviaryConfig::new("test")
            .aviary_type(AviaryType::Display)
            .status(AviaryStatus::Molting);
        assert_eq!(c.aviary_type, AviaryType::Display);
        assert_eq!(c.status, AviaryStatus::Molting);
    }

    #[test]
    fn test_bird_new() {
        let b = AviaryBird::new("b1", "Title", "Content");
        assert_eq!(b.id, "b1");
    }

    #[test]
    fn test_bird_builder() {
        let b = AviaryBird::new("b1", "Title", "Content")
            .perch(1);
        assert_eq!(b.perch, 1);
    }

    #[test]
    fn test_bird_flying() {
        let mut b = AviaryBird::new("b1", "Title", "Content");
        b.make_grounded();
        assert!(!b.flying);
        b.make_flying();
        assert!(b.flying);
    }

    #[test]
    fn test_keeper_new() {
        let k = AviaryKeeper::new("key", "name", "b1");
        assert_eq!(k.bird_id, "b1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = AviaryStats::default();
        let bird = AviaryBird::new("b1", "Title", "Content");
        s.update(&[bird], AviaryType::Flight);
        assert_eq!(s.total_birds, 1);
        assert_eq!(s.flying, 1);
    }

    #[test]
    fn test_aviary_new() {
        let a = SettingsAviary::new(AviaryConfig::default());
        assert_eq!(a.bird_count(), 0);
    }

    #[test]
    fn test_aviary_add_bird() {
        let mut a = SettingsAviary::new(AviaryConfig::default());
        a.add_bird(AviaryBird::new("b1", "Title", "Content"));
        assert_eq!(a.bird_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = AviaryRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = AviaryRegistry::new();
        r.register("a1", SettingsAviary::new(AviaryConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_aviary_query() {
        assert!(is_aviary_query("settings aviary"));
        assert!(!is_aviary_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = aviary_fun_fact();
        assert!(fact.contains("aviary"));
    }
}
