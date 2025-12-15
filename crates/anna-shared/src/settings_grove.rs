// v0.0.765: Settings Grove (Phase 341)
// Tree grove for settings forestry

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Grove type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GroveType {
    /// Oak grove
    #[default]
    Oak,
    /// Olive grove
    Olive,
    /// Citrus grove
    Citrus,
    /// Sacred grove
    Sacred,
}

impl std::fmt::Display for GroveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Oak => write!(f, "oak"),
            Self::Olive => write!(f, "olive"),
            Self::Citrus => write!(f, "citrus"),
            Self::Sacred => write!(f, "sacred"),
        }
    }
}

/// Grove status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GroveStatus {
    /// Planted status
    #[default]
    Planted,
    /// Maturing status
    Maturing,
    /// Productive status
    Productive,
    /// Resting status
    Resting,
}

impl std::fmt::Display for GroveStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Planted => write!(f, "planted"),
            Self::Maturing => write!(f, "maturing"),
            Self::Productive => write!(f, "productive"),
            Self::Resting => write!(f, "resting"),
        }
    }
}

/// Grove config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroveConfig {
    /// Name
    pub name: String,
    /// Grove type
    pub grove_type: GroveType,
    /// Status
    pub status: GroveStatus,
    /// Max trees
    pub max_trees: usize,
}

impl GroveConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            grove_type: GroveType::Oak,
            status: GroveStatus::Planted,
            max_trees: 100,
        }
    }

    /// Set type
    pub fn grove_type(mut self, gt: GroveType) -> Self {
        self.grove_type = gt;
        self
    }

    /// Set status
    pub fn status(mut self, s: GroveStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max trees
    pub fn max_trees(mut self, max: usize) -> Self {
        self.max_trees = max;
        self
    }
}

impl Default for GroveConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Grove tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroveTree {
    /// Tree ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Row number
    pub row: u32,
    /// Healthy
    pub healthy: bool,
}

impl GroveTree {
    /// Create new tree
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            row: 0,
            healthy: true,
        }
    }

    /// Set row
    pub fn row(mut self, r: u32) -> Self {
        self.row = r;
        self
    }

    /// Make healthy
    pub fn make_healthy(&mut self) {
        self.healthy = true;
    }

    /// Make diseased
    pub fn make_diseased(&mut self) {
        self.healthy = false;
    }
}

/// Grove tender
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroveTender {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Tree ID
    pub tree_id: String,
}

impl GroveTender {
    /// Create new tender
    pub fn new(key: impl Into<String>, name: impl Into<String>, tree_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            tree_id: tree_id.into(),
        }
    }
}

/// Grove stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GroveStats {
    /// Total trees
    pub total_trees: usize,
    /// Healthy trees
    pub healthy: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl GroveStats {
    /// Update from trees
    pub fn update(&mut self, trees: &[GroveTree], grove_type: GroveType) {
        self.total_trees = trees.len();
        self.healthy = trees.iter().filter(|t| t.healthy).count();
        *self.by_type.entry(grove_type.to_string()).or_insert(0) += 1;
    }

    /// Healthy rate
    pub fn healthy_rate(&self) -> f64 {
        if self.total_trees == 0 { 0.0 } else { self.healthy as f64 / self.total_trees as f64 * 100.0 }
    }
}

/// Settings grove
#[derive(Debug, Clone, Default)]
pub struct SettingsGrove {
    /// Config
    config: GroveConfig,
    /// Trees
    trees: Vec<GroveTree>,
    /// Tenders
    tenders: Vec<GroveTender>,
    /// Stats
    stats: GroveStats,
}

impl SettingsGrove {
    /// Create new grove system
    pub fn new(config: GroveConfig) -> Self {
        Self {
            config,
            trees: Vec::new(),
            tenders: Vec::new(),
            stats: GroveStats::default(),
        }
    }

    /// Add tree
    pub fn add_tree(&mut self, tree: GroveTree) -> bool {
        if self.trees.len() >= self.config.max_trees {
            return false;
        }
        self.trees.push(tree);
        self.update_stats();
        true
    }

