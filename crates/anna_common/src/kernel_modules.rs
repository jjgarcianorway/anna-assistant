//! Kernel and Boot System Detection
//!
//! Detects kernel versions, modules, boot configuration, and boot health.
//!
//! ## Detection Capabilities
//!
//! ### Kernel Detection
//! - Installed kernels (LTS vs mainline)
//! - Currently running kernel version
//! - Available kernel versions in /boot
//! - Kernel package information
//!
//! ### Kernel Modules
//! - Currently loaded modules
//! - Module dependencies
//! - Broken/failing modules
//! - DKMS module status
//! - Module load errors from journal
//!
//! ### Boot Configuration
//! - Boot entries (systemd-boot, GRUB)
//! - Boot entry validation
//! - Bootloader health
//! - Boot errors and warnings from journal
//! - Failed boot attempts
//!
//! ## Example
//!
//! ```rust
//! use anna_common::kernel_modules::KernelModules;
//!
//! let kernel_info = KernelModules::detect();
//! println!("Running kernel: {}", kernel_info.current_kernel);
//! println!("Installed kernels: {}", kernel_info.installed_kernels.len());
//! println!("Loaded modules: {}", kernel_info.loaded_modules.len());
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::process::Command;

/// Complete kernel and boot system information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelModules {
    /// Currently running kernel version
    pub current_kernel: String,

    /// All installed kernels on the system
    pub installed_kernels: Vec<InstalledKernel>,

    /// Currently loaded kernel modules
    pub loaded_modules: Vec<LoadedModule>,

    /// Broken or failing kernel modules
    pub broken_modules: Vec<BrokenModule>,

    /// DKMS module status
    pub dkms_status: DkmsStatus,

    /// Boot entries configuration
    pub boot_entries: Vec<BootEntry>,

    /// Boot health and errors
    pub boot_health: BootHealth,

    /// Module loading errors from journal
    pub module_errors: Vec<ModuleError>,
}

/// Information about an installed kernel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledKernel {
    /// Kernel version string
    pub version: String,

    /// Kernel type (mainline, lts, zen, hardened, etc.)
    pub kernel_type: KernelType,

    /// Package name
    pub package_name: Option<String>,

    /// Whether this kernel is currently running
    pub is_current: bool,

    /// Kernel image path in /boot
    pub image_path: Option<String>,

    /// Initramfs path in /boot
    pub initramfs_path: Option<String>,

    /// Whether all required files exist
    pub is_complete: bool,
}

/// Type of kernel
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KernelType {
    /// Standard mainline kernel
    Mainline,

    /// Long-term support kernel
    Lts,

    /// Zen kernel (optimized for desktop)
    Zen,

    /// Hardened kernel (security-focused)
    Hardened,

    /// Custom or unknown kernel
    Custom(String),
}

/// A currently loaded kernel module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadedModule {
    /// Module name
    pub name: String,

    /// Module size in bytes
    pub size: u64,

    /// Number of instances using this module
    pub used_by_count: u32,

    /// List of modules using this module
    pub used_by: Vec<String>,

    /// Module state
    pub state: ModuleState,
}

/// Kernel module state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleState {
    /// Module is loaded and functioning
    Live,

    /// Module is being loaded
    Loading,

    /// Module is being unloaded
    Unloading,

    /// Unknown state
    Unknown,
}

/// A broken or failing kernel module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokenModule {
    /// Module name
    pub name: String,

    /// Error description
    pub error: String,

    /// Error source
    pub source: ErrorSource,
}

/// Source of module error
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSource {
    /// Error from dmesg
    Dmesg,

    /// Error from systemd journal
    Journal,

    /// Missing dependency
    MissingDependency,

    /// Failed to load
    LoadFailure,
}

/// DKMS (Dynamic Kernel Module Support) status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DkmsStatus {
    /// Whether DKMS is installed
    pub dkms_installed: bool,

    /// DKMS modules
    pub modules: Vec<DkmsModule>,

    /// Failed DKMS builds
    pub failed_builds: Vec<DkmsFailure>,
}

