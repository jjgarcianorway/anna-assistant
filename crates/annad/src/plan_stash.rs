//! Plan Stash - State capture and rollback storage.
//! Phase 17: Post-action verification and rollback.
//!
//! Captures pre-execution state for rollback capability.
//! Stash location: /var/lib/anna/rollback/<plan_id>/<step_index>/

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info, warn};

/// Rollback stash for a plan execution.
#[derive(Debug, Clone)]
pub struct PlanStash {
    /// Plan ID this stash belongs to.
    pub plan_id: String,
    /// Base directory for this stash.
    pub stash_dir: PathBuf,
    /// Captured states per step.
    pub step_states: Vec<StepState>,
}

/// Pre-execution state for a single step.
#[derive(Debug, Clone)]
pub struct StepState {
    /// Step index.
    pub step_index: usize,
    /// File backups (path -> backup location).
    pub file_backups: Vec<FileBackup>,
    /// Systemd unit states.
    pub unit_states: Vec<UnitState>,
    /// Directory for this step's stash.
    pub step_dir: PathBuf,
}

/// Backup of a file before modification.
#[derive(Debug, Clone)]
pub struct FileBackup {
    /// Original file path.
    pub original_path: PathBuf,
    /// Backup file path in stash.
    pub backup_path: PathBuf,
    /// Whether file existed before (false = created new).
    pub existed: bool,
}

/// State of a systemd unit before modification.
#[derive(Debug, Clone)]
pub struct UnitState {
    /// Unit name (e.g., "sleep.target").
    pub unit: String,
    /// Was it masked?
    pub was_masked: bool,
    /// Was it enabled?
    pub was_enabled: bool,
    /// Previous symlink target (for mask).
    pub previous_target: Option<String>,
}

impl PlanStash {
    /// Create a new stash for a plan.
    pub fn new(plan_id: &str) -> Self {
        let stash_dir = Self::stash_base_dir().join(plan_id);
        Self {
            plan_id: plan_id.to_string(),
            stash_dir,
            step_states: Vec::new(),
        }
    }

    /// Base directory for all rollback stashes.
    fn stash_base_dir() -> PathBuf {
        PathBuf::from("/var/lib/anna/rollback")
    }

    /// Initialize stash directory.
    pub fn init(&self) -> Result<(), String> {
        fs::create_dir_all(&self.stash_dir)
            .map_err(|e| format!("Failed to create stash dir: {}", e))?;
        info!("Initialized stash at {:?}", self.stash_dir);
        Ok(())
    }

    /// Create step state directory.
    pub fn create_step_state(&mut self, step_index: usize) -> &mut StepState {
        let step_dir = self.stash_dir.join(format!("step_{}", step_index));
        let _ = fs::create_dir_all(&step_dir);

        self.step_states.push(StepState {
            step_index,
            file_backups: Vec::new(),
            unit_states: Vec::new(),
            step_dir,
        });

        self.step_states.last_mut().unwrap()
    }

    /// Get step state by index.
    pub fn get_step_state(&self, step_index: usize) -> Option<&StepState> {
        self.step_states.iter().find(|s| s.step_index == step_index)
    }

    /// Clean up stash after successful execution.
    pub fn cleanup(&self) -> Result<(), String> {
        if self.stash_dir.exists() {
            fs::remove_dir_all(&self.stash_dir)
                .map_err(|e| format!("Failed to cleanup stash: {}", e))?;
            info!("Cleaned up stash at {:?}", self.stash_dir);
        }
        Ok(())
    }
}

impl StepState {
    /// Backup a file before modification.
    pub fn backup_file(&mut self, path: &str) -> Result<FileBackup, String> {
        let original = PathBuf::from(path);
        let filename = original
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let backup_path = self.step_dir.join(format!("backup_{}", filename));

        let existed = original.exists();

        if existed {
            fs::copy(&original, &backup_path)
                .map_err(|e| format!("Failed to backup {}: {}", path, e))?;
            debug!("Backed up {:?} to {:?}", original, backup_path);
        } else {
            // Mark as "will be created" - rollback means delete
            fs::write(&backup_path, b"__ABSENT__")
                .map_err(|e| format!("Failed to mark absent: {}", e))?;
            debug!("Marked {:?} as absent (new file)", original);
        }

        let backup = FileBackup {
            original_path: original,
            backup_path,
            existed,
        };
        self.file_backups.push(backup.clone());
        Ok(backup)
    }

