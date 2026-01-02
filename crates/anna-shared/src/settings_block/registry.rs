// v0.0.755: Settings Block (Phase 331)
// Block registry and utility functions

use std::collections::HashMap;
use super::block::SettingsBlock;

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
