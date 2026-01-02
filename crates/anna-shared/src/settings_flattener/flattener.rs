// v0.0.679: Settings Flattener Implementation
// Core flattening logic

use std::collections::HashMap;
use crate::settings_flattener::types::{FlattenerConfig, FlattenerStats, FlattenResult, FlattenMode};

/// Settings flattener
#[derive(Debug, Clone, Default)]
pub struct SettingsFlattener {
    /// Config
    config: FlattenerConfig,
    /// Stats
    stats: FlattenerStats,
}

impl SettingsFlattener {
    /// Create new flattener
    pub fn new(config: FlattenerConfig) -> Self {
        Self {
            config,
            stats: FlattenerStats::default(),
        }
    }

    /// Flatten nested map (simulated with dot-separated keys)
    pub fn flatten(&mut self, settings: &HashMap<String, String>) -> FlattenResult {
        let separator = self.config.get_separator();
        let mut flattened = HashMap::new();
        let mut max_depth = 0;

        for (key, value) in settings {
            // Count depth based on separator occurrences
            let depth = key.matches('.').count() + 1;
            max_depth = max_depth.max(depth);

            // Convert key based on mode
            let new_key = match self.config.mode {
                FlattenMode::DotNotation => key.clone(),
                FlattenMode::Underscore => key.replace('.', "_"),
                FlattenMode::Bracket => {
                    let parts: Vec<&str> = key.split('.').collect();
                    if parts.len() > 1 {
                        format!("[{}]", parts.join("]["))
                    } else {
                        format!("[{}]", key)
                    }
                }
                FlattenMode::Slash => key.replace('.', "/"),
            };

            flattened.insert(new_key, value.clone());
        }

        let result = FlattenResult::new(flattened, max_depth, self.config.mode);
        self.stats.record(&result);
        result
    }

    /// Flatten with prefix
    pub fn flatten_with_prefix(&mut self, settings: &HashMap<String, String>, prefix: &str) -> FlattenResult {
        let separator = self.config.get_separator();
        let mut flattened = HashMap::new();
        let mut max_depth = 0;

        for (key, value) in settings {
            let depth = key.matches('.').count() + 2; // +1 for prefix, +1 base
            max_depth = max_depth.max(depth);

            let new_key = format!("{}{}{}", prefix, separator, key);
            flattened.insert(new_key, value.clone());
        }

        let result = FlattenResult::new(flattened, max_depth, self.config.mode);
        self.stats.record(&result);
        result
    }

    /// Unflatten (convert flat keys back to nested structure representation)
    pub fn unflatten(&mut self, settings: &HashMap<String, String>) -> FlattenResult {
        // For string-based settings, we just return as-is
        // but track the operation
        let max_depth = settings.keys()
            .map(|k| k.matches(self.config.get_separator()).count() + 1)
            .max()
            .unwrap_or(0);

        let result = FlattenResult::new(settings.clone(), max_depth, self.config.mode);
        self.stats.record(&result);
        result
    }

    /// Get stats
    pub fn stats(&self) -> &FlattenerStats {
        &self.stats
    }
}
