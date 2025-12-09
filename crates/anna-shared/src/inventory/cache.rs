//! Inventory cache (v0.0.188).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::constants::{INVENTORY_TTL_SECS, VIP_TOOLS};
use super::helpers::{check_tool_installed, current_timestamp};
use super::system_info::SystemInfo;
use super::types::{InventoryItem, InventoryState};

/// Inventory cache for installed tools
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InventoryCache {
    /// Map of tool name -> inventory item
    pub items: BTreeMap<String, InventoryItem>,
    /// Unix timestamp when cache was last updated
    pub last_updated: u64,
    /// Version of the inventory format
    #[serde(default)]
    pub version: u8,
    /// System information (v0.0.41)
    #[serde(default)]
    pub system_info: Option<SystemInfo>,
}

impl InventoryCache {
    /// Create new empty cache
    pub fn new() -> Self {
        Self {
            items: BTreeMap::new(),
            last_updated: current_timestamp(),
            version: 1,
            system_info: None,
        }
    }

    /// Check if a tool is installed (from cache)
    pub fn is_installed(&self, name: &str) -> Option<bool> {
        self.items
            .get(name)
            .map(|item| item.state == InventoryState::Installed && !item.is_stale())
    }

    /// Get all installed tools
    pub fn installed_tools(&self) -> Vec<&str> {
        self.items
            .iter()
            .filter(|(_, item)| item.state == InventoryState::Installed && !item.is_stale())
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Get installed editors only
    pub fn installed_editors(&self) -> Vec<&str> {
        let editor_names = ["vim", "vi", "nano", "emacs", "nvim", "code", "micro"];
        self.items
            .iter()
            .filter(|(name, item)| {
                editor_names.contains(&name.as_str())
                    && item.state == InventoryState::Installed
                    && !item.is_stale()
            })
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Verify and update a single tool
    pub fn verify_tool(&mut self, name: &str) -> bool {
        if let Some(path) = check_tool_installed(name) {
            self.items
                .insert(name.to_string(), InventoryItem::installed(name, &path));
            true
        } else {
            self.items
                .insert(name.to_string(), InventoryItem::not_installed(name));
            false
        }
    }

    /// Refresh VIP tools
    pub fn refresh_vip_tools(&mut self) {
        for &tool in VIP_TOOLS {
            self.verify_tool(tool);
        }
        self.last_updated = current_timestamp();
    }

    /// Refresh system info (v0.0.41)
    pub fn refresh_system_info(&mut self) {
        self.system_info = Some(SystemInfo::collect());
        self.last_updated = current_timestamp();
    }

    /// Full refresh: VIP tools + system info (v0.0.41)
    pub fn full_refresh(&mut self) {
        self.refresh_vip_tools();
        self.refresh_system_info();
    }

    /// Get system info (collecting if not present)
    pub fn get_system_info(&mut self) -> &SystemInfo {
        if self.system_info.is_none() {
            self.refresh_system_info();
        }
        self.system_info.as_ref().unwrap()
    }

    /// Mark stale items
    pub fn mark_stale_items(&mut self) {
        for item in self.items.values_mut() {
            if item.is_stale() {
                item.mark_stale();
            }
        }
    }

    /// Count of installed tools
    pub fn installed_count(&self) -> usize {
        self.items
            .values()
            .filter(|item| item.state == InventoryState::Installed)
            .count()
    }
}

/// Check if inventory cache is fresh (not stale) - v0.0.41
pub fn is_inventory_fresh(cache: &InventoryCache) -> bool {
    let now = current_timestamp();
    now.saturating_sub(cache.last_updated) <= INVENTORY_TTL_SECS
}
