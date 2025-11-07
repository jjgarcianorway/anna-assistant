//! System Detection Module
//!
//! Intelligently detects hardware capabilities and existing tools
//! to enable adaptive configuration generation.
//!
//! Part of the Anna Hyprland System (RC.9.6+)
//!
//! Also used globally to make resource-aware recommendations:
//! "Potato computer, potato tools" 🥔

use std::process::Command;
use tracing::{info, warn};

/// Resource tier based on hardware capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ResourceTier {
    /// ≤8GB RAM, integrated GPU, ≤4 cores
    /// Target: Low resource usage, minimal animations, CLI-focused
    Efficient,

    /// 8-16GB RAM, discrete GPU, 4-8 cores
    /// Target: Balanced performance, smooth animations, mixed GUI/CLI
    Balanced,

    /// ≥16GB RAM, high-end GPU, ≥8 cores
    /// Target: Maximum visual quality, elaborate effects, full features
    Performance,
}

/// GPU vendor detection
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Unknown,
}

/// Complete system profile for intelligent configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemProfile {
    // Hardware
    pub ram_gb: u64,
    pub gpu_vendor: GpuVendor,
    pub gpu_model: Option<String>,
    pub cpu_cores: usize,
    pub resolution: Option<(u32, u32)>,

    // Existing tools (detected, user already has these)
    pub terminal: Option<String>,
    pub file_manager: Option<String>,
    pub launcher: Option<String>,
    pub text_editor: Option<String>,
    pub browser: Option<String>,

    // Derived tier and score
    pub tier: ResourceTier,
    pub score: u32,  // 0-100: capability score for filtering recommendations

    // Metadata
    pub detected_at: chrono::DateTime<chrono::Utc>,
}

impl SystemProfile {
    /// Detect system capabilities and existing tools
    pub fn detect() -> Self {
        info!("Detecting system capabilities...");

        let ram_gb = detect_ram_gb();
        let gpu_vendor = detect_gpu_vendor();
        let gpu_model = detect_gpu_model(&gpu_vendor);
        let cpu_cores = num_cpus::get();
        let resolution = detect_resolution();

        // Determine tier based on hardware
        let tier = determine_tier(ram_gb, &gpu_vendor, cpu_cores);

        info!(
            "Hardware: {}GB RAM, {:?} GPU, {} cores → {:?} tier",
            ram_gb, gpu_vendor, cpu_cores, tier
        );

        // Detect existing tools
        let terminal = detect_terminal();
        let file_manager = detect_file_manager();
        let launcher = detect_launcher();
        let text_editor = detect_text_editor();
        let browser = detect_browser();

        if let Some(ref term) = terminal {
            info!("Detected existing terminal: {}", term);
        }
        if let Some(ref fm) = file_manager {
            info!("Detected existing file manager: {}", fm);
        }

        // Calculate capability score (0-100)
        let score = calculate_score(ram_gb, &gpu_vendor, cpu_cores);

        info!("System score: {}/100 ({:?} tier)", score, tier);

        Self {
            ram_gb,
            gpu_vendor,
            gpu_model,
            cpu_cores,
            resolution,
            terminal,
            file_manager,
            launcher,
            text_editor,
            browser,
            tier,
            score,
            detected_at: chrono::Utc::now(),
        }
    }

    /// Load cached profile or detect fresh
    pub fn load_or_detect() -> Self {
        match Self::load_cached() {
            Ok(profile) => {
                info!("Loaded cached system profile (score: {})", profile.score);
                profile
            }
            Err(e) => {
                info!("No cached profile ({}), detecting fresh", e);
                let profile = Self::detect();
                let _ = profile.save_cache();
                profile
            }
        }
    }

    /// Load cached profile from disk
    fn load_cached() -> anyhow::Result<Self> {
        let cache_path = std::path::Path::new("/var/lib/anna/system_profile.json");
        let data = std::fs::read_to_string(cache_path)?;
        let profile: SystemProfile = serde_json::from_str(&data)?;
        Ok(profile)
    }

    /// Save profile to cache
    pub fn save_cache(&self) -> anyhow::Result<()> {
        let cache_dir = std::path::Path::new("/var/lib/anna");
        std::fs::create_dir_all(cache_dir)?;

        let cache_path = cache_dir.join("system_profile.json");
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(&cache_path, data)?;

        info!("Saved system profile to {}", cache_path.display());
        Ok(())
    }

    /// Check if profile should be refreshed (hardware might have changed)
    pub fn should_refresh(&self) -> bool {
        let age = chrono::Utc::now() - self.detected_at;

        // Refresh if older than 7 days OR if detection seems stale
        age.num_days() > 7
    }

