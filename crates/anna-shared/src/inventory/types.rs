//! Inventory types (v0.0.188).

use serde::{Deserialize, Serialize};

use super::constants::INVENTORY_TTL_SECS;
use super::helpers::current_timestamp;

/// State of an inventory item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum InventoryState {
    /// Tool is installed and verified
    Installed,
    /// Tool was installed but may have been removed
    #[default]
    Stale,
    /// Tool is not installed
    NotInstalled,
}

/// An item in the inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    /// Tool name (e.g., "vim")
    pub name: String,
    /// Full path to the binary (if installed)
    pub path: Option<String>,
    /// Current state
    pub state: InventoryState,
    /// Unix timestamp when last verified
    pub verified_at: u64,
    /// Seconds until considered stale
    pub stale_after: u64,
}

impl InventoryItem {
    /// Create a new installed item
    pub fn installed(name: &str, path: &str) -> Self {
        Self {
            name: name.to_string(),
            path: Some(path.to_string()),
            state: InventoryState::Installed,
            verified_at: current_timestamp(),
            stale_after: INVENTORY_TTL_SECS, // v0.0.41: 10 minutes
        }
    }

    /// Create a not-installed item
    pub fn not_installed(name: &str) -> Self {
        Self {
            name: name.to_string(),
            path: None,
            state: InventoryState::NotInstalled,
            verified_at: current_timestamp(),
            stale_after: INVENTORY_TTL_SECS, // v0.0.41: 10 minutes
        }
    }

    /// Check if this item is stale
    pub fn is_stale(&self) -> bool {
        let now = current_timestamp();
        now.saturating_sub(self.verified_at) > self.stale_after
    }

    /// Mark as stale
    pub fn mark_stale(&mut self) {
        self.state = InventoryState::Stale;
    }
}
