//! Configuration summary types (v0.0.211).

use serde::{Deserialize, Serialize};

/// Configuration summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigInfo {
    /// Debug mode enabled
    pub debug_mode: bool,
    /// Clean REPL mode (non-debug)
    pub repl_clean_mode: bool,
    /// Autonomy level (0-100)
    pub autonomy_level: u8,
}
