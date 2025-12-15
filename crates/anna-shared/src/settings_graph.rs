// v0.0.663: Settings Graph (Phase 239)
// Graph for modeling settings relationships and dependencies

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Link type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum LinkType {
    /// Reference link
    #[default]
    Reference,
    /// Alias link
    Alias,
    /// Dependency link
    Dependency,
    /// Override link
    Override,
    /// Computed link
    Computed,
}

impl std::fmt::Display for LinkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reference => write!(f, "reference"),
            Self::Alias => write!(f, "alias"),
            Self::Dependency => write!(f, "dependency"),
            Self::Override => write!(f, "override"),
            Self::Computed => write!(f, "computed"),
        }
    }
}

/// Link direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LinkDirection {
    /// Unidirectional (source -> target)
    #[default]
    Unidirectional,
    /// Bidirectional (source <-> target)
    Bidirectional,
    /// Reverse (target -> source)
    Reverse,
}

impl std::fmt::Display for LinkDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unidirectional => write!(f, "unidirectional"),
            Self::Bidirectional => write!(f, "bidirectional"),
            Self::Reverse => write!(f, "reverse"),
        }
    }
}

/// Linker config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkerConfig {
    /// Default link type
    pub default_link_type: LinkType,
    /// Default direction
    pub default_direction: LinkDirection,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Allow circular links
    pub allow_circular: bool,
    /// Auto-resolve
    pub auto_resolve: bool,
}

impl LinkerConfig {
    /// Create new config
    pub fn new(link_type: LinkType) -> Self {
        Self {
            default_link_type: link_type,
            default_direction: LinkDirection::Unidirectional,
            category: None,
            allow_circular: false,
            auto_resolve: true,
        }
    }

    /// Set direction
    pub fn direction(mut self, direction: LinkDirection) -> Self {
        self.default_direction = direction;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set allow circular
    pub fn allow_circular(mut self, allow: bool) -> Self {
        self.allow_circular = allow;
        self
    }

    /// Set auto resolve
    pub fn auto_resolve(mut self, resolve: bool) -> Self {
        self.auto_resolve = resolve;
        self
    }
}

impl Default for LinkerConfig {
    fn default() -> Self {
        Self::new(LinkType::Reference)
    }
}

/// Settings link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsLink {
    /// Link ID
    pub id: String,
    /// Source key
    pub source: String,
    /// Target key
    pub target: String,
    /// Link type
    pub link_type: LinkType,
    /// Direction
    pub direction: LinkDirection,
    /// Description
    pub description: Option<String>,
}

impl SettingsLink {
    /// Create new link
    pub fn new(id: impl Into<String>, source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            source: source.into(),
            target: target.into(),
            link_type: LinkType::Reference,
            direction: LinkDirection::Unidirectional,
            description: None,
        }
    }

    /// With link type
    pub fn with_type(mut self, link_type: LinkType) -> Self {
        self.link_type = link_type;
        self
    }

    /// With direction
    pub fn with_direction(mut self, direction: LinkDirection) -> Self {
        self.direction = direction;
        self
    }

    /// With description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// Link result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkResult {
    /// Links created
    pub links_created: Vec<String>,
    /// Links updated
    pub links_updated: Vec<String>,
    /// Links failed
    pub links_failed: Vec<String>,
    /// Total links
    pub total_links: usize,
}

impl LinkResult {
    /// Create new result
    pub fn new() -> Self {
        Self {
            links_created: Vec::new(),
            links_updated: Vec::new(),
            links_failed: Vec::new(),
            total_links: 0,
        }
    }

    /// Add created
    pub fn add_created(&mut self, id: String) {
        self.links_created.push(id);
        self.total_links += 1;
    }

    /// Add updated
    pub fn add_updated(&mut self, id: String) {
        self.links_updated.push(id);
    }

    /// Add failed
    pub fn add_failed(&mut self, id: String) {
        self.links_failed.push(id);
    }

    /// Has failures
    pub fn has_failures(&self) -> bool {
        !self.links_failed.is_empty()
    }

    /// Success
    pub fn success(&self) -> bool {
        self.links_failed.is_empty()
    }
}

