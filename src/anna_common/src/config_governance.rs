//! Configuration governance for Anna Assistant
//!
//! This module implements the three-tier config system:
//! 1. System defaults: /etc/anna/anna.yml (read-only)
//! 2. User preferences: ~/.config/anna/prefs.yml (user-editable via annactl)
//! 3. Effective merged: /var/lib/anna/state/effective_config.json
//!
//! All configuration changes go through annactl to maintain consistency.

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Configuration file banner (inserted at top of YAML files)
pub const CONFIG_BANNER: &str = r#"# ─────────────────────────────────────────────────────────
# Managed by Anna. Please use `annactl config ...` to change behavior.
# Manual edits may be overwritten. See `annactl help config`.
# ─────────────────────────────────────────────────────────
"#;

/// Configuration value with origin tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValue {
    pub value: serde_json::Value,
    pub origin: ConfigOrigin,
}

/// Where a configuration value came from
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigOrigin {
    System,   // /etc/anna/anna.yml
    User,     // ~/.config/anna/prefs.yml
    Runtime,  // Command-line flag or environment variable
}

/// Effective configuration with origin tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectiveConfig {
    pub values: HashMap<String, serde_json::Value>,
    pub origins: HashMap<String, ConfigOrigin>,
    pub merged_at: String,  // ISO 8601 timestamp
}

/// Configuration paths
pub struct ConfigPaths {
    pub system_default: PathBuf,
    pub user_prefs: PathBuf,
    pub effective_snapshot: PathBuf,
    pub state_dir: PathBuf,
}

impl ConfigPaths {
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .context("Cannot determine home directory")?;

        Ok(Self {
            system_default: PathBuf::from("/etc/anna/anna.yml"),
            user_prefs: PathBuf::from(home).join(".config/anna/prefs.yml"),
            effective_snapshot: PathBuf::from("/var/lib/anna/state/effective_config.json"),
            state_dir: PathBuf::from("/var/lib/anna/state"),
        })
    }
}

/// Load YAML config from file
fn load_yaml_config(path: &Path) -> Result<HashMap<String, serde_json::Value>> {
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    // Parse YAML
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse YAML in {}", path.display()))?;

    // Convert to flat key-value map
    let map = flatten_yaml(&yaml, String::new());

    Ok(map)
}

/// Flatten nested YAML into dot-notation keys
fn flatten_yaml(value: &serde_yaml::Value, prefix: String) -> HashMap<String, serde_json::Value> {
    let mut result = HashMap::new();

    match value {
        serde_yaml::Value::Mapping(map) => {
            for (key, val) in map {
                if let serde_yaml::Value::String(key_str) = key {
                    let new_prefix = if prefix.is_empty() {
                        key_str.clone()
                    } else {
                        format!("{}.{}", prefix, key_str)
                    };

                    let flattened = flatten_yaml(val, new_prefix.clone());
                    result.extend(flattened);
                }
            }
        }
        _ => {
            // Convert YAML value to JSON value
            if let Ok(json_str) = serde_json::to_string(&value) {
                if let Ok(json_val) = serde_json::from_str(&json_str) {
                    result.insert(prefix, json_val);
                }
            }
        }
    }

    result
}

/// Merge configurations with precedence: system < user < runtime
pub fn merge_configs(
    system: HashMap<String, serde_json::Value>,
    user: HashMap<String, serde_json::Value>,
    runtime: HashMap<String, serde_json::Value>,
) -> EffectiveConfig {
    let mut values = HashMap::new();
    let mut origins = HashMap::new();

    // Start with system defaults
    for (key, value) in system {
        values.insert(key.clone(), value);
        origins.insert(key, ConfigOrigin::System);
    }

    // Override with user prefs
    for (key, value) in user {
        values.insert(key.clone(), value);
        origins.insert(key, ConfigOrigin::User);
    }

    // Override with runtime
    for (key, value) in runtime {
        values.insert(key.clone(), value);
        origins.insert(key, ConfigOrigin::Runtime);
    }

    EffectiveConfig {
        values,
        origins,
        merged_at: chrono::Utc::now().to_rfc3339(),
    }
}

/// Load effective configuration
pub fn load_effective_config() -> Result<EffectiveConfig> {
    let paths = ConfigPaths::new()?;

    let system = load_yaml_config(&paths.system_default)
        .unwrap_or_else(|_| HashMap::new());
    let user = load_yaml_config(&paths.user_prefs)
        .unwrap_or_else(|_| HashMap::new());
    let runtime = HashMap::new(); // Populated by CLI flags

    Ok(merge_configs(system, user, runtime))
}

/// Save effective configuration to snapshot
pub fn save_effective_snapshot(config: &EffectiveConfig) -> Result<()> {
    let paths = ConfigPaths::new()?;

    // Ensure state directory exists
    if !paths.state_dir.exists() {
        // Try to create, may need privileges
        if let Err(e) = fs::create_dir_all(&paths.state_dir) {
            // Will be handled by privilege escalation in caller
            bail!("Cannot create state directory: {}", e);
        }
    }

    // Serialize to JSON
    let json = serde_json::to_string_pretty(config)
        .context("Failed to serialize config")?;

    // Try to write with file locking
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&paths.effective_snapshot)
        .context("Failed to open effective config file for writing")?;

    // File locking (advisory)
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        unsafe {
            libc::flock(file.as_raw_fd(), libc::LOCK_EX);
        }
    }

    file.write_all(json.as_bytes())
        .context("Failed to write effective config")?;

    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        unsafe {
            libc::flock(file.as_raw_fd(), libc::LOCK_UN);
        }
    }

    Ok(())
}

