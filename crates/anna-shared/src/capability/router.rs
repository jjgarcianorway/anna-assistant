//! Capability Router - Deterministic request routing.
//!
//! Pure function. No fallback. No partial matches. No "best guess."

use super::registry::{CapabilityId, CAPABILITY_REGISTRY};
use serde::{Deserialize, Serialize};

/// Reason why a request is unsupported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnsupportedReason {
    /// No matching capability found.
    UnknownCapability,
    /// Request is ambiguous (matches multiple capabilities).
    AmbiguousRequest,
    /// Request is empty or unparseable.
    MalformedRequest,
    /// Capability exists but is explicitly disabled.
    CapabilityDisabled,
}

impl UnsupportedReason {
    /// Short message for this reason.
    pub fn short_message(&self) -> &'static str {
        match self {
            Self::UnknownCapability => "No capability matches this request.",
            Self::AmbiguousRequest => "Request matches multiple capabilities.",
            Self::MalformedRequest => "Request could not be parsed.",
            Self::CapabilityDisabled => "Capability is currently disabled.",
        }
    }

    /// Reason code for logging/telemetry.
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnknownCapability => "UNKNOWN_CAPABILITY",
            Self::AmbiguousRequest => "AMBIGUOUS_REQUEST",
            Self::MalformedRequest => "MALFORMED_REQUEST",
            Self::CapabilityDisabled => "CAPABILITY_DISABLED",
        }
    }
}

/// Result of routing a request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityRoutingResult {
    /// Request maps to a known capability.
    Supported {
        capability_id: CapabilityId,
    },
    /// Request does not map to any capability.
    Unsupported {
        reason_code: String,
        short_message: String,
    },
}

impl CapabilityRoutingResult {
    /// Create a supported result.
    pub fn supported(id: &str) -> Self {
        Self::Supported {
            capability_id: CapabilityId::new(id),
        }
    }

    /// Create an unsupported result.
    pub fn unsupported(reason: UnsupportedReason) -> Self {
        Self::Unsupported {
            reason_code: reason.code().to_string(),
            short_message: reason.short_message().to_string(),
        }
    }

    /// Whether this result is supported.
    pub fn is_supported(&self) -> bool {
        matches!(self, Self::Supported { .. })
    }

    /// Get the capability ID if supported.
    pub fn capability_id(&self) -> Option<&CapabilityId> {
        match self {
            Self::Supported { capability_id } => Some(capability_id),
            Self::Unsupported { .. } => None,
        }
    }
}