/// A DKMS module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DkmsModule {
    /// Module name
    pub name: String,

    /// Module version
    pub version: String,

    /// Kernel versions it's built for
    pub kernel_versions: Vec<String>,

    /// Build status
    pub status: DkmsBuildStatus,
}

/// DKMS build status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DkmsBuildStatus {
    /// Built and installed
    Installed,

    /// Built but not installed
    Built,

    /// Failed to build
    Failed,

    /// Not built yet
    NotBuilt,
}

/// A failed DKMS build
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DkmsFailure {
    /// Module name
    pub module_name: String,

    /// Module version
    pub version: String,

    /// Kernel version it failed for
    pub kernel_version: String,

    /// Error description
    pub error: String,
}

/// A boot entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootEntry {
    /// Entry ID or title
    pub id: String,

    /// Entry title/description
    pub title: String,

    /// Bootloader type
    pub bootloader: BootloaderType,

    /// Kernel image path
    pub kernel_path: Option<String>,

    /// Initramfs path
    pub initramfs_path: Option<String>,

    /// Whether entry appears valid
    pub is_valid: bool,

    /// Validation errors
    pub validation_errors: Vec<String>,
}

/// Type of bootloader
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BootloaderType {
    /// systemd-boot
    SystemdBoot,

    /// GRUB
    Grub,

    /// rEFInd
    Refind,

    /// Other or unknown
    Other(String),
}

/// Boot health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootHealth {
    /// Last boot timestamp
    pub last_boot_time: Option<String>,

    /// Boot errors from journal
    pub boot_errors: Vec<BootError>,

    /// Boot warnings from journal
    pub boot_warnings: Vec<String>,

    /// Failed boot attempts (from journal)
    pub failed_boots: u32,

    /// Boot duration in seconds
    pub boot_duration_seconds: Option<f64>,
}

/// A boot error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootError {
    /// Error message
    pub message: String,

    /// Error severity
    pub severity: ErrorSeverity,

    /// Timestamp
    pub timestamp: Option<String>,

    /// Unit or service that failed
    pub unit: Option<String>,
}

/// Error severity level
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Critical,
    Error,
    Warning,
}

/// A module loading error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleError {
    /// Module name
    pub module_name: String,

    /// Error message
    pub error: String,

    /// Timestamp
    pub timestamp: Option<String>,
}

impl KernelModules {
    /// Detect all kernel and boot information
    pub fn detect() -> Self {
        let current_kernel = get_current_kernel();
        let installed_kernels = detect_installed_kernels(&current_kernel);
        let loaded_modules = get_loaded_modules();
        let broken_modules = detect_broken_modules();
        let dkms_status = detect_dkms_status();
        let boot_entries = detect_boot_entries();
        let boot_health = detect_boot_health();
        let module_errors = get_module_errors();

        Self {
            current_kernel,
            installed_kernels,
            loaded_modules,
            broken_modules,
            dkms_status,
            boot_entries,
            boot_health,
            module_errors,
        }
    }
}

/// Get current running kernel version
fn get_current_kernel() -> String {
    match Command::new("uname").arg("-r").output() {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => "unknown".to_string(),
    }
}

