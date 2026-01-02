// v0.0.663: Settings Graph - Settings Link
// Link structures and results for settings graph

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::link_types::{LinkDirection, LinkType};

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
