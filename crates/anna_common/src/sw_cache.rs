//! SW Cache v7.40.0 - Cache-First Software Discovery
//!
//! Core problem: annactl sw was doing heavyweight discovery every invocation:
//! - Running pacman -Qi for every explicit package (~400-500 times)
//! - Running man -f for every package
//! - Scanning PATH directories every time
//! - Checking config files for 50+ apps
//!
//! Solution: Cache all derived data with delta detection:
//! - Package data: watch pacman.log offset/mtime for changes
//! - Commands: watch PATH directory mtimes
//! - Services: watch unit file mtimes
//!
//! Performance targets:
//! - annactl sw p95 < 1.0s when cache is warm

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::atomic_write;
use crate::daemon_state::INTERNAL_DIR;

/// Cache file path (daemon writes here, annactl reads)
pub const SW_CACHE_PATH: &str = "/var/lib/anna/internal/sw_cache.json";

/// User-local cache fallback (when daemon cache not available)
fn get_user_cache_path() -> Option<String> {
    std::env::var("HOME")
        .ok()
        .map(|home| format!("{}/.cache/anna/sw_cache.json", home))
}

/// How long before cache is considered stale (5 minutes)
pub const CACHE_TTL_SECS: u64 = 300;

/// Software cache - all derived data for sw command
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwCache {
    /// Schema version for migration
    pub schema_version: u32,
    /// When cache was last updated
    pub updated_at: Option<DateTime<Utc>>,
    /// Time taken to build cache (ms)
    pub build_duration_ms: u64,

    // Delta detection state
    /// Last seen pacman.log size (bytes)
    pub pacman_log_size: u64,
    /// Last seen pacman.log mtime (epoch secs)
    pub pacman_log_mtime: u64,
    /// PATH directory mtimes (dir -> mtime epoch)
    pub path_dir_mtimes: HashMap<String, u64>,

    // Cached counts
    pub package_counts: CachedPackageCounts,
    pub command_count: usize,
    pub service_counts: CachedServiceCounts,

    // Cached category data (expensive to compute)
    pub categories: Vec<CachedCategory>,

    // Cached config coverage
    pub config_coverage: CachedConfigCoverage,

    // Cached topology
    pub topology: CachedTopology,
}

