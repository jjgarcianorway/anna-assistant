//! Core types for daemon state management.

use anna_shared::session::SessionStore;
use anna_shared::status::{DaemonState, RecoveryStatus, UpdateCheckState};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::time::Instant;

use super::DEFAULT_UPDATE_CHECK_INTERVAL;

/// A cached answer with timestamp
#[derive(Clone)]
pub struct CachedAnswer {
    pub answer: String,
    pub cached_at: Instant,
}

/// Commands that rarely change and can be cached longer (5 minutes)
/// Used by core_loop.rs for cache TTL decisions
/// v0.0.928: Expanded list for better cache hit rate
pub const STATIC_COMMANDS: &[&str] = &[
    // System info
    "uname -r",
    "uname -a",
    "uname -m",
    "cat /etc/os-release",
    "hostnamectl",
    "hostname",
    // Hardware
    "lscpu",
    "lsblk",
    "lspci",
    "lsusb",
    "lsmod",
    "cat /proc/cpuinfo",
    "cat /proc/meminfo",
    // GPU info
    "lspci | grep -i vga",
    "lspci | grep -i nvidia",
    "lspci | grep -i amd",
    // Package info
    "pacman -Q",
    "pacman -Qe",
    "pacman -Qm",
    // Resource usage (semi-static - changes slowly)
    "free -h",
    "df -h",
    "findmnt",
    // Config files (rarely change)
    "cat /etc/fstab",
    "cat /etc/hostname",
    "cat /etc/locale.conf",
    "cat /etc/vconsole.conf",
    "cat /etc/mkinitcpio.conf",
    // Kernel
    "cat /proc/cmdline",
    "cat /proc/version",
    // Network config (changes infrequently)
    "cat /etc/resolv.conf",
    "ip link",
    // Desktop environment
    "echo $XDG_SESSION_TYPE",
    "echo $XDG_CURRENT_DESKTOP",
];

/// Update state
pub struct UpdateState {
    pub enabled: bool,
    pub check_interval_secs: u64,
    pub last_check_at: Option<DateTime<Utc>>,
    pub next_check_at: Option<DateTime<Utc>>,
    pub latest_version: Option<String>,
    pub latest_checked_at: Option<DateTime<Utc>>,
    pub update_available: bool,
    pub check_state: UpdateCheckState,
}

impl Default for UpdateState {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_secs: DEFAULT_UPDATE_CHECK_INTERVAL,
            last_check_at: None,
            next_check_at: None,
            latest_version: None,
            latest_checked_at: None,
            update_available: false,
            check_state: UpdateCheckState::NeverChecked,
        }
    }
}

/// Inner state
pub struct StateInner {
    pub state: DaemonState,
    pub started_at: Instant,
    pub ollama_running: bool,
    pub model: Option<String>,
    pub last_error: Option<String>,
    pub update: UpdateState,
    pub gpu: Option<String>,
    pub vram_mb: Option<u64>,
    /// Persistent session storage
    pub sessions: SessionStore,
    /// Number of active connections (for graceful shutdown)
    pub active_connections: u32,
    /// Flag indicating restart is pending (clients should finish quickly)
    pub restart_pending: bool,
    /// Counter for periodic session saves
    pub(crate) session_save_counter: u32,
    /// Answer cache for identical questions (normalized question -> (answer, timestamp))
    pub(crate) answer_cache: HashMap<String, CachedAnswer>,
    /// Human-readable description of current initialization step
    pub init_status: String,
    /// v0.3.36: Self-healing recovery metrics
    pub recovery_status: RecoveryStatus,
    /// Event-driven system command cache
    pub cache: crate::cache::SystemCache,
}
