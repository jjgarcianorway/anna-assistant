//! Socket health and build metadata.

use serde::{Deserialize, Serialize};

use crate::version::BuildInfo;

/// v0.3.21: Build metadata from compile time
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildMetadata {
    /// Version string
    pub version: String,
    /// Git commit SHA (short)
    pub git_sha: String,
    /// Whether there were uncommitted changes at build
    pub git_dirty: bool,
    /// Build timestamp (RFC3339)
    pub build_time: String,
    /// Version file integrity check result
    pub integrity_ok: bool,
    /// Integrity error message if any
    pub integrity_error: Option<String>,
}

impl BuildMetadata {
    /// Create from BuildInfo
    pub fn from_build_info() -> Self {
        let info = BuildInfo::get();
        let integrity = crate::version::verify_version_integrity();
        Self {
            version: info.version.to_string(),
            git_sha: info.git_sha.to_string(),
            git_dirty: info.git_dirty,
            build_time: info.build_time.to_string(),
            integrity_ok: integrity.is_ok(),
            integrity_error: integrity.err(),
        }
    }

    /// Format as display string
    pub fn display(&self) -> String {
        let dirty = if self.git_dirty { "*" } else { "" };
        format!("{}+{}{}", self.version, self.git_sha, dirty)
    }
}

/// v0.3.21: Socket health status
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SocketHealth {
    /// Socket path
    pub path: String,
    /// Whether socket file exists
    pub exists: bool,
    /// Socket status
    pub status: SocketStatus,
    /// Last successful ping timestamp
    pub last_ping: Option<String>,
    /// Last error message if any
    pub last_error: Option<String>,
}

/// v0.3.21: Socket status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SocketStatus {
    #[default]
    Unknown,
    Healthy,
    Unresponsive,
    NotFound,
    PermissionDenied,
}

impl std::fmt::Display for SocketStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SocketStatus::Unknown => write!(f, "UNKNOWN"),
            SocketStatus::Healthy => write!(f, "HEALTHY"),
            SocketStatus::Unresponsive => write!(f, "UNRESPONSIVE"),
            SocketStatus::NotFound => write!(f, "NOT_FOUND"),
            SocketStatus::PermissionDenied => write!(f, "PERMISSION_DENIED"),
        }
    }
}

impl SocketHealth {
    /// Check socket health
    pub fn check(path: &str) -> Self {
        let socket_path = std::path::Path::new(path);
        let exists = socket_path.exists();

        let status = if !exists {
            SocketStatus::NotFound
        } else {
            // Check if we can access the socket
            match std::fs::metadata(socket_path) {
                Ok(meta) => {
                    if meta.permissions().readonly() {
                        SocketStatus::PermissionDenied
                    } else {
                        // Basic existence check passed, connection test needed
                        SocketStatus::Unknown
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                    SocketStatus::PermissionDenied
                }
                Err(_) => SocketStatus::Unknown,
            }
        };

        Self {
            path: path.to_string(),
            exists,
            status,
            last_ping: None,
            last_error: None,
        }
    }

    /// Mark as healthy after successful connection
    pub fn mark_healthy(&mut self) {
        self.status = SocketStatus::Healthy;
        self.last_ping = Some(chrono::Utc::now().to_rfc3339());
        self.last_error = None;
    }

    /// Mark as unresponsive with error
    pub fn mark_unresponsive(&mut self, error: &str) {
        self.status = SocketStatus::Unresponsive;
        self.last_error = Some(error.to_string());
    }
}
