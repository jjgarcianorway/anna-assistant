// v0.0.663: Settings Graph - Linker Implementation
// Main linker and registry for settings graph

use std::collections::HashMap;

use super::linker_config::LinkerConfig;
use super::link_types::LinkType;
use super::settings_link::{LinkResult, LinkerStats, SettingsLink};

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
