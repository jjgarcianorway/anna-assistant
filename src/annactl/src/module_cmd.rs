// Anna v0.10.1 - annactl module enable/disable commands

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

const MODULES_PATH: &str = "/etc/anna/modules.yaml";

#[derive(Debug, Deserialize, Serialize)]
struct ModulesConfig {
    version: String,
    modules: HashMap<String, ModuleUserConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ModuleUserConfig {
    state: String,
    reason: String,
}

pub fn enable_module(name: &str) -> Result<()> {
    let mut config = load_config()?;

    if let Some(module) = config.modules.get_mut(name) {
        if module.state == "enabled" {
            println!("✓ Module '{}' is already enabled", name);
            return Ok(());
        }

        module.state = "enabled".to_string();
        module.reason = "Enabled by user".to_string();
    } else {
        // Add new entry
        config.modules.insert(
            name.to_string(),
            ModuleUserConfig {
                state: "enabled".to_string(),
                reason: "Enabled by user".to_string(),
            },
        );
    }

    save_config(&config)?;

    println!("✓ Module '{}' enabled", name);
    println!("\n  Restart daemon: sudo systemctl restart annad");
    println!("  Check status:   annactl capabilities\n");

    Ok(())
}

pub fn disable_module(name: &str, reason: Option<String>) -> Result<()> {
    let mut config = load_config()?;

    // Check if module is required
    let is_required = check_if_required(name)?;
    if is_required {
        anyhow::bail!("Cannot disable required module '{}'", name);
    }

    let reason_text = reason.unwrap_or_else(|| "Disabled by user".to_string());

    if let Some(module) = config.modules.get_mut(name) {
        if module.state == "disabled" {
            println!("⚠ Module '{}' is already disabled", name);
            return Ok(());
        }

        module.state = "disabled".to_string();
        module.reason = reason_text.clone();
    } else {
        // Add new entry
        config.modules.insert(
            name.to_string(),
            ModuleUserConfig {
                state: "disabled".to_string(),
                reason: reason_text.clone(),
            },
        );
    }

    save_config(&config)?;

    println!("✓ Module '{}' disabled", name);
    println!("  Reason: {}", reason_text);
    println!("\n  Restart daemon: sudo systemctl restart annad");
    println!("  Check status:   annactl capabilities\n");

    Ok(())
}

fn load_config() -> Result<ModulesConfig> {
    if !std::path::Path::new(MODULES_PATH).exists() {
        // Create default config
        return Ok(ModulesConfig {
            version: "0.10.1".to_string(),
            modules: HashMap::new(),
        });
    }

    let content = fs::read_to_string(MODULES_PATH)
        .context("Failed to read modules.yaml")?;

    serde_yaml::from_str(&content)
        .context("Failed to parse modules.yaml")
}

fn save_config(config: &ModulesConfig) -> Result<()> {
    let content = serde_yaml::to_string(config)
        .context("Failed to serialize modules.yaml")?;

    fs::write(MODULES_PATH, content)
        .context("Failed to write modules.yaml (run with sudo?)")?;

    Ok(())
}

fn check_if_required(name: &str) -> Result<bool> {
    let cap_path = "/usr/lib/anna/CAPABILITIES.toml";
    if !std::path::Path::new(cap_path).exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(cap_path)?;
    let registry: toml::Value = toml::from_str(&content)?;

    if let Some(modules) = registry.get("modules").and_then(|m| m.as_table()) {
        if let Some(module_def) = modules.get(name) {
            let required = module_def
                .get("required")
                .and_then(|r| r.as_bool())
                .unwrap_or(false);

            return Ok(required);
        }
    }

    Ok(false)
}
