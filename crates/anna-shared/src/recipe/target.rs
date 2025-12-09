//! Recipe target and rollback types (v0.0.177).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Target for a recipe action (v0.0.27)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipeTarget {
    pub app_id: String,
    pub config_path_template: String,
}

impl RecipeTarget {
    pub fn new(app_id: impl Into<String>, config_path: impl Into<String>) -> Self {
        Self {
            app_id: app_id.into(),
            config_path_template: config_path.into(),
        }
    }

    /// Expand config path template with environment variables
    pub fn expand_path(&self) -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(
            self.config_path_template
                .replace("$HOME", &home)
                .replace("~", &home),
        )
    }
}

/// Rollback information for reversible changes (v0.0.27)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollbackInfo {
    pub backup_path: PathBuf,
    pub description: String,
    #[serde(default)]
    pub tested: bool,
}

impl RollbackInfo {
    pub fn new(backup_path: PathBuf, description: impl Into<String>) -> Self {
        Self {
            backup_path,
            description: description.into(),
            tested: false,
        }
    }
}
