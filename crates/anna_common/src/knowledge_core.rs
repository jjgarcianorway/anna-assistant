//! Knowledge Core v5.0.0 - System Profiler and Knowledge Engine
//!
//! Anna as a pure observer:
//! - Watches the machine
//! - Builds structured knowledge base
//! - No Q&A, no router, no XP, no levels
//!
//! ## Data Sources
//! - Packages (pacman, AUR)
//! - Binaries on PATH
//! - Process usage over time
//! - Resource usage (CPU, memory)
//! - Config files for known tools

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// Constants
// ============================================================================

/// Knowledge store path
pub const KNOWLEDGE_STORE_PATH: &str = "/var/lib/anna/knowledge/knowledge_v5.json";

/// Telemetry store path
pub const TELEMETRY_STORE_PATH: &str = "/var/lib/anna/knowledge/telemetry_v5.json";

// ============================================================================
// Categories
// ============================================================================

/// Software category classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Editor,
    Terminal,
    Shell,
    Wm,         // Window Manager
    Compositor,
    Browser,
    Tool,
    Service,
    Unknown,
}

impl Category {
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Editor => "editor",
            Category::Terminal => "terminal",
            Category::Shell => "shell",
            Category::Wm => "wm",
            Category::Compositor => "compositor",
            Category::Browser => "browser",
            Category::Tool => "tool",
            Category::Service => "service",
            Category::Unknown => "unknown",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Category::Editor => "Editors",
            Category::Terminal => "Terminals",
            Category::Shell => "Shells",
            Category::Wm => "WMs",
            Category::Compositor => "Compositors",
            Category::Browser => "Browsers",
            Category::Tool => "Tools",
            Category::Service => "Services",
            Category::Unknown => "Unknown",
        }
    }
}

// ============================================================================
// Knowledge Object
// ============================================================================

/// A single piece of knowledge about a tool/package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeObject {
    /// Canonical name (e.g., "vim", "nano", "hyprland")
    pub name: String,

    /// Category classification
    pub category: Category,

    /// How it was detected
    pub detected_as: DetectionSource,

    /// Is it currently installed?
    pub installed: bool,

    /// Package name (if installed via package manager)
    pub package_name: Option<String>,

    /// Binary path (if found)
    pub binary_path: Option<String>,

    /// Number of times the binary was seen executed
    pub usage_count: u64,

    /// Total CPU time observed (milliseconds)
    pub total_cpu_time_ms: u64,

    /// Peak memory usage (bytes)
    pub total_mem_bytes_peak: u64,

    /// First seen timestamp (Unix seconds)
    pub first_seen_at: u64,

    /// Last seen timestamp (Unix seconds)
    pub last_seen_at: u64,

    /// Known config file paths
    pub config_paths: Vec<String>,

    /// Arch Wiki reference (e.g., "archwiki:Vim")
    pub wiki_ref: Option<String>,
}

impl KnowledgeObject {
    pub fn new(name: &str, category: Category) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            name: name.to_string(),
            category,
            detected_as: DetectionSource::Unknown,
            installed: false,
            package_name: None,
            binary_path: None,
            usage_count: 0,
            total_cpu_time_ms: 0,
            total_mem_bytes_peak: 0,
            first_seen_at: now,
            last_seen_at: now,
            config_paths: vec![],
            wiki_ref: None,
        }
    }

    /// Record a usage observation
    pub fn record_usage(&mut self, cpu_time_ms: u64, mem_bytes: u64) {
        self.usage_count += 1;
        self.total_cpu_time_ms += cpu_time_ms;
        if mem_bytes > self.total_mem_bytes_peak {
            self.total_mem_bytes_peak = mem_bytes;
        }
        self.last_seen_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Format for display
    pub fn format_summary(&self) -> String {
        let installed = if self.installed { "yes" } else { "no" };
        let wiki = self.wiki_ref.as_deref().unwrap_or("-");
        format!(
            "{:<12} installed: {:<3}  runs: {:<6} wiki: {}",
            self.name, installed, self.usage_count, wiki
        )
    }
}

/// How the software was detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DetectionSource {
    Package,
    Binary,
    Both,
    Unknown,
}

// ============================================================================
// Knowledge Store
// ============================================================================

/// The main knowledge database
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgeStore {
    /// Objects indexed by canonical name
    pub objects: HashMap<String, KnowledgeObject>,

    /// Store metadata
    pub created_at: u64,
    pub last_updated: u64,

    /// Discovery stats
    pub first_discovery_at: Option<u64>,
    pub last_discovery_at: Option<u64>,
}

