//! Helper status section for status display (v0.0.434).

use super::super::helper_config::HelperConfig;
use super::super::helper_entry::HelperCatalog;
use super::super::helper_manager::HelperManager;
use super::super::helper_state::HelperInstalledBy;
use serde::{Deserialize, Serialize};

/// Helper status section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperStatusSection {
    /// Helpers installed by Anna.
    pub anna_installed: Vec<HelperStatusEntry>,
    /// Helpers installed by user.
    pub user_installed: Vec<HelperStatusEntry>,
    /// Policy summary.
    pub policy: String,
}

impl HelperStatusSection {
    /// Build from manager and config.
    pub fn build(manager: &HelperManager, catalog: &HelperCatalog, config: &HelperConfig) -> Self {
        let mut anna_installed = Vec::new();
        let mut user_installed = Vec::new();

        for (id, state) in &manager.helpers {
            let purpose = catalog
                .get(id)
                .map(|h| h.purpose.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            let entry = HelperStatusEntry {
                id: id.clone(),
                purpose,
                use_count: state.use_count,
                last_used: state.last_used.clone(),
            };

            match state.installed_by {
                HelperInstalledBy::Anna => anna_installed.push(entry),
                HelperInstalledBy::User => user_installed.push(entry),
            }
        }

        Self {
            anna_installed,
            user_installed,
            policy: config.format_summary(),
        }
    }
}

/// Single helper status entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperStatusEntry {
    /// Helper ID.
    pub id: String,
    /// Purpose.
    pub purpose: String,
    /// Usage count.
    pub use_count: u64,
    /// Last used timestamp.
    pub last_used: Option<String>,
}
