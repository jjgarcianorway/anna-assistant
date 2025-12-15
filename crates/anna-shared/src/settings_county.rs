// v0.0.749: Settings County (Phase 325)
// County level for settings governance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// County type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CountyType {
    /// Metropolitan county
    #[default]
    Metropolitan,
    /// Rural county
    Rural,
    /// Historic county
    Historic,
    /// Administrative county
    Administrative,
}

impl std::fmt::Display for CountyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Metropolitan => write!(f, "metropolitan"),
            Self::Rural => write!(f, "rural"),
            Self::Historic => write!(f, "historic"),
            Self::Administrative => write!(f, "administrative"),
        }
    }
}

/// County status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CountyStatus {
    /// Established status
    #[default]
    Established,
    /// Active status
    Active,
    /// Merged status
    Merged,
    /// Abolished status
    Abolished,
}

impl std::fmt::Display for CountyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Established => write!(f, "established"),
            Self::Active => write!(f, "active"),
            Self::Merged => write!(f, "merged"),
            Self::Abolished => write!(f, "abolished"),
        }
    }
}

/// County config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountyConfig {
    /// Name
    pub name: String,
    /// County type
    pub county_type: CountyType,
    /// Status
    pub status: CountyStatus,
    /// Max ordinances
    pub max_ordinances: usize,
}

impl CountyConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            county_type: CountyType::Metropolitan,
            status: CountyStatus::Established,
            max_ordinances: 100,
        }
    }

    /// Set type
    pub fn county_type(mut self, ct: CountyType) -> Self {
        self.county_type = ct;
        self
    }

    /// Set status
    pub fn status(mut self, s: CountyStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max ordinances
    pub fn max_ordinances(mut self, max: usize) -> Self {
        self.max_ordinances = max;
        self
    }
}

impl Default for CountyConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// County ordinance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountyOrdinance {
    /// Ordinance ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Township number
    pub township: u32,
    /// Enacted
    pub enacted: bool,
}

impl CountyOrdinance {
    /// Create new ordinance
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            township: 0,
            enacted: true,
        }
    }

    /// Set township
    pub fn township(mut self, t: u32) -> Self {
        self.township = t;
        self
    }

    /// Make enacted
    pub fn make_enacted(&mut self) {
        self.enacted = true;
    }

    /// Make repealed
    pub fn make_repealed(&mut self) {
        self.enacted = false;
    }
}

/// County commissioner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountyCommissioner {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Ordinance ID
    pub ordinance_id: String,
}

impl CountyCommissioner {
    /// Create new commissioner
    pub fn new(key: impl Into<String>, name: impl Into<String>, ordinance_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            ordinance_id: ordinance_id.into(),
        }
    }
}

/// County stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CountyStats {
    /// Total ordinances
    pub total_ordinances: usize,
    /// Enacted ordinances
    pub enacted: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl CountyStats {
    /// Update from ordinances
    pub fn update(&mut self, ordinances: &[CountyOrdinance], county_type: CountyType) {
        self.total_ordinances = ordinances.len();
        self.enacted = ordinances.iter().filter(|o| o.enacted).count();
        *self.by_type.entry(county_type.to_string()).or_insert(0) += 1;
    }

    /// Enacted rate
    pub fn enacted_rate(&self) -> f64 {
        if self.total_ordinances == 0 { 0.0 } else { self.enacted as f64 / self.total_ordinances as f64 * 100.0 }
    }
}

/// Settings county
#[derive(Debug, Clone, Default)]
pub struct SettingsCounty {
    /// Config
    config: CountyConfig,
    /// Ordinances
    ordinances: Vec<CountyOrdinance>,
    /// Commissioners
    commissioners: Vec<CountyCommissioner>,
    /// Stats
    stats: CountyStats,
}

impl SettingsCounty {
    /// Create new county system
    pub fn new(config: CountyConfig) -> Self {
        Self {
            config,
            ordinances: Vec::new(),
            commissioners: Vec::new(),
            stats: CountyStats::default(),
        }
    }

    /// Add ordinance
    pub fn add_ordinance(&mut self, ordinance: CountyOrdinance) -> bool {
        if self.ordinances.len() >= self.config.max_ordinances {
            return false;
        }
        self.ordinances.push(ordinance);
        self.update_stats();
        true
    }