    /// Get appropriate status bar for this tier
    pub fn recommended_bar(&self) -> &str {
        match self.tier {
            ResourceTier::Efficient => "yambar",
            ResourceTier::Balanced => "waybar",
            ResourceTier::Performance => "hyprpanel-git",
        }
    }

    /// Get appropriate launcher for this tier
    pub fn recommended_launcher(&self) -> &str {
        // Prefer existing if available
        if let Some(ref launcher) = self.launcher {
            return launcher;
        }

        match self.tier {
            ResourceTier::Efficient => "fuzzel",
            ResourceTier::Balanced => "wofi",
            ResourceTier::Performance => "rofi-wayland",
        }
    }

    /// Get terminal to use (prefer existing)
    pub fn get_terminal(&self) -> &str {
        self.terminal.as_deref().unwrap_or("kitty")
    }

    /// Get file manager to use (prefer existing)
    pub fn get_file_manager(&self) -> &str {
        self.file_manager.as_deref().unwrap_or("thunar")
    }

    /// Check if NVIDIA GPU detected
    pub fn has_nvidia(&self) -> bool {
        self.gpu_vendor == GpuVendor::Nvidia
    }
}

/// Detect total RAM in gigabytes
fn detect_ram_gb() -> u64 {
    // Read from /proc/meminfo
    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(kb_str) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = kb_str.parse::<u64>() {
                        return kb / 1024 / 1024; // Convert KB to GB
                    }
                }
            }
        }
    }

    warn!("Failed to detect RAM, assuming 8GB");
    8
}

/// Detect GPU vendor
fn detect_gpu_vendor() -> GpuVendor {
    // Check lspci for GPU info
    if let Ok(output) = Command::new("lspci").output() {
        let output_str = String::from_utf8_lossy(&output.stdout).to_lowercase();

        if output_str.contains("nvidia") {
            return GpuVendor::Nvidia;
        }
        if output_str.contains("amd") || output_str.contains("radeon") {
            return GpuVendor::Amd;
        }
        if output_str.contains("intel") {
            return GpuVendor::Intel;
        }
    }

    GpuVendor::Unknown
}

/// Detect GPU model name
fn detect_gpu_model(vendor: &GpuVendor) -> Option<String> {
    if let Ok(output) = Command::new("lspci").output() {
        let output_str = String::from_utf8_lossy(&output.stdout);

        for line in output_str.lines() {
            let lower = line.to_lowercase();
            match vendor {
                GpuVendor::Nvidia if lower.contains("nvidia") && lower.contains("vga") => {
                    // Extract model name after the colon
                    if let Some(model) = line.split(':').nth(2) {
                        return Some(model.trim().to_string());
                    }
                }
                GpuVendor::Amd if (lower.contains("amd") || lower.contains("radeon")) && lower.contains("vga") => {
                    if let Some(model) = line.split(':').nth(2) {
                        return Some(model.trim().to_string());
                    }
                }
                GpuVendor::Intel if lower.contains("intel") && lower.contains("vga") => {
                    if let Some(model) = line.split(':').nth(2) {
                        return Some(model.trim().to_string());
                    }
                }
                _ => {}
            }
        }
    }
    None
}

/// Detect screen resolution (first monitor)
fn detect_resolution() -> Option<(u32, u32)> {
    // Try wayland first (wlr-randr for wlroots compositors)
    if let Ok(output) = Command::new("wlr-randr").output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.contains("current") {
                // Parse format like "1920x1080 @ 60.000000 Hz"
                if let Some(res_part) = line.split_whitespace().next() {
                    if let Some((w, h)) = res_part.split_once('x') {
                        if let (Ok(width), Ok(height)) = (w.parse(), h.parse()) {
                            return Some((width, height));
                        }
                    }
                }
            }
        }
    }

    // Fallback: try xrandr for X11
    if let Ok(output) = Command::new("xrandr").args(["--query"]).output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.contains("*") {
                // Current resolution has asterisk
                if let Some(res_part) = line.split_whitespace().next() {
                    if let Some((w, h)) = res_part.split_once('x') {
                        if let (Ok(width), Ok(height)) = (w.parse(), h.parse()) {
                            return Some((width, height));
                        }
                    }
                }
            }
        }
    }

    None
}

/// Determine resource tier based on hardware
fn determine_tier(ram_gb: u64, gpu: &GpuVendor, cpu_cores: usize) -> ResourceTier {
    // Performance tier: ≥16GB RAM AND (dedicated GPU OR ≥8 cores)
    if ram_gb >= 16 && (matches!(gpu, GpuVendor::Nvidia | GpuVendor::Amd) || cpu_cores >= 8) {
        return ResourceTier::Performance;
    }

    // Efficient tier: ≤8GB RAM OR (Intel GPU AND ≤4 cores)
    if ram_gb <= 8 || (matches!(gpu, GpuVendor::Intel | GpuVendor::Unknown) && cpu_cores <= 4) {
        return ResourceTier::Efficient;
    }

    // Everything else: Balanced
    ResourceTier::Balanced
}