impl KnowledgeStore {
    /// Create a new empty store
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            objects: HashMap::new(),
            created_at: now,
            last_updated: now,
            first_discovery_at: None,
            last_discovery_at: None,
        }
    }

    /// Load from disk
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string(KNOWLEDGE_STORE_PATH) {
            serde_json::from_str(&content).unwrap_or_else(|_| Self::new())
        } else {
            Self::new()
        }
    }

    /// Save to disk
    pub fn save(&self) -> std::io::Result<()> {
        if let Some(parent) = Path::new(KNOWLEDGE_STORE_PATH).parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(KNOWLEDGE_STORE_PATH, json)
    }

    /// Add or update an object
    pub fn upsert(&mut self, obj: KnowledgeObject) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let is_new = !self.objects.contains_key(&obj.name);

        if is_new {
            // Track discovery times
            if self.first_discovery_at.is_none() {
                self.first_discovery_at = Some(now);
            }
            self.last_discovery_at = Some(now);
        }

        self.objects.insert(obj.name.clone(), obj);
        self.last_updated = now;
    }

    /// Get object by name
    pub fn get(&self, name: &str) -> Option<&KnowledgeObject> {
        self.objects.get(name)
    }

    /// Get mutable object by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut KnowledgeObject> {
        self.objects.get_mut(name)
    }

    /// Count objects by category
    pub fn count_by_category(&self) -> HashMap<Category, usize> {
        let mut counts = HashMap::new();
        for obj in self.objects.values() {
            *counts.entry(obj.category.clone()).or_insert(0) += 1;
        }
        counts
    }

    /// Get objects by category
    pub fn get_by_category(&self, category: &Category) -> Vec<&KnowledgeObject> {
        self.objects
            .values()
            .filter(|o| &o.category == category)
            .collect()
    }

    /// Get top N objects by usage
    pub fn top_by_usage(&self, n: usize) -> Vec<&KnowledgeObject> {
        let mut sorted: Vec<_> = self.objects.values().collect();
        sorted.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
        sorted.into_iter().take(n).collect()
    }

    /// Get top N objects by CPU time
    pub fn top_by_cpu(&self, n: usize) -> Vec<&KnowledgeObject> {
        let mut sorted: Vec<_> = self.objects.values().collect();
        sorted.sort_by(|a, b| b.total_cpu_time_ms.cmp(&a.total_cpu_time_ms));
        sorted.into_iter().take(n).collect()
    }

    /// Get top N objects by memory
    pub fn top_by_memory(&self, n: usize) -> Vec<&KnowledgeObject> {
        let mut sorted: Vec<_> = self.objects.values().collect();
        sorted.sort_by(|a, b| b.total_mem_bytes_peak.cmp(&a.total_mem_bytes_peak));
        sorted.into_iter().take(n).collect()
    }

    /// Total object count
    pub fn total_objects(&self) -> usize {
        self.objects.len()
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.objects.clear();
        self.first_discovery_at = None;
        self.last_discovery_at = None;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.last_updated = now;
    }
}

// ============================================================================
// Telemetry Aggregates
// ============================================================================

/// Aggregated telemetry for process observations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TelemetryAggregates {
    /// Total processes observed
    pub processes_observed: u64,

    /// Unique commands seen
    pub unique_commands: u64,

    /// Total samples collected
    pub total_samples: u64,

    /// Daemon start time (Unix seconds)
    pub daemon_start_at: u64,

    /// Last update time
    pub last_updated: u64,

    /// Command execution counts
    pub command_counts: HashMap<String, u64>,
}

impl TelemetryAggregates {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            processes_observed: 0,
            unique_commands: 0,
            total_samples: 0,
            daemon_start_at: now,
            last_updated: now,
            command_counts: HashMap::new(),
        }
    }

    /// Load from disk
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string(TELEMETRY_STORE_PATH) {
            serde_json::from_str(&content).unwrap_or_else(|_| Self::new())
        } else {
            Self::new()
        }
    }

    /// Save to disk
    pub fn save(&self) -> std::io::Result<()> {
        if let Some(parent) = Path::new(TELEMETRY_STORE_PATH).parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(TELEMETRY_STORE_PATH, json)
    }

    /// Record a process observation
    pub fn record_process(&mut self, command: &str) {
        self.processes_observed += 1;
        self.total_samples += 1;

        let count = self.command_counts.entry(command.to_string()).or_insert(0);
        if *count == 0 {
            self.unique_commands += 1;
        }
        *count += 1;

        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Get most used command
    pub fn most_used_command(&self) -> Option<(&String, &u64)> {
        self.command_counts.iter().max_by_key(|(_, count)| *count)
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.processes_observed = 0;
        self.unique_commands = 0;
        self.total_samples = 0;
        self.command_counts.clear();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.daemon_start_at = now;
        self.last_updated = now;
    }

    /// Get uptime string
    pub fn uptime_string(&self) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let uptime_secs = now.saturating_sub(self.daemon_start_at);

        let hours = uptime_secs / 3600;
        let minutes = (uptime_secs % 3600) / 60;

        if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        }
    }
}

