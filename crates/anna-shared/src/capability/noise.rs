//! Noise Containment - Warnings only where relevant.
//!
//! Each capability declares which warning categories it cares about.
//! Warnings outside those categories are filtered out.

use super::registry::{CapabilityId, WarningCategory, CAPABILITY_REGISTRY};
use serde::{Deserialize, Serialize};

/// A system warning that may or may not be relevant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemWarning {
    /// Warning category.
    pub category: WarningCategory,
    /// Warning message.
    pub message: String,
    /// Severity (0-100, higher = more severe).
    pub severity: u8,
}

impl SystemWarning {
    /// Create a new warning.
    pub fn new(category: WarningCategory, message: &str, severity: u8) -> Self {
        Self {
            category,
            message: message.to_string(),
            severity,
        }
    }
}

/// Result of filtering warnings for relevance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarningRelevance {
    /// Warnings that are relevant to the capability.
    pub relevant: Vec<SystemWarning>,
    /// Warnings that were filtered out.
    pub filtered: usize,
}

impl WarningRelevance {
    /// Whether any relevant warnings exist.
    pub fn has_warnings(&self) -> bool {
        !self.relevant.is_empty()
    }

    /// Count of relevant warnings.
    pub fn count(&self) -> usize {
        self.relevant.len()
    }
}

/// Filter warnings for relevance to a capability.
///
/// Only warnings in categories the capability cares about are included.
/// This prevents noise spillover between unrelated domains.
pub fn filter_warnings(
    capability_id: &CapabilityId,
    warnings: &[SystemWarning],
) -> WarningRelevance {
    // Look up capability in registry
    let capability = match CAPABILITY_REGISTRY.get(capability_id) {
        Some(cap) => cap,
        None => {
            // Unknown capability - return no warnings
            return WarningRelevance {
                relevant: Vec::new(),
                filtered: warnings.len(),
            };
        }
    };

    // Filter by relevant categories
    let relevant_categories = &capability.relevant_warnings;

    let mut relevant = Vec::new();
    let mut filtered = 0;

    for warning in warnings {
        // Check if warning category is relevant
        let is_relevant = relevant_categories.contains(&WarningCategory::All)
            || relevant_categories.contains(&warning.category);

        if is_relevant {
            relevant.push(warning.clone());
        } else {
            filtered += 1;
        }
    }

    // Sort by severity (highest first)
    relevant.sort_by(|a, b| b.severity.cmp(&a.severity));

    WarningRelevance { relevant, filtered }
}

/// Get all warning categories relevant to a capability.
pub fn get_relevant_categories(capability_id: &CapabilityId) -> Vec<WarningCategory> {
    CAPABILITY_REGISTRY
        .get(capability_id)
        .map(|cap| cap.relevant_warnings.clone())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_warnings() -> Vec<SystemWarning> {
        vec![
            SystemWarning::new(WarningCategory::Storage, "Disk usage at 85%", 70),
            SystemWarning::new(WarningCategory::Network, "DNS resolution slow", 30),
            SystemWarning::new(WarningCategory::Service, "docker.service failed", 80),
            SystemWarning::new(WarningCategory::Display, "HiDPI not configured", 40),
            SystemWarning::new(WarningCategory::Security, "SSH root login enabled", 90),
        ]
    }

    #[test]
    fn test_disk_capability_sees_only_storage() {
        let id = CapabilityId::new("status.disk");
        let result = filter_warnings(&id, &test_warnings());

        assert_eq!(result.count(), 1);
        assert_eq!(result.filtered, 4);
        assert_eq!(result.relevant[0].message, "Disk usage at 85%");
    }

    #[test]
    fn test_network_capability_sees_only_network() {
        let id = CapabilityId::new("status.network");
        let result = filter_warnings(&id, &test_warnings());

        assert_eq!(result.count(), 1);
        assert_eq!(result.filtered, 4);
        assert_eq!(result.relevant[0].message, "DNS resolution slow");
    }

    #[test]
    fn test_system_status_sees_all() {
        let id = CapabilityId::new("status.system");
        let result = filter_warnings(&id, &test_warnings());

        // status.system has WarningCategory::All
        assert_eq!(result.count(), 5);
        assert_eq!(result.filtered, 0);
    }

    #[test]
    fn test_display_capability_sees_display() {
        let id = CapabilityId::new("display.scale.gdm");
        let result = filter_warnings(&id, &test_warnings());

        assert_eq!(result.count(), 1);
        assert_eq!(result.filtered, 4);
        assert_eq!(result.relevant[0].message, "HiDPI not configured");
    }

    #[test]
    fn test_unknown_capability_filters_all() {
        let id = CapabilityId::new("nonexistent.capability");
        let result = filter_warnings(&id, &test_warnings());

        assert_eq!(result.count(), 0);
        assert_eq!(result.filtered, 5);
    }

    #[test]
    fn test_warnings_sorted_by_severity() {
        let id = CapabilityId::new("status.system");
        let result = filter_warnings(&id, &test_warnings());

        // Should be sorted highest to lowest
        assert_eq!(result.relevant[0].severity, 90); // Security
        assert_eq!(result.relevant[1].severity, 80); // Service
        assert_eq!(result.relevant[2].severity, 70); // Storage
        assert_eq!(result.relevant[3].severity, 40); // Display
        assert_eq!(result.relevant[4].severity, 30); // Network
    }

    #[test]
    fn test_identity_sees_security_and_status() {
        let id = CapabilityId::new("status.identity");
        let result = filter_warnings(&id, &test_warnings());

        // status.identity has StatusIdentity and Security
        assert_eq!(result.count(), 1); // Only Security warning matches
        assert_eq!(result.relevant[0].message, "SSH root login enabled");
    }

    #[test]
    fn test_empty_warnings() {
        let id = CapabilityId::new("status.disk");
        let result = filter_warnings(&id, &[]);

        assert_eq!(result.count(), 0);
        assert_eq!(result.filtered, 0);
        assert!(!result.has_warnings());
    }
}