/// Detect installed terminal emulator (preference order)
fn detect_terminal() -> Option<String> {
    let terminals = vec![
        "kitty",
        "foot",
        "alacritty",
        "wezterm",
        "termite",
        "xterm",
    ];

    for term in terminals {
        if is_installed(term) {
            return Some(term.to_string());
        }
    }
    None
}

/// Detect installed file manager
fn detect_file_manager() -> Option<String> {
    let file_managers = vec![
        "thunar",
        "pcmanfm",
        "nautilus",
        "dolphin",
        "nemo",
    ];

    for fm in file_managers {
        if is_installed(fm) {
            return Some(fm.to_string());
        }
    }
    None
}

/// Detect installed launcher
fn detect_launcher() -> Option<String> {
    let launchers = vec![
        "rofi",
        "wofi",
        "fuzzel",
        "tofi",
        "bemenu",
    ];

    for launcher in launchers {
        if is_installed(launcher) {
            return Some(launcher.to_string());
        }
    }
    None
}

/// Detect text editor
fn detect_text_editor() -> Option<String> {
    let editors = vec![
        "nvim",
        "vim",
        "nano",
        "emacs",
        "code",
        "gedit",
    ];

    for editor in editors {
        if is_installed(editor) {
            return Some(editor.to_string());
        }
    }
    None
}

/// Detect browser
fn detect_browser() -> Option<String> {
    let browsers = vec![
        "firefox",
        "chromium",
        "brave",
        "google-chrome",
        "librewolf",
    ];

    for browser in browsers {
        if is_installed(browser) {
            return Some(browser.to_string());
        }
    }
    None
}

/// Check if a command is installed and in PATH
fn is_installed(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Calculate system capability score (0-100)
///
/// Score determines which recommendations are appropriate:
/// - 0-30: Efficient tier (potato systems) - only essential, lightweight recommendations
/// - 31-65: Balanced tier (mid-range) - moderate recommendations
/// - 66-100: Performance tier (high-end) - all recommendations including resource-intensive
///
/// Scoring formula:
/// - RAM: 0-40 points (0GB=0, 4GB=10, 8GB=20, 16GB=30, 32GB=40, 64GB+=40)
/// - GPU: 0-40 points (None/Intel=0, AMD=20, NVIDIA=30, High-end=40)
/// - CPU: 0-20 points (1 core=2, 2=5, 4=10, 8=15, 16+=20)
fn calculate_score(ram_gb: u64, gpu: &GpuVendor, cpu_cores: usize) -> u32 {
    let mut score: u32 = 0;

    // RAM score (0-40 points)
    score += match ram_gb {
        0..=3 => 0,
        4..=7 => 10,
        8..=15 => 20,
        16..=31 => 30,
        32..=63 => 35,
        _ => 40,
    };

    // GPU score (0-40 points)
    score += match gpu {
        GpuVendor::Unknown => 0,
        GpuVendor::Intel => 5,    // Integrated GPU
        GpuVendor::Amd => 25,      // Discrete AMD
        GpuVendor::Nvidia => 35,   // Discrete NVIDIA (assume mid-high)
    };

    // CPU score (0-20 points)
    score += match cpu_cores {
        0..=1 => 2,
        2..=3 => 5,
        4..=7 => 10,
        8..=15 => 15,
        _ => 20,
    };

    // Cap at 100
    score.min(100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoring() {
        // Potato laptop: 4GB RAM, Intel iGPU, 2 cores
        assert_eq!(calculate_score(4, &GpuVendor::Intel, 2), 20); // Efficient

        // Mid-range: 16GB RAM, AMD GPU, 6 cores
        assert_eq!(calculate_score(16, &GpuVendor::Amd, 6), 65); // Balanced

        // High-end: 32GB RAM, NVIDIA, 12 cores
        assert_eq!(calculate_score(32, &GpuVendor::Nvidia, 12), 85); // Performance
    }

    #[test]
    fn test_tier_boundaries() {
        // Score 0-30 = Efficient
        assert!(calculate_score(4, &GpuVendor::Intel, 2) <= 30);

        // Score 31-65 = Balanced
        let mid_score = calculate_score(12, &GpuVendor::Amd, 6);
        assert!(mid_score > 30 && mid_score <= 65);

        // Score 66-100 = Performance
        assert!(calculate_score(32, &GpuVendor::Nvidia, 12) > 65);
    }
}