impl Default for LinkResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Linker stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LinkerStats {
    /// Total links created
    pub total_links: usize,
    /// Total resolutions
    pub total_resolutions: usize,
    /// By link type
    pub by_type: HashMap<String, usize>,
}

impl LinkerStats {
    /// Record link
    pub fn record(&mut self, link_type: LinkType) {
        self.total_links += 1;
        *self.by_type.entry(link_type.to_string()).or_insert(0) += 1;
    }

    /// Record resolution
    pub fn record_resolution(&mut self) {
        self.total_resolutions += 1;
    }

    /// Resolutions per link
    pub fn resolutions_per_link(&self) -> f64 {
        if self.total_links == 0 {
            0.0
        } else {
            self.total_resolutions as f64 / self.total_links as f64
        }
    }
}

/// Settings linker
#[derive(Debug, Clone, Default)]
pub struct SettingsLinker {
    /// Config
    config: LinkerConfig,
    /// Links
    links: HashMap<String, SettingsLink>,
    /// Stats
    stats: LinkerStats,
    /// Next link ID
    next_id: usize,
}

impl SettingsLinker {
    /// Create new linker
    pub fn new(config: LinkerConfig) -> Self {
        Self {
            config,
            links: HashMap::new(),
            stats: LinkerStats::default(),
            next_id: 1,
        }
    }

    /// Create link
    pub fn link(&mut self, source: &str, target: &str) -> LinkResult {
        let mut result = LinkResult::new();
        let id = format!("link_{}", self.next_id);
        self.next_id += 1;

        // Check for circular if not allowed
        if !self.config.allow_circular && self.would_be_circular(source, target) {
            result.add_failed(id);
            return result;
        }

        let link = SettingsLink::new(&id, source, target)
            .with_type(self.config.default_link_type)
            .with_direction(self.config.default_direction);

        self.links.insert(id.clone(), link);
        self.stats.record(self.config.default_link_type);
        result.add_created(id);

        result
    }

    /// Create link with type
    pub fn link_with_type(&mut self, source: &str, target: &str, link_type: LinkType) -> LinkResult {
        let mut result = LinkResult::new();
        let id = format!("link_{}", self.next_id);
        self.next_id += 1;

        let link = SettingsLink::new(&id, source, target)
            .with_type(link_type)
            .with_direction(self.config.default_direction);

        self.links.insert(id.clone(), link);
        self.stats.record(link_type);
        result.add_created(id);

        result
    }

    /// Check if link would be circular
    fn would_be_circular(&self, source: &str, target: &str) -> bool {
        let mut visited = std::collections::HashSet::new();
        let mut queue = vec![target.to_string()];

        while let Some(current) = queue.pop() {
            if current == source {
                return true;
            }
            if visited.insert(current.clone()) {
                for link in self.links.values() {
                    if link.source == current {
                        queue.push(link.target.clone());
                    }
                }
            }
        }
        false
    }

    /// Resolve link
    pub fn resolve(&self, key: &str, settings: &HashMap<String, String>) -> Option<String> {
        for link in self.links.values() {
            if link.source == key {
                if let Some(value) = settings.get(&link.target) {
                    return Some(value.clone());
                }
            }
        }
        settings.get(key).cloned()
    }

    /// Get link
    pub fn get_link(&self, id: &str) -> Option<&SettingsLink> {
        self.links.get(id)
    }

    /// Remove link
    pub fn remove_link(&mut self, id: &str) -> bool {
        self.links.remove(id).is_some()
    }

    /// Get all links for key
    pub fn links_for(&self, key: &str) -> Vec<&SettingsLink> {
        self.links
            .values()
            .filter(|l| l.source == key || l.target == key)
            .collect()
    }

    /// Link count
    pub fn link_count(&self) -> usize {
        self.links.len()
    }

    /// Get stats
    pub fn stats(&self) -> &LinkerStats {
        &self.stats
    }

    /// Clear links
    pub fn clear(&mut self) {
        self.links.clear();
    }
}

/// Settings linker registry
#[derive(Debug, Clone, Default)]
pub struct SettingsLinkerRegistry {
    /// Linkers by ID
    linkers: HashMap<String, SettingsLinker>,
}