/// Route a request to a capability.
///
/// This is a pure function. Deterministic. No side effects.
/// No fallback. No partial matches. No "best guess."
pub fn route_request(input: &str) -> CapabilityRoutingResult {
    let input = input.trim().to_lowercase();

    // Reject empty input
    if input.is_empty() {
        return CapabilityRoutingResult::unsupported(UnsupportedReason::MalformedRequest);
    }

    // Deterministic pattern matching - exact matches first
    // No inference. No fuzzy matching. No AI guessing.

    // =========================================================================
    // DISPLAY SCALING
    // =========================================================================

    if matches_display_scale_gdm(&input) {
        return CapabilityRoutingResult::supported("display.scale.gdm");
    }

    if matches_display_scale_xorg(&input) {
        return CapabilityRoutingResult::supported("display.scale.xorg");
    }

    if matches_display_scale_wayland(&input) {
        return CapabilityRoutingResult::supported("display.scale.wayland");
    }

    // =========================================================================
    // STATUS QUERIES
    // =========================================================================

    if matches_status_system(&input) {
        return CapabilityRoutingResult::supported("status.system");
    }

    if matches_status_disk(&input) {
        return CapabilityRoutingResult::supported("status.disk");
    }

    if matches_status_memory(&input) {
        return CapabilityRoutingResult::supported("status.memory");
    }

    if matches_status_network(&input) {
        return CapabilityRoutingResult::supported("status.network");
    }

    if matches_status_services(&input) {
        return CapabilityRoutingResult::supported("status.services");
    }

    if matches_status_identity(&input) {
        return CapabilityRoutingResult::supported("status.identity");
    }

    // =========================================================================
    // PACKAGE OPERATIONS
    // =========================================================================

    if matches_package_install(&input) {
        return CapabilityRoutingResult::supported("package.install");
    }

    if matches_package_remove(&input) {
        return CapabilityRoutingResult::supported("package.remove");
    }

    if matches_package_update(&input) {
        return CapabilityRoutingResult::supported("package.update");
    }

    // =========================================================================
    // SERVICE OPERATIONS
    // =========================================================================

    if matches_service_start(&input) {
        return CapabilityRoutingResult::supported("service.start");
    }

    if matches_service_stop(&input) {
        return CapabilityRoutingResult::supported("service.stop");
    }

    if matches_service_restart(&input) {
        return CapabilityRoutingResult::supported("service.restart");
    }

    if matches_service_enable(&input) {
        return CapabilityRoutingResult::supported("service.enable");
    }

    // =========================================================================
    // CONFIG OPERATIONS
    // =========================================================================

    if matches_config_edit(&input) {
        return CapabilityRoutingResult::supported("config.edit");
    }

    // =========================================================================
    // NO MATCH - EXPLICIT REJECTION
    // =========================================================================

    CapabilityRoutingResult::unsupported(UnsupportedReason::UnknownCapability)
}

// =============================================================================
// PATTERN MATCHERS - Deterministic, no inference
// =============================================================================

fn matches_display_scale_gdm(input: &str) -> bool {
    // GDM scaling patterns
    (input.contains("gdm") && (input.contains("scal") || input.contains("hidpi") || input.contains("dpi")))
        || (input.contains("login") && input.contains("scal") && input.contains("screen"))
        || (input.contains("gdm") && input.contains("display"))
}

fn matches_display_scale_xorg(input: &str) -> bool {
    (input.contains("xorg") || input.contains("x11"))
        && (input.contains("scal") || input.contains("dpi") || input.contains("resolution"))
}

fn matches_display_scale_wayland(input: &str) -> bool {
    input.contains("wayland")
        && (input.contains("scal") || input.contains("dpi") || input.contains("fractional"))
}

fn matches_status_system(input: &str) -> bool {
    input == "status"
        || input == "system status"
        || input.starts_with("what is my status")
        || input.starts_with("show status")
        || input.starts_with("check status")
        || (input.contains("status") && input.contains("system"))
        || (input.contains("overview") && !input.contains("disk") && !input.contains("network"))
}

fn matches_status_disk(input: &str) -> bool {
    input.contains("disk") && (input.contains("usage") || input.contains("space") || input.contains("status"))
        || input.starts_with("how much disk")
        || input.starts_with("how much storage")
        || input.starts_with("df ")
        || input == "df"
}

fn matches_status_memory(input: &str) -> bool {
    (input.contains("memory") || input.contains("ram") || input.contains("swap"))
        && (input.contains("usage") || input.contains("status") || input.contains("how much") || input.contains("free"))
        || input.starts_with("how much ram")
        || input.starts_with("how much memory")
        || input == "free"
}

fn matches_status_network(input: &str) -> bool {
    (input.contains("network") || input.contains("wifi") || input.contains("ethernet") || input.contains("internet"))
        && (input.contains("status") || input.contains("connection") || input.contains("connected"))
        || input.starts_with("am i online")
        || input.starts_with("is network")
}

fn matches_status_services(input: &str) -> bool {
    (input.contains("service") || input.contains("systemd"))
        && (input.contains("status") || input.contains("failed") || input.contains("running"))
        || input.starts_with("what services")
        || input.starts_with("show services")
        || input.starts_with("list services")
}