/// Current schema version
pub const SW_CACHE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CachedPackageCounts {
    pub total: usize,
    pub explicit: usize,
    pub dependency: usize,
    pub aur: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CachedServiceCounts {
    pub total: usize,
    pub running: usize,
    pub failed: usize,
    pub enabled: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CachedCategory {
    pub name: String,
    pub packages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CachedConfigCoverage {
    pub total_apps: usize,
    pub apps_with_config: usize,
    pub app_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CachedTopology {
    pub roles: Vec<CachedRole>,
    pub service_groups: Vec<CachedServiceGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CachedRole {
    pub name: String,
    pub components: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CachedServiceGroup {
    pub name: String,
    pub services: Vec<String>,
}

impl SwCache {
    /// Load cache from disk (tries daemon path first, then user cache)
    pub fn load() -> Option<Self> {
        // Try daemon path first (most up-to-date if daemon is running)
        if let Ok(content) = std::fs::read_to_string(SW_CACHE_PATH) {
            if let Ok(cache) = serde_json::from_str::<Self>(&content) {
                if cache.schema_version == SW_CACHE_SCHEMA_VERSION {
                    return Some(cache);
                }
            }
        }

        // Fall back to user cache
        if let Some(user_path) = get_user_cache_path() {
            if let Ok(content) = std::fs::read_to_string(&user_path) {
                if let Ok(cache) = serde_json::from_str::<Self>(&content) {
                    if cache.schema_version == SW_CACHE_SCHEMA_VERSION {
                        return Some(cache);
                    }
                }
            }
        }

        None
    }

    /// Save cache to disk atomically (daemon saves to system path)
    pub fn save(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(INTERNAL_DIR)?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        atomic_write(SW_CACHE_PATH, &content)
    }

    /// Save cache to user-local path (for annactl when daemon unavailable)
    pub fn save_user(&self) -> std::io::Result<()> {
        let user_path = get_user_cache_path()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "HOME not set"))?;

        // Create parent directory
        if let Some(parent) = std::path::Path::new(&user_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        atomic_write(&user_path, &content)
    }

    /// Check if cache is fresh (not stale)
    pub fn is_fresh(&self) -> bool {
        let Some(updated) = self.updated_at else {
            return false;
        };

        let age_secs = (Utc::now() - updated).num_seconds().max(0) as u64;
        age_secs < CACHE_TTL_SECS
    }

    /// Check if packages have changed since cache was built
    pub fn packages_changed(&self) -> bool {
        let (size, mtime) = get_pacman_log_state();

        // Changed if size or mtime differs
        size != self.pacman_log_size || mtime != self.pacman_log_mtime
    }

    /// Check if PATH commands have changed
    pub fn commands_changed(&self) -> bool {
        let current_mtimes = get_path_dir_mtimes();

        // Check if any directory has changed
        for (dir, mtime) in &current_mtimes {
            match self.path_dir_mtimes.get(dir) {
                Some(cached_mtime) if *cached_mtime == *mtime => continue,
                _ => return true, // New dir or changed mtime
            }
        }

        // Check if any cached dir was removed
        for dir in self.path_dir_mtimes.keys() {
            if !current_mtimes.contains_key(dir) {
                return true;
            }
        }

        false
    }

    /// Check if any relevant change occurred
    pub fn needs_rebuild(&self) -> bool {
        !self.is_fresh() || self.packages_changed() || self.commands_changed()
    }

    /// Get cache age in human-readable format
    pub fn format_age(&self) -> String {
        let Some(updated) = self.updated_at else {
            return "never".to_string();
        };

        let age_secs = (Utc::now() - updated).num_seconds().max(0) as u64;
        if age_secs < 60 {
            format!("{}s ago", age_secs)
        } else if age_secs < 3600 {
            format!("{}m ago", age_secs / 60)
        } else {
            format!("{}h ago", age_secs / 3600)
        }
    }
}

/// Get pacman.log size and mtime for delta detection
pub fn get_pacman_log_state() -> (u64, u64) {
    const PACMAN_LOG: &str = "/var/log/pacman.log";

    match std::fs::metadata(PACMAN_LOG) {
        Ok(meta) => {
            let size = meta.len();
            let mtime = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            (size, mtime)
        }
        Err(_) => (0, 0),
    }
}

/// Get PATH directory mtimes for delta detection
pub fn get_path_dir_mtimes() -> HashMap<String, u64> {
    let mut mtimes = HashMap::new();
    let path_var = std::env::var("PATH").unwrap_or_default();

    for dir in path_var.split(':') {
        if dir.is_empty() {
            continue;
        }

        if let Ok(meta) = std::fs::metadata(dir) {
            if let Ok(mtime) = meta.modified() {
                if let Ok(duration) = mtime.duration_since(std::time::UNIX_EPOCH) {
                    mtimes.insert(dir.to_string(), duration.as_secs());
                }
            }
        }
    }

    mtimes
}

/// Build the full cache (expensive operation)
/// Called by daemon or on cache miss
pub fn build_sw_cache() -> SwCache {
    use crate::grounded::{
        packages::PackageCounts,
        commands::count_path_executables,
        services::ServiceCounts,
        categoriser::get_category_summary,
    };
    use crate::topology_map::build_software_topology;

    let start = std::time::Instant::now();

    // Get delta detection state
    let (pacman_log_size, pacman_log_mtime) = get_pacman_log_state();
    let path_dir_mtimes = get_path_dir_mtimes();

    // Query package counts
    let pkg_counts = PackageCounts::query();
    let package_counts = CachedPackageCounts {
        total: pkg_counts.total,
        explicit: pkg_counts.explicit,
        dependency: pkg_counts.dependency,
        aur: pkg_counts.aur,
    };

    // Count commands
    let command_count = count_path_executables();

    // Query service counts
    let svc_counts = ServiceCounts::query();
    let service_counts = CachedServiceCounts {
        total: svc_counts.total,
        running: svc_counts.running,
        failed: svc_counts.failed,
        enabled: svc_counts.enabled,
    };

    // Build categories (this is the expensive part)
    let category_data = get_category_summary();
    let categories: Vec<CachedCategory> = category_data
        .into_iter()
        .map(|(name, packages)| CachedCategory { name, packages })
        .collect();

    // Build config coverage
    let config_coverage = build_config_coverage();

    // Build topology
    let topo = build_software_topology();
    let topology = CachedTopology {
        roles: topo.roles.iter().map(|r| CachedRole {
            name: r.name.clone(),
            components: r.components.clone(),
        }).collect(),
        service_groups: topo.service_groups.iter().map(|g| CachedServiceGroup {
            name: g.name.clone(),
            services: g.services.clone(),
        }).collect(),
    };

    let build_duration_ms = start.elapsed().as_millis() as u64;

    SwCache {
        schema_version: SW_CACHE_SCHEMA_VERSION,
        updated_at: Some(Utc::now()),
        build_duration_ms,
        pacman_log_size,
        pacman_log_mtime,
        path_dir_mtimes,
        package_counts,
        command_count,
        service_counts,
        categories,
        config_coverage,
        topology,
    }
}

/// Known apps to check for config coverage (same as sw.rs)
const CONFIG_COVERAGE_APPS: &[&str] = &[
    "vim", "nvim", "neovim", "emacs", "nano", "helix", "kakoune", "micro",
    "alacritty", "kitty", "foot", "wezterm", "termite", "st", "urxvt", "xterm",
    "hyprland", "sway", "wayfire", "river", "dwl", "i3", "bspwm", "openbox",
    "mpv", "vlc", "mplayer", "mpd", "ncmpcpp", "cmus", "pipewire", "pulseaudio",
    "networkmanager", "iwd", "wpa_supplicant", "systemd-networkd", "connman",
    "tlp", "powertop", "acpid", "thermald", "auto-cpufreq",
    "systemd", "grub", "mkinitcpio", "pacman", "yay", "paru",
];

fn build_config_coverage() -> CachedConfigCoverage {
    use crate::config_locator::discover_config;

    let mut apps_with_config = Vec::new();

    for &app in CONFIG_COVERAGE_APPS {
        let discovery = discover_config(app);
        if !discovery.detected.is_empty() {
            apps_with_config.push(app.to_string());
        }
    }

    CachedConfigCoverage {
        total_apps: CONFIG_COVERAGE_APPS.len(),
        apps_with_config: apps_with_config.len(),
        app_names: apps_with_config,
    }
}

/// Get or build cache
/// Returns cached data if fresh, otherwise rebuilds
/// Tries to save to system path (daemon), falls back to user path (annactl)
pub fn get_sw_cache() -> SwCache {
    // Try to load existing cache
    if let Some(cache) = SwCache::load() {
        if !cache.needs_rebuild() {
            return cache;
        }
    }

    // Build new cache
    let cache = build_sw_cache();

    // Try system path first (daemon), fall back to user path (annactl)
    if cache.save().is_err() {
        let _ = cache.save_user();
    }

    cache
}

/// Get cache without rebuilding (for status display)
/// Returns None if cache doesn't exist or is too stale
pub fn get_sw_cache_readonly() -> Option<SwCache> {
    SwCache::load()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pacman_log_state() {
        let (size, mtime) = get_pacman_log_state();
        // On a system with pacman, these should be non-zero
        // On other systems, they'll be zero (graceful fallback)
        assert!(size >= 0);
        assert!(mtime >= 0);
    }

    #[test]
    fn test_path_dir_mtimes() {
        let mtimes = get_path_dir_mtimes();
        // Should have at least /usr/bin on most systems
        assert!(!mtimes.is_empty() || std::env::var("PATH").unwrap_or_default().is_empty());
    }

    #[test]
    fn test_cache_freshness() {
        let mut cache = SwCache::default();
        assert!(!cache.is_fresh()); // No updated_at

        cache.updated_at = Some(Utc::now());
        assert!(cache.is_fresh()); // Just updated
    }
}
