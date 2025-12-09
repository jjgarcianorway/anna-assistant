//! Helpers persistence (v0.0.221).

use std::path::PathBuf;

use super::registry::HelpersRegistry;

/// Path to the helpers store file.
pub fn helpers_store_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    PathBuf::from(home).join(".anna").join("helpers.json")
}

/// Load helpers registry from disk.
pub fn load_helpers() -> HelpersRegistry {
    let path = helpers_store_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(registry) = serde_json::from_str(&content) {
                return registry;
            }
        }
    }
    HelpersRegistry::new()
}

/// Save helpers registry to disk.
pub fn save_helpers(registry: &HelpersRegistry) -> std::io::Result<()> {
    let path = helpers_store_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(registry).map_err(|e| std::io::Error::other(e))?;
    std::fs::write(&path, content)
}

/// Clear helpers store (for reset) (v0.0.28)
pub fn clear_helpers_store() -> std::io::Result<()> {
    let path = helpers_store_path();
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}
