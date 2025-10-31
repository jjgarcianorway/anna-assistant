//! Configuration management commands (standalone, no daemon required)

use anyhow::Result;
use anna_common::{
    anna_narrative, anna_info, anna_ok, anna_warn,
    get_config_value, set_user_config, reset_user_config,
    load_effective_config, ConfigOrigin,
};

/// Get a configuration value
pub async fn config_get(key: &str) -> Result<()> {
    anna_narrative(format!("Let me check the value of '{}'...", key));

    match get_config_value(key) {
        Ok(config_value) => {
            let origin_str = match config_value.origin {
                ConfigOrigin::System => "system default",
                ConfigOrigin::User => "your preference",
                ConfigOrigin::Runtime => "command-line override",
            };

            anna_ok(format!("{} = {}", key, config_value.value));
            anna_info(format!("Source: {}", origin_str));
            Ok(())
        }
        Err(_) => {
            anna_warn(format!("I couldn't find a setting called '{}'.", key));
            anna_info("Try 'annactl config list' to see all available settings.");
            Ok(())
        }
    }
}

/// Set a configuration value
pub async fn config_set(key: &str, value: &str) -> Result<()> {
    anna_narrative(format!("I'll update '{}' for you...", key));

    // Parse value as JSON
    let json_value: serde_json::Value = serde_json::from_str(value)
        .or_else(|_| {
            // If not valid JSON, treat as string
            Ok::<_, serde_json::Error>(serde_json::Value::String(value.to_string()))
        })?;

    // Try to set
    match set_user_config(key, json_value.clone()) {
        Ok(_) => {
            anna_ok(format!("Set {} = {}", key, json_value));
            anna_info("Your preference is saved in ~/.config/anna/prefs.yml");
        }
        Err(e) => {
            if e.to_string().contains("Cannot create state directory") {
                anna_warn("I need a little extra permission to save this change safely.");
                anna_info("I'll use your system's standard prompt and come right back.");

                // Try with privilege escalation (handled by caller)
                return Err(e);
            } else {
                anna_warn(format!("Couldn't save that setting: {}", e));
            }
        }
    }

    Ok(())
}

/// Reset configuration (all or specific key)
pub async fn config_reset(key: Option<&str>) -> Result<()> {
    if let Some(specific_key) = key {
        anna_narrative(format!("I'll reset '{}' back to its default...", specific_key));
        reset_user_config(Some(specific_key))?;
        anna_ok(format!("Reset {} to system default", specific_key));
    } else {
        anna_narrative("I'll reset all your preferences back to defaults...");
        reset_user_config(None)?;
        anna_ok("All preferences reset to system defaults");
        anna_info("You can customize again with 'annactl config set' or 'annactl persona set'");
    }

    Ok(())
}

/// Export effective configuration
pub async fn config_export(path: Option<&str>) -> Result<()> {
    anna_narrative("Let me gather your effective configuration...");

    let effective = load_effective_config()?;
    let json = serde_json::to_string_pretty(&effective)?;

    if let Some(file_path) = path {
        std::fs::write(file_path, &json)?;
        anna_ok(format!("Exported configuration to {}", file_path));
    } else {
        // Print to stdout
        println!("{}", json);
    }

    Ok(())
}

/// Import configuration from file
pub async fn config_import(path: &str, replace: bool) -> Result<()> {
    anna_narrative(format!("I'll import settings from {}...", path));

    let content = std::fs::read_to_string(path)?;
    let imported: serde_json::Value = serde_json::from_str(&content)?;

    // Extract values
    if let Some(values) = imported.get("values").and_then(|v| v.as_object()) {
        let mode = if replace { "replacing" } else { "merging with" };
        anna_info(format!("Found {} settings, {} your current preferences", values.len(), mode));

        if replace {
            // Reset first
            reset_user_config(None)?;
        }

        // Import each value
        for (key, value) in values {
            set_user_config(key, value.clone())?;
        }

        anna_ok(format!("Imported {} settings successfully", values.len()));
    } else {
        anna_warn("That file doesn't look like a valid configuration export.");
        anna_info("Make sure it was created with 'annactl config export'.");
    }

    Ok(())
}

/// List all configuration values
pub async fn config_list() -> Result<()> {
    anna_narrative("Here's your current configuration:");

    let effective = load_effective_config()?;

    // Group by prefix
    let mut groups: std::collections::HashMap<String, Vec<(String, serde_json::Value, ConfigOrigin)>> = std::collections::HashMap::new();

    for (key, value) in &effective.values {
        let origin = effective.origins.get(key).copied().unwrap_or(ConfigOrigin::System);
        let prefix = key.split('.').next().unwrap_or("other").to_string();

        groups.entry(prefix)
            .or_insert_with(Vec::new)
            .push((key.clone(), value.clone(), origin));
    }

    // Sort groups
    let mut group_names: Vec<String> = groups.keys().cloned().collect();
    group_names.sort();

    for group_name in group_names {
        println!("\n[{}]", group_name);

        let mut items = groups[&group_name].clone();
        items.sort_by(|a, b| a.0.cmp(&b.0));

        for (key, value, origin) in items {
            let origin_marker = match origin {
                ConfigOrigin::System => "",
                ConfigOrigin::User => " (your preference)",
                ConfigOrigin::Runtime => " (runtime override)",
            };
            println!("  {} = {}{}", key, value, origin_marker);
        }
    }

    println!();
    anna_info("Change settings with: annactl config set <key> <value>");
    anna_info("Reset to defaults with: annactl config reset [<key>]");

    Ok(())
}
