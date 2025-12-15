// v0.0.754: Settings Neighborhood (Phase 330)
// Residential neighborhood for settings community

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Neighborhood type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NeighborhoodType {
    /// Residential neighborhood
    #[default]
    Residential,
    /// Commercial neighborhood
    Commercial,
    /// Industrial neighborhood
    Industrial,
    /// Mixed-use neighborhood
    MixedUse,
}

impl std::fmt::Display for NeighborhoodType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Residential => write!(f, "residential"),
            Self::Commercial => write!(f, "commercial"),
            Self::Industrial => write!(f, "industrial"),
            Self::MixedUse => write!(f, "mixed-use"),
        }
    }
}

/// Neighborhood status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NeighborhoodStatus {
    /// Planned status
    #[default]
    Planned,
    /// Developing status
    Developing,
    /// Established status
    Established,
    /// Revitalized status
    Revitalized,
}

impl std::fmt::Display for NeighborhoodStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Planned => write!(f, "planned"),
            Self::Developing => write!(f, "developing"),
            Self::Established => write!(f, "established"),
            Self::Revitalized => write!(f, "revitalized"),
        }
    }
}

/// Neighborhood config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborhoodConfig {
    /// Name
    pub name: String,
    /// Neighborhood type
    pub neighborhood_type: NeighborhoodType,
    /// Status
    pub status: NeighborhoodStatus,
    /// Max initiatives
    pub max_initiatives: usize,
}

impl NeighborhoodConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            neighborhood_type: NeighborhoodType::Residential,
            status: NeighborhoodStatus::Planned,
            max_initiatives: 100,
        }
    }

    /// Set type
    pub fn neighborhood_type(mut self, nt: NeighborhoodType) -> Self {
        self.neighborhood_type = nt;
        self
    }

    /// Set status
    pub fn status(mut self, s: NeighborhoodStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max initiatives
    pub fn max_initiatives(mut self, max: usize) -> Self {
        self.max_initiatives = max;
        self
    }
}

impl Default for NeighborhoodConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Neighborhood initiative
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborhoodInitiative {
    /// Initiative ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Block number
    pub block: u32,
    /// Approved
    pub approved: bool,
}

impl NeighborhoodInitiative {
    /// Create new initiative
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            block: 0,
            approved: true,
        }
    }

    /// Set block
    pub fn block(mut self, b: u32) -> Self {
        self.block = b;
        self
    }

    /// Make approved
    pub fn make_approved(&mut self) {
        self.approved = true;
    }

    /// Make rejected
    pub fn make_rejected(&mut self) {
        self.approved = false;
    }
}

/// Neighborhood organizer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborhoodOrganizer {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Initiative ID
    pub initiative_id: String,
}

impl NeighborhoodOrganizer {
    /// Create new organizer
    pub fn new(key: impl Into<String>, name: impl Into<String>, initiative_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            initiative_id: initiative_id.into(),
        }
    }
}

/// Neighborhood stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NeighborhoodStats {
    /// Total initiatives
    pub total_initiatives: usize,
    /// Approved initiatives
    pub approved: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl NeighborhoodStats {
    /// Update from initiatives
    pub fn update(&mut self, initiatives: &[NeighborhoodInitiative], neighborhood_type: NeighborhoodType) {
        self.total_initiatives = initiatives.len();
        self.approved = initiatives.iter().filter(|i| i.approved).count();
        *self.by_type.entry(neighborhood_type.to_string()).or_insert(0) += 1;
    }

    /// Approved rate
    pub fn approved_rate(&self) -> f64 {
        if self.total_initiatives == 0 { 0.0 } else { self.approved as f64 / self.total_initiatives as f64 * 100.0 }
    }
}

/// Settings neighborhood
#[derive(Debug, Clone, Default)]
pub struct SettingsNeighborhood {
    /// Config
    config: NeighborhoodConfig,
    /// Initiatives
    initiatives: Vec<NeighborhoodInitiative>,
    /// Organizers
    organizers: Vec<NeighborhoodOrganizer>,
    /// Stats
    stats: NeighborhoodStats,
}

impl SettingsNeighborhood {
    /// Create new neighborhood system
    pub fn new(config: NeighborhoodConfig) -> Self {
        Self {
            config,
            initiatives: Vec::new(),
            organizers: Vec::new(),
            stats: NeighborhoodStats::default(),
        }
    }

    /// Add initiative
    pub fn add_initiative(&mut self, initiative: NeighborhoodInitiative) -> bool {
        if self.initiatives.len() >= self.config.max_initiatives {
            return false;
        }
        self.initiatives.push(initiative);
        self.update_stats();
        true
    }

    /// Get initiative
    pub fn get_initiative(&self, id: &str) -> Option<&NeighborhoodInitiative> {
        self.initiatives.iter().find(|i| i.id == id)
    }

