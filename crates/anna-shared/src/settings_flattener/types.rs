// v0.0.679: Settings Flattener Types
// Type definitions for settings flattener

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
