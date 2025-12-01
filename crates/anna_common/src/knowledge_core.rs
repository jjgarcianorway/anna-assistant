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
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// Constants
// ============================================================================

/// Knowledge store path
pub const KNOWLEDGE_STORE_PATH: &str = "/var/lib/anna/knowledge/knowledge_v5.json";

/// Telemetry store path
pub const TELEMETRY_STORE_PATH: &str = "/var/lib/anna/knowledge/telemetry_v5.json";

/// Inventory progress state path (v5.4.0)
pub const INVENTORY_PROGRESS_PATH: &str = "/var/lib/anna/knowledge/inventory_progress.json";

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

/// v5.1.0: Object type classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ObjectType {
    Command,
    Package,
    Service,
}

impl ObjectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ObjectType::Command => "command",
            ObjectType::Package => "package",
            ObjectType::Service => "service",
        }
    }
}

/// A single piece of knowledge about a tool/package/service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeObject {
    /// Canonical name (e.g., "vim", "nano", "hyprland")
    pub name: String,

    /// Category classification
    pub category: Category,

    /// v5.1.0: Object types (can be multiple: command + package + service)
    #[serde(default)]
    pub object_types: Vec<ObjectType>,

    /// How it was detected
    pub detected_as: DetectionSource,

    /// Is it currently installed?
    pub installed: bool,

    /// Package name (if installed via package manager)
    pub package_name: Option<String>,

    /// v5.1.0: Package version
    pub package_version: Option<String>,

    /// v5.1.0: Package install time (Unix seconds)
    pub installed_at: Option<u64>,

    /// v5.1.0: Package removal time (Unix seconds)
    pub removed_at: Option<u64>,

    /// v5.1.0: All executable paths for this object
    #[serde(default)]
    pub paths: Vec<String>,

    /// Binary path (legacy - first path, kept for compatibility)
    pub binary_path: Option<String>,

    /// v5.1.0: Systemd service unit name (if applicable)
    pub service_unit: Option<String>,

    /// v5.1.0: Service enabled status
    pub service_enabled: Option<bool>,

    /// v5.1.0: Service active status
    pub service_active: Option<bool>,

    /// v5.1.0: Inventory source (path_scan, pacman_db, systemd)
    #[serde(default)]
    pub inventory_source: Vec<String>,

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

    /// v5.1.0: When configs were discovered
    pub config_discovered_at: Option<u64>,

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
            object_types: vec![],
            detected_as: DetectionSource::Unknown,
            installed: false,
            package_name: None,
            package_version: None,
            installed_at: None,
            removed_at: None,
            paths: vec![],
            binary_path: None,
            service_unit: None,
            service_enabled: None,
            service_active: None,
            inventory_source: vec![],
            usage_count: 0,
            total_cpu_time_ms: 0,
            total_mem_bytes_peak: 0,
            first_seen_at: now,
            last_seen_at: now,
            config_paths: vec![],
            config_discovered_at: None,
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

    /// Save to disk using atomic write
    /// v5.5.2: Uses atomic write (temp file + rename) to prevent corruption
    pub fn save(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        crate::atomic_write::atomic_write(KNOWLEDGE_STORE_PATH, &json)
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

    /// v5.1.0: Count objects by ObjectType
    /// v5.5.1: Only count INSTALLED packages (not removed ones)
    pub fn count_by_type(&self) -> (usize, usize, usize) {
        let mut commands = 0;
        let mut packages = 0;
        let mut services = 0;

        for obj in self.objects.values() {
            // Commands: must exist on PATH (installed = true for binaries)
            if obj.object_types.contains(&ObjectType::Command) && obj.installed {
                commands += 1;
            }
            // Packages: must be currently installed
            if obj.object_types.contains(&ObjectType::Package) && obj.installed {
                packages += 1;
            }
            // Services: count all (services exist as unit files)
            if obj.object_types.contains(&ObjectType::Service) {
                services += 1;
            }
        }
        (commands, packages, services)
    }

    /// v5.1.0: Count objects with runs observed
    pub fn count_with_usage(&self) -> usize {
        self.objects.values().filter(|o| o.usage_count > 0).count()
    }

    /// v5.1.0: Get services
    pub fn get_services(&self) -> Vec<&KnowledgeObject> {
        self.objects
            .values()
            .filter(|o| o.object_types.contains(&ObjectType::Service))
            .collect()
    }
}

// ============================================================================
// Inventory Progress (v5.1.0)
// ============================================================================

/// v5.1.0: Inventory scan phase
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InventoryPhase {
    Idle,
    ScanningPath,
    ScanningPackages,
    ScanningServices,
    /// v5.1.1: Priority scan for user-requested object
    PriorityScan,
    Complete,
}

/// v5.1.1: Inventory job priority
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InventoryPriority {
    /// User-requested object discovery/refresh
    High,
    /// Background full inventory scan
    Low,
}