/// Set a configuration value in user prefs
pub fn set_user_config(key: &str, value: serde_json::Value) -> Result<()> {
    let paths = ConfigPaths::new()?;

    // Ensure user config directory exists
    if let Some(parent) = paths.user_prefs.parent() {
        fs::create_dir_all(parent)
            .context("Failed to create user config directory")?;
    }

    // Load existing user prefs
    let mut user_config = if paths.user_prefs.exists() {
        let content = fs::read_to_string(&paths.user_prefs)?;
        serde_yaml::from_str(&content).unwrap_or_else(|_| serde_yaml::Value::Mapping(Default::default()))
    } else {
        serde_yaml::Value::Mapping(Default::default())
    };

    // Update value (convert dot-notation key to nested structure)
    set_nested_value(&mut user_config, key, value)?;

    // Serialize to YAML
    let yaml = serde_yaml::to_string(&user_config)
        .context("Failed to serialize config")?;

    // Write with banner
    let content = format!("{}\n{}", CONFIG_BANNER, yaml);

    fs::write(&paths.user_prefs, content)
        .context("Failed to write user prefs")?;

    // Regenerate effective snapshot
    let effective = load_effective_config()?;
    save_effective_snapshot(&effective)?;

    Ok(())
}

/// Set nested value in YAML structure using dot notation
fn set_nested_value(root: &mut serde_yaml::Value, key: &str, value: serde_json::Value) -> Result<()> {
    let parts: Vec<&str> = key.split('.').collect();

    if parts.is_empty() {
        bail!("Empty key");
    }

    // Convert JSON value to YAML value
    let yaml_value: serde_yaml::Value = serde_json::from_str(
        &serde_json::to_string(&value)?
    )?;

    // Navigate to the right place in the tree
    let mut current = root;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Last part - set the value
            if let serde_yaml::Value::Mapping(map) = current {
                map.insert(
                    serde_yaml::Value::String(part.to_string()),
                    yaml_value.clone(),
                );
            }
        } else {
            // Navigate deeper, creating maps as needed
            if let serde_yaml::Value::Mapping(map) = current {
                let key = serde_yaml::Value::String(part.to_string());
                if !map.contains_key(&key) {
                    map.insert(key.clone(), serde_yaml::Value::Mapping(Default::default()));
                }
                current = map.get_mut(&key).unwrap();
            }
        }
    }

    Ok(())
}

/// Get a configuration value with origin
pub fn get_config_value(key: &str) -> Result<ConfigValue> {
    let effective = load_effective_config()?;

    let value = effective.values.get(key)
        .ok_or_else(|| anyhow::anyhow!("Configuration key '{}' not found", key))?
        .clone();

    let origin = effective.origins.get(key)
        .copied()
        .unwrap_or(ConfigOrigin::System);

    Ok(ConfigValue { value, origin })
}

/// Reset user configuration (all or specific key)
pub fn reset_user_config(key: Option<&str>) -> Result<()> {
    let paths = ConfigPaths::new()?;

    if let Some(specific_key) = key {
        // Reset specific key - load, remove, save
        if paths.user_prefs.exists() {
            let content = fs::read_to_string(&paths.user_prefs)?;
            let mut user_config: serde_yaml::Value = serde_yaml::from_str(&content)
                .unwrap_or_else(|_| serde_yaml::Value::Mapping(Default::default()));

            remove_nested_value(&mut user_config, specific_key);

            let yaml = serde_yaml::to_string(&user_config)?;
            let content = format!("{}\n{}", CONFIG_BANNER, yaml);
            fs::write(&paths.user_prefs, content)?;
        }
    } else {
        // Reset all - just recreate with banner and empty structure
        let content = format!("{}\n# User preferences (examples):\n# ui:\n#   emojis: true\n#   colors: true\n", CONFIG_BANNER);
        fs::write(&paths.user_prefs, content)?;
    }

    // Regenerate effective snapshot
    let effective = load_effective_config()?;
    save_effective_snapshot(&effective)?;

    Ok(())
}

/// Remove nested value from YAML structure
fn remove_nested_value(root: &mut serde_yaml::Value, key: &str) {
    let parts: Vec<&str> = key.split('.').collect();
    if parts.is_empty() {
        return;
    }

    let mut current = root;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Last part - remove it
            if let serde_yaml::Value::Mapping(map) = current {
                map.remove(&serde_yaml::Value::String(part.to_string()));
            }
        } else {
            // Navigate deeper
            if let serde_yaml::Value::Mapping(map) = current {
                let key = serde_yaml::Value::String(part.to_string());
                if let Some(next) = map.get_mut(&key) {
                    current = next;
                } else {
                    return; // Path doesn't exist
                }
            }
        }
    }
}

/// Ensure config files have banners
pub fn ensure_banners() -> Result<()> {
    let paths = ConfigPaths::new()?;

    // User prefs
    if paths.user_prefs.exists() {
        let content = fs::read_to_string(&paths.user_prefs)?;
        if !content.contains("Managed by Anna") {
            let new_content = format!("{}\n{}", CONFIG_BANNER, content);
            fs::write(&paths.user_prefs, new_content)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_configs() {
        let mut system = HashMap::new();
        system.insert("ui.colors".to_string(), serde_json::json!(true));
        system.insert("ui.emojis".to_string(), serde_json::json!(true));

        let mut user = HashMap::new();
        user.insert("ui.emojis".to_string(), serde_json::json!(false));

        let runtime = HashMap::new();

        let effective = merge_configs(system, user, runtime);

        assert_eq!(effective.values.get("ui.colors"), Some(&serde_json::json!(true)));
        assert_eq!(effective.values.get("ui.emojis"), Some(&serde_json::json!(false)));
        assert_eq!(effective.origins.get("ui.colors"), Some(&ConfigOrigin::System));
        assert_eq!(effective.origins.get("ui.emojis"), Some(&ConfigOrigin::User));
    }
}