/// Detect all installed kernels
fn detect_installed_kernels(current_kernel: &str) -> Vec<InstalledKernel> {
    let mut kernels = Vec::new();

    // Method 1: Check /boot for kernel images
    if let Ok(entries) = fs::read_dir("/boot") {
        let mut kernel_versions = HashSet::new();

        for entry in entries.flatten() {
            let filename = entry.file_name();
            let filename_str = filename.to_string_lossy();

            // Match vmlinuz-* files
            if filename_str.starts_with("vmlinuz-") {
                let version = filename_str.strip_prefix("vmlinuz-").unwrap().to_string();
                kernel_versions.insert(version);
            }
        }

        for version in kernel_versions {
            let kernel_type = classify_kernel_type(&version);
            let is_current = version == current_kernel;

            let image_path = format!("/boot/vmlinuz-{}", version);
            let initramfs_path = find_initramfs(&version);

            let is_complete = fs::metadata(&image_path).is_ok() && initramfs_path.is_some();

            kernels.push(InstalledKernel {
                version,
                kernel_type,
                package_name: None,
                is_current,
                image_path: Some(image_path),
                initramfs_path,
                is_complete,
            });
        }
    }

    // Method 2: Check pacman for installed kernel packages
    if let Ok(output) = Command::new("pacman").args(["-Q"]).output() {
        if output.status.success() {
            let packages = String::from_utf8_lossy(&output.stdout);
            for line in packages.lines() {
                if let Some((pkg_name, version)) = line.split_once(' ') {
                    if is_kernel_package(pkg_name) {
                        // Try to find matching kernel in existing list
                        let found = kernels
                            .iter_mut()
                            .find(|k| k.version.contains(&extract_version_from_package(version)));

                        if let Some(kernel) = found {
                            kernel.package_name = Some(pkg_name.to_string());
                        }
                    }
                }
            }
        }
    }

    kernels
}

/// Classify kernel type based on version string
fn classify_kernel_type(version: &str) -> KernelType {
    if version.contains("-lts") {
        KernelType::Lts
    } else if version.contains("-zen") {
        KernelType::Zen
    } else if version.contains("-hardened") {
        KernelType::Hardened
    } else if version.contains("-ARCH") || !version.contains('-') {
        KernelType::Mainline
    } else {
        KernelType::Custom(version.to_string())
    }
}

/// Find initramfs for a kernel version
fn find_initramfs(kernel_version: &str) -> Option<String> {
    // Try common initramfs naming patterns
    let patterns = vec![
        format!("/boot/initramfs-{}.img", kernel_version),
        format!("/boot/initramfs-{}-fallback.img", kernel_version),
        format!("/boot/initrd.img-{}", kernel_version),
    ];

    for pattern in patterns {
        if fs::metadata(&pattern).is_ok() {
            return Some(pattern);
        }
    }

    None
}

/// Check if a package is a kernel package
fn is_kernel_package(pkg_name: &str) -> bool {
    matches!(
        pkg_name,
        "linux" | "linux-lts" | "linux-zen" | "linux-hardened" | "linux-mainline"
    )
}

/// Extract version number from package version string
fn extract_version_from_package(pkg_version: &str) -> String {
    // Package version format: "6.6.8.arch1-1" -> extract "6.6.8"
    pkg_version.split('-').next().unwrap_or("").to_string()
}

/// Get currently loaded kernel modules
fn get_loaded_modules() -> Vec<LoadedModule> {
    let mut modules = Vec::new();

    // Parse /proc/modules
    if let Ok(content) = fs::read_to_string("/proc/modules") {
        for line in content.lines() {
            if let Some(module) = parse_proc_module_line(line) {
                modules.push(module);
            }
        }
    }

    modules
}

/// Parse a line from /proc/modules
fn parse_proc_module_line(line: &str) -> Option<LoadedModule> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 4 {
        return None;
    }

    let name = parts[0].to_string();
    let size = parts[1].parse().unwrap_or(0);
    let used_by_count = parts[2].parse().unwrap_or(0);

    let used_by = if parts[3] == "-" {
        Vec::new()
    } else {
        parts[3].split(',').map(|s| s.to_string()).collect()
    };

    let state = if parts.len() > 4 {
        match parts[4] {
            "Live" => ModuleState::Live,
            "Loading" => ModuleState::Loading,
            "Unloading" => ModuleState::Unloading,
            _ => ModuleState::Unknown,
        }
    } else {
        ModuleState::Live
    };

    Some(LoadedModule {
        name,
        size,
        used_by_count,
        used_by,
        state,
    })
}

