// v0.0.679: Settings Flattener (Phase 255)
// Flatten nested settings structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Flatten mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FlattenMode {
    /// Flatten using dot notation
    #[default]
    DotNotation,
    /// Flatten using underscore
    Underscore,
    /// Flatten using bracket notation
    Bracket,
    /// Flatten using slash notation
    Slash,
}

impl std::fmt::Display for FlattenMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DotNotation => write!(f, "dot_notation"),
            Self::Underscore => write!(f, "underscore"),
            Self::Bracket => write!(f, "bracket"),
            Self::Slash => write!(f, "slash"),
        }
    }
}

/// Depth limit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DepthLimit {
    /// No limit
    #[default]
    Unlimited,
    /// Limited to N levels
    Limited(usize),
}

impl std::fmt::Display for DepthLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unlimited => write!(f, "unlimited"),
            Self::Limited(n) => write!(f, "limited({})", n),
        }
    }
}

/// Flattener config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlattenerConfig {
    /// Flatten mode
    pub mode: FlattenMode,
    /// Depth limit
    pub depth_limit: DepthLimit,
    /// Separator for custom modes
    pub separator: String,
    /// Preserve arrays as lists
    pub preserve_arrays: bool,
}

impl FlattenerConfig {
    /// Create new config
    pub fn new(mode: FlattenMode) -> Self {
        Self {
            mode,
            depth_limit: DepthLimit::Unlimited,
            separator: ".".to_string(),
            preserve_arrays: false,
        }
    }

    /// Set depth limit
    pub fn depth_limit(mut self, limit: DepthLimit) -> Self {
        self.depth_limit = limit;
        self
    }

    /// Set separator
    pub fn separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }

    /// Set preserve arrays
    pub fn preserve_arrays(mut self, preserve: bool) -> Self {
        self.preserve_arrays = preserve;
        self
    }

    /// Get separator for mode
    pub fn get_separator(&self) -> &str {
        match self.mode {
            FlattenMode::DotNotation => ".",
            FlattenMode::Underscore => "_",
            FlattenMode::Bracket => "][",
            FlattenMode::Slash => "/",
        }
    }
}

impl Default for FlattenerConfig {
    fn default() -> Self {
        Self::new(FlattenMode::DotNotation)
    }
}

/// Flatten result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlattenResult {
    /// Flattened settings
    pub settings: HashMap<String, String>,
    /// Original depth
    pub original_depth: usize,
    /// Keys flattened
    pub keys_flattened: usize,
    /// Mode used
    pub mode: FlattenMode,
}

impl FlattenResult {
    /// Create new result
    pub fn new(settings: HashMap<String, String>, original_depth: usize, mode: FlattenMode) -> Self {
        let keys_flattened = settings.len();
        Self {
            settings,
            original_depth,
            keys_flattened,
            mode,
        }
    }

    /// Is flat
    pub fn is_flat(&self) -> bool {
        self.original_depth <= 1
    }

    /// Get value
    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    /// Keys
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.settings.keys()
    }
}

impl Default for FlattenResult {
    fn default() -> Self {
        Self::new(HashMap::new(), 0, FlattenMode::DotNotation)
    }
}

/// Flattener stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FlattenerStats {
    /// Total flatten operations
    pub total_operations: usize,
    /// Total keys flattened
    pub total_keys_flattened: usize,
    /// Max depth seen
    pub max_depth_seen: usize,
    /// By mode
    pub by_mode: HashMap<String, usize>,
}

impl FlattenerStats {
    /// Record flatten
    pub fn record(&mut self, result: &FlattenResult) {
        self.total_operations += 1;
        self.total_keys_flattened += result.keys_flattened;
        self.max_depth_seen = self.max_depth_seen.max(result.original_depth);
        *self.by_mode.entry(result.mode.to_string()).or_insert(0) += 1;
    }

    /// Average keys per operation
    pub fn average_keys(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            self.total_keys_flattened as f64 / self.total_operations as f64
        }
    }
}

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

/// Flattener registry
#[derive(Debug, Clone, Default)]
pub struct FlattenerRegistry {
    /// Flatteners by ID
    flatteners: HashMap<String, SettingsFlattener>,
}

impl FlattenerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register flattener
    pub fn register(&mut self, id: impl Into<String>, flattener: SettingsFlattener) {
        self.flatteners.insert(id.into(), flattener);
    }

    /// Unregister flattener
    pub fn unregister(&mut self, id: &str) -> bool {
        self.flatteners.remove(id).is_some()
    }

    /// Get flattener
    pub fn get(&self, id: &str) -> Option<&SettingsFlattener> {
        self.flatteners.get(id)
    }

    /// Get flattener mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsFlattener> {
        self.flatteners.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.flatteners.len()
    }
}

