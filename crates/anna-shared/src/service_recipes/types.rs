//! Service-related type definitions (v0.0.214).

use serde::{Deserialize, Serialize};

/// Service action types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceAction {
    /// Start the service
    Start,
    /// Stop the service
    Stop,
    /// Restart the service
    Restart,
    /// Enable at boot
    Enable,
    /// Disable at boot
    Disable,
    /// Reload configuration
    Reload,
}

impl ServiceAction {
    pub fn display_name(&self) -> &'static str {
        match self {
            ServiceAction::Start => "start",
            ServiceAction::Stop => "stop",
            ServiceAction::Restart => "restart",
            ServiceAction::Enable => "enable",
            ServiceAction::Disable => "disable",
            ServiceAction::Reload => "reload",
        }
    }

    /// Get systemctl command
    pub fn systemctl_cmd(&self, service: &str) -> String {
        format!("systemctl {} {}", self.display_name(), service)
    }

    /// Get the opposite action (for rollback)
    pub fn opposite(&self) -> Option<ServiceAction> {
        match self {
            ServiceAction::Start => Some(ServiceAction::Stop),
            ServiceAction::Stop => Some(ServiceAction::Start),
            ServiceAction::Enable => Some(ServiceAction::Disable),
            ServiceAction::Disable => Some(ServiceAction::Enable),
            ServiceAction::Restart => None,
            ServiceAction::Reload => None,
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "start" => Some(ServiceAction::Start),
            "stop" => Some(ServiceAction::Stop),
            "restart" => Some(ServiceAction::Restart),
            "enable" => Some(ServiceAction::Enable),
            "disable" => Some(ServiceAction::Disable),
            "reload" => Some(ServiceAction::Reload),
            _ => None,
        }
    }
}

impl std::fmt::Display for ServiceAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Service category for common services
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceCategory {
    /// System services
    System,
    /// Network services
    Network,
    /// Display/GUI services
    Display,
    /// Audio services
    Audio,
    /// Security services
    Security,
    /// User services
    User,
}

impl ServiceCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            ServiceCategory::System => "System",
            ServiceCategory::Network => "Network",
            ServiceCategory::Display => "Display",
            ServiceCategory::Audio => "Audio",
            ServiceCategory::Security => "Security",
            ServiceCategory::User => "User",
        }
    }
}

/// Risk level for service operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceRisk {
    /// Safe to modify
    Low,
    /// May cause issues
    Medium,
    /// Critical service - warn user
    High,
    /// Protected - refuse to modify
    Protected,
}

impl ServiceRisk {
    pub fn display_name(&self) -> &'static str {
        match self {
            ServiceRisk::Low => "low",
            ServiceRisk::Medium => "medium",
            ServiceRisk::High => "high",
            ServiceRisk::Protected => "protected",
        }
    }

    /// Can we proceed with user confirmation?
    pub fn allows_modification(&self) -> bool {
        !matches!(self, ServiceRisk::Protected)
    }
}
