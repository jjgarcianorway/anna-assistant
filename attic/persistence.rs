use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};

/// Persistence directory
const STATE_DIR: &str = "/var/lib/anna/state";

/// Maximum age of state files before rotation (7 days)
const MAX_STATE_AGE_DAYS: i64 = 7;

/// Component state metadata
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StateMetadata {
    pub component: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
}

/// Generic state wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub metadata: StateMetadata,
    pub data: serde_json::Value,
}

/// Initialize persistence system
pub fn init() -> Result<()> {
    let state_dir = Path::new(STATE_DIR);
    if !state_dir.exists() {
        fs::create_dir_all(state_dir)
            .context("Failed to create state directory")?;
    }

    // Rotate old state files
    rotate_old_states()?;

    Ok(())
}

/// Save component state
pub fn save_state(component: &str, data: serde_json::Value) -> Result<()> {
    let state = State {
        metadata: StateMetadata {
            component: component.to_string(),
            timestamp: Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        data,
    };

    let state_file = get_state_file(component);

    // Ensure parent directory exists
    if let Some(parent) = state_file.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let json = serde_json::to_string_pretty(&state)?;
    fs::write(&state_file, json)
        .context(format!("Failed to write state for {}", component))?;

    Ok(())
}

/// Load component state
pub fn load_state(component: &str) -> Result<Option<State>> {
    let state_file = get_state_file(component);

    if !state_file.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(&state_file)
        .context(format!("Failed to read state for {}", component))?;

    let state: State = serde_json::from_str(&contents)
        .context(format!("Failed to parse state for {}", component))?;

    Ok(Some(state))
}

/// List all saved component states
pub fn list_states() -> Result<Vec<String>> {
    let state_dir = Path::new(STATE_DIR);
    if !state_dir.exists() {
        return Ok(vec![]);
    }

    let mut components = Vec::new();

    for entry in fs::read_dir(state_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                components.push(stem.to_string());
            }
        }
    }

    components.sort();
    Ok(components)
}

/// Delete component state
#[allow(dead_code)]
pub fn delete_state(component: &str) -> Result<()> {
    let state_file = get_state_file(component);

    if state_file.exists() {
        fs::remove_file(&state_file)
            .context(format!("Failed to delete state for {}", component))?;
    }

    Ok(())
}

/// Get state file path for a component
fn get_state_file(component: &str) -> PathBuf {
    Path::new(STATE_DIR).join(format!("{}.json", component))
}

/// Rotate old state files (older than MAX_STATE_AGE_DAYS)
fn rotate_old_states() -> Result<()> {
    let state_dir = Path::new(STATE_DIR);
    if !state_dir.exists() {
        return Ok(());
    }

    let now = Utc::now();

    for entry in fs::read_dir(state_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
            // Try to read the state to check its age
            if let Ok(contents) = fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str::<State>(&contents) {
                    let age = now.signed_duration_since(state.metadata.timestamp);

                    if age.num_days() > MAX_STATE_AGE_DAYS {
                        // Archive old state with timestamp
                        let archived_name = format!(
                            "{}_archived_{}.json",
                            state.metadata.component,
                            state.metadata.timestamp.format("%Y%m%d")
                        );
                        let archived_path = state_dir.join(archived_name);

                        // Move to archived name
                        let _ = fs::rename(&path, archived_path);
                    }
                }
            }
        }
    }

    // Clean up very old archives (older than 30 days)
    for entry in fs::read_dir(state_dir)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.contains("_archived_") {
                if let Ok(metadata) = fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(elapsed) = modified.elapsed() {
                            if elapsed.as_secs() > 30 * 24 * 60 * 60 {
                                // Older than 30 days
                                let _ = fs::remove_file(&path);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Save a simple key-value state for a component
#[allow(dead_code)]
pub fn save_kv_state(component: &str, kv: HashMap<String, String>) -> Result<()> {
    let data = serde_json::to_value(kv)?;
    save_state(component, data)
}

/// Load a simple key-value state for a component
#[allow(dead_code)]
pub fn load_kv_state(component: &str) -> Result<Option<HashMap<String, String>>> {
    if let Some(state) = load_state(component)? {
        let kv: HashMap<String, String> = serde_json::from_value(state.data)
            .context("Failed to parse key-value state")?;
        Ok(Some(kv))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_metadata() {
        let metadata = StateMetadata {
            component: "test".to_string(),
            timestamp: Utc::now(),
            version: "0.9.0".to_string(),
        };

        assert_eq!(metadata.component, "test");
        assert_eq!(metadata.version, "0.9.0");
    }

    #[test]
    fn test_state_file_path() {
        let path = get_state_file("test_component");
        assert!(path.to_string_lossy().contains("test_component.json"));
    }
}
