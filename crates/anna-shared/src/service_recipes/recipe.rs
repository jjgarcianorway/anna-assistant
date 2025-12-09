//! ServiceRecipe struct definition (v0.0.214).

use serde::{Deserialize, Serialize};

use super::types::{ServiceAction, ServiceCategory, ServiceRisk};

/// A service recipe with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRecipe {
    /// Service unit name (e.g., "sshd.service")
    pub name: String,
    /// Display name
    pub display_name: String,
    /// Category
    pub category: ServiceCategory,
    /// Description
    pub description: String,
    /// Risk level
    pub risk: ServiceRisk,
    /// Common alternative names
    pub aliases: Vec<String>,
}

impl ServiceRecipe {
    /// Create a new service recipe
    pub fn new(name: &str, display_name: &str, category: ServiceCategory, desc: &str) -> Self {
        Self {
            name: name.to_string(),
            display_name: display_name.to_string(),
            category,
            description: desc.to_string(),
            risk: ServiceRisk::Low,
            aliases: Vec::new(),
        }
    }

    /// Set risk level
    pub fn with_risk(mut self, risk: ServiceRisk) -> Self {
        self.risk = risk;
        self
    }

    /// Add alias names
    pub fn with_aliases(mut self, aliases: &[&str]) -> Self {
        self.aliases = aliases.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Get command for an action
    pub fn command_for(&self, action: ServiceAction) -> String {
        action.systemctl_cmd(&self.name)
    }

    /// Get rollback command if available
    pub fn rollback_command(&self, action: ServiceAction) -> Option<String> {
        action.opposite().map(|a| a.systemctl_cmd(&self.name))
    }
}
