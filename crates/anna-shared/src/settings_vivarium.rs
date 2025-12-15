// v0.0.776: Settings Vivarium (Phase 352)
// Living vivarium for settings animal habitat

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Vivarium type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum VivariumType {
    /// Reptile vivarium
    #[default]
    Reptile,
    /// Amphibian vivarium
    Amphibian,
    /// Invertebrate vivarium
    Invertebrate,
    /// Mixed vivarium
    Mixed,
}

impl std::fmt::Display for VivariumType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reptile => write!(f, "reptile"),
            Self::Amphibian => write!(f, "amphibian"),
            Self::Invertebrate => write!(f, "invertebrate"),
            Self::Mixed => write!(f, "mixed"),
        }
    }
}

/// Vivarium status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum VivariumStatus {
    /// Setup status
    #[default]
    Setup,
    /// Established status
    Established,
    /// Breeding status
    Breeding,
    /// Resting status
    Resting,
}

impl std::fmt::Display for VivariumStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Setup => write!(f, "setup"),
            Self::Established => write!(f, "established"),
            Self::Breeding => write!(f, "breeding"),
            Self::Resting => write!(f, "resting"),
        }
    }
}

/// Vivarium config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VivariumConfig {
    /// Name
    pub name: String,
    /// Vivarium type
    pub vivarium_type: VivariumType,
    /// Status
    pub status: VivariumStatus,
    /// Max creatures
    pub max_creatures: usize,
}

impl VivariumConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            vivarium_type: VivariumType::Reptile,
            status: VivariumStatus::Setup,
            max_creatures: 100,
        }
    }

    /// Set type
    pub fn vivarium_type(mut self, vt: VivariumType) -> Self {
        self.vivarium_type = vt;
        self
    }

    /// Set status
    pub fn status(mut self, s: VivariumStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max creatures
    pub fn max_creatures(mut self, max: usize) -> Self {
        self.max_creatures = max;
        self
    }
}

impl Default for VivariumConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Vivarium creature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VivariumCreature {
    /// Creature ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Enclosure number
    pub enclosure: u32,
    /// Thriving
    pub thriving: bool,
}

impl VivariumCreature {
    /// Create new creature
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            enclosure: 0,
            thriving: true,
        }
    }

    /// Set enclosure
    pub fn enclosure(mut self, e: u32) -> Self {
        self.enclosure = e;
        self
    }

    /// Make thriving
    pub fn make_thriving(&mut self) {
        self.thriving = true;
    }

    /// Make struggling
    pub fn make_struggling(&mut self) {
        self.thriving = false;
    }
}

/// Vivarium keeper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VivariumKeeper {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Creature ID
    pub creature_id: String,
}

impl VivariumKeeper {
    /// Create new keeper
    pub fn new(key: impl Into<String>, name: impl Into<String>, creature_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            creature_id: creature_id.into(),
        }
    }
}

/// Vivarium stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VivariumStats {
    /// Total creatures
    pub total_creatures: usize,
    /// Thriving creatures
    pub thriving: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl VivariumStats {
    /// Update from creatures
    pub fn update(&mut self, creatures: &[VivariumCreature], vivarium_type: VivariumType) {
        self.total_creatures = creatures.len();
        self.thriving = creatures.iter().filter(|c| c.thriving).count();
        *self.by_type.entry(vivarium_type.to_string()).or_insert(0) += 1;
    }

    /// Thriving rate
    pub fn thriving_rate(&self) -> f64 {
        if self.total_creatures == 0 { 0.0 } else { self.thriving as f64 / self.total_creatures as f64 * 100.0 }
    }
}

/// Settings vivarium
#[derive(Debug, Clone, Default)]
pub struct SettingsVivarium {
    /// Config
    config: VivariumConfig,
    /// Creatures
    creatures: Vec<VivariumCreature>,
    /// Keepers
    keepers: Vec<VivariumKeeper>,
    /// Stats
    stats: VivariumStats,
}

impl SettingsVivarium {
    /// Create new vivarium system
    pub fn new(config: VivariumConfig) -> Self {
        Self {
            config,
            creatures: Vec::new(),
            keepers: Vec::new(),
            stats: VivariumStats::default(),
        }
    }

    /// Add creature
    pub fn add_creature(&mut self, creature: VivariumCreature) -> bool {
        if self.creatures.len() >= self.config.max_creatures {
            return false;
        }
        self.creatures.push(creature);
        self.update_stats();
        true
    }

