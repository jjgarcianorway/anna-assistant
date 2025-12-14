// v0.0.605: Settings Linker (Phase 181)
// Link settings together with references and aliases

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Link type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkType {
    /// Direct reference
    Reference,
    /// Alias (alternative name)
    Alias,
    /// Computed from other settings
    Computed,
    /// Inherited from parent
    Inherited,
    /// Synchronized (two-way)
    Sync,
}

impl std::fmt::Display for LinkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reference => write!(f, "reference"),
            Self::Alias => write!(f, "alias"),
            Self::Computed => write!(f, "computed"),
            Self::Inherited => write!(f, "inherited"),
            Self::Sync => write!(f, "sync"),
        }
    }
}

/// Link status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkStatus {
    /// Active and valid
    Active,
    /// Broken (target missing)
    Broken,
    /// Pending resolution
    Pending,
    /// Disabled
    Disabled,
}

impl std::fmt::Display for LinkStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Broken => write!(f, "broken"),
            Self::Pending => write!(f, "pending"),
            Self::Disabled => write!(f, "disabled"),
        }
    }
}

/// Link definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkDef {
    /// Unique ID
    pub id: String,
    /// Source key
    pub source: String,
    /// Source category
    pub source_category: SettingsCategory,
    /// Target key
    pub target: String,
    /// Target category
    pub target_category: SettingsCategory,
    /// Link type
    pub link_type: LinkType,
    /// Status
    pub status: LinkStatus,
    /// Description
    pub description: String,
}

impl LinkDef {
    /// Create new link
    pub fn new(
        id: impl Into<String>,
        source: impl Into<String>,
        source_cat: SettingsCategory,
        target: impl Into<String>,
        target_cat: SettingsCategory,
        link_type: LinkType,
    ) -> Self {
        Self {
            id: id.into(),
            source: source.into(),
            source_category: source_cat,
            target: target.into(),
            target_category: target_cat,
            link_type,
            status: LinkStatus::Active,
            description: String::new(),
        }
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set status
    pub fn status(mut self, status: LinkStatus) -> Self {
        self.status = status;
        self
    }

    /// Is active
    pub fn is_active(&self) -> bool {
        self.status == LinkStatus::Active
    }

    /// Is broken
    pub fn is_broken(&self) -> bool {
        self.status == LinkStatus::Broken
    }
}

/// Link resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkResolution {
    /// Link ID
    pub link_id: String,
    /// Resolved value
    pub value: Option<String>,
    /// Success
    pub success: bool,
    /// Error message
    pub error: Option<String>,
}

impl LinkResolution {
    /// Create success resolution
    pub fn success(link_id: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            link_id: link_id.into(),
            value: Some(value.into()),
            success: true,
            error: None,
        }
    }

    /// Create failure resolution
    pub fn failure(link_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            link_id: link_id.into(),
            value: None,
            success: false,
            error: Some(error.into()),
        }
    }
}

/// Link registry
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LinkRegistry {
    /// Links by ID
    links: HashMap<String, LinkDef>,
}

impl LinkRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Add link
    pub fn add(&mut self, link: LinkDef) {
        self.links.insert(link.id.clone(), link);
    }

    /// Remove link
    pub fn remove(&mut self, id: &str) -> Option<LinkDef> {
        self.links.remove(id)
    }

    /// Get link
    pub fn get(&self, id: &str) -> Option<&LinkDef> {
        self.links.get(id)
    }

    /// Find links from source
    pub fn from_source(&self, key: &str, category: SettingsCategory) -> Vec<&LinkDef> {
        self.links
            .values()
            .filter(|l| l.source == key && l.source_category == category)
            .collect()
    }

    /// Find links to target
    pub fn to_target(&self, key: &str, category: SettingsCategory) -> Vec<&LinkDef> {
        self.links
            .values()
            .filter(|l| l.target == key && l.target_category == category)
            .collect()
    }

    /// Get broken links
    pub fn broken(&self) -> Vec<&LinkDef> {
        self.links.values().filter(|l| l.is_broken()).collect()
    }

    /// Link count
    pub fn count(&self) -> usize {
        self.links.len()
    }

    /// Active count
    pub fn active_count(&self) -> usize {
        self.links.values().filter(|l| l.is_active()).count()
    }
}

