// v0.0.785: Settings Retreat (Phase 361)
// Peaceful retreat for settings relaxation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Retreat type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RetreatType {
    /// Peaceful retreat
    #[default]
    Peaceful,
    /// Mountain retreat
    Mountain,
    /// Coastal retreat
    Coastal,
    /// Forest retreat
    Forest,
}

impl std::fmt::Display for RetreatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Peaceful => write!(f, "peaceful"),
            Self::Mountain => write!(f, "mountain"),
            Self::Coastal => write!(f, "coastal"),
            Self::Forest => write!(f, "forest"),
        }
    }
}

/// Retreat status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RetreatStatus {
    /// Open status
    #[default]
    Open,
    /// Relaxing status
    Relaxing,
    /// Meditating status
    Meditating,
    /// Rejuvenating status
    Rejuvenating,
}

impl std::fmt::Display for RetreatStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Relaxing => write!(f, "relaxing"),
            Self::Meditating => write!(f, "meditating"),
            Self::Rejuvenating => write!(f, "rejuvenating"),
        }
    }
}

/// Retreat config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetreatConfig {
    /// Name
    pub name: String,
    /// Retreat type
    pub retreat_type: RetreatType,
    /// Status
    pub status: RetreatStatus,
    /// Max visitors
    pub max_visitors: usize,
}

impl RetreatConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            retreat_type: RetreatType::Peaceful,
            status: RetreatStatus::Open,
            max_visitors: 100,
        }
    }

    /// Set type
    pub fn retreat_type(mut self, rt: RetreatType) -> Self {
        self.retreat_type = rt;
        self
    }

    /// Set status
    pub fn status(mut self, s: RetreatStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max visitors
    pub fn max_visitors(mut self, max: usize) -> Self {
        self.max_visitors = max;
        self
    }
}

impl Default for RetreatConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Retreat visitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetreatVisitor {
    /// Visitor ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Cabin number
    pub cabin: u32,
    /// Relaxed
    pub relaxed: bool,
}

impl RetreatVisitor {
    /// Create new visitor
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            cabin: 0,
            relaxed: true,
        }
    }

    /// Set cabin
    pub fn cabin(mut self, c: u32) -> Self {
        self.cabin = c;
        self
    }

    /// Make relaxed
    pub fn make_relaxed(&mut self) {
        self.relaxed = true;
    }

    /// Make stressed
    pub fn make_stressed(&mut self) {
        self.relaxed = false;
    }
}

/// Retreat guide
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetreatGuide {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Visitor ID
    pub visitor_id: String,
}

impl RetreatGuide {
    /// Create new guide
    pub fn new(key: impl Into<String>, name: impl Into<String>, visitor_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            visitor_id: visitor_id.into(),
        }
    }
}

/// Retreat stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RetreatStats {
    /// Total visitors
    pub total_visitors: usize,
    /// Relaxed visitors
    pub relaxed: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl RetreatStats {
    /// Update from visitors
    pub fn update(&mut self, visitors: &[RetreatVisitor], retreat_type: RetreatType) {
        self.total_visitors = visitors.len();
        self.relaxed = visitors.iter().filter(|v| v.relaxed).count();
        *self.by_type.entry(retreat_type.to_string()).or_insert(0) += 1;
    }

    /// Relaxation rate
    pub fn relaxation_rate(&self) -> f64 {
        if self.total_visitors == 0 { 0.0 } else { self.relaxed as f64 / self.total_visitors as f64 * 100.0 }
    }
}

/// Settings retreat
#[derive(Debug, Clone, Default)]
pub struct SettingsRetreat {
    /// Config
    config: RetreatConfig,
    /// Visitors
    visitors: Vec<RetreatVisitor>,
    /// Guides
    guides: Vec<RetreatGuide>,
    /// Stats
    stats: RetreatStats,
}

impl SettingsRetreat {
    /// Create new retreat system
    pub fn new(config: RetreatConfig) -> Self {
        Self {
            config,
            visitors: Vec::new(),
            guides: Vec::new(),
            stats: RetreatStats::default(),
        }
    }

    /// Add visitor
    pub fn add_visitor(&mut self, visitor: RetreatVisitor) -> bool {
        if self.visitors.len() >= self.config.max_visitors {
            return false;
        }
        self.visitors.push(visitor);
        self.update_stats();
        true
    }

