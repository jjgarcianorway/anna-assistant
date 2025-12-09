//! Inventory persistence (v0.0.188).

use std::path::PathBuf;

use super::cache::InventoryCache;

/// Get inventory cache file path
pub fn inventory_path() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".anna")
        .join("inventory.json")
}

/// Load inventory cache from disk
pub fn load_inventory() -> Option<InventoryCache> {
    let path = inventory_path();
    if !path.exists() {
        return None;
    }
    let data = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Save inventory cache to disk
pub fn save_inventory(cache: &InventoryCache) -> std::io::Result<()> {
    let path = inventory_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(cache)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, json)
}

/// Clear inventory cache
pub fn clear_inventory() -> std::io::Result<()> {
    let path = inventory_path();
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// Load or create fresh inventory (with VIP tools check)
pub fn load_or_create_inventory() -> InventoryCache {
    if let Some(mut cache) = load_inventory() {
        cache.mark_stale_items();
        return cache;
    }

    // Create fresh inventory with VIP tools
    let mut cache = InventoryCache::new();
    cache.refresh_vip_tools();
    let _ = save_inventory(&cache);
    cache
}

/// Filter options to only installed ones
pub fn filter_installed_options(options: &[&str]) -> Vec<String> {
    let cache = load_or_create_inventory();
    options
        .iter()
        .filter(|&&opt| cache.is_installed(opt).unwrap_or(false))
        .map(|&s| s.to_string())
        .collect()
}