/// Settings linker
#[derive(Debug, Clone, Default)]
pub struct SettingsLinker {
    /// Registry
    registry: LinkRegistry,
    /// Resolution cache
    cache: HashMap<String, LinkResolution>,
}

impl SettingsLinker {
    /// Create new linker
    pub fn new() -> Self {
        Self::default()
    }

    /// Get registry
    pub fn registry(&self) -> &LinkRegistry {
        &self.registry
    }

    /// Get registry mut
    pub fn registry_mut(&mut self) -> &mut LinkRegistry {
        &mut self.registry
    }

    /// Cache resolution
    pub fn cache_resolution(&mut self, resolution: LinkResolution) {
        self.cache.insert(resolution.link_id.clone(), resolution);
    }

    /// Get cached resolution
    pub fn get_cached(&self, link_id: &str) -> Option<&LinkResolution> {
        self.cache.get(link_id)
    }

    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

/// Format linker
pub fn format_linker(linker: &SettingsLinker) -> String {
    let mut output = String::new();
    output.push_str("Settings Linker:\n");
    output.push_str(&format!("  Links: {}\n", linker.registry.count()));
    output.push_str(&format!("  Active: {}\n", linker.registry.active_count()));
    output.push_str(&format!("  Broken: {}\n", linker.registry.broken().len()));
    output.push_str(&format!("  Cached: {}\n", linker.cache_size()));
    output
}

/// Check if query is about linker
pub fn is_linker_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("link settings")
        || lower.contains("settings link")
        || lower.contains("alias")
}

/// Fun fact about linker
pub fn linker_fun_fact() -> &'static str {
    "Anna can link settings together so changes propagate automatically!"
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
    fn test_link_status_display() {
        assert_eq!(format!("{}", LinkStatus::Active), "active");
        assert_eq!(format!("{}", LinkStatus::Broken), "broken");
    }

    #[test]
    fn test_link_def_new() {
        let l = LinkDef::new(
            "l1", "src", SettingsCategory::Personality,
            "tgt", SettingsCategory::Privacy, LinkType::Reference,
        );
        assert!(l.is_active());
    }

    #[test]
    fn test_link_def_broken() {
        let l = LinkDef::new(
            "l1", "s", SettingsCategory::Risk,
            "t", SettingsCategory::Risk, LinkType::Alias,
        ).status(LinkStatus::Broken);
        assert!(l.is_broken());
    }

    #[test]
    fn test_resolution_success() {
        let r = LinkResolution::success("l1", "value");
        assert!(r.success);
    }

    #[test]
    fn test_resolution_failure() {
        let r = LinkResolution::failure("l1", "not found");
        assert!(!r.success);
    }

    #[test]
    fn test_registry_new() {
        let r = LinkRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_add_remove() {
        let mut r = LinkRegistry::new();
        r.add(LinkDef::new(
            "l1", "s", SettingsCategory::Personality,
            "t", SettingsCategory::Personality, LinkType::Reference,
        ));
        assert_eq!(r.count(), 1);
        r.remove("l1");
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_linker_new() {
        let l = SettingsLinker::new();
        assert_eq!(l.cache_size(), 0);
    }

    #[test]
    fn test_linker_cache() {
        let mut l = SettingsLinker::new();
        l.cache_resolution(LinkResolution::success("l1", "v"));
        assert_eq!(l.cache_size(), 1);
    }

    #[test]
    fn test_is_linker_query() {
        assert!(is_linker_query("link settings"));
        assert!(!is_linker_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = linker_fun_fact();
        assert!(fact.contains("link"));
    }
}
