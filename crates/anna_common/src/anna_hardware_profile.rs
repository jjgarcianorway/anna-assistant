//! Anna Hardware Profile Module (6.11.0)
//!
//! Tracks system hardware over time to detect changes:
//! - RAM increases/decreases
//! - CPU core changes
//! - Last chosen LLM model
//!
//! Used for LLM model recommendations and reflection items.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Anna's hardware profile
///
/// Persisted to disk to track changes across daemon restarts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnnaHardwareProfile {
    /// Total RAM in GiB
    pub total_ram_gib: u64,
    /// Number of CPU cores
    pub cpu_cores: u64,
    /// Last LLM model configured/chosen
    pub last_llm_model: String,
}

impl AnnaHardwareProfile {
    /// Get the profile file path
    ///
    /// Prefers /var/lib/anna/ (for daemon), falls back to ~/.local/share/anna/
    fn profile_path() -> PathBuf {
        // Try system location first
        let system_path = Path::new("/var/lib/anna/hardware_profile.json");
        if system_path.parent().map(|p| p.exists()).unwrap_or(false) {
            return system_path.to_path_buf();
        }

        // Fall back to user location
        if let Ok(home) = std::env::var("HOME") {
            let user_path = Path::new(&home).join(".local/share/anna/hardware_profile.json");
            if let Some(parent) = user_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            return user_path;
        }

        // Last resort: current directory
        PathBuf::from("hardware_profile.json")
    }

    /// Read hardware profile from disk
    ///
    /// Returns None if no profile exists yet (first run).
    pub fn read() -> Option<Self> {
        let path = Self::profile_path();
        if !path.exists() {
            return None;
        }

        fs::read_to_string(&path)
            .ok()
            .and_then(|contents| serde_json::from_str(&contents).ok())
    }

    /// Write hardware profile to disk
    pub fn write(&self) -> Result<()> {
        let path = Self::profile_path();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Cannot create directory: {}", parent.display()))?;
        }

        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize hardware profile")?;

        fs::write(&path, json)
            .with_context(|| format!("Cannot write hardware profile to {}", path.display()))?;

        Ok(())
    }
}

/// Detect current hardware
///
/// Returns a profile with current RAM/CPU but no last_llm_model.
/// Call `.write()` after setting the model to persist.
pub fn detect_current_hardware() -> AnnaHardwareProfile {
    let total_ram_gib = detect_ram_gib();
    let cpu_cores = detect_cpu_cores();

    AnnaHardwareProfile {
        total_ram_gib,
        cpu_cores,
        last_llm_model: String::new(), // To be filled by caller
    }
}

/// Detect total RAM in GiB
fn detect_ram_gib() -> u64 {
    // Try to read /proc/meminfo
    if let Ok(contents) = fs::read_to_string("/proc/meminfo") {
        for line in contents.lines() {
            if line.starts_with("MemTotal:") {
                // Format: "MemTotal:       32821508 kB"
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<u64>() {
                        // Convert KB to GiB (1 GiB = 1048576 KB)
                        return kb / 1048576;
                    }
                }
            }
        }
    }

    // Fallback: use sysinfo crate if available
    #[cfg(feature = "sysinfo")]
    {
        use sysinfo::{System, SystemExt};
        let mut sys = System::new_all();
        sys.refresh_all();
        return sys.total_memory() / (1024 * 1024 * 1024);
    }

    // Last resort: assume 8 GiB
    8
}

/// Detect number of CPU cores
fn detect_cpu_cores() -> u64 {
    // Try to use num_cpus crate
    num_cpus::get() as u64
}

/// Compare two profiles and generate human-readable change description
///
/// Returns None if no significant changes.
pub fn compare_profiles(old: &AnnaHardwareProfile, new: &AnnaHardwareProfile) -> Option<String> {
    let mut changes = Vec::new();

    // Check RAM changes (significant if ±8 GiB or more)
    let ram_diff = new.total_ram_gib as i64 - old.total_ram_gib as i64;
    if ram_diff.abs() >= 8 {
        if ram_diff > 0 {
            changes.push(format!("RAM upgraded: {} → {} GiB", old.total_ram_gib, new.total_ram_gib));
        } else {
            changes.push(format!("RAM decreased: {} → {} GiB", old.total_ram_gib, new.total_ram_gib));
        }
    }

    // Check CPU changes (significant if ±4 cores or more)
    let cpu_diff = new.cpu_cores as i64 - old.cpu_cores as i64;
    if cpu_diff.abs() >= 4 {
        if cpu_diff > 0 {
            changes.push(format!("CPU cores increased: {} → {}", old.cpu_cores, new.cpu_cores));
        } else {
            changes.push(format!("CPU cores decreased: {} → {}", old.cpu_cores, new.cpu_cores));
        }
    }

    if changes.is_empty() {
        None
    } else {
        Some(changes.join(", "))
    }
}

