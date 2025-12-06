//! Service configuration recipes.
//!
//! v0.0.98: Safe systemd service management.
//!
//! # Supported Operations
//! - Start/stop/restart services
//! - Enable/disable at boot
//! - Check status
//!
//! # Safety
//! - All changes require user confirmation
//! - Only manages user-requested services
//! - No system-critical services can be disabled

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
            ServiceAction::Restart => None, // No opposite
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

/// Known services with metadata
pub fn known_services() -> Vec<ServiceRecipe> {
    vec![
        // Network
        ServiceRecipe::new("sshd.service", "SSH Server", ServiceCategory::Network,
            "OpenSSH server daemon")
            .with_risk(ServiceRisk::Medium)
            .with_aliases(&["ssh", "openssh"]),
        ServiceRecipe::new("NetworkManager.service", "NetworkManager", ServiceCategory::Network,
            "Network configuration daemon")
            .with_risk(ServiceRisk::High)
            .with_aliases(&["network", "nm"]),
        ServiceRecipe::new("systemd-networkd.service", "systemd-networkd", ServiceCategory::Network,
            "Systemd network daemon")
            .with_risk(ServiceRisk::High),
        ServiceRecipe::new("cups.service", "CUPS", ServiceCategory::Network,
            "Printing service")
            .with_risk(ServiceRisk::Low)
            .with_aliases(&["printing", "printer"]),
        ServiceRecipe::new("bluetooth.service", "Bluetooth", ServiceCategory::Network,
            "Bluetooth daemon")
            .with_risk(ServiceRisk::Low),

        // Display
        ServiceRecipe::new("gdm.service", "GDM", ServiceCategory::Display,
            "GNOME Display Manager")
            .with_risk(ServiceRisk::High)
            .with_aliases(&["gnome-dm"]),
        ServiceRecipe::new("sddm.service", "SDDM", ServiceCategory::Display,
            "Simple Desktop Display Manager")
            .with_risk(ServiceRisk::High),
        ServiceRecipe::new("lightdm.service", "LightDM", ServiceCategory::Display,
            "Light Display Manager")
            .with_risk(ServiceRisk::High),

        // Audio
        ServiceRecipe::new("pipewire.service", "PipeWire", ServiceCategory::Audio,
            "Audio/video daemon")
            .with_risk(ServiceRisk::Low)
            .with_aliases(&["audio"]),
        ServiceRecipe::new("pulseaudio.service", "PulseAudio", ServiceCategory::Audio,
            "Sound server")
            .with_risk(ServiceRisk::Low),

        // System
        ServiceRecipe::new("cronie.service", "Cron", ServiceCategory::System,
            "Task scheduler")
            .with_risk(ServiceRisk::Medium)
            .with_aliases(&["cron", "crond"]),
        ServiceRecipe::new("docker.service", "Docker", ServiceCategory::System,
            "Container runtime")
            .with_risk(ServiceRisk::Low),
        ServiceRecipe::new("libvirtd.service", "Libvirt", ServiceCategory::System,
            "Virtualization daemon")
            .with_risk(ServiceRisk::Low)
            .with_aliases(&["libvirt", "virt"]),

        // Protected - refuse to modify
        ServiceRecipe::new("systemd-journald.service", "Journald", ServiceCategory::System,
            "System logging")
            .with_risk(ServiceRisk::Protected),
        ServiceRecipe::new("dbus.service", "D-Bus", ServiceCategory::System,
            "Message bus")
            .with_risk(ServiceRisk::Protected),
        ServiceRecipe::new("systemd-udevd.service", "Udev", ServiceCategory::System,
            "Device manager")
            .with_risk(ServiceRisk::Protected),
    ]
}

/// Find a service recipe by name or alias
pub fn find_service(name: &str) -> Option<ServiceRecipe> {
    let name_lower = name.to_lowercase();
    known_services()
        .into_iter()
        .find(|s| {
            s.name.to_lowercase().starts_with(&name_lower)
                || s.display_name.to_lowercase() == name_lower
                || s.aliases.iter().any(|a| a.to_lowercase() == name_lower)
        })
}

/// Generate confirmation prompt for service action
pub fn confirmation_prompt(recipe: &ServiceRecipe, action: ServiceAction) -> String {
    let mut prompt = format!(
        "{} {}?\n\
         Service: {}\n\
         Description: {}\n",
        action.display_name().chars().next().unwrap().to_uppercase().collect::<String>()
            + &action.display_name()[1..],
        recipe.display_name,
        recipe.name,
        recipe.description
    );

    // Add risk warning if needed
    match recipe.risk {
        ServiceRisk::High => {
            prompt.push_str("\nWARNING: This is a critical service. ");
            prompt.push_str("Modifying it may affect system stability.\n");
        }
        ServiceRisk::Protected => {
            prompt.push_str("\nERROR: This service is protected and cannot be modified.\n");
            return prompt;
        }
        _ => {}
    }

    // Add rollback info
    if let Some(rollback) = recipe.rollback_command(action) {
        prompt.push_str(&format!("To undo: sudo {}\n", rollback));
    }

    prompt.push_str("\nProceed? [y/N]");
    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_action_display() {
        assert_eq!(ServiceAction::Start.display_name(), "start");
        assert_eq!(ServiceAction::Enable.display_name(), "enable");
    }

    #[test]
    fn test_service_action_opposite() {
        assert_eq!(ServiceAction::Start.opposite(), Some(ServiceAction::Stop));
        assert_eq!(ServiceAction::Enable.opposite(), Some(ServiceAction::Disable));
        assert_eq!(ServiceAction::Restart.opposite(), None);
    }

    #[test]
    fn test_find_service() {
        assert!(find_service("sshd").is_some());
        assert!(find_service("ssh").is_some()); // alias
        assert!(find_service("docker").is_some());
        assert!(find_service("nonexistent").is_none());
    }

    #[test]
    fn test_protected_service() {
        let dbus = find_service("dbus").unwrap();
        assert_eq!(dbus.risk, ServiceRisk::Protected);
        assert!(!dbus.risk.allows_modification());
    }

    #[test]
    fn test_systemctl_command() {
        let docker = find_service("docker").unwrap();
        assert_eq!(docker.command_for(ServiceAction::Start), "systemctl start docker.service");
    }

    #[test]
    fn test_known_services_count() {
        let services = known_services();
        assert!(services.len() >= 10);
    }
}
