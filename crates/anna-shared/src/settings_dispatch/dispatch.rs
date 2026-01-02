// v0.0.714: Settings Dispatch Core (Phase 290)
// Main dispatch implementation

use super::config::DispatchConfig;
use super::item::{DispatchItem, DispatchMetadata};
use super::stats::DispatchStats;

/// Settings dispatch
#[derive(Debug, Clone, Default)]
pub struct SettingsDispatch {
    /// Config
    config: DispatchConfig,
    /// Items
    items: Vec<DispatchItem>,
    /// Metadata
    metadata: Vec<DispatchMetadata>,
    /// Stats
    stats: DispatchStats,
}

impl SettingsDispatch {
    /// Create new dispatch system
    pub fn new(config: DispatchConfig) -> Self {
        Self {
            config,
            items: Vec::new(),
            metadata: Vec::new(),
            stats: DispatchStats::default(),
        }
    }

    /// Add item
    pub fn add_item(&mut self, item: DispatchItem) -> bool {
        if self.items.len() >= self.config.max_dispatches {
            return false;
        }
        self.items.push(item);
        self.update_stats();
        true
    }

    /// Get item
    pub fn get_item(&self, id: &str) -> Option<&DispatchItem> {
        self.items.iter().find(|i| i.id == id)
    }

    /// Get item mut
    pub fn get_item_mut(&mut self, id: &str) -> Option<&mut DispatchItem> {
        self.items.iter_mut().find(|i| i.id == id)
    }

    /// Add metadata
    pub fn add_metadata(&mut self, meta: DispatchMetadata) {
        self.metadata.push(meta);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.items, self.config.dispatch_type);
    }

    /// Get stats
    pub fn stats(&self) -> &DispatchStats {
        &self.stats
    }

    /// Item count
    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatch_new() {
        let d = SettingsDispatch::new(DispatchConfig::default());
        assert_eq!(d.item_count(), 0);
    }

    #[test]
    fn test_dispatch_add_item() {
        let mut d = SettingsDispatch::new(DispatchConfig::default());
        d.add_item(DispatchItem::new("i1", "target", "payload"));
        assert_eq!(d.item_count(), 1);
    }
}