/// Recommend LLM model based on hardware
///
/// Policy for 6.11.0:
/// - < 8 GiB RAM: llama3.1:3b
/// - 8-15 GiB: llama3.1:8b
/// - 16-31 GiB: llama3.1:8b (safe default)
/// - ≥ 32 GiB and ≥ 16 cores: llama3.1:70b
///
/// This is advisory only - does not change config automatically.
pub fn recommend_llm_model(total_ram_gib: u64, cpu_cores: u64) -> String {
    if total_ram_gib < 8 {
        "llama3.1:3b".to_string()
    } else if total_ram_gib >= 32 && cpu_cores >= 16 {
        "llama3.1:70b".to_string()
    } else {
        // 8-31 GiB: safe default
        "llama3.1:8b".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_current_hardware() {
        let profile = detect_current_hardware();

        // RAM should be at least 1 GiB
        assert!(profile.total_ram_gib >= 1);

        // CPU cores should be at least 1
        assert!(profile.cpu_cores >= 1);
    }

    #[test]
    fn test_recommend_llm_model() {
        // Low RAM
        assert_eq!(recommend_llm_model(4, 4), "llama3.1:3b");
        assert_eq!(recommend_llm_model(7, 8), "llama3.1:3b");

        // Medium RAM
        assert_eq!(recommend_llm_model(8, 4), "llama3.1:8b");
        assert_eq!(recommend_llm_model(16, 8), "llama3.1:8b");
        assert_eq!(recommend_llm_model(24, 12), "llama3.1:8b");

        // High RAM + cores
        assert_eq!(recommend_llm_model(32, 16), "llama3.1:70b");
        assert_eq!(recommend_llm_model(64, 32), "llama3.1:70b");

        // High RAM but low cores (don't recommend 70b)
        assert_eq!(recommend_llm_model(32, 8), "llama3.1:8b");
    }

    #[test]
    fn test_compare_profiles_no_change() {
        let old = AnnaHardwareProfile {
            total_ram_gib: 16,
            cpu_cores: 8,
            last_llm_model: "llama3.1:8b".to_string(),
        };
        let new = AnnaHardwareProfile {
            total_ram_gib: 16,
            cpu_cores: 8,
            last_llm_model: "llama3.1:8b".to_string(),
        };

        assert_eq!(compare_profiles(&old, &new), None);
    }

    #[test]
    fn test_compare_profiles_ram_upgrade() {
        let old = AnnaHardwareProfile {
            total_ram_gib: 16,
            cpu_cores: 8,
            last_llm_model: "llama3.1:8b".to_string(),
        };
        let new = AnnaHardwareProfile {
            total_ram_gib: 32,
            cpu_cores: 8,
            last_llm_model: "llama3.1:8b".to_string(),
        };

        let result = compare_profiles(&old, &new);
        assert!(result.is_some());
        assert!(result.unwrap().contains("RAM upgraded"));
    }

    #[test]
    fn test_compare_profiles_cpu_increase() {
        let old = AnnaHardwareProfile {
            total_ram_gib: 16,
            cpu_cores: 8,
            last_llm_model: "llama3.1:8b".to_string(),
        };
        let new = AnnaHardwareProfile {
            total_ram_gib: 16,
            cpu_cores: 16,
            last_llm_model: "llama3.1:8b".to_string(),
        };

        let result = compare_profiles(&old, &new);
        assert!(result.is_some());
        assert!(result.unwrap().contains("CPU cores increased"));
    }

    #[test]
    fn test_profile_serialization() {
        let profile = AnnaHardwareProfile {
            total_ram_gib: 16,
            cpu_cores: 8,
            last_llm_model: "llama3.1:8b".to_string(),
        };

        // Test JSON round-trip
        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: AnnaHardwareProfile = serde_json::from_str(&json).unwrap();

        assert_eq!(profile, deserialized);
    }
}