    /// Get creature
    pub fn get_creature(&self, id: &str) -> Option<&VivariumCreature> {
        self.creatures.iter().find(|c| c.id == id)
    }

    /// Get creature mut
    pub fn get_creature_mut(&mut self, id: &str) -> Option<&mut VivariumCreature> {
        self.creatures.iter_mut().find(|c| c.id == id)
    }

    /// Add keeper
    pub fn add_keeper(&mut self, keeper: VivariumKeeper) {
        self.keepers.push(keeper);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.creatures, self.config.vivarium_type);
    }

    /// Get stats
    pub fn stats(&self) -> &VivariumStats {
        &self.stats
    }

    /// Creature count
    pub fn creature_count(&self) -> usize {
        self.creatures.len()
    }
}

/// Vivarium registry
#[derive(Debug, Clone, Default)]
pub struct VivariumRegistry {
    /// Vivariums by ID
    vivariums: HashMap<String, SettingsVivarium>,
}

impl VivariumRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register vivarium
    pub fn register(&mut self, id: impl Into<String>, vivarium: SettingsVivarium) {
        self.vivariums.insert(id.into(), vivarium);
    }

    /// Unregister vivarium
    pub fn unregister(&mut self, id: &str) -> bool {
        self.vivariums.remove(id).is_some()
    }

    /// Get vivarium
    pub fn get(&self, id: &str) -> Option<&SettingsVivarium> {
        self.vivariums.get(id)
    }

    /// Get vivarium mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsVivarium> {
        self.vivariums.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.vivariums.len()
    }
}

/// Format vivarium registry
pub fn format_vivarium_registry(registry: &VivariumRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Vivarium Registry:\n");
    output.push_str(&format!("  Vivariums: {}\n", registry.count()));
    output
}

/// Check if query is about vivarium
pub fn is_vivarium_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings vivarium") || lower.contains("vivarium settings") || lower.contains("living vivarium")
}

/// Fun fact about vivarium
pub fn vivarium_fun_fact() -> &'static str {
    "Anna's settings vivarium maintains animal habitat boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vivarium_type_display() {
        assert_eq!(format!("{}", VivariumType::Reptile), "reptile");
        assert_eq!(format!("{}", VivariumType::Amphibian), "amphibian");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", VivariumStatus::Setup), "setup");
        assert_eq!(format!("{}", VivariumStatus::Established), "established");
    }

    #[test]
    fn test_config_new() {
        let c = VivariumConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = VivariumConfig::new("test")
            .vivarium_type(VivariumType::Invertebrate)
            .status(VivariumStatus::Breeding);
        assert_eq!(c.vivarium_type, VivariumType::Invertebrate);
        assert_eq!(c.status, VivariumStatus::Breeding);
    }

    #[test]
    fn test_creature_new() {
        let c = VivariumCreature::new("c1", "Title", "Content");
        assert_eq!(c.id, "c1");
    }

    #[test]
    fn test_creature_builder() {
        let c = VivariumCreature::new("c1", "Title", "Content")
            .enclosure(1);
        assert_eq!(c.enclosure, 1);
    }

    #[test]
    fn test_creature_thriving() {
        let mut c = VivariumCreature::new("c1", "Title", "Content");
        c.make_struggling();
        assert!(!c.thriving);
        c.make_thriving();
        assert!(c.thriving);
    }

    #[test]
    fn test_keeper_new() {
        let k = VivariumKeeper::new("key", "name", "c1");
        assert_eq!(k.creature_id, "c1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = VivariumStats::default();
        let creature = VivariumCreature::new("c1", "Title", "Content");
        s.update(&[creature], VivariumType::Reptile);
        assert_eq!(s.total_creatures, 1);
        assert_eq!(s.thriving, 1);
    }

    #[test]
    fn test_vivarium_new() {
        let v = SettingsVivarium::new(VivariumConfig::default());
        assert_eq!(v.creature_count(), 0);
    }

    #[test]
    fn test_vivarium_add_creature() {
        let mut v = SettingsVivarium::new(VivariumConfig::default());
        v.add_creature(VivariumCreature::new("c1", "Title", "Content"));
        assert_eq!(v.creature_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = VivariumRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = VivariumRegistry::new();
        r.register("v1", SettingsVivarium::new(VivariumConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_vivarium_query() {
        assert!(is_vivarium_query("settings vivarium"));
        assert!(!is_vivarium_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = vivarium_fun_fact();
        assert!(fact.contains("vivarium"));
    }
}