    /// Get visitor
    pub fn get_visitor(&self, id: &str) -> Option<&RetreatVisitor> {
        self.visitors.iter().find(|v| v.id == id)
    }

    /// Get visitor mut
    pub fn get_visitor_mut(&mut self, id: &str) -> Option<&mut RetreatVisitor> {
        self.visitors.iter_mut().find(|v| v.id == id)
    }

    /// Add guide
    pub fn add_guide(&mut self, guide: RetreatGuide) {
        self.guides.push(guide);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.visitors, self.config.retreat_type);
    }

    /// Get stats
    pub fn stats(&self) -> &RetreatStats {
        &self.stats
    }

    /// Visitor count
    pub fn visitor_count(&self) -> usize {
        self.visitors.len()
    }
}

/// Retreat registry
#[derive(Debug, Clone, Default)]
pub struct RetreatRegistry {
    /// Retreats by ID
    retreats: HashMap<String, SettingsRetreat>,
}

impl RetreatRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register retreat
    pub fn register(&mut self, id: impl Into<String>, retreat: SettingsRetreat) {
        self.retreats.insert(id.into(), retreat);
    }

    /// Unregister retreat
    pub fn unregister(&mut self, id: &str) -> bool {
        self.retreats.remove(id).is_some()
    }

    /// Get retreat
    pub fn get(&self, id: &str) -> Option<&SettingsRetreat> {
        self.retreats.get(id)
    }

    /// Get retreat mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsRetreat> {
        self.retreats.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.retreats.len()
    }
}

/// Format retreat registry
pub fn format_retreat_registry(registry: &RetreatRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Retreat Registry:\n");
    output.push_str(&format!("  Retreats: {}\n", registry.count()));
    output
}

/// Check if query is about retreat
pub fn is_retreat_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings retreat") || lower.contains("retreat settings") || lower.contains("peaceful retreat")
}

/// Fun fact about retreat
pub fn retreat_fun_fact() -> &'static str {
    "Anna's settings retreat provides peaceful relaxation for configurations!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retreat_type_display() {
        assert_eq!(format!("{}", RetreatType::Peaceful), "peaceful");
        assert_eq!(format!("{}", RetreatType::Mountain), "mountain");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", RetreatStatus::Open), "open");
        assert_eq!(format!("{}", RetreatStatus::Rejuvenating), "rejuvenating");
    }

    #[test]
    fn test_config_new() {
        let c = RetreatConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = RetreatConfig::new("test")
            .retreat_type(RetreatType::Mountain)
            .status(RetreatStatus::Meditating);
        assert_eq!(c.retreat_type, RetreatType::Mountain);
        assert_eq!(c.status, RetreatStatus::Meditating);
    }

    #[test]
    fn test_visitor_new() {
        let v = RetreatVisitor::new("v1", "Title", "Content");
        assert_eq!(v.id, "v1");
    }

    #[test]
    fn test_visitor_builder() {
        let v = RetreatVisitor::new("v1", "Title", "Content")
            .cabin(1);
        assert_eq!(v.cabin, 1);
    }

    #[test]
    fn test_visitor_relaxation() {
        let mut v = RetreatVisitor::new("v1", "Title", "Content");
        v.make_stressed();
        assert!(!v.relaxed);
        v.make_relaxed();
        assert!(v.relaxed);
    }

    #[test]
    fn test_guide_new() {
        let g = RetreatGuide::new("key", "name", "v1");
        assert_eq!(g.visitor_id, "v1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = RetreatStats::default();
        let visitor = RetreatVisitor::new("v1", "Title", "Content");
        s.update(&[visitor], RetreatType::Peaceful);
        assert_eq!(s.total_visitors, 1);
        assert_eq!(s.relaxed, 1);
    }

    #[test]
    fn test_retreat_new() {
        let r = SettingsRetreat::new(RetreatConfig::default());
        assert_eq!(r.visitor_count(), 0);
    }

    #[test]
    fn test_retreat_add_visitor() {
        let mut r = SettingsRetreat::new(RetreatConfig::default());
        r.add_visitor(RetreatVisitor::new("v1", "Title", "Content"));
        assert_eq!(r.visitor_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = RetreatRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = RetreatRegistry::new();
        r.register("r1", SettingsRetreat::new(RetreatConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_retreat_query() {
        assert!(is_retreat_query("settings retreat"));
        assert!(!is_retreat_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = retreat_fun_fact();
        assert!(fact.contains("retreat"));
    }
}
