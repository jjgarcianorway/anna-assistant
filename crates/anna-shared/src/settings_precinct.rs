// v0.0.753: Settings Precinct (Phase 329)
// Voting precinct for settings participation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Precinct type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PrecinctType {
    /// Voting precinct
    #[default]
    Voting,
    /// Police precinct
    Police,
    /// Fire precinct
    Fire,
    /// School precinct
    School,
}

impl std::fmt::Display for PrecinctType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Voting => write!(f, "voting"),
            Self::Police => write!(f, "police"),
            Self::Fire => write!(f, "fire"),
            Self::School => write!(f, "school"),
        }
    }
}

/// Precinct status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PrecinctStatus {
    /// Designated status
    #[default]
    Designated,
    /// Active status
    Active,
    /// Consolidated status
    Consolidated,
    /// Dissolved status
    Dissolved,
}

impl std::fmt::Display for PrecinctStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Designated => write!(f, "designated"),
            Self::Active => write!(f, "active"),
            Self::Consolidated => write!(f, "consolidated"),
            Self::Dissolved => write!(f, "dissolved"),
        }
    }
}

/// Precinct config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrecinctConfig {
    /// Name
    pub name: String,
    /// Precinct type
    pub precinct_type: PrecinctType,
    /// Status
    pub status: PrecinctStatus,
    /// Max ballots
    pub max_ballots: usize,
}

impl PrecinctConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            precinct_type: PrecinctType::Voting,
            status: PrecinctStatus::Designated,
            max_ballots: 100,
        }
    }

    /// Set type
    pub fn precinct_type(mut self, pt: PrecinctType) -> Self {
        self.precinct_type = pt;
        self
    }

    /// Set status
    pub fn status(mut self, s: PrecinctStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max ballots
    pub fn max_ballots(mut self, max: usize) -> Self {
        self.max_ballots = max;
        self
    }
}

impl Default for PrecinctConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Precinct ballot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrecinctBallot {
    /// Ballot ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// District number
    pub district: u32,
    /// Certified
    pub certified: bool,
}

impl PrecinctBallot {
    /// Create new ballot
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            district: 0,
            certified: true,
        }
    }

    /// Set district
    pub fn district(mut self, d: u32) -> Self {
        self.district = d;
        self
    }

    /// Make certified
    pub fn make_certified(&mut self) {
        self.certified = true;
    }

    /// Make contested
    pub fn make_contested(&mut self) {
        self.certified = false;
    }
}

/// Precinct captain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrecinctCaptain {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Ballot ID
    pub ballot_id: String,
}

impl PrecinctCaptain {
    /// Create new captain
    pub fn new(key: impl Into<String>, name: impl Into<String>, ballot_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            ballot_id: ballot_id.into(),
        }
    }
}

/// Precinct stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrecinctStats {
    /// Total ballots
    pub total_ballots: usize,
    /// Certified ballots
    pub certified: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl PrecinctStats {
    /// Update from ballots
    pub fn update(&mut self, ballots: &[PrecinctBallot], precinct_type: PrecinctType) {
        self.total_ballots = ballots.len();
        self.certified = ballots.iter().filter(|b| b.certified).count();
        *self.by_type.entry(precinct_type.to_string()).or_insert(0) += 1;
    }

    /// Certified rate
    pub fn certified_rate(&self) -> f64 {
        if self.total_ballots == 0 { 0.0 } else { self.certified as f64 / self.total_ballots as f64 * 100.0 }
    }
}

/// Settings precinct
#[derive(Debug, Clone, Default)]
pub struct SettingsPrecinct {
    /// Config
    config: PrecinctConfig,
    /// Ballots
    ballots: Vec<PrecinctBallot>,
    /// Captains
    captains: Vec<PrecinctCaptain>,
    /// Stats
    stats: PrecinctStats,
}

impl SettingsPrecinct {
    /// Create new precinct system
    pub fn new(config: PrecinctConfig) -> Self {
        Self {
            config,
            ballots: Vec::new(),
            captains: Vec::new(),
            stats: PrecinctStats::default(),
        }
    }

    /// Add ballot
    pub fn add_ballot(&mut self, ballot: PrecinctBallot) -> bool {
        if self.ballots.len() >= self.config.max_ballots {
            return false;
        }
        self.ballots.push(ballot);
        self.update_stats();
        true
    }

