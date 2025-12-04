//! Snapshots v7.41.0 - Daemon-Owned Data Snapshots
//!
//! Architecture rule: annad WRITES snapshots, annactl ONLY READS.
//! annactl must NEVER do heavyweight scanning - it's a thin display client.
//!
//! Snapshot locations:
//! - /var/lib/anna/internal/snapshots/sw.json
//! - /var/lib/anna/internal/snapshots/hw.json
//! - /var/lib/anna/internal/snapshots/status.json
//!
//! Meta locations (daemon-only, for delta detection):
//! - /var/lib/anna/internal/meta/sw_meta.json
//! - /var/lib/anna/internal/meta/hw_meta.json
//!
//! All writes are atomic: temp file + fsync + rename

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::os::unix::fs::MetadataExt;

use crate::atomic_write;

/// Base directory for snapshots
pub const SNAPSHOTS_DIR: &str = "/var/lib/anna/internal/snapshots";
pub const META_DIR: &str = "/var/lib/anna/internal/meta";

/// Current schema versions
pub const SW_SNAPSHOT_VERSION: u32 = 2; // v7.41.0: new format
pub const HW_SNAPSHOT_VERSION: u32 = 1;

// =============================================================================
// SOFTWARE SNAPSHOT
// =============================================================================

/// Software snapshot - everything annactl sw needs to display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwSnapshot {
    /// Schema version for migration
    pub data_version: u32,
    /// When this snapshot was generated
    pub generated_at: DateTime<Utc>,
    /// How long the scan took (ms)
    pub scan_duration_ms: u64,

    // Package data
    pub packages: PackageSnapshot,

    // Command data
    pub commands: CommandSnapshot,

    // Service data
    pub services: ServiceSnapshot,

    // Categories (pre-computed)
    pub categories: Vec<CategoryEntry>,

    // Config coverage (pre-computed)
    pub config_coverage: ConfigCoverage,

    // Topology (pre-computed)
    pub topology: TopologySnapshot,

    // Steam/game platforms (pre-computed)
    pub platforms: PlatformSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackageSnapshot {
    pub total: usize,
    pub explicit: usize,
    pub dependency: usize,
    pub aur: usize,
    /// Top packages by some metric (for compact display)
    pub highlights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommandSnapshot {
    pub total: usize,
    /// Sample commands for display
    pub highlights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceSnapshot {
    pub total: usize,
    pub running: usize,
    pub failed: usize,
    pub enabled: usize,
    /// Failed service names for display
    pub failed_services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CategoryEntry {
    pub name: String,
    pub count: usize,
    pub packages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigCoverage {
    pub total_apps: usize,
    pub apps_with_config: usize,
    pub app_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TopologySnapshot {
    pub roles: Vec<RoleEntry>,
    pub service_groups: Vec<ServiceGroupEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RoleEntry {
    pub name: String,
    pub components: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceGroupEntry {
    pub name: String,
    pub services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlatformSnapshot {
    pub steam_installed: bool,
    pub steam_game_count: usize,
    pub steam_total_size_bytes: u64,
    pub steam_top_games: Vec<GameEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameEntry {
    pub name: String,
    pub size_bytes: u64,
}

impl SwSnapshot {
    pub fn path() -> String {
        format!("{}/sw.json", SNAPSHOTS_DIR)
    }

    /// Load snapshot (annactl uses this)
    pub fn load() -> Option<Self> {
        let content = std::fs::read_to_string(Self::path()).ok()?;
        let snapshot: Self = serde_json::from_str(&content).ok()?;

        // Check schema version
        if snapshot.data_version != SW_SNAPSHOT_VERSION {
            return None;
        }
        Some(snapshot)
    }

    /// Save snapshot atomically (daemon uses this)
    pub fn save(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(SNAPSHOTS_DIR)?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        atomic_write(&Self::path(), &content)
    }

    /// Get age in human-readable format
    pub fn format_age(&self) -> String {
        let age_secs = (Utc::now() - self.generated_at).num_seconds().max(0) as u64;
        if age_secs < 60 {
            format!("{}s ago", age_secs)
        } else if age_secs < 3600 {
            format!("{}m ago", age_secs / 60)
        } else {
            format!("{}h ago", age_secs / 3600)
        }
    }

    /// Check if snapshot is reasonably fresh (< 10 minutes)
    pub fn is_fresh(&self) -> bool {
        let age_secs = (Utc::now() - self.generated_at).num_seconds().max(0) as u64;
        age_secs < 600
    }
}

// =============================================================================
// SOFTWARE META (daemon-only, for delta detection)
// =============================================================================

/// Metadata for delta detection - daemon internal use only
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwMeta {
    /// pacman.log fingerprint
    pub pacman_log: PacmanLogFingerprint,
    /// PATH directory fingerprints
    pub path_dirs: HashMap<String, DirFingerprint>,
    /// systemd fingerprint
    pub systemd: SystemdFingerprint,
    /// Last full scan time
    pub last_full_scan: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PacmanLogFingerprint {
    pub inode: u64,
    pub size: u64,
    pub mtime: u64,
    pub last_byte_offset: u64,
    /// Hash of last line for rotation detection
    pub last_line_hash: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DirFingerprint {
    pub inode: u64,
    pub mtime: u64,
    pub file_count: usize,
    /// Simple hash of filenames for quick change detection
    pub names_hash: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemdFingerprint {
    /// Hash of systemctl list-unit-files output
    pub unit_files_hash: u64,
    /// mtime of /etc/systemd/system
    pub etc_mtime: u64,
    /// mtime of /usr/lib/systemd/system
    pub usr_lib_mtime: u64,
}

impl SwMeta {
    pub fn path() -> String {
        format!("{}/sw_meta.json", META_DIR)
    }

    pub fn load() -> Option<Self> {
        let content = std::fs::read_to_string(Self::path()).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn save(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(META_DIR)?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        atomic_write(&Self::path(), &content)
    }
}

// =============================================================================
// DELTA DETECTION HELPERS
// =============================================================================

/// Get current pacman.log fingerprint
pub fn get_pacman_log_fingerprint() -> PacmanLogFingerprint {
    const PACMAN_LOG: &str = "/var/log/pacman.log";

    match std::fs::metadata(PACMAN_LOG) {
        Ok(meta) => {
            let mtime = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            // Read last line for hash
            let last_line_hash = std::fs::read_to_string(PACMAN_LOG)
                .ok()
                .and_then(|content| content.lines().last().map(|l| simple_hash(l)));

            PacmanLogFingerprint {
                inode: meta.ino(),
                size: meta.len(),
                mtime,
                last_byte_offset: meta.len(),
                last_line_hash,
            }
        }
        Err(_) => PacmanLogFingerprint::default(),
    }
}

/// Get fingerprint for a directory
pub fn get_dir_fingerprint(path: &str) -> Option<DirFingerprint> {
    let meta = std::fs::metadata(path).ok()?;
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Count files and hash names
    let mut file_count = 0;
    let mut names_hash: u64 = 0;

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_file() || ft.is_symlink() {
                    file_count += 1;
                    if let Some(name) = entry.file_name().to_str() {
                        names_hash = names_hash.wrapping_add(simple_hash(name));
                    }
                }
            }
        }
    }

    Some(DirFingerprint {
        inode: meta.ino(),
        mtime,
        file_count,
        names_hash,
    })
}

/// Get current systemd fingerprint
pub fn get_systemd_fingerprint() -> SystemdFingerprint {
    let etc_mtime = std::fs::metadata("/etc/systemd/system")
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let usr_lib_mtime = std::fs::metadata("/usr/lib/systemd/system")
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Get hash of unit files list
    let unit_files_hash = std::process::Command::new("systemctl")
        .args(["list-unit-files", "--no-pager", "--no-legend"])
        .output()
        .ok()
        .map(|o| simple_hash(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or(0);

    SystemdFingerprint {
        unit_files_hash,
        etc_mtime,
        usr_lib_mtime,
    }
}

/// Check if pacman.log needs full rebuild (rotation/truncation)
pub fn pacman_log_needs_full_rebuild(
    old: &PacmanLogFingerprint,
    new: &PacmanLogFingerprint,
) -> bool {
    // Inode changed = file rotated
    if old.inode != new.inode && old.inode != 0 {
        return true;
    }
    // Size smaller than offset = truncated
    if new.size < old.last_byte_offset {
        return true;
    }
    false
}

/// Check if pacman.log has new entries
pub fn pacman_log_has_changes(old: &PacmanLogFingerprint, new: &PacmanLogFingerprint) -> bool {
    new.size != old.size || new.mtime != old.mtime
}

/// Simple non-cryptographic hash for fingerprinting
pub fn simple_hash(s: &str) -> u64 {
    let mut hash: u64 = 5381;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    hash
}

// =============================================================================
// HARDWARE SNAPSHOT
// =============================================================================

/// Hardware snapshot for annactl hw
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HwSnapshot {
    pub data_version: u32,
    pub generated_at: DateTime<Utc>,
    pub scan_duration_ms: u64,

    pub cpu: CpuSnapshot,
    pub memory: MemorySnapshot,
    pub storage: Vec<StorageEntry>,
    pub network: Vec<NetworkEntry>,
    pub gpu: Option<GpuSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CpuSnapshot {
    pub model: String,
    pub cores: usize,
    pub threads: usize,
    pub frequency_mhz: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemorySnapshot {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub swap_total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageEntry {
    pub name: String,
    pub model: Option<String>,
    pub size_bytes: u64,
    pub mount_point: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkEntry {
    pub name: String,
    pub mac: Option<String>,
    pub ipv4: Option<String>,
    pub is_up: bool,
    pub is_wireless: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GpuSnapshot {
    pub name: String,
    pub driver: Option<String>,
    pub vram_mb: Option<u64>,
}

impl HwSnapshot {
    pub fn path() -> String {
        format!("{}/hw.json", SNAPSHOTS_DIR)
    }

    pub fn load() -> Option<Self> {
        let content = std::fs::read_to_string(Self::path()).ok()?;
        let snapshot: Self = serde_json::from_str(&content).ok()?;
        if snapshot.data_version != HW_SNAPSHOT_VERSION {
            return None;
        }
        Some(snapshot)
    }

    pub fn save(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(SNAPSHOTS_DIR)?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        atomic_write(&Self::path(), &content)
    }

    pub fn format_age(&self) -> String {
        let age_secs = (Utc::now() - self.generated_at).num_seconds().max(0) as u64;
        if age_secs < 60 {
            format!("{}s ago", age_secs)
        } else if age_secs < 3600 {
            format!("{}m ago", age_secs / 60)
        } else {
            format!("{}h ago", age_secs / 3600)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_hash() {
        let h1 = simple_hash("hello");
        let h2 = simple_hash("hello");
        let h3 = simple_hash("world");

        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_pacman_log_fingerprint() {
        let fp = get_pacman_log_fingerprint();
        // Should work even if pacman.log doesn't exist
        assert!(fp.size >= 0);
    }
}