impl InventoryPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            InventoryPhase::Idle => "idle",
            InventoryPhase::ScanningPath => "scanning PATH",
            InventoryPhase::ScanningPackages => "scanning packages",
            InventoryPhase::ScanningServices => "scanning services",
            InventoryPhase::PriorityScan => "priority_scan",
            InventoryPhase::Complete => "complete",
        }
    }
}

impl InventoryPriority {
    pub fn is_high(&self) -> bool {
        matches!(self, InventoryPriority::High)
    }
}

/// v5.1.1: Checkpoint for resuming full inventory scan
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanCheckpoint {
    /// Phase we were in
    pub phase: Option<InventoryPhase>,
    /// Index/offset where we stopped
    pub offset: usize,
    /// Timestamp when paused
    pub paused_at: u64,
}

/// v5.1.0 + v5.1.1: Inventory progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryProgress {
    /// Current phase
    pub phase: InventoryPhase,
    /// Percent complete (0-100)
    pub percent: u8,
    /// Items processed in current phase
    pub items_processed: usize,
    /// Total items in current phase
    pub items_total: usize,
    /// Estimated time remaining (seconds)
    pub eta_secs: Option<u64>,
    /// Scan started at (Unix seconds)
    pub started_at: Option<u64>,
    /// Last update timestamp
    pub last_update: u64,
    /// Is initial scan complete?
    pub initial_scan_complete: bool,
    /// v5.1.1: Current priority scan target (if any)
    #[serde(default)]
    pub priority_target: Option<String>,
    /// v5.1.1: Checkpoint for resuming full scan
    #[serde(default)]
    pub scan_checkpoint: Option<ScanCheckpoint>,
}

impl Default for InventoryProgress {
    fn default() -> Self {
        Self::new()
    }
}

