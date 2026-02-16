//! Wire protocol between annad and anna-executor.
//!
//! Only structured enum variants cross this boundary — no shell strings, ever.
//! The enum is the allowlist. Any unrecognized variant is rejected at deserialization.

use serde::{Deserialize, Serialize};

/// A privileged operation request from annad to anna-executor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutorRequest {
    /// Restart a named system service via systemctl.
    RestartService { name: String },
    /// Vacuum the systemd journal, keeping entries newer than `keep_days` days.
    CleanJournal { keep_days: u32 },
    /// Run paccache to keep `keep_versions` cached package versions.
    CleanPackageCache { keep_versions: u32 },
    /// Delete files in /tmp older than 1 day.
    CleanTmpFiles,
}

/// Response from anna-executor to annad.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutorResponse {
    /// Operation succeeded. `output` contains stdout of the command.
    Ok { output: String },
    /// Request was structurally valid but not permitted (e.g. service not in allowlist).
    Denied { reason: String },
    /// Operation failed at the OS level.
    Error { message: String },
}