    /// Capture systemd unit state before masking/enabling.
    pub fn capture_unit_state(&mut self, unit: &str) -> Result<UnitState, String> {
        let was_masked = is_unit_masked(unit);
        let was_enabled = is_unit_enabled(unit);
        let previous_target = if was_masked {
            get_mask_target(unit)
        } else {
            None
        };

        let state = UnitState {
            unit: unit.to_string(),
            was_masked,
            was_enabled,
            previous_target,
        };

        debug!(
            "Captured unit state: {} (masked={}, enabled={})",
            unit, was_masked, was_enabled
        );

        self.unit_states.push(state.clone());
        Ok(state)
    }

    /// Restore file from backup.
    pub fn restore_file(&self, backup: &FileBackup) -> Result<(), String> {
        if !backup.existed {
            // File was created by step - delete it
            if backup.original_path.exists() {
                fs::remove_file(&backup.original_path)
                    .map_err(|e| format!("Failed to remove new file: {}", e))?;
                info!("Removed newly created file {:?}", backup.original_path);
            }
        } else {
            // Restore from backup
            fs::copy(&backup.backup_path, &backup.original_path)
                .map_err(|e| format!("Failed to restore file: {}", e))?;
            info!("Restored {:?} from backup", backup.original_path);
        }
        Ok(())
    }

    /// Restore systemd unit state.
    pub fn restore_unit_state(&self, state: &UnitState) -> Result<(), String> {
        if state.was_masked {
            // Was masked before, re-mask if now unmasked
            if !is_unit_masked(&state.unit) {
                run_systemctl(&["mask", &state.unit])?;
                info!("Re-masked unit {}", state.unit);
            }
        } else {
            // Was not masked, unmask if now masked
            if is_unit_masked(&state.unit) {
                run_systemctl(&["unmask", &state.unit])?;
                info!("Unmasked unit {}", state.unit);
            }
        }

        if state.was_enabled {
            if !is_unit_enabled(&state.unit) {
                run_systemctl(&["enable", &state.unit])?;
            }
        } else {
            if is_unit_enabled(&state.unit) {
                run_systemctl(&["disable", &state.unit])?;
            }
        }

        Ok(())
    }
}

/// Check if a systemd unit is masked.
fn is_unit_masked(unit: &str) -> bool {
    Command::new("systemctl")
        .args(["is-enabled", unit])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("masked"))
        .unwrap_or(false)
}

/// Check if a systemd unit is enabled.
fn is_unit_enabled(unit: &str) -> bool {
    Command::new("systemctl")
        .args(["is-enabled", unit])
        .output()
        .map(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).trim() == "enabled")
        .unwrap_or(false)
}

/// Get the mask symlink target for a unit.
fn get_mask_target(unit: &str) -> Option<String> {
    let path = PathBuf::from("/etc/systemd/system").join(unit);
    fs::read_link(&path)
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

/// Run systemctl command with pkexec.
fn run_systemctl(args: &[&str]) -> Result<(), String> {
    let cmd = format!("systemctl {}", args.join(" "));
    Command::new("pkexec")
        .args(["sh", "-c", &cmd])
        .output()
        .map_err(|e| format!("Failed to run systemctl: {}", e))
        .and_then(|o| {
            if o.status.success() {
                Ok(())
            } else {
                Err(format!(
                    "systemctl failed: {}",
                    String::from_utf8_lossy(&o.stderr)
                ))
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_stash_creation() {
        let stash = PlanStash::new("test-plan-123");
        assert!(stash.stash_dir.to_string_lossy().contains("test-plan-123"));
    }

    #[test]
    fn test_unit_state_detection() {
        // These won't work in CI but document the API
        let _ = is_unit_masked("nonexistent.target");
        let _ = is_unit_enabled("nonexistent.target");
    }
}
