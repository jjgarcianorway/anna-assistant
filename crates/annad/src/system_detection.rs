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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Unknown,
}

/// Complete system profile for intelligent configuration
#[derive(Debug, Clone)]
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

    // Derived tier
    pub tier: ResourceTier,
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
        }
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