impl InventoryProgress {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            phase: InventoryPhase::Idle,
            percent: 0,
            items_processed: 0,
            items_total: 0,
            eta_secs: None,
            started_at: None,
            last_update: now,
            initial_scan_complete: false,
            priority_target: None,
            scan_checkpoint: None,
        }
    }

    /// Start a new scan phase
    pub fn start_phase(&mut self, phase: InventoryPhase, total_items: usize) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if self.started_at.is_none() {
            self.started_at = Some(now);
        }
        self.phase = phase;
        self.items_total = total_items;
        self.items_processed = 0;
        self.last_update = now;
        self.update_percent();
    }

    /// v5.1.1: Start a priority scan for a specific object
    pub fn start_priority_scan(&mut self, target: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Save checkpoint if we're in the middle of a scan
        if self.phase != InventoryPhase::Idle && self.phase != InventoryPhase::Complete {
            self.scan_checkpoint = Some(ScanCheckpoint {
                phase: Some(self.phase.clone()),
                offset: self.items_processed,
                paused_at: now,
            });
        }

        self.phase = InventoryPhase::PriorityScan;
        self.priority_target = Some(target.to_string());
        self.last_update = now;
        self.eta_secs = Some(1); // Priority scans are fast
    }

    /// v5.1.1: End priority scan and resume normal operation
    pub fn end_priority_scan(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.priority_target = None;

        // Resume from checkpoint if available
        if let Some(checkpoint) = self.scan_checkpoint.take() {
            if let Some(phase) = checkpoint.phase {
                self.phase = phase;
                self.items_processed = checkpoint.offset;
            } else {
                self.phase = InventoryPhase::Idle;
            }
        } else if self.initial_scan_complete {
            self.phase = InventoryPhase::Complete;
        } else {
            self.phase = InventoryPhase::Idle;
        }

        self.last_update = now;
    }

    /// Update progress
    pub fn update(&mut self, processed: usize) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.items_processed = processed;
        self.last_update = now;
        self.update_percent();

        // Calculate ETA based on progress
        if let Some(started) = self.started_at {
            if self.percent > 0 && self.percent < 100 {
                let elapsed = now.saturating_sub(started);
                let estimated_total = (elapsed as f64 * 100.0) / self.percent as f64;
                self.eta_secs = Some((estimated_total - elapsed as f64) as u64);
            }
        }
    }

    /// Mark scan complete
    pub fn complete(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.phase = InventoryPhase::Complete;
        self.percent = 100;
        self.eta_secs = None;
        self.last_update = now;
        self.initial_scan_complete = true;
        self.priority_target = None;
        self.scan_checkpoint = None;
    }

    fn update_percent(&mut self) {
        if self.items_total > 0 {
            self.percent = ((self.items_processed as f64 / self.items_total as f64) * 100.0) as u8;
        }
    }

    /// Format progress for display
    pub fn format_status(&self) -> String {
        if let Some(target) = &self.priority_target {
            format!("priority_scan ({})", target)
        } else if self.initial_scan_complete {
            "Complete".to_string()
        } else if self.phase == InventoryPhase::Idle {
            "Waiting...".to_string()
        } else {
            let eta = self.eta_secs
                .map(|s| format!(" (ETA: {}s)", s))
                .unwrap_or_default();
            format!("{} {}%{}", self.phase.as_str(), self.percent, eta)
        }
    }

    /// v5.1.1: Check if in priority scan mode
    pub fn is_priority_scan(&self) -> bool {
        self.phase == InventoryPhase::PriorityScan && self.priority_target.is_some()
    }

    /// v5.4.0: Load progress from disk
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string(INVENTORY_PROGRESS_PATH) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::new()
        }
    }

    /// v5.4.0: Save progress to disk
    /// v5.5.2: Uses atomic write (temp file + rename) to prevent corruption
    pub fn save(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        crate::atomic_write::atomic_write(INVENTORY_PROGRESS_PATH, &json)
    }

    /// v5.4.0: Format ETA for human display
    pub fn format_eta(&self) -> String {
        match self.eta_secs {
            Some(0) => "< 1s".to_string(),
            Some(s) if s < 60 => format!("{}s", s),
            Some(s) if s < 3600 => format!("{}m {}s", s / 60, s % 60),
            Some(s) => format!("{}h {}m", s / 3600, (s % 3600) / 60),
            None => "unknown".to_string(),
        }
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

    /// Save to disk using atomic write
    /// v5.5.2: Uses atomic write (temp file + rename) to prevent corruption
    pub fn save(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        crate::atomic_write::atomic_write(TELEMETRY_STORE_PATH, &json)
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

    // v5.4.1: Expanded tool categories

    // Development tools
    if matches!(lower.as_str(),
        "git" | "cargo" | "rustc" | "rustup" |
        "python" | "python3" | "pip" | "pip3" |
        "node" | "nodejs" | "npm" | "npx" | "yarn" | "pnpm" |
        "go" | "gcc" | "g++" | "clang" | "clang++" |
        "make" | "cmake" | "meson" | "ninja" |
        "docker" | "podman" | "kubectl" |
        "gdb" | "lldb" | "valgrind" | "strace" | "ltrace"
    ) {
        return (Category::Tool, None);
    }

    // Core utilities (coreutils + common Unix tools)
    if matches!(lower.as_str(),
        "ls" | "cat" | "cp" | "mv" | "rm" | "mkdir" | "rmdir" |
        "chmod" | "chown" | "chgrp" | "touch" | "ln" |
        "head" | "tail" | "less" | "more" |
        "grep" | "egrep" | "fgrep" | "rg" | "ripgrep" |
        "find" | "fd" | "locate" | "which" | "whereis" |
        "sed" | "awk" | "cut" | "sort" | "uniq" | "wc" | "tr" |
        "tar" | "gzip" | "gunzip" | "zip" | "unzip" | "xz" | "zstd" |
        "curl" | "wget" | "ssh" | "scp" | "rsync" |
        "ps" | "top" | "htop" | "btop" | "kill" | "pkill" |
        "df" | "du" | "free" | "lsblk" | "mount" | "umount" |
        "date" | "cal" | "uptime" | "hostname" |
        "echo" | "printf" | "tee" | "xargs" |
        "diff" | "patch" | "md5sum" | "sha256sum" |
        "env" | "export" | "source" | "alias" |
        "man" | "info" | "help" | "whatis" | "apropos"
    ) {
        return (Category::Tool, None);
    }

    // Modern CLI replacements
    if matches!(lower.as_str(),
        "eza" | "exa" | "lsd" |         // ls replacements
        "bat" | "batcat" |               // cat replacement
        "fzf" | "sk" |                   // fuzzy finders
        "jq" | "yq" |                    // JSON/YAML tools
        "delta" |                         // diff replacement
        "duf" |                           // df replacement
        "dust" |                          // du replacement
        "procs" |                         // ps replacement
        "zoxide" | "z" |                  // cd replacement
        "starship" |                      // prompt
        "tldr" |                          // man replacement
        "neofetch" | "fastfetch" | "pfetch"  // system info
    ) {
        return (Category::Tool, None);
    }

    // Image/media viewers
    if matches!(lower.as_str(),
        "swayimg" | "imv" | "feh" | "sxiv" | "nsxiv" |
        "mpv" | "vlc" | "ffmpeg" | "ffplay" |
        "fotoxx" | "gimp" | "inkscape"
    ) {
        return (Category::Tool, None);
    }

    // System services
    if matches!(lower.as_str(),
        "systemd" | "systemctl" | "journalctl" |
        "sshd" | "openssh" | "NetworkManager" |
        "pipewire" | "wireplumber" | "pulseaudio" |
        "bluetooth" | "bluez" | "cups" |
        "ollama" | "docker" | "containerd"
    ) {
        return (Category::Service, None);
    }

    // File managers
    if matches!(lower.as_str(),
        "ranger" | "lf" | "nnn" | "vifm" |
        "nautilus" | "dolphin" | "thunar" | "pcmanfm"
    ) {
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