// ============================================================================
// Tool Classification
// ============================================================================

/// Classify a tool by name
pub fn classify_tool(name: &str) -> (Category, Option<&'static str>) {
    let lower = name.to_lowercase();

    // Editors
    if matches!(lower.as_str(), "vim" | "nvim" | "neovim") {
        return (Category::Editor, Some("archwiki:Vim"));
    }
    if lower == "nano" {
        return (Category::Editor, Some("archwiki:Nano"));
    }
    if matches!(lower.as_str(), "code" | "code-oss" | "vscode" | "visual-studio-code") {
        return (Category::Editor, Some("archwiki:Visual_Studio_Code"));
    }
    if lower == "emacs" {
        return (Category::Editor, Some("archwiki:Emacs"));
    }
    if matches!(lower.as_str(), "helix" | "hx") {
        return (Category::Editor, Some("archwiki:Helix"));
    }
    if lower == "kate" {
        return (Category::Editor, Some("archwiki:Kate"));
    }
    if lower == "gedit" {
        return (Category::Editor, None);
    }

    // Terminals
    if lower == "alacritty" {
        return (Category::Terminal, Some("archwiki:Alacritty"));
    }
    if lower == "kitty" {
        return (Category::Terminal, Some("archwiki:Kitty"));
    }
    if lower == "wezterm" {
        return (Category::Terminal, Some("archwiki:WezTerm"));
    }
    if lower == "foot" {
        return (Category::Terminal, Some("archwiki:Foot"));
    }
    if matches!(lower.as_str(), "gnome-terminal" | "gnome_terminal") {
        return (Category::Terminal, Some("archwiki:GNOME_Terminal"));
    }
    if lower == "konsole" {
        return (Category::Terminal, Some("archwiki:Konsole"));
    }
    if lower == "terminator" {
        return (Category::Terminal, None);
    }
    if lower == "st" {
        return (Category::Terminal, Some("archwiki:St"));
    }

    // Shells
    if lower == "zsh" {
        return (Category::Shell, Some("archwiki:Zsh"));
    }
    if lower == "bash" {
        return (Category::Shell, Some("archwiki:Bash"));
    }
    if lower == "fish" {
        return (Category::Shell, Some("archwiki:Fish"));
    }
    if lower == "nushell" || lower == "nu" {
        return (Category::Shell, Some("archwiki:Nushell"));
    }

    // Window Managers
    if lower == "i3" {
        return (Category::Wm, Some("archwiki:I3"));
    }
    if lower == "sway" {
        return (Category::Wm, Some("archwiki:Sway"));
    }
    if lower == "awesome" {
        return (Category::Wm, Some("archwiki:Awesome"));
    }
    if lower == "bspwm" {
        return (Category::Wm, Some("archwiki:Bspwm"));
    }
    if lower == "dwm" {
        return (Category::Wm, Some("archwiki:Dwm"));
    }
    if lower == "openbox" {
        return (Category::Wm, Some("archwiki:Openbox"));
    }

    // Compositors
    if lower == "hyprland" {
        return (Category::Compositor, Some("archwiki:Hyprland"));
    }
    if lower == "wayfire" {
        return (Category::Compositor, Some("archwiki:Wayfire"));
    }
    if lower == "river" {
        return (Category::Compositor, Some("archwiki:River"));
    }
    if lower == "picom" {
        return (Category::Compositor, Some("archwiki:Picom"));
    }

    // Browsers
    if lower == "firefox" {
        return (Category::Browser, Some("archwiki:Firefox"));
    }
    if matches!(lower.as_str(), "chromium" | "chrome" | "google-chrome") {
        return (Category::Browser, Some("archwiki:Chromium"));
    }
    if lower == "brave" {
        return (Category::Browser, Some("archwiki:Brave"));
    }
    if lower == "vivaldi" {
        return (Category::Browser, None);
    }
    if matches!(lower.as_str(), "qutebrowser" | "qute") {
        return (Category::Browser, Some("archwiki:Qutebrowser"));
    }

    // Services/Tools
    if lower == "waybar" {
        return (Category::Service, Some("archwiki:Waybar"));
    }
    if lower == "polybar" {
        return (Category::Service, Some("archwiki:Polybar"));
    }
    if lower == "dunst" {
        return (Category::Service, Some("archwiki:Dunst"));
    }
    if lower == "rofi" {
        return (Category::Service, Some("archwiki:Rofi"));
    }
    if lower == "wofi" {
        return (Category::Service, None);
    }

    // Common tools
    if matches!(lower.as_str(), "git" | "cargo" | "rustc" | "python" | "node" | "npm") {
        return (Category::Tool, None);
    }

    (Category::Unknown, None)
}

