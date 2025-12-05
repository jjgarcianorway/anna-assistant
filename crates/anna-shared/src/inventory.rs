//! Installed software inventory cache (v0.0.39, enhanced v0.0.41).
//!
//! Caches information about installed tools to prevent asking about
//! non-installed options and to speed up clarification flows.
//!
//! v0.0.41: Added SystemInfo (hostname, user, arch, kernel, package_count,
//! desktops, gpu_present). TTL reduced to 10 minutes for faster updates.
//!
//! Storage: ~/.anna/inventory.json

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

/// Default inventory TTL in seconds (10 minutes) - v0.0.41
pub const INVENTORY_TTL_SECS: u64 = 600;

/// VIP tools to check on inventory refresh
pub const VIP_TOOLS: &[&str] = &[
    "vim", "vi", "nano", "emacs", "nvim", "code", "micro", // Editors
    "git", "hg", "svn",                                     // VCS
    "pacman", "yay", "paru",                                // Arch package managers
    "systemctl", "journalctl",                              // Systemd
    "ip", "nmcli", "iwctl", "ping",                         // Network
    "docker", "podman",                                     // Containers
    "ssh", "rsync",                                         // Remote
    "python", "python3", "node", "npm", "cargo", "rustc",   // Languages
];

/// Desktop environment packages to detect
pub const DESKTOP_PACKAGES: &[&str] = &[
    "gnome-shell", "plasma-desktop", "xfce4-session", "cinnamon",
    "mate-session-manager", "budgie-desktop", "lxqt-session", "sway", "i3",
];

/// State of an inventory item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InventoryState {
    /// Tool is installed and verified
    Installed,
    /// Tool was installed but may have been removed
    Stale,
    /// Tool is not installed
    NotInstalled,
}

impl Default for InventoryState {
    fn default() -> Self {
        Self::Stale
    }
}

/// An item in the inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    /// Tool name (e.g., "vim")
    pub name: String,
    /// Full path to the binary (if installed)
    pub path: Option<String>,
    /// Current state
    pub state: InventoryState,
    /// Unix timestamp when last verified
    pub verified_at: u64,
    /// Seconds until considered stale
    pub stale_after: u64,
}

impl InventoryItem {
    /// Create a new installed item
    pub fn installed(name: &str, path: &str) -> Self {
        Self {
            name: name.to_string(),
            path: Some(path.to_string()),
            state: InventoryState::Installed,
            verified_at: current_timestamp(),
            stale_after: INVENTORY_TTL_SECS, // v0.0.41: 10 minutes
        }
    }

    /// Create a not-installed item
    pub fn not_installed(name: &str) -> Self {
        Self {
            name: name.to_string(),
            path: None,
            state: InventoryState::NotInstalled,
            verified_at: current_timestamp(),
            stale_after: INVENTORY_TTL_SECS, // v0.0.41: 10 minutes
        }
    }

    /// Check if this item is stale
    pub fn is_stale(&self) -> bool {
        let now = current_timestamp();
        now.saturating_sub(self.verified_at) > self.stale_after
    }

    /// Mark as stale
    pub fn mark_stale(&mut self) {
        self.state = InventoryState::Stale;
    }
}

/// System information snapshot (v0.0.41)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Hostname
    pub hostname: String,
    /// Current user
    pub user: String,
    /// System architecture (e.g., x86_64)
    pub arch: String,
    /// Kernel version
    pub kernel: String,
    /// Total package count (if available)
    pub package_count: Option<u32>,
    /// Detected desktop environments
    pub desktops: Vec<String>,
    /// Whether GPU is present (from lspci)
    pub gpu_present: Option<bool>,
    /// GPU vendor if detected
    pub gpu_vendor: Option<String>,
}

impl SystemInfo {
    /// Collect system info from probes
    pub fn collect() -> Self {
        let hostname = run_command("hostname").unwrap_or_else(|| "unknown".to_string());
        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
        let arch = run_command("uname -m").unwrap_or_else(|| "unknown".to_string());
        let kernel = run_command("uname -r").unwrap_or_else(|| "unknown".to_string());

        // Package count (Arch Linux)
        let package_count = run_command("pacman -Qq")
            .map(|out| out.lines().count() as u32);

        // Detect desktops
        let desktops = detect_desktops();

        // GPU detection
        let (gpu_present, gpu_vendor) = detect_gpu();

        Self {
            hostname,
            user,
            arch,
            kernel,
            package_count,
            desktops,
            gpu_present,
            gpu_vendor,
        }
    }
}

