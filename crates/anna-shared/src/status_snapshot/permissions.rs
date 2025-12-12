//! Permission and access types (v0.0.211).
//! v0.0.463: Enhanced with folder permissions per VISION.md Phase 29.

use serde::{Deserialize, Serialize};

/// Folder permission info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderPermission {
    /// Path to folder
    pub path: String,
    /// Whether folder exists
    pub exists: bool,
    /// Whether user can read
    pub readable: bool,
    /// Whether user can write
    pub writable: bool,
    /// Owner (user:group)
    pub owner: Option<String>,
    /// Unix permissions (e.g., "755")
    pub mode: Option<String>,
}

impl FolderPermission {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            exists: false,
            readable: false,
            writable: false,
            owner: None,
            mode: None,
        }
    }

    pub fn with_exists(mut self, exists: bool) -> Self {
        self.exists = exists;
        self
    }

    pub fn with_readable(mut self, readable: bool) -> Self {
        self.readable = readable;
        self
    }

    pub fn with_writable(mut self, writable: bool) -> Self {
        self.writable = writable;
        self
    }

    pub fn with_owner(mut self, owner: Option<String>) -> Self {
        self.owner = owner;
        self
    }

    pub fn with_mode(mut self, mode: Option<String>) -> Self {
        self.mode = mode;
        self
    }
}

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
    /// Key folder permissions (v0.0.463)
    #[serde(default)]
    pub folders: Vec<FolderPermission>,
}

impl PermissionsInfo {
    pub fn current() -> Self {
        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
        Self {
            user,
            groups: Vec::new(), // Will be populated by caller
            can_talk_to_daemon: false,
            data_dir_ok: false,
            folders: Vec::new(),
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

    pub fn with_folders(mut self, folders: Vec<FolderPermission>) -> Self {
        self.folders = folders;
        self
    }

    /// Check key Anna folders and populate permissions
    pub fn check_anna_folders(&mut self) {
        let key_paths = [
            "/run/anna",
            "/var/lib/anna",
            &format!("{}/.anna", std::env::var("HOME").unwrap_or_default()),
        ];

        for path in key_paths {
            if path.is_empty() {
                continue;
            }
            let mut folder = FolderPermission::new(path);
            let path_obj = std::path::Path::new(path);

            folder.exists = path_obj.exists();
            if folder.exists {
                folder.readable = path_obj.read_dir().is_ok();
                // Check writable by trying to access metadata
                if let Ok(meta) = path_obj.metadata() {
                    folder.writable = !meta.permissions().readonly();
                }
            }
            self.folders.push(folder);
        }
    }
}
