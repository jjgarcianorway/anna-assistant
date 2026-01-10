//! Configuration file scanning - modprobe, udev, systemd, X11.

use anyhow::Result;

use super::{ConfigFile, ConfigProfile};

/// Scan existing configurations
pub fn scan_configs() -> Result<ConfigProfile> {
    let mut configs = ConfigProfile::default();

    // Scan modprobe.d
    configs.modprobe = scan_directory("/etc/modprobe.d", &["conf"]);

    // Scan udev rules (custom ones)
    configs.udev_rules = scan_directory("/etc/udev/rules.d", &["rules"]);

    // Scan systemd overrides
    configs.systemd_overrides = scan_systemd_overrides();

    // Scan X11 configs
    configs.xorg_configs = scan_directory("/etc/X11/xorg.conf.d", &["conf"]);

    Ok(configs)
}

/// Scan a directory for config files
pub fn scan_directory(path: &str, extensions: &[&str]) -> Vec<ConfigFile> {
    let mut files = Vec::new();

    let dir = match std::fs::read_dir(path) {
        Ok(d) => d,
        Err(_) => return files,
    };

    for entry in dir.flatten() {
        let path = entry.path();
        if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if extensions.is_empty() || extensions.contains(&ext) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    // Skip empty files and very large files
                    if !content.trim().is_empty() && content.len() < 10000 {
                        files.push(ConfigFile {
                            path: path.display().to_string(),
                            content,
                        });
                    }
                }
            }
        }
    }

    files
}

/// Scan systemd override directories
fn scan_systemd_overrides() -> Vec<ConfigFile> {
    let mut files = Vec::new();

    // Check /etc/systemd/system for overrides
    let system_dir = "/etc/systemd/system";
    if let Ok(dir) = std::fs::read_dir(system_dir) {
        for entry in dir.flatten() {
            let path = entry.path();

            // Look for .d directories (override directories)
            if path.is_dir() && path.display().to_string().ends_with(".d") {
                files.extend(scan_directory(&path.display().to_string(), &["conf"]));
            }

            // Also check direct override files
            if path.is_file() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.ends_with(".service") || name.ends_with(".timer") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if !content.trim().is_empty() && content.len() < 10000 {
                            files.push(ConfigFile {
                                path: path.display().to_string(),
                                content,
                            });
                        }
                    }
                }
            }
        }
    }

    files
}