/// Detect broken kernel modules
fn detect_broken_modules() -> Vec<BrokenModule> {
    let mut broken = Vec::new();

    // Check dmesg for module loading errors
    if let Ok(output) = Command::new("dmesg").output() {
        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout);
            for line in content.lines() {
                if line.contains("module") && (line.contains("error") || line.contains("failed")) {
                    if let Some(module_name) = extract_module_from_dmesg(line) {
                        broken.push(BrokenModule {
                            name: module_name,
                            error: line.to_string(),
                            source: ErrorSource::Dmesg,
                        });
                    }
                }
            }
        }
    }

    // Check journal for module errors
    if let Ok(output) = Command::new("journalctl")
        .args(["-b", "-p", "err", "--no-pager", "-n", "1000"])
        .output()
    {
        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout);
            for line in content.lines() {
                if line.contains("modprobe") || line.contains("insmod") {
                    if let Some(module_name) = extract_module_from_journal(line) {
                        broken.push(BrokenModule {
                            name: module_name,
                            error: line.to_string(),
                            source: ErrorSource::Journal,
                        });
                    }
                }
            }
        }
    }

    broken
}

/// Extract module name from dmesg line
fn extract_module_from_dmesg(line: &str) -> Option<String> {
    // Very basic extraction - look for common patterns
    if let Some(start) = line.find("module ") {
        let rest = &line[start + 7..];
        if let Some(end) = rest.find(' ') {
            return Some(rest[..end].to_string());
        }
    }
    None
}

/// Extract module name from journal line
fn extract_module_from_journal(line: &str) -> Option<String> {
    // Look for modprobe <module_name>
    if let Some(idx) = line.find("modprobe ") {
        let rest = &line[idx + 9..];
        if let Some(end) = rest.find(' ') {
            return Some(rest[..end].to_string());
        } else {
            return Some(rest.to_string());
        }
    }
    None
}

/// Detect DKMS status
fn detect_dkms_status() -> DkmsStatus {
    // Check if DKMS is installed
    let dkms_installed = Command::new("which")
        .arg("dkms")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !dkms_installed {
        return DkmsStatus {
            dkms_installed: false,
            modules: Vec::new(),
            failed_builds: Vec::new(),
        };
    }

    let mut modules = Vec::new();
    let mut failed_builds = Vec::new();

    // Get DKMS status
    if let Ok(output) = Command::new("dkms").arg("status").output() {
        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout);
            for line in content.lines() {
                if let Some(module) = parse_dkms_status_line(line) {
                    modules.push(module);
                }
            }
        }
    }

    // Check for failed builds in journal
    if let Ok(output) = Command::new("journalctl")
        .args(["-b", "-u", "dkms", "--no-pager"])
        .output()
    {
        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout);
            for line in content.lines() {
                if line.contains("Error!") || line.contains("failed") {
                    // Parse DKMS failure - this is simplified
                    if let Some(failure) = parse_dkms_failure(line) {
                        failed_builds.push(failure);
                    }
                }
            }
        }
    }

    DkmsStatus {
        dkms_installed,
        modules,
        failed_builds,
    }
}

/// Parse DKMS status line
fn parse_dkms_status_line(line: &str) -> Option<DkmsModule> {
    // DKMS status format: "module/version, kernel-version, arch: status"
    // Example: "virtualbox/7.0.12, 6.6.8-arch1-1, x86_64: installed"

    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() < 3 {
        return None;
    }

    let module_parts: Vec<&str> = parts[0].split('/').collect();
    if module_parts.len() != 2 {
        return None;
    }

    let name = module_parts[0].trim().to_string();
    let version = module_parts[1].trim().to_string();
    let kernel_version = parts[1].trim().to_string();

    let status_part = parts[2].split(':').next_back().unwrap_or("").trim();
    let status = match status_part {
        "installed" => DkmsBuildStatus::Installed,
        "built" => DkmsBuildStatus::Built,
        _ => DkmsBuildStatus::NotBuilt,
    };

    Some(DkmsModule {
        name,
        version,
        kernel_versions: vec![kernel_version],
        status,
    })
}