    /// Get ordinance
    pub fn get_ordinance(&self, id: &str) -> Option<&CountyOrdinance> {
        self.ordinances.iter().find(|o| o.id == id)
    }

    /// Get ordinance mut
    pub fn get_ordinance_mut(&mut self, id: &str) -> Option<&mut CountyOrdinance> {
        self.ordinances.iter_mut().find(|o| o.id == id)
    }

    /// Add commissioner
    pub fn add_commissioner(&mut self, commissioner: CountyCommissioner) {
        self.commissioners.push(commissioner);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.ordinances, self.config.county_type);
    }

    /// Get stats
    pub fn stats(&self) -> &CountyStats {
        &self.stats
    }

    /// Ordinance count
    pub fn ordinance_count(&self) -> usize {
        self.ordinances.len()
    }
}

/// County registry
#[derive(Debug, Clone, Default)]
pub struct CountyRegistry {
    /// Counties by ID
    counties: HashMap<String, SettingsCounty>,
}

impl CountyRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register county
    pub fn register(&mut self, id: impl Into<String>, county: SettingsCounty) {
        self.counties.insert(id.into(), county);
    }

    /// Unregister county
    pub fn unregister(&mut self, id: &str) -> bool {
        self.counties.remove(id).is_some()
    }

    /// Get county
    pub fn get(&self, id: &str) -> Option<&SettingsCounty> {
        self.counties.get(id)
    }

    /// Get county mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCounty> {
        self.counties.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.counties.len()
    }
}

/// Format county registry
pub fn format_county_registry(registry: &CountyRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings County Registry:\n");
    output.push_str(&format!("  Counties: {}\n", registry.count()));
    output
}

/// Check if query is about county
pub fn is_county_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings county") || lower.contains("county settings") || lower.contains("county level")
}

/// Fun fact about county
pub fn county_fun_fact() -> &'static str {
    "Anna's settings county establishes county-level governance!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_county_type_display() {
        assert_eq!(format!("{}", CountyType::Metropolitan), "metropolitan");
        assert_eq!(format!("{}", CountyType::Rural), "rural");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", CountyStatus::Established), "established");
        assert_eq!(format!("{}", CountyStatus::Active), "active");
    }

    #[test]
    fn test_config_new() {
        let c = CountyConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = CountyConfig::new("test")
            .county_type(CountyType::Historic)
            .status(CountyStatus::Active);
        assert_eq!(c.county_type, CountyType::Historic);
        assert_eq!(c.status, CountyStatus::Active);
    }

    #[test]
    fn test_ordinance_new() {
        let o = CountyOrdinance::new("o1", "Title", "Content");
        assert_eq!(o.id, "o1");
    }

    #[test]
    fn test_ordinance_builder() {
        let o = CountyOrdinance::new("o1", "Title", "Content")
            .township(1);
        assert_eq!(o.township, 1);
    }

    #[test]
    fn test_ordinance_enacted() {
        let mut o = CountyOrdinance::new("o1", "Title", "Content");
        o.make_repealed();
        assert!(!o.enacted);
        o.make_enacted();
        assert!(o.enacted);
    }

    #[test]
    fn test_commissioner_new() {
        let c = CountyCommissioner::new("key", "name", "o1");
        assert_eq!(c.ordinance_id, "o1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = CountyStats::default();
        let ordinance = CountyOrdinance::new("o1", "Title", "Content");
        s.update(&[ordinance], CountyType::Metropolitan);
        assert_eq!(s.total_ordinances, 1);
        assert_eq!(s.enacted, 1);
    }

    #[test]
    fn test_county_new() {
        let c = SettingsCounty::new(CountyConfig::default());
        assert_eq!(c.ordinance_count(), 0);
    }

    #[test]
    fn test_county_add_ordinance() {
        let mut c = SettingsCounty::new(CountyConfig::default());
        c.add_ordinance(CountyOrdinance::new("o1", "Title", "Content"));
        assert_eq!(c.ordinance_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = CountyRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = CountyRegistry::new();
        r.register("c1", SettingsCounty::new(CountyConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_county_query() {
        assert!(is_county_query("settings county"));
        assert!(!is_county_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = county_fun_fact();
        assert!(fact.contains("county"));
    }
}