    /// Get initiative mut
    pub fn get_initiative_mut(&mut self, id: &str) -> Option<&mut NeighborhoodInitiative> {
        self.initiatives.iter_mut().find(|i| i.id == id)
    }

    /// Add organizer
    pub fn add_organizer(&mut self, organizer: NeighborhoodOrganizer) {
        self.organizers.push(organizer);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.initiatives, self.config.neighborhood_type);
    }

    /// Get stats
    pub fn stats(&self) -> &NeighborhoodStats {
        &self.stats
    }

    /// Initiative count
    pub fn initiative_count(&self) -> usize {
        self.initiatives.len()
    }
}

/// Neighborhood registry
#[derive(Debug, Clone, Default)]
pub struct NeighborhoodRegistry {
    /// Neighborhoods by ID
    neighborhoods: HashMap<String, SettingsNeighborhood>,
}

impl NeighborhoodRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register neighborhood
    pub fn register(&mut self, id: impl Into<String>, neighborhood: SettingsNeighborhood) {
        self.neighborhoods.insert(id.into(), neighborhood);
    }

    /// Unregister neighborhood
    pub fn unregister(&mut self, id: &str) -> bool {
        self.neighborhoods.remove(id).is_some()
    }

    /// Get neighborhood
    pub fn get(&self, id: &str) -> Option<&SettingsNeighborhood> {
        self.neighborhoods.get(id)
    }

    /// Get neighborhood mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsNeighborhood> {
        self.neighborhoods.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.neighborhoods.len()
    }
}

/// Format neighborhood registry
pub fn format_neighborhood_registry(registry: &NeighborhoodRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Neighborhood Registry:\n");
    output.push_str(&format!("  Neighborhoods: {}\n", registry.count()));
    output
}

/// Check if query is about neighborhood
pub fn is_neighborhood_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings neighborhood") || lower.contains("neighborhood settings") || lower.contains("residential neighborhood")
}

/// Fun fact about neighborhood
pub fn neighborhood_fun_fact() -> &'static str {
    "Anna's settings neighborhood establishes community participation!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neighborhood_type_display() {
        assert_eq!(format!("{}", NeighborhoodType::Residential), "residential");
        assert_eq!(format!("{}", NeighborhoodType::Commercial), "commercial");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", NeighborhoodStatus::Planned), "planned");
        assert_eq!(format!("{}", NeighborhoodStatus::Established), "established");
    }

    #[test]
    fn test_config_new() {
        let c = NeighborhoodConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = NeighborhoodConfig::new("test")
            .neighborhood_type(NeighborhoodType::Commercial)
            .status(NeighborhoodStatus::Developing);
        assert_eq!(c.neighborhood_type, NeighborhoodType::Commercial);
        assert_eq!(c.status, NeighborhoodStatus::Developing);
    }

    #[test]
    fn test_initiative_new() {
        let i = NeighborhoodInitiative::new("i1", "Title", "Content");
        assert_eq!(i.id, "i1");
    }

    #[test]
    fn test_initiative_builder() {
        let i = NeighborhoodInitiative::new("i1", "Title", "Content")
            .block(1);
        assert_eq!(i.block, 1);
    }

    #[test]
    fn test_initiative_approved() {
        let mut i = NeighborhoodInitiative::new("i1", "Title", "Content");
        i.make_rejected();
        assert!(!i.approved);
        i.make_approved();
        assert!(i.approved);
    }

    #[test]
    fn test_organizer_new() {
        let o = NeighborhoodOrganizer::new("key", "name", "i1");
        assert_eq!(o.initiative_id, "i1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = NeighborhoodStats::default();
        let initiative = NeighborhoodInitiative::new("i1", "Title", "Content");
        s.update(&[initiative], NeighborhoodType::Residential);
        assert_eq!(s.total_initiatives, 1);
        assert_eq!(s.approved, 1);
    }

    #[test]
    fn test_neighborhood_new() {
        let n = SettingsNeighborhood::new(NeighborhoodConfig::default());
        assert_eq!(n.initiative_count(), 0);
    }

    #[test]
    fn test_neighborhood_add_initiative() {
        let mut n = SettingsNeighborhood::new(NeighborhoodConfig::default());
        n.add_initiative(NeighborhoodInitiative::new("i1", "Title", "Content"));
        assert_eq!(n.initiative_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = NeighborhoodRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = NeighborhoodRegistry::new();
        r.register("n1", SettingsNeighborhood::new(NeighborhoodConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_neighborhood_query() {
        assert!(is_neighborhood_query("settings neighborhood"));
        assert!(!is_neighborhood_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = neighborhood_fun_fact();
        assert!(fact.contains("neighborhood"));
    }
}
