//! Helpers information types (v0.0.211).

use crate::helpers::{HelpersRegistry, InstallSource};
use serde::{Deserialize, Serialize};

/// Helper package summary (lite version for snapshot)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperPackageLite {
    pub id: String,
    pub name: String,
    pub available: bool,
    pub source: InstallSource,
}

/// Helpers summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HelpersInfo {
    /// Total helpers tracked
    pub total: usize,
    /// Count by source
    pub anna_installed: usize,
    pub user_installed: usize,
    pub bundled: usize,
    /// List of helpers
    pub list: Vec<HelperPackageLite>,
}

impl HelpersInfo {
    pub fn from_registry(registry: &HelpersRegistry) -> Self {
        let list: Vec<HelperPackageLite> = registry
            .packages
            .iter()
            .map(|p| HelperPackageLite {
                id: p.id.clone(),
                name: p.name.clone(),
                available: p.available,
                source: p.install_source,
            })
            .collect();

        Self {
            total: registry.len(),
            anna_installed: registry.anna_installed().len(),
            user_installed: registry
                .packages
                .iter()
                .filter(|p| p.install_source == InstallSource::User)
                .count(),
            bundled: registry
                .packages
                .iter()
                .filter(|p| p.install_source == InstallSource::Bundled)
                .count(),
            list,
        }
    }
}