    /// Get tree
    pub fn get_tree(&self, id: &str) -> Option<&GroveTree> {
        self.trees.iter().find(|t| t.id == id)
    }

    /// Get tree mut
    pub fn get_tree_mut(&mut self, id: &str) -> Option<&mut GroveTree> {
        self.trees.iter_mut().find(|t| t.id == id)
    }

    /// Add tender
    pub fn add_tender(&mut self, tender: GroveTender) {
        self.tenders.push(tender);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.trees, self.config.grove_type);
    }

    /// Get stats
    pub fn stats(&self) -> &GroveStats {
        &self.stats
    }

    /// Tree count
    pub fn tree_count(&self) -> usize {
        self.trees.len()
    }
}

/// Grove registry
#[derive(Debug, Clone, Default)]
pub struct GroveRegistry {
    /// Groves by ID
    groves: HashMap<String, SettingsGrove>,
}

impl GroveRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register grove
    pub fn register(&mut self, id: impl Into<String>, grove: SettingsGrove) {
        self.groves.insert(id.into(), grove);
    }

    /// Unregister grove
    pub fn unregister(&mut self, id: &str) -> bool {
        self.groves.remove(id).is_some()
    }

    /// Get grove
    pub fn get(&self, id: &str) -> Option<&SettingsGrove> {
        self.groves.get(id)
    }

    /// Get grove mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsGrove> {
        self.groves.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.groves.len()
    }
}

/// Format grove registry
pub fn format_grove_registry(registry: &GroveRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Grove Registry:\n");
    output.push_str(&format!("  Groves: {}\n", registry.count()));
    output
}

/// Check if query is about grove
pub fn is_grove_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings grove") || lower.contains("grove settings") || lower.contains("tree grove")
}

/// Fun fact about grove
pub fn grove_fun_fact() -> &'static str {
    "Anna's settings grove establishes forestry boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grove_type_display() {
        assert_eq!(format!("{}", GroveType::Oak), "oak");
        assert_eq!(format!("{}", GroveType::Olive), "olive");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", GroveStatus::Planted), "planted");
        assert_eq!(format!("{}", GroveStatus::Productive), "productive");
    }

    #[test]
    fn test_config_new() {
        let c = GroveConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = GroveConfig::new("test")
            .grove_type(GroveType::Citrus)
            .status(GroveStatus::Maturing);
        assert_eq!(c.grove_type, GroveType::Citrus);
        assert_eq!(c.status, GroveStatus::Maturing);
    }

    #[test]
    fn test_tree_new() {
        let t = GroveTree::new("t1", "Title", "Content");
        assert_eq!(t.id, "t1");
    }

    #[test]
    fn test_tree_builder() {
        let t = GroveTree::new("t1", "Title", "Content")
            .row(1);
        assert_eq!(t.row, 1);
    }

    #[test]
    fn test_tree_healthy() {
        let mut t = GroveTree::new("t1", "Title", "Content");
        t.make_diseased();
        assert!(!t.healthy);
        t.make_healthy();
        assert!(t.healthy);
    }

    #[test]
    fn test_tender_new() {
        let t = GroveTender::new("key", "name", "t1");
        assert_eq!(t.tree_id, "t1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = GroveStats::default();
        let tree = GroveTree::new("t1", "Title", "Content");
        s.update(&[tree], GroveType::Oak);
        assert_eq!(s.total_trees, 1);
        assert_eq!(s.healthy, 1);
    }

    #[test]
    fn test_grove_new() {
        let g = SettingsGrove::new(GroveConfig::default());
        assert_eq!(g.tree_count(), 0);
    }

    #[test]
    fn test_grove_add_tree() {
        let mut g = SettingsGrove::new(GroveConfig::default());
        g.add_tree(GroveTree::new("t1", "Title", "Content"));
        assert_eq!(g.tree_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = GroveRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = GroveRegistry::new();
        r.register("g1", SettingsGrove::new(GroveConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_grove_query() {
        assert!(is_grove_query("settings grove"));
        assert!(!is_grove_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = grove_fun_fact();
        assert!(fact.contains("grove"));
    }
}