    /// Get ballot
    pub fn get_ballot(&self, id: &str) -> Option<&PrecinctBallot> {
        self.ballots.iter().find(|b| b.id == id)
    }

    /// Get ballot mut
    pub fn get_ballot_mut(&mut self, id: &str) -> Option<&mut PrecinctBallot> {
        self.ballots.iter_mut().find(|b| b.id == id)
    }

    /// Add captain
    pub fn add_captain(&mut self, captain: PrecinctCaptain) {
        self.captains.push(captain);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.ballots, self.config.precinct_type);
    }

    /// Get stats
    pub fn stats(&self) -> &PrecinctStats {
        &self.stats
    }

    /// Ballot count
    pub fn ballot_count(&self) -> usize {
        self.ballots.len()
    }
}

/// Precinct registry
#[derive(Debug, Clone, Default)]
pub struct PrecinctRegistry {
    /// Precincts by ID
    precincts: HashMap<String, SettingsPrecinct>,
}

impl PrecinctRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register precinct
    pub fn register(&mut self, id: impl Into<String>, precinct: SettingsPrecinct) {
        self.precincts.insert(id.into(), precinct);
    }

    /// Unregister precinct
    pub fn unregister(&mut self, id: &str) -> bool {
        self.precincts.remove(id).is_some()
    }

    /// Get precinct
    pub fn get(&self, id: &str) -> Option<&SettingsPrecinct> {
        self.precincts.get(id)
    }

    /// Get precinct mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsPrecinct> {
        self.precincts.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.precincts.len()
    }
}

/// Format precinct registry
pub fn format_precinct_registry(registry: &PrecinctRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Precinct Registry:\n");
    output.push_str(&format!("  Precincts: {}\n", registry.count()));
    output
}

/// Check if query is about precinct
pub fn is_precinct_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings precinct") || lower.contains("precinct settings") || lower.contains("voting precinct")
}

/// Fun fact about precinct
pub fn precinct_fun_fact() -> &'static str {
    "Anna's settings precinct establishes voting participation!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precinct_type_display() {
        assert_eq!(format!("{}", PrecinctType::Voting), "voting");
        assert_eq!(format!("{}", PrecinctType::Police), "police");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", PrecinctStatus::Designated), "designated");
        assert_eq!(format!("{}", PrecinctStatus::Active), "active");
    }

    #[test]
    fn test_config_new() {
        let c = PrecinctConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = PrecinctConfig::new("test")
            .precinct_type(PrecinctType::Fire)
            .status(PrecinctStatus::Consolidated);
        assert_eq!(c.precinct_type, PrecinctType::Fire);
        assert_eq!(c.status, PrecinctStatus::Consolidated);
    }

    #[test]
    fn test_ballot_new() {
        let b = PrecinctBallot::new("b1", "Title", "Content");
        assert_eq!(b.id, "b1");
    }

    #[test]
    fn test_ballot_builder() {
        let b = PrecinctBallot::new("b1", "Title", "Content")
            .district(1);
        assert_eq!(b.district, 1);
    }

    #[test]
    fn test_ballot_certified() {
        let mut b = PrecinctBallot::new("b1", "Title", "Content");
        b.make_contested();
        assert!(!b.certified);
        b.make_certified();
        assert!(b.certified);
    }

    #[test]
    fn test_captain_new() {
        let c = PrecinctCaptain::new("key", "name", "b1");
        assert_eq!(c.ballot_id, "b1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = PrecinctStats::default();
        let ballot = PrecinctBallot::new("b1", "Title", "Content");
        s.update(&[ballot], PrecinctType::Voting);
        assert_eq!(s.total_ballots, 1);
        assert_eq!(s.certified, 1);
    }

    #[test]
    fn test_precinct_new() {
        let p = SettingsPrecinct::new(PrecinctConfig::default());
        assert_eq!(p.ballot_count(), 0);
    }

    #[test]
    fn test_precinct_add_ballot() {
        let mut p = SettingsPrecinct::new(PrecinctConfig::default());
        p.add_ballot(PrecinctBallot::new("b1", "Title", "Content"));
        assert_eq!(p.ballot_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = PrecinctRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = PrecinctRegistry::new();
        r.register("p1", SettingsPrecinct::new(PrecinctConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_precinct_query() {
        assert!(is_precinct_query("settings precinct"));
        assert!(!is_precinct_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = precinct_fun_fact();
        assert!(fact.contains("precinct"));
    }
}
