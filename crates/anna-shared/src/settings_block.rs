// v0.0.755: Settings Block (Phase 331)
// City block for settings subdivision

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Block type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BlockType {
    /// Residential block
    #[default]
    Residential,
    /// Commercial block
    Commercial,
    /// Industrial block
    Industrial,
    /// Civic block
    Civic,
}

impl std::fmt::Display for BlockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Residential => write!(f, "residential"),
            Self::Commercial => write!(f, "commercial"),
            Self::Industrial => write!(f, "industrial"),
            Self::Civic => write!(f, "civic"),
        }
    }
}

/// Block status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BlockStatus {
    /// Surveyed status
    #[default]
    Surveyed,
    /// Developed status
    Developed,
    /// Subdivided status
    Subdivided,
    /// Consolidated status
    Consolidated,
}

impl std::fmt::Display for BlockStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Surveyed => write!(f, "surveyed"),
            Self::Developed => write!(f, "developed"),
            Self::Subdivided => write!(f, "subdivided"),
            Self::Consolidated => write!(f, "consolidated"),
        }
    }
}

/// Block config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockConfig {
    /// Name
    pub name: String,
    /// Block type
    pub block_type: BlockType,
    /// Status
    pub status: BlockStatus,
    /// Max plats
    pub max_plats: usize,
}

impl BlockConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            block_type: BlockType::Residential,
            status: BlockStatus::Surveyed,
            max_plats: 100,
        }
    }

    /// Set type
    pub fn block_type(mut self, bt: BlockType) -> Self {
        self.block_type = bt;
        self
    }

    /// Set status
    pub fn status(mut self, s: BlockStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max plats
    pub fn max_plats(mut self, max: usize) -> Self {
        self.max_plats = max;
        self
    }
}

impl Default for BlockConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Block plat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockPlat {
    /// Plat ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Lot number
    pub lot: u32,
    /// Recorded
    pub recorded: bool,
}

impl BlockPlat {
    /// Create new plat
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            lot: 0,
            recorded: true,
        }
    }

    /// Set lot
    pub fn lot(mut self, l: u32) -> Self {
        self.lot = l;
        self
    }

    /// Make recorded
    pub fn make_recorded(&mut self) {
        self.recorded = true;
    }

    /// Make pending
    pub fn make_pending(&mut self) {
        self.recorded = false;
    }
}

/// Block surveyor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockSurveyor {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Plat ID
    pub plat_id: String,
}

impl BlockSurveyor {
    /// Create new surveyor
    pub fn new(key: impl Into<String>, name: impl Into<String>, plat_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            plat_id: plat_id.into(),
        }
    }
}

/// Block stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockStats {
    /// Total plats
    pub total_plats: usize,
    /// Recorded plats
    pub recorded: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl BlockStats {
    /// Update from plats
    pub fn update(&mut self, plats: &[BlockPlat], block_type: BlockType) {
        self.total_plats = plats.len();
        self.recorded = plats.iter().filter(|p| p.recorded).count();
        *self.by_type.entry(block_type.to_string()).or_insert(0) += 1;
    }

    /// Recorded rate
    pub fn recorded_rate(&self) -> f64 {
        if self.total_plats == 0 { 0.0 } else { self.recorded as f64 / self.total_plats as f64 * 100.0 }
    }
}

/// Settings block
#[derive(Debug, Clone, Default)]
pub struct SettingsBlock {
    /// Config
    config: BlockConfig,
    /// Plats
    plats: Vec<BlockPlat>,
    /// Surveyors
    surveyors: Vec<BlockSurveyor>,
    /// Stats
    stats: BlockStats,
}

impl SettingsBlock {
    /// Create new block system
    pub fn new(config: BlockConfig) -> Self {
        Self {
            config,
            plats: Vec::new(),
            surveyors: Vec::new(),
            stats: BlockStats::default(),
        }
    }

    /// Add plat
    pub fn add_plat(&mut self, plat: BlockPlat) -> bool {
        if self.plats.len() >= self.config.max_plats {
            return false;
        }
        self.plats.push(plat);
        self.update_stats();
        true
    }