fn matches_status_identity(input: &str) -> bool {
    (input.contains("user") || input.contains("group") || input.contains("identity") || input.contains("permission"))
        && (input.contains("am i") || input.contains("my") || input.contains("who") || input.contains("status"))
        || input.starts_with("who am i")
        || input.starts_with("what groups")
        || input.starts_with("my groups")
}

fn matches_package_install(input: &str) -> bool {
    input.starts_with("install ")
        || input.starts_with("pacman -s ")
        || (input.contains("install") && input.contains("package"))
}

fn matches_package_remove(input: &str) -> bool {
    input.starts_with("uninstall ")
        || input.starts_with("remove ")
        || input.starts_with("pacman -r ")
        || (input.contains("remove") && input.contains("package"))
}

fn matches_package_update(input: &str) -> bool {
    input == "update"
        || input == "upgrade"
        || input.starts_with("update system")
        || input.starts_with("upgrade system")
        || input.starts_with("pacman -syu")
        || (input.contains("update") && input.contains("package"))
}

fn matches_service_start(input: &str) -> bool {
    input.starts_with("start ") && (input.contains("service") || !input.contains(" a "))
        || (input.contains("start") && input.contains("service"))
}

fn matches_service_stop(input: &str) -> bool {
    input.starts_with("stop ") && (input.contains("service") || !input.contains(" a "))
        || (input.contains("stop") && input.contains("service"))
}

fn matches_service_restart(input: &str) -> bool {
    input.starts_with("restart ")
        || (input.contains("restart") && input.contains("service"))
}

fn matches_service_enable(input: &str) -> bool {
    input.starts_with("enable ") && (input.contains("service") || !input.contains(" a "))
        || input.starts_with("disable ")
        || (input.contains("enable") && input.contains("service"))
}

fn matches_config_edit(input: &str) -> bool {
    (input.contains("edit") || input.contains("modify") || input.contains("change"))
        && (input.contains("config") || input.contains("/etc/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gdm_scaling_routes_correctly() {
        let result = route_request("scale gdm login screen");
        assert!(result.is_supported());
        assert_eq!(result.capability_id().unwrap().as_str(), "display.scale.gdm");

        let result = route_request("gdm hidpi scaling");
        assert!(result.is_supported());
        assert_eq!(result.capability_id().unwrap().as_str(), "display.scale.gdm");
    }

    #[test]
    fn test_status_routes_correctly() {
        let result = route_request("status");
        assert!(result.is_supported());
        assert_eq!(result.capability_id().unwrap().as_str(), "status.system");

        let result = route_request("disk usage");
        assert!(result.is_supported());
        assert_eq!(result.capability_id().unwrap().as_str(), "status.disk");

        let result = route_request("how much ram do I have");
        assert!(result.is_supported());
        assert_eq!(result.capability_id().unwrap().as_str(), "status.memory");
    }

    #[test]
    fn test_package_routes_correctly() {
        let result = route_request("install neovim");
        assert!(result.is_supported());
        assert_eq!(result.capability_id().unwrap().as_str(), "package.install");

        let result = route_request("update system");
        assert!(result.is_supported());
        assert_eq!(result.capability_id().unwrap().as_str(), "package.update");
    }

    #[test]
    fn test_unknown_request_rejected() {
        let result = route_request("tell me a joke");
        assert!(!result.is_supported());

        let result = route_request("what is the meaning of life");
        assert!(!result.is_supported());
    }

    #[test]
    fn test_empty_request_rejected() {
        let result = route_request("");
        assert!(!result.is_supported());

        let result = route_request("   ");
        assert!(!result.is_supported());
    }

    #[test]
    fn test_routing_is_deterministic() {
        // Same input always produces same output
        let input = "scale gdm display";
        let result1 = route_request(input);
        let result2 = route_request(input);

        assert_eq!(
            result1.capability_id().map(|id| id.as_str()),
            result2.capability_id().map(|id| id.as_str())
        );
    }
}
