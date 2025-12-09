//! Version information types (v0.0.211).

use serde::{Deserialize, Serialize};

/// Version information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VersionInfo {
    /// annactl version
    pub annactl: String,
    /// annad version
    pub annad: String,
    /// anna-shared version
    pub anna_shared: String,
    /// Current git tag (local)
    pub git_tag_current: Option<String>,
    /// Remote git tag (latest from GitHub)
    pub git_tag_remote: Option<String>,
}

impl VersionInfo {
    pub fn new(version: &str) -> Self {
        Self {
            annactl: version.to_string(),
            annad: version.to_string(),
            anna_shared: version.to_string(),
            git_tag_current: Some(format!("v{}", version)),
            git_tag_remote: None,
        }
    }

    pub fn with_remote(mut self, remote: Option<String>) -> Self {
        self.git_tag_remote = remote;
        self
    }
}
