// Installation Source Detection
// Phase 3.10: AUR-Aware Auto-Upgrade System
//
// Detects how Anna was installed to determine if auto-updates are allowed

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

/// Installation source for Anna
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallationSource {
    /// Installed via AUR package manager (pacman/yay)
    AUR { package_name: String },

    /// Manual installation (GitHub release, curl, or local build)
    Manual { path: String },

    /// Unknown source
    Unknown,
}

impl InstallationSource {
    /// Check if auto-updates are allowed for this installation source
    pub fn allows_auto_update(&self) -> bool {
        matches!(self, InstallationSource::Manual { .. })
    }

    /// Get human-readable description
    pub fn description(&self) -> String {
        match self {
            InstallationSource::AUR { package_name } => {
                format!("AUR Package ({})", package_name)
            }
            InstallationSource::Manual { path } => {
                format!("Manual Installation ({})", path)
            }
            InstallationSource::Unknown => "Unknown".to_string(),
        }
    }

    /// Get update command suggestion
    pub fn update_command(&self) -> String {
        match self {
            InstallationSource::AUR { package_name } => {
                // Try to detect which AUR helper was used
                if Command::new("yay").arg("--version").output().is_ok() {
                    format!("yay -S {}", package_name)
                } else if Command::new("paru").arg("--version").output().is_ok() {
                    format!("paru -S {}", package_name)
                } else {
                    format!("sudo pacman -S {}", package_name)
                }
            }
            InstallationSource::Manual { .. } => {
                "annactl upgrade".to_string()
            }
            InstallationSource::Unknown => {
                "Unknown - check installation method".to_string()
            }
        }
    }
}

/// Detect installation source for a binary
pub fn detect_installation_source(binary_path: &str) -> InstallationSource {
    // Method 1: Check if managed by pacman
    if let Some(package_name) = check_pacman_ownership(binary_path) {
        return InstallationSource::AUR { package_name };
    }

    // Method 2: Check file path patterns
    let path = Path::new(binary_path);

    // AUR packages typically install to /usr/bin
    if path.starts_with("/usr/bin") {
        // Double-check with pacman
        if let Ok(output) = Command::new("pacman")
            .args(&["-Qo", binary_path])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(pkg) = extract_package_name(&stdout) {
                    return InstallationSource::AUR {
                        package_name: pkg.to_string(),
                    };
                }
            }
        }
    }

    // Manual installations typically in /usr/local/bin or custom paths
    if path.starts_with("/usr/local") || path.starts_with("/opt") {
        return InstallationSource::Manual {
            path: binary_path.to_string(),
        };
    }

    // Method 3: Check build metadata (embedded at compile time)
    if let Some(source) = check_build_metadata() {
        return source;
    }

    // Default to manual if in unusual location
    InstallationSource::Manual {
        path: binary_path.to_string(),
    }
}

/// Check if binary is owned by a pacman package
fn check_pacman_ownership(binary_path: &str) -> Option<String> {
    let output = Command::new("pacman")
        .args(&["-Qo", binary_path])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    extract_package_name(&stdout)
}

/// Extract package name from pacman output
fn extract_package_name(pacman_output: &str) -> Option<String> {
    // Format: "/usr/bin/annactl is owned by anna-assistant-bin 3.9.1-1"
    let parts: Vec<&str> = pacman_output.split_whitespace().collect();

    // Find "owned by" and get next token
    for (i, &part) in parts.iter().enumerate() {
        if part == "by" && i + 1 < parts.len() {
            return Some(parts[i + 1].to_string());
        }
    }

    None
}

/// Check build metadata embedded at compile time
fn check_build_metadata() -> Option<InstallationSource> {
    // This can be set via RUSTFLAGS or build.rs
    // For now, we don't embed this, but it's a placeholder for future enhancement

    #[cfg_attr(not(test), allow(unexpected_cfgs))]
    #[cfg(feature = "aur-build")]
    {
        return Some(InstallationSource::AUR {
            package_name: "anna-assistant-bin".to_string(),
        });
    }

    None
}

/// Detect current binary installation source
pub fn detect_current_installation() -> InstallationSource {
    // Try to get current executable path
    match std::env::current_exe() {
        Ok(path) => {
            let path_str = path.to_string_lossy().to_string();
            detect_installation_source(&path_str)
        }
        Err(_) => InstallationSource::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installation_source_allows_auto_update() {
        let manual = InstallationSource::Manual {
            path: "/usr/local/bin/annactl".to_string(),
        };
        assert!(manual.allows_auto_update());

        let aur = InstallationSource::AUR {
            package_name: "anna-assistant-bin".to_string(),
        };
        assert!(!aur.allows_auto_update());
    }

    #[test]
    fn test_extract_package_name() {
        let output = "/usr/bin/annactl is owned by anna-assistant-bin 3.9.1-1";
        assert_eq!(
            extract_package_name(output),
            Some("anna-assistant-bin".to_string())
        );
    }

    #[test]
    fn test_path_based_detection() {
        let source = detect_installation_source("/usr/local/bin/annactl");
        assert!(matches!(source, InstallationSource::Manual { .. }));
    }
}
