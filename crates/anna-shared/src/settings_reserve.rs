// v0.0.782: Settings Reserve (Phase 358)
// Nature reserve for settings preservation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reserve type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ReserveType {
    /// Nature reserve
    #[default]
    Nature,
    /// Game reserve
    Game,
    /// Forest reserve
    Forest,
    /// Marine reserve
    Marine,
}

impl std::fmt::Display for ReserveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nature => write!(f, "nature"),
            Self::Game => write!(f, "game"),
            Self::Forest => write!(f, "forest"),
            Self::Marine => write!(f, "marine"),
        }
    }
}

/// Reserve status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ReserveStatus {
    /// Protected status
    #[default]
    Protected,
    /// Managed status
    Managed,
    /// Restored status
    Restored,
    /// Conserved status
    Conserved,
}

impl std::fmt::Display for ReserveStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Protected => write!(f, "protected"),
            Self::Managed => write!(f, "managed"),
            Self::Restored => write!(f, "restored"),
            Self::Conserved => write!(f, "conserved"),
        }
    }
}

/// Reserve config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReserveConfig {
    /// Name
    pub name: String,
    /// Reserve type
    pub reserve_type: ReserveType,
    /// Status
    pub status: ReserveStatus,
    /// Max species
    pub max_species: usize,
}

impl ReserveConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            reserve_type: ReserveType::Nature,
            status: ReserveStatus::Protected,
            max_species: 100,
        }
    }

    /// Set type
    pub fn reserve_type(mut self, rt: ReserveType) -> Self {
        self.reserve_type = rt;
        self
    }

    /// Set status
    pub fn status(mut self, s: ReserveStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max species
    pub fn max_species(mut self, max: usize) -> Self {
        self.max_species = max;
        self
    }
}

impl Default for ReserveConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Reserve species
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReserveSpecies {
    /// Species ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Territory number
    pub territory: u32,
    /// Thriving
    pub thriving: bool,
}

impl ReserveSpecies {
    /// Create new species
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            territory: 0,
            thriving: true,
        }
    }

    /// Set territory
    pub fn territory(mut self, t: u32) -> Self {
        self.territory = t;
        self
    }

    /// Make thriving
    pub fn make_thriving(&mut self) {
        self.thriving = true;
    }

    /// Make endangered
    pub fn make_endangered(&mut self) {
        self.thriving = false;
    }
}

/// Reserve ranger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReserveRanger {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Species ID
    pub species_id: String,
}

impl ReserveRanger {
    /// Create new ranger
    pub fn new(key: impl Into<String>, name: impl Into<String>, species_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            species_id: species_id.into(),
        }
    }
}

/// Reserve stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReserveStats {
    /// Total species
    pub total_species: usize,
    /// Thriving species
    pub thriving: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ReserveStats {
    /// Update from species
    pub fn update(&mut self, species: &[ReserveSpecies], reserve_type: ReserveType) {
        self.total_species = species.len();
        self.thriving = species.iter().filter(|s| s.thriving).count();
        *self.by_type.entry(reserve_type.to_string()).or_insert(0) += 1;
    }

    /// Thriving rate
    pub fn thriving_rate(&self) -> f64 {
        if self.total_species == 0 { 0.0 } else { self.thriving as f64 / self.total_species as f64 * 100.0 }
    }
}

/// Settings reserve
#[derive(Debug, Clone, Default)]
pub struct SettingsReserve {
    /// Config
    config: ReserveConfig,
    /// Species
    species: Vec<ReserveSpecies>,
    /// Rangers
    rangers: Vec<ReserveRanger>,
    /// Stats
    stats: ReserveStats,
}

impl SettingsReserve {
    /// Create new reserve system
    pub fn new(config: ReserveConfig) -> Self {
        Self {
            config,
            species: Vec::new(),
            rangers: Vec::new(),
            stats: ReserveStats::default(),
        }
    }

    /// Add species
    pub fn add_species(&mut self, species: ReserveSpecies) -> bool {
        if self.species.len() >= self.config.max_species {
            return false;
        }
        self.species.push(species);
        self.update_stats();
        true
    }