/// Format flattener registry
pub fn format_flattener_registry(registry: &FlattenerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Flattener Registry:\n");
    output.push_str(&format!("  Flatteners: {}\n", registry.count()));
    output
}

/// Check if query is about flattener
pub fn is_flattener_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("flatten settings") || lower.contains("settings flattener") || lower.contains("unnest settings")
}

/// Fun fact about flattener
pub fn flattener_fun_fact() -> &'static str {
    "Anna's settings flattener converts nested structures into flat key-value pairs!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatten_mode_display() {
        assert_eq!(format!("{}", FlattenMode::DotNotation), "dot_notation");
        assert_eq!(format!("{}", FlattenMode::Underscore), "underscore");
    }

    #[test]
    fn test_depth_limit_display() {
        assert_eq!(format!("{}", DepthLimit::Unlimited), "unlimited");
        assert_eq!(format!("{}", DepthLimit::Limited(5)), "limited(5)");
    }

    #[test]
    fn test_config_new() {
        let c = FlattenerConfig::new(FlattenMode::DotNotation);
        assert_eq!(c.mode, FlattenMode::DotNotation);
    }

    #[test]
    fn test_config_builder() {
        let c = FlattenerConfig::new(FlattenMode::Underscore)
            .depth_limit(DepthLimit::Limited(3))
            .separator(":");
        assert_eq!(c.depth_limit, DepthLimit::Limited(3));
        assert_eq!(c.separator, ":");
    }

    #[test]
    fn test_config_get_separator() {
        assert_eq!(FlattenerConfig::new(FlattenMode::DotNotation).get_separator(), ".");
        assert_eq!(FlattenerConfig::new(FlattenMode::Underscore).get_separator(), "_");
        assert_eq!(FlattenerConfig::new(FlattenMode::Slash).get_separator(), "/");
    }

    #[test]
    fn test_result_new() {
        let r = FlattenResult::new(HashMap::new(), 2, FlattenMode::DotNotation);
        assert_eq!(r.original_depth, 2);
        assert!(!r.is_flat());
    }

    #[test]
    fn test_result_is_flat() {
        let r = FlattenResult::new(HashMap::new(), 1, FlattenMode::DotNotation);
        assert!(r.is_flat());
    }

    #[test]
    fn test_stats_record() {
        let mut s = FlattenerStats::default();
        let mut settings = HashMap::new();
        settings.insert("a.b".to_string(), "v".to_string());
        let r = FlattenResult::new(settings, 2, FlattenMode::DotNotation);
        s.record(&r);
        assert_eq!(s.total_operations, 1);
        assert_eq!(s.max_depth_seen, 2);
    }

    #[test]
    fn test_flattener_new() {
        let f = SettingsFlattener::new(FlattenerConfig::default());
        assert_eq!(f.stats().total_operations, 0);
    }

    #[test]
    fn test_flattener_flatten_dot() {
        let mut f = SettingsFlattener::new(FlattenerConfig::new(FlattenMode::DotNotation));
        let mut settings = HashMap::new();
        settings.insert("app.db.host".to_string(), "localhost".to_string());
        settings.insert("app.db.port".to_string(), "5432".to_string());

        let result = f.flatten(&settings);
        assert_eq!(result.keys_flattened, 2);
        assert!(result.get("app.db.host").is_some());
    }

    #[test]
    fn test_flattener_flatten_underscore() {
        let mut f = SettingsFlattener::new(FlattenerConfig::new(FlattenMode::Underscore));
        let mut settings = HashMap::new();
        settings.insert("app.db.host".to_string(), "localhost".to_string());

        let result = f.flatten(&settings);
        assert!(result.get("app_db_host").is_some());
    }

    #[test]
    fn test_flattener_flatten_slash() {
        let mut f = SettingsFlattener::new(FlattenerConfig::new(FlattenMode::Slash));
        let mut settings = HashMap::new();
        settings.insert("app.db.host".to_string(), "localhost".to_string());

        let result = f.flatten(&settings);
        assert!(result.get("app/db/host").is_some());
    }

    #[test]
    fn test_flattener_with_prefix() {
        let mut f = SettingsFlattener::new(FlattenerConfig::default());
        let mut settings = HashMap::new();
        settings.insert("host".to_string(), "localhost".to_string());

        let result = f.flatten_with_prefix(&settings, "db");
        assert!(result.get("db.host").is_some());
    }

    #[test]
    fn test_registry_new() {
        let r = FlattenerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = FlattenerRegistry::new();
        r.register("f1", SettingsFlattener::new(FlattenerConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_flattener_query() {
        assert!(is_flattener_query("flatten settings"));
        assert!(!is_flattener_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = flattener_fun_fact();
        assert!(fact.contains("flattener"));
    }
}
