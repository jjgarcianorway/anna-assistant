//! Recovery plan parser - loads YAML definitions
//!
//! Phase 0.6: Recovery Framework - Plan parser
//! Citation: [archwiki:System_maintenance#Backup]

use super::types::RecoveryPlan;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Load all recovery plans from assets directory
pub fn load_all_plans() -> Result<Vec<RecoveryPlan>> {
    let recovery_dir = get_recovery_assets_dir();
    let mut plans = Vec::new();

    if !recovery_dir.exists() {
        // Return embedded plans as fallback
        return Ok(get_embedded_plans());
    }

    for entry in std::fs::read_dir(&recovery_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            match load_plan_from_file(&path) {
                Ok(plan) => plans.push(plan),
                Err(e) => {
                    tracing::warn!("Failed to load recovery plan from {:?}: {}", path, e);
                }
            }
        }
    }

    if plans.is_empty() {
        // Fallback to embedded plans
        Ok(get_embedded_plans())
    } else {
        Ok(plans)
    }
}

/// Load a single recovery plan from file
pub fn load_plan_from_file(path: &Path) -> Result<RecoveryPlan> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read recovery plan from {:?}", path))?;

    let plan: RecoveryPlan = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse recovery plan from {:?}", path))?;

    Ok(plan)
}

/// Load a specific recovery plan by ID
pub fn load_plan(plan_id: &str) -> Result<RecoveryPlan> {
    let all_plans = load_all_plans()?;

    all_plans
        .into_iter()
        .find(|p| p.name == plan_id)
        .ok_or_else(|| anyhow::anyhow!("Recovery plan not found: {}", plan_id))
}

/// Get recovery assets directory path
fn get_recovery_assets_dir() -> PathBuf {
    // Check multiple possible locations
    let possible_paths = vec![
        PathBuf::from("/usr/local/lib/anna/recovery"),
        PathBuf::from("./assets/recovery"),
        PathBuf::from("../assets/recovery"),
        PathBuf::from("../../assets/recovery"),
    ];

    for path in possible_paths {
        if path.exists() {
            return path;
        }
    }

    // Default fallback
    PathBuf::from("/usr/local/lib/anna/recovery")
}

/// Get embedded recovery plans as fallback (Phase 0.6 - minimal set)
fn get_embedded_plans() -> Vec<RecoveryPlan> {
    vec![
        RecoveryPlan {
            name: "bootloader".to_string(),
            description: "Bootloader repair and reinstallation".to_string(),
            citation: "[archwiki:GRUB#Installation]".to_string(),
            steps: vec![],
        },
        RecoveryPlan {
            name: "initramfs".to_string(),
            description: "Rebuild initramfs images".to_string(),
            citation: "[archwiki:Mkinitcpio]".to_string(),
            steps: vec![],
        },
        RecoveryPlan {
            name: "pacman-db".to_string(),
            description: "Repair pacman database".to_string(),
            citation: "[archwiki:Pacman/Tips_and_tricks#Database_errors]".to_string(),
            steps: vec![],
        },
        RecoveryPlan {
            name: "fstab".to_string(),
            description: "Repair and validate /etc/fstab".to_string(),
            citation: "[archwiki:Fstab]".to_string(),
            steps: vec![],
        },
        RecoveryPlan {
            name: "systemd".to_string(),
            description: "Restore systemd units and services".to_string(),
            citation: "[archwiki:Systemd#Basic_systemctl_usage]".to_string(),
            steps: vec![],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_plans() {
        let plans = get_embedded_plans();
        assert_eq!(plans.len(), 5);
        assert!(plans.iter().any(|p| p.name == "bootloader"));
        assert!(plans.iter().any(|p| p.name == "initramfs"));
        assert!(plans.iter().any(|p| p.name == "pacman-db"));
    }

    #[test]
    fn test_load_all_plans() {
        // May succeed or fall back to embedded
        let result = load_all_plans();
        assert!(result.is_ok());
        let plans = result.unwrap();
        assert!(!plans.is_empty());
    }
}
