// v0.0.765: Settings Grove (Phase 341)
// Main settings grove structure

use super::config::GroveConfig;
use super::tree::GroveTree;
use super::tender::GroveTender;
use super::stats::GroveStats;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