/// Parse DKMS failure from journal
fn parse_dkms_failure(line: &str) -> Option<DkmsFailure> {
    // Very simplified - in reality this would need more sophisticated parsing
    Some(DkmsFailure {
        module_name: "unknown".to_string(),
        version: "unknown".to_string(),
        kernel_version: "unknown".to_string(),
        error: line.to_string(),
    })
}

/// Detect boot entries
fn detect_boot_entries() -> Vec<BootEntry> {
    let mut entries = Vec::new();

    // Check for systemd-boot
    if fs::metadata("/boot/loader/entries").is_ok() {
        entries.extend(detect_systemd_boot_entries());
    }

    // Check for GRUB
    if fs::metadata("/boot/grub/grub.cfg").is_ok() {
        entries.extend(detect_grub_entries());
    }

    entries
}

/// Detect systemd-boot entries
fn detect_systemd_boot_entries() -> Vec<BootEntry> {
    let mut entries = Vec::new();

    if let Ok(dir_entries) = fs::read_dir("/boot/loader/entries") {
        for entry in dir_entries.flatten() {
            if let Some(ext) = entry.path().extension() {
                if ext == "conf" {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if let Some(boot_entry) =
                            parse_systemd_boot_entry(&content, &entry.file_name().to_string_lossy())
                        {
                            entries.push(boot_entry);
                        }
                    }
                }
            }
        }
    }

    entries
}

/// Parse systemd-boot entry file
fn parse_systemd_boot_entry(content: &str, filename: &str) -> Option<BootEntry> {
    let mut title = None;
    let mut kernel_path = None;
    let mut initramfs_path = None;
    let mut validation_errors = Vec::new();

    for line in content.lines() {
        if let Some((key, value)) = line.split_once(char::is_whitespace) {
            let key = key.trim();
            let value = value.trim();

            match key {
                "title" => title = Some(value.to_string()),
                "linux" => kernel_path = Some(value.to_string()),
                "initrd" => initramfs_path = Some(value.to_string()),
                _ => {}
            }
        }
    }

    // Validate entry
    if let Some(ref path) = kernel_path {
        let full_path = format!("/boot{}", path);
        if fs::metadata(&full_path).is_err() {
            validation_errors.push(format!("Kernel image not found: {}", full_path));
        }
    } else {
        validation_errors.push("No kernel path specified".to_string());
    }

    if let Some(ref path) = initramfs_path {
        let full_path = format!("/boot{}", path);
        if fs::metadata(&full_path).is_err() {
            validation_errors.push(format!("Initramfs not found: {}", full_path));
        }
    }

    Some(BootEntry {
        id: filename.to_string(),
        title: title.unwrap_or_else(|| filename.to_string()),
        bootloader: BootloaderType::SystemdBoot,
        kernel_path,
        initramfs_path,
        is_valid: validation_errors.is_empty(),
        validation_errors,
    })
}

/// Detect GRUB entries
fn detect_grub_entries() -> Vec<BootEntry> {
    let mut entries = Vec::new();

    if let Ok(content) = fs::read_to_string("/boot/grub/grub.cfg") {
        let mut current_entry: Option<BootEntry> = None;

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("menuentry") {
                // Save previous entry
                if let Some(entry) = current_entry.take() {
                    entries.push(entry);
                }

                // Extract title
                if let Some(start) = line.find('\'') {
                    if let Some(end) = line[start + 1..].find('\'') {
                        let title = line[start + 1..start + 1 + end].to_string();
                        current_entry = Some(BootEntry {
                            id: format!("grub-{}", entries.len()),
                            title,
                            bootloader: BootloaderType::Grub,
                            kernel_path: None,
                            initramfs_path: None,
                            is_valid: true,
                            validation_errors: Vec::new(),
                        });
                    }
                }
            } else if line.starts_with("linux") {
                if let Some(ref mut entry) = current_entry {
                    if let Some(path) = line.split_whitespace().nth(1) {
                        entry.kernel_path = Some(path.to_string());
                    }
                }
            } else if line.starts_with("initrd") {
                if let Some(ref mut entry) = current_entry {
                    if let Some(path) = line.split_whitespace().nth(1) {
                        entry.initramfs_path = Some(path.to_string());
                    }
                }
            }
        }

        // Save last entry
        if let Some(entry) = current_entry {
            entries.push(entry);
        }
    }

    entries
}