/// Inventory cache for installed tools
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InventoryCache {
    /// Map of tool name -> inventory item
    pub items: BTreeMap<String, InventoryItem>,
    /// Unix timestamp when cache was last updated
    pub last_updated: u64,
    /// Version of the inventory format
    #[serde(default)]
    pub version: u8,
    /// System information (v0.0.41)
    #[serde(default)]
    pub system_info: Option<SystemInfo>,
}

impl InventoryCache {
    /// Create new empty cache
    pub fn new() -> Self {
        Self {
            items: BTreeMap::new(),
            last_updated: current_timestamp(),
            version: 1,
            system_info: None,
        }
    }

    /// Check if a tool is installed (from cache)
    pub fn is_installed(&self, name: &str) -> Option<bool> {
        self.items.get(name).map(|item| {
            item.state == InventoryState::Installed && !item.is_stale()
        })
    }

    /// Get all installed tools
    pub fn installed_tools(&self) -> Vec<&str> {
        self.items
            .iter()
            .filter(|(_, item)| item.state == InventoryState::Installed && !item.is_stale())
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Get installed editors only
    pub fn installed_editors(&self) -> Vec<&str> {
        let editor_names = ["vim", "vi", "nano", "emacs", "nvim", "code", "micro"];
        self.items
            .iter()
            .filter(|(name, item)| {
                editor_names.contains(&name.as_str())
                    && item.state == InventoryState::Installed
                    && !item.is_stale()
            })
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Verify and update a single tool
    pub fn verify_tool(&mut self, name: &str) -> bool {
        if let Some(path) = check_tool_installed(name) {
            self.items.insert(name.to_string(), InventoryItem::installed(name, &path));
            true
        } else {
            self.items.insert(name.to_string(), InventoryItem::not_installed(name));
            false
        }
    }

    /// Refresh VIP tools
    pub fn refresh_vip_tools(&mut self) {
        for &tool in VIP_TOOLS {
            self.verify_tool(tool);
        }
        self.last_updated = current_timestamp();
    }

    /// Refresh system info (v0.0.41)
    pub fn refresh_system_info(&mut self) {
        self.system_info = Some(SystemInfo::collect());
        self.last_updated = current_timestamp();
    }

    /// Full refresh: VIP tools + system info (v0.0.41)
    pub fn full_refresh(&mut self) {
        self.refresh_vip_tools();
        self.refresh_system_info();
    }

    /// Get system info (collecting if not present)
    pub fn get_system_info(&mut self) -> &SystemInfo {
        if self.system_info.is_none() {
            self.refresh_system_info();
        }
        self.system_info.as_ref().unwrap()
    }

    /// Mark stale items
    pub fn mark_stale_items(&mut self) {
        for item in self.items.values_mut() {
            if item.is_stale() {
                item.mark_stale();
            }
        }
    }

    /// Count of installed tools
    pub fn installed_count(&self) -> usize {
        self.items
            .values()
            .filter(|item| item.state == InventoryState::Installed)
            .count()
    }
}

/// Check if a tool is installed using `command -v`
pub fn check_tool_installed(name: &str) -> Option<String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {}", name))
        .output()
        .ok()?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Some(path);
        }
    }
    None
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// === Persistence ===

/// Get inventory cache file path
pub fn inventory_path() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".anna")
        .join("inventory.json")
}

/// Load inventory cache from disk
pub fn load_inventory() -> Option<InventoryCache> {
    let path = inventory_path();
    if !path.exists() {
        return None;
    }
    let data = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Save inventory cache to disk
pub fn save_inventory(cache: &InventoryCache) -> std::io::Result<()> {
    let path = inventory_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(cache)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, json)
}

