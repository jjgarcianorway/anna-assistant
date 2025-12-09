//! Permission and access types (v0.0.211).

use serde::{Deserialize, Serialize};

/// Permission and access information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionsInfo {
    /// Current user
    pub user: String,
    /// User groups
    pub groups: Vec<String>,
    /// Can connect to daemon socket
    pub can_talk_to_daemon: bool,
    /// Data directory is accessible
    pub data_dir_ok: bool,
}

impl PermissionsInfo {
    pub fn current() -> Self {
        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
        Self {
            user,
            groups: Vec::new(), // Will be populated by caller
            can_talk_to_daemon: false,
            data_dir_ok: false,
        }
    }

    pub fn with_groups(mut self, groups: Vec<String>) -> Self {
        self.groups = groups;
        self
    }

    pub fn with_daemon_access(mut self, can_talk: bool) -> Self {
        self.can_talk_to_daemon = can_talk;
        self
    }

    pub fn with_data_dir_ok(mut self, ok: bool) -> Self {
        self.data_dir_ok = ok;
        self
    }
}