impl SettingsLinkerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register linker
    pub fn register(&mut self, id: impl Into<String>, linker: SettingsLinker) {
        self.linkers.insert(id.into(), linker);
    }

    /// Unregister linker
    pub fn unregister(&mut self, id: &str) -> bool {
        self.linkers.remove(id).is_some()
    }

    /// Get linker
    pub fn get(&self, id: &str) -> Option<&SettingsLinker> {
        self.linkers.get(id)
    }

    /// Get linker mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsLinker> {
        self.linkers.get_mut(id)
    }

    /// Linker count
    pub fn count(&self) -> usize {
        self.linkers.len()
    }
}

/// Format graph registry
pub fn format_graph_registry(registry: &SettingsLinkerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Graph Registry:\n");
    output.push_str(&format!("  Graphs: {}\n", registry.count()));
    output
}

/// Check if query is about graph
pub fn is_graph_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("graph") || lower.contains("settings graph") || lower.contains("dependency graph")
}

/// Fun fact about graph
pub fn graph_fun_fact() -> &'static str {
    "Anna's settings graphs model complex dependency relationships!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_type_display() {
        assert_eq!(format!("{}", LinkType::Reference), "reference");
        assert_eq!(format!("{}", LinkType::Alias), "alias");
    }

    #[test]
    fn test_link_direction_display() {
        assert_eq!(format!("{}", LinkDirection::Unidirectional), "unidirectional");
        assert_eq!(format!("{}", LinkDirection::Bidirectional), "bidirectional");
    }

    #[test]
    fn test_config_new() {
        let c = LinkerConfig::new(LinkType::Reference);
        assert!(!c.allow_circular);
    }

    #[test]
    fn test_config_builder() {
        let c = LinkerConfig::new(LinkType::Dependency)
            .direction(LinkDirection::Bidirectional)
            .allow_circular(true);
        assert_eq!(c.default_direction, LinkDirection::Bidirectional);
        assert!(c.allow_circular);
    }

    #[test]
    fn test_link_new() {
        let l = SettingsLink::new("link_1", "source", "target");
        assert_eq!(l.source, "source");
        assert_eq!(l.target, "target");
    }

    #[test]
    fn test_link_with_type() {
        let l = SettingsLink::new("link_1", "s", "t").with_type(LinkType::Alias);
        assert_eq!(l.link_type, LinkType::Alias);
    }

    #[test]
    fn test_result_new() {
        let r = LinkResult::new();
        assert_eq!(r.total_links, 0);
    }

    #[test]
    fn test_result_add_created() {
        let mut r = LinkResult::new();
        r.add_created("link_1".to_string());
        assert_eq!(r.total_links, 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = LinkerStats::default();
        s.record(LinkType::Reference);
        assert_eq!(s.total_links, 1);
    }

    #[test]
    fn test_linker_new() {
        let l = SettingsLinker::new(LinkerConfig::new(LinkType::Reference));
        assert_eq!(l.link_count(), 0);
    }

    #[test]
    fn test_linker_link() {
        let mut l = SettingsLinker::new(LinkerConfig::new(LinkType::Reference));
        let r = l.link("source", "target");
        assert!(r.success());
        assert_eq!(l.link_count(), 1);
    }

    #[test]
    fn test_linker_resolve() {
        let mut l = SettingsLinker::new(LinkerConfig::new(LinkType::Reference));
        l.link("alias", "actual");

        let mut settings = HashMap::new();
        settings.insert("actual".to_string(), "value".to_string());

        let resolved = l.resolve("alias", &settings);
        assert_eq!(resolved, Some("value".to_string()));
    }

    #[test]
    fn test_linker_circular_prevention() {
        let mut l = SettingsLinker::new(LinkerConfig::new(LinkType::Reference));
        l.link("a", "b");
        l.link("b", "c");
        let r = l.link("c", "a"); // Would create circular
        assert!(r.has_failures());
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsLinkerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsLinkerRegistry::new();
        r.register("l1", SettingsLinker::new(LinkerConfig::new(LinkType::Reference)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_graph_query() {
        assert!(is_graph_query("settings graph"));
        assert!(!is_graph_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = graph_fun_fact();
        assert!(fact.contains("graph"));
    }
}