    /// Get plat
    pub fn get_plat(&self, id: &str) -> Option<&BlockPlat> {
        self.plats.iter().find(|p| p.id == id)
    }

    /// Get plat mut
    pub fn get_plat_mut(&mut self, id: &str) -> Option<&mut BlockPlat> {
        self.plats.iter_mut().find(|p| p.id == id)
    }

    /// Add surveyor
    pub fn add_surveyor(&mut self, surveyor: BlockSurveyor) {
        self.surveyors.push(surveyor);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.plats, self.config.block_type);
    }

    /// Get stats
    pub fn stats(&self) -> &BlockStats {
        &self.stats
    }

    /// Plat count
    pub fn plat_count(&self) -> usize {
        self.plats.len()
    }
}

/// Block registry
#[derive(Debug, Clone, Default)]
pub struct BlockRegistry {
    /// Blocks by ID
    blocks: HashMap<String, SettingsBlock>,
}

impl BlockRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register block
    pub fn register(&mut self, id: impl Into<String>, block: SettingsBlock) {
        self.blocks.insert(id.into(), block);
    }

    /// Unregister block
    pub fn unregister(&mut self, id: &str) -> bool {
        self.blocks.remove(id).is_some()
    }

    /// Get block
    pub fn get(&self, id: &str) -> Option<&SettingsBlock> {
        self.blocks.get(id)
    }

    /// Get block mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsBlock> {
        self.blocks.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.blocks.len()
    }
}

/// Format block registry
pub fn format_block_registry(registry: &BlockRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Block Registry:\n");
    output.push_str(&format!("  Blocks: {}\n", registry.count()));
    output
}

/// Check if query is about block
pub fn is_block_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings block") || lower.contains("block settings") || lower.contains("city block")
}

/// Fun fact about block
pub fn block_fun_fact() -> &'static str {
    "Anna's settings block establishes subdivision boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_type_display() {
        assert_eq!(format!("{}", BlockType::Residential), "residential");
        assert_eq!(format!("{}", BlockType::Commercial), "commercial");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", BlockStatus::Surveyed), "surveyed");
        assert_eq!(format!("{}", BlockStatus::Developed), "developed");
    }

    #[test]
    fn test_config_new() {
        let c = BlockConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = BlockConfig::new("test")
            .block_type(BlockType::Commercial)
            .status(BlockStatus::Subdivided);
        assert_eq!(c.block_type, BlockType::Commercial);
        assert_eq!(c.status, BlockStatus::Subdivided);
    }

    #[test]
    fn test_plat_new() {
        let p = BlockPlat::new("p1", "Title", "Content");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_plat_builder() {
        let p = BlockPlat::new("p1", "Title", "Content")
            .lot(1);
        assert_eq!(p.lot, 1);
    }

    #[test]
    fn test_plat_recorded() {
        let mut p = BlockPlat::new("p1", "Title", "Content");
        p.make_pending();
        assert!(!p.recorded);
        p.make_recorded();
        assert!(p.recorded);
    }

    #[test]
    fn test_surveyor_new() {
        let s = BlockSurveyor::new("key", "name", "p1");
        assert_eq!(s.plat_id, "p1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = BlockStats::default();
        let plat = BlockPlat::new("p1", "Title", "Content");
        s.update(&[plat], BlockType::Residential);
        assert_eq!(s.total_plats, 1);
        assert_eq!(s.recorded, 1);
    }

    #[test]
    fn test_block_new() {
        let b = SettingsBlock::new(BlockConfig::default());
        assert_eq!(b.plat_count(), 0);
    }

    #[test]
    fn test_block_add_plat() {
        let mut b = SettingsBlock::new(BlockConfig::default());
        b.add_plat(BlockPlat::new("p1", "Title", "Content"));
        assert_eq!(b.plat_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = BlockRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = BlockRegistry::new();
        r.register("b1", SettingsBlock::new(BlockConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_block_query() {
        assert!(is_block_query("settings block"));
        assert!(!is_block_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = block_fun_fact();
        assert!(fact.contains("block"));
    }
}