/// Get known config paths for a tool
pub fn get_config_paths(name: &str) -> Vec<String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
    let lower = name.to_lowercase();

    match lower.as_str() {
        "vim" => vec![
            format!("{}/.vimrc", home),
        ],
        "nvim" | "neovim" => vec![
            format!("{}/.config/nvim/init.vim", home),
            format!("{}/.config/nvim/init.lua", home),
        ],
        "zsh" => vec![
            format!("{}/.zshrc", home),
            format!("{}/.zshenv", home),
        ],
        "bash" => vec![
            format!("{}/.bashrc", home),
            format!("{}/.bash_profile", home),
        ],
        "fish" => vec![
            format!("{}/.config/fish/config.fish", home),
        ],
        "alacritty" => vec![
            format!("{}/.config/alacritty/alacritty.toml", home),
            format!("{}/.config/alacritty/alacritty.yml", home),
        ],
        "kitty" => vec![
            format!("{}/.config/kitty/kitty.conf", home),
        ],
        "hyprland" => vec![
            format!("{}/.config/hypr/hyprland.conf", home),
        ],
        "sway" => vec![
            format!("{}/.config/sway/config", home),
        ],
        "i3" => vec![
            format!("{}/.config/i3/config", home),
            format!("{}/.i3/config", home),
        ],
        "waybar" => vec![
            format!("{}/.config/waybar/config", home),
            format!("{}/.config/waybar/style.css", home),
        ],
        "polybar" => vec![
            format!("{}/.config/polybar/config.ini", home),
        ],
        _ => vec![],
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_display() {
        assert_eq!(Category::Editor.as_str(), "editor");
        assert_eq!(Category::Terminal.display_name(), "Terminals");
    }

    #[test]
    fn test_classify_editors() {
        assert_eq!(classify_tool("vim").0, Category::Editor);
        assert_eq!(classify_tool("nvim").0, Category::Editor);
        assert_eq!(classify_tool("nano").0, Category::Editor);
        assert_eq!(classify_tool("code").0, Category::Editor);
    }

    #[test]
    fn test_classify_terminals() {
        assert_eq!(classify_tool("alacritty").0, Category::Terminal);
        assert_eq!(classify_tool("kitty").0, Category::Terminal);
        assert_eq!(classify_tool("foot").0, Category::Terminal);
    }

    #[test]
    fn test_classify_shells() {
        assert_eq!(classify_tool("zsh").0, Category::Shell);
        assert_eq!(classify_tool("bash").0, Category::Shell);
        assert_eq!(classify_tool("fish").0, Category::Shell);
    }

    #[test]
    fn test_classify_browsers() {
        assert_eq!(classify_tool("firefox").0, Category::Browser);
        assert_eq!(classify_tool("chromium").0, Category::Browser);
    }

    #[test]
    fn test_classify_compositors() {
        assert_eq!(classify_tool("hyprland").0, Category::Compositor);
        assert_eq!(classify_tool("picom").0, Category::Compositor);
    }

    #[test]
    fn test_knowledge_object_usage() {
        let mut obj = KnowledgeObject::new("vim", Category::Editor);
        assert_eq!(obj.usage_count, 0);

        obj.record_usage(100, 1024);
        assert_eq!(obj.usage_count, 1);
        assert_eq!(obj.total_cpu_time_ms, 100);
        assert_eq!(obj.total_mem_bytes_peak, 1024);

        obj.record_usage(50, 512);
        assert_eq!(obj.usage_count, 2);
        assert_eq!(obj.total_cpu_time_ms, 150);
        assert_eq!(obj.total_mem_bytes_peak, 1024); // Peak unchanged
    }

    #[test]
    fn test_knowledge_store_operations() {
        let mut store = KnowledgeStore::new();
        assert_eq!(store.total_objects(), 0);

        let obj = KnowledgeObject::new("vim", Category::Editor);
        store.upsert(obj);
        assert_eq!(store.total_objects(), 1);

        let counts = store.count_by_category();
        assert_eq!(counts.get(&Category::Editor), Some(&1));
    }

    #[test]
    fn test_telemetry_aggregates() {
        let mut telem = TelemetryAggregates::new();
        assert_eq!(telem.processes_observed, 0);

        telem.record_process("vim");
        telem.record_process("vim");
        telem.record_process("zsh");

        assert_eq!(telem.processes_observed, 3);
        assert_eq!(telem.unique_commands, 2);
        assert_eq!(telem.command_counts.get("vim"), Some(&2));
    }

    #[test]
    fn test_config_paths() {
        let paths = get_config_paths("zsh");
        assert!(!paths.is_empty());
        assert!(paths.iter().any(|p| p.contains(".zshrc")));
    }
}