/// Detect boot health
fn detect_boot_health() -> BootHealth {
    let mut boot_errors = Vec::new();
    let mut boot_warnings = Vec::new();
    let mut failed_boots = 0;

    // Get last boot time
    let last_boot_time = get_last_boot_time();

    // Get boot duration
    let boot_duration_seconds = get_boot_duration();

    // Check journal for boot errors
    if let Ok(output) = Command::new("journalctl")
        .args(["-b", "-p", "err", "--no-pager", "-n", "500"])
        .output()
    {
        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout);
            for line in content.lines() {
                if is_boot_related_error(line) {
                    if let Some(error) = parse_boot_error(line) {
                        boot_errors.push(error);
                    }
                }
            }
        }
    }

    // Check for warnings
    if let Ok(output) = Command::new("journalctl")
        .args(["-b", "-p", "warning", "--no-pager", "-n", "200"])
        .output()
    {
        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout);
            for line in content.lines() {
                if is_boot_related_error(line) {
                    boot_warnings.push(line.to_string());
                }
            }
        }
    }

    // Count failed boots from previous boots
    if let Ok(output) = Command::new("journalctl")
        .args(["--list-boots", "--no-pager"])
        .output()
    {
        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout);
            failed_boots = content.lines().count() as u32;
        }
    }

    BootHealth {
        last_boot_time,
        boot_errors,
        boot_warnings,
        failed_boots,
        boot_duration_seconds,
    }
}

/// Get last boot time
fn get_last_boot_time() -> Option<String> {
    Command::new("who")
        .arg("-b")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let output = String::from_utf8_lossy(&o.stdout);
            output.split_whitespace().nth(2).map(|s| s.to_string())
        })
}

/// Get boot duration
fn get_boot_duration() -> Option<f64> {
    Command::new("systemd-analyze")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let output = String::from_utf8_lossy(&o.stdout);
            // Parse "Startup finished in 2.345s (kernel) + 5.678s (userspace) = 8.023s"
            if let Some(total) = output.split('=').nth(1) {
                if let Some(time_str) = total.trim().strip_suffix('s') {
                    return time_str.parse().ok();
                }
            }
            None
        })
}

/// Check if log line is boot-related error
fn is_boot_related_error(line: &str) -> bool {
    line.contains("boot")
        || line.contains("kernel")
        || line.contains("initramfs")
        || line.contains("systemd[1]")
        || line.contains("Failed to start")
}

/// Parse boot error from journal line
fn parse_boot_error(line: &str) -> Option<BootError> {
    let severity = if line.contains("critical") {
        ErrorSeverity::Critical
    } else if line.contains("error") {
        ErrorSeverity::Error
    } else {
        ErrorSeverity::Warning
    };

    // Extract unit name if present
    let unit = if let Some(start) = line.find("systemd[1]: ") {
        let rest = &line[start + 12..];
        rest.split(':').next().map(|s| s.trim().to_string())
    } else {
        None
    };

    Some(BootError {
        message: line.to_string(),
        severity,
        timestamp: None,
        unit,
    })
}

/// Get module loading errors from journal
fn get_module_errors() -> Vec<ModuleError> {
    let mut errors = Vec::new();

    if let Ok(output) = Command::new("journalctl")
        .args(["-b", "-p", "err", "--no-pager", "-n", "500"])
        .output()
    {
        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout);
            for line in content.lines() {
                if line.contains("module") && (line.contains("error") || line.contains("failed")) {
                    if let Some(module_name) = extract_module_from_journal(line) {
                        errors.push(ModuleError {
                            module_name,
                            error: line.to_string(),
                            timestamp: None,
                        });
                    }
                }
            }
        }
    }

    errors
}