/// Clear inventory cache
pub fn clear_inventory() -> std::io::Result<()> {
    let path = inventory_path();
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// Load or create fresh inventory (with VIP tools check)
pub fn load_or_create_inventory() -> InventoryCache {
    if let Some(mut cache) = load_inventory() {
        cache.mark_stale_items();
        return cache;
    }

    // Create fresh inventory with VIP tools
    let mut cache = InventoryCache::new();
    cache.refresh_vip_tools();
    let _ = save_inventory(&cache);
    cache
}

/// Filter options to only installed ones
pub fn filter_installed_options(options: &[&str]) -> Vec<String> {
    let cache = load_or_create_inventory();
    options
        .iter()
        .filter(|&&opt| cache.is_installed(opt).unwrap_or(false))
        .map(|&s| s.to_string())
        .collect()
}

// === Helper functions for SystemInfo (v0.0.41) ===

/// Run a command and return stdout (trimmed)
fn run_command(cmd: &str) -> Option<String> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }
    let output = Command::new(parts[0])
        .args(&parts[1..])
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Detect installed desktop environments
fn detect_desktops() -> Vec<String> {
    let mut desktops = Vec::new();

    // Check XDG_CURRENT_DESKTOP first
    if let Ok(de) = std::env::var("XDG_CURRENT_DESKTOP") {
        for d in de.split(':') {
            if !d.is_empty() {
                desktops.push(d.to_string());
            }
        }
    }

    // Check for DE packages
    for &pkg in DESKTOP_PACKAGES {
        if check_tool_installed(pkg).is_some() {
            if !desktops.iter().any(|d| d.to_lowercase().contains(&pkg.to_lowercase())) {
                desktops.push(pkg.to_string());
            }
        }
    }

    desktops.sort();
    desktops.dedup();
    desktops
}

/// Detect GPU presence and vendor
fn detect_gpu() -> (Option<bool>, Option<String>) {
    // Try lspci for GPU detection
    if let Some(output) = run_command("lspci") {
        let lower = output.to_lowercase();
        if lower.contains("vga") || lower.contains("3d controller") || lower.contains("display controller") {
            // Determine vendor
            let vendor = if lower.contains("nvidia") {
                Some("NVIDIA".to_string())
            } else if lower.contains("amd") || lower.contains("radeon") {
                Some("AMD".to_string())
            } else if lower.contains("intel") {
                Some("Intel".to_string())
            } else {
                Some("Unknown".to_string())
            };
            return (Some(true), vendor);
        }
        return (Some(false), None);
    }
    (None, None) // Could not detect
}

/// Check if inventory cache is fresh (not stale) - v0.0.41
pub fn is_inventory_fresh(cache: &InventoryCache) -> bool {
    let now = current_timestamp();
    now.saturating_sub(cache.last_updated) <= INVENTORY_TTL_SECS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_cache_new() {
        let cache = InventoryCache::new();
        assert!(cache.items.is_empty());
        assert_eq!(cache.version, 1);
    }

    #[test]
    fn test_inventory_item_installed() {
        let item = InventoryItem::installed("vim", "/usr/bin/vim");
        assert_eq!(item.state, InventoryState::Installed);
        assert_eq!(item.path, Some("/usr/bin/vim".to_string()));
        assert!(!item.is_stale());
    }

    #[test]
    fn test_inventory_item_stale() {
        let mut item = InventoryItem::installed("vim", "/usr/bin/vim");
        item.verified_at = 0; // Very old
        assert!(item.is_stale());
    }

    #[test]
    fn test_inventory_verify_tool() {
        let mut cache = InventoryCache::new();
        // Test with a tool that's almost certainly installed
        let has_sh = cache.verify_tool("sh");
        assert!(has_sh); // sh should be on any Unix system
        assert_eq!(cache.is_installed("sh"), Some(true));
    }

    #[test]
    fn test_installed_editors_filter() {
        let mut cache = InventoryCache::new();
        cache.items.insert("vim".to_string(), InventoryItem::installed("vim", "/usr/bin/vim"));
        cache.items.insert("nano".to_string(), InventoryItem::not_installed("nano"));

        let editors = cache.installed_editors();
        assert!(editors.contains(&"vim"));
        assert!(!editors.contains(&"nano"));
    }
}
