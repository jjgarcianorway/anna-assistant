//! System information (v0.0.188).

use serde::{Deserialize, Serialize};

use super::constants::DESKTOP_PACKAGES;
use super::helpers::{check_tool_installed, run_command};

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
        let package_count = run_command("pacman -Qq").map(|out| out.lines().count() as u32);

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
        if check_tool_installed(pkg).is_some()
            && !desktops
                .iter()
                .any(|d| d.to_lowercase().contains(&pkg.to_lowercase()))
        {
            desktops.push(pkg.to_string());
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
        if lower.contains("vga")
            || lower.contains("3d controller")
            || lower.contains("display controller")
        {
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