    /// Get species
    pub fn get_species(&self, id: &str) -> Option<&ReserveSpecies> {
        self.species.iter().find(|s| s.id == id)
    }

    /// Get species mut
    pub fn get_species_mut(&mut self, id: &str) -> Option<&mut ReserveSpecies> {
        self.species.iter_mut().find(|s| s.id == id)
    }

    /// Add ranger
    pub fn add_ranger(&mut self, ranger: ReserveRanger) {
        self.rangers.push(ranger);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.species, self.config.reserve_type);
    }

    /// Get stats
    pub fn stats(&self) -> &ReserveStats {
        &self.stats
    }

    /// Species count
    pub fn species_count(&self) -> usize {
        self.species.len()
    }
}

/// Reserve registry
#[derive(Debug, Clone, Default)]
pub struct ReserveRegistry {
    /// Reserves by ID
    reserves: HashMap<String, SettingsReserve>,
}

impl ReserveRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register reserve
    pub fn register(&mut self, id: impl Into<String>, reserve: SettingsReserve) {
        self.reserves.insert(id.into(), reserve);
    }

    /// Unregister reserve
    pub fn unregister(&mut self, id: &str) -> bool {
        self.reserves.remove(id).is_some()
    }

    /// Get reserve
    pub fn get(&self, id: &str) -> Option<&SettingsReserve> {
        self.reserves.get(id)
    }

    /// Get reserve mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsReserve> {
        self.reserves.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.reserves.len()
    }
}

/// Format reserve registry
pub fn format_reserve_registry(registry: &ReserveRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Reserve Registry:\n");
    output.push_str(&format!("  Reserves: {}\n", registry.count()));
    output
}

/// Check if query is about reserve
pub fn is_reserve_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings reserve") || lower.contains("reserve settings") || lower.contains("nature reserve")
}

/// Fun fact about reserve
pub fn reserve_fun_fact() -> &'static str {
    "Anna's settings reserve preserves conservation boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reserve_type_display() {
        assert_eq!(format!("{}", ReserveType::Nature), "nature");
        assert_eq!(format!("{}", ReserveType::Game), "game");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ReserveStatus::Protected), "protected");
        assert_eq!(format!("{}", ReserveStatus::Conserved), "conserved");
    }

    #[test]
    fn test_config_new() {
        let c = ReserveConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ReserveConfig::new("test")
            .reserve_type(ReserveType::Game)
            .status(ReserveStatus::Managed);
        assert_eq!(c.reserve_type, ReserveType::Game);
        assert_eq!(c.status, ReserveStatus::Managed);
    }

    #[test]
    fn test_species_new() {
        let s = ReserveSpecies::new("s1", "Title", "Content");
        assert_eq!(s.id, "s1");
    }

    #[test]
    fn test_species_builder() {
        let s = ReserveSpecies::new("s1", "Title", "Content")
            .territory(1);
        assert_eq!(s.territory, 1);
    }

    #[test]
    fn test_species_thriving() {
        let mut s = ReserveSpecies::new("s1", "Title", "Content");
        s.make_endangered();
        assert!(!s.thriving);
        s.make_thriving();
        assert!(s.thriving);
    }

    #[test]
    fn test_ranger_new() {
        let r = ReserveRanger::new("key", "name", "s1");
        assert_eq!(r.species_id, "s1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = ReserveStats::default();
        let species = ReserveSpecies::new("s1", "Title", "Content");
        s.update(&[species], ReserveType::Nature);
        assert_eq!(s.total_species, 1);
        assert_eq!(s.thriving, 1);
    }

    #[test]
    fn test_reserve_new() {
        let r = SettingsReserve::new(ReserveConfig::default());
        assert_eq!(r.species_count(), 0);
    }

    #[test]
    fn test_reserve_add_species() {
        let mut r = SettingsReserve::new(ReserveConfig::default());
        r.add_species(ReserveSpecies::new("s1", "Title", "Content"));
        assert_eq!(r.species_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = ReserveRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ReserveRegistry::new();
        r.register("r1", SettingsReserve::new(ReserveConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_reserve_query() {
        assert!(is_reserve_query("settings reserve"));
        assert!(!is_reserve_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = reserve_fun_fact();
        assert!(fact.contains("reserve"));
    }
}
