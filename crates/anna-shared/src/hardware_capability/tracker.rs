//! Hardware capability tracker

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::capability::HardwareCapability;
use super::types::{HardwareCategory, HardwareStatus};

/// Hardware capability tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HardwareCapabilityTracker {
    /// All capabilities
    pub capabilities: Vec<HardwareCapability>,
    /// Count by category
    pub by_category: HashMap<String, u64>,
    /// Count by status
    pub by_status: HashMap<String, u64>,
    /// Last full scan
    pub last_scan: Option<u64>,
}

impl HardwareCapabilityTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a capability
    pub fn register(&mut self, capability: HardwareCapability) {
        *self.by_category.entry(capability.category.name().to_string()).or_insert(0) += 1;
        *self.by_status.entry(capability.status.name().to_string()).or_insert(0) += 1;
        self.capabilities.push(capability);
    }

    /// Update capability status
    pub fn update_status(&mut self, name: &str, status: HardwareStatus, timestamp: u64) -> bool {
        let found = self.capabilities.iter().position(|c| c.name == name);
        if let Some(idx) = found {
            let old_status = self.capabilities[idx].status;
            if let Some(count) = self.by_status.get_mut(old_status.name()) {
                *count = count.saturating_sub(1);
            }
            *self.by_status.entry(status.name().to_string()).or_insert(0) += 1;

            self.capabilities[idx].status = status;
            self.capabilities[idx].last_check = timestamp;
            true
        } else {
            false
        }
    }

    /// Get capability by name
    pub fn get(&self, name: &str) -> Option<&HardwareCapability> {
        self.capabilities.iter().find(|c| c.name == name)
    }

    /// Check if capability exists
    pub fn has(&self, name: &str) -> bool {
        self.capabilities
            .iter()
            .any(|c| c.name == name && c.status == HardwareStatus::Detected)
    }

    /// Check if helper is useful
    pub fn is_helper_useful(&self, helper: &str) -> bool {
        self.capabilities.iter().any(|c| {
            c.status == HardwareStatus::Detected
                && c.relevant_helpers.iter().any(|h| h == helper)
        })
    }

    /// Get useless helpers (no hardware)
    pub fn useless_helpers<'a>(&self, proposed: &'a [String]) -> Vec<&'a String> {
        proposed
            .iter()
            .filter(|h| !self.is_helper_useful(h))
            .collect()
    }

    /// Get capabilities by category
    pub fn by_hw_category(&self, category: HardwareCategory) -> Vec<&HardwareCapability> {
        self.capabilities.iter().filter(|c| c.category == category).collect()
    }

    /// Get detected capabilities
    pub fn detected(&self) -> Vec<&HardwareCapability> {
        self.capabilities.iter().filter(|c| c.status == HardwareStatus::Detected).collect()
    }

    /// Get missing capabilities
    pub fn not_detected(&self) -> Vec<&HardwareCapability> {
        self.capabilities.iter().filter(|c| c.status == HardwareStatus::NotDetected).collect()
    }

    /// Total capability count
    pub fn total_count(&self) -> usize {
        self.capabilities.len()
    }

    /// Detected count
    pub fn detected_count(&self) -> usize {
        self.capabilities.iter().filter(|c| c.status == HardwareStatus::Detected).count()
    }

    /// Record a full scan
    pub fn record_scan(&mut self, timestamp: u64) {
        self.last_scan = Some(timestamp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::{HardwareCategory, HardwareStatus};

    fn make_capability(name: &str, category: HardwareCategory, status: HardwareStatus) -> HardwareCapability {
        HardwareCapability {
            name: name.to_string(),
            category,
            status,
            device: Some("Test Device".to_string()),
            last_check: 1234567890,
            relevant_helpers: vec!["helper1".to_string()],
        }
    }

    #[test]
    fn test_register_capability() {
        let mut tracker = HardwareCapabilityTracker::new();
        tracker.register(make_capability("ethernet", HardwareCategory::Network, HardwareStatus::Detected));

        assert_eq!(tracker.total_count(), 1);
        assert!(tracker.get("ethernet").is_some());
    }

    #[test]
    fn test_has_capability() {
        let mut tracker = HardwareCapabilityTracker::new();
        tracker.register(make_capability("ethernet", HardwareCategory::Network, HardwareStatus::Detected));
        tracker.register(make_capability("wifi", HardwareCategory::Wireless, HardwareStatus::NotDetected));

        assert!(tracker.has("ethernet"));
        assert!(!tracker.has("wifi"));
    }

    #[test]
    fn test_update_status() {
        let mut tracker = HardwareCapabilityTracker::new();
        tracker.register(make_capability("ethernet", HardwareCategory::Network, HardwareStatus::Detected));

        assert!(tracker.update_status("ethernet", HardwareStatus::Disabled, 2000));
        assert_eq!(tracker.get("ethernet").unwrap().status, HardwareStatus::Disabled);
    }

    #[test]
    fn test_is_helper_useful() {
        let mut tracker = HardwareCapabilityTracker::new();
        let mut cap = make_capability("ethernet", HardwareCategory::Network, HardwareStatus::Detected);
        cap.relevant_helpers = vec!["ethtool".to_string()];
        tracker.register(cap);

        assert!(tracker.is_helper_useful("ethtool"));
        assert!(!tracker.is_helper_useful("iwconfig"));
    }

    #[test]
    fn test_useless_helpers() {
        let mut tracker = HardwareCapabilityTracker::new();
        let mut cap = make_capability("ethernet", HardwareCategory::Network, HardwareStatus::Detected);
        cap.relevant_helpers = vec!["ethtool".to_string()];
        tracker.register(cap);

        let proposed = vec!["ethtool".to_string(), "iwconfig".to_string()];
        let useless = tracker.useless_helpers(&proposed);
        assert_eq!(useless.len(), 1);
        assert_eq!(useless[0], "iwconfig");
    }

    #[test]
    fn test_by_category() {
        let mut tracker = HardwareCapabilityTracker::new();
        tracker.register(make_capability("ethernet", HardwareCategory::Network, HardwareStatus::Detected));
        tracker.register(make_capability("sound", HardwareCategory::Audio, HardwareStatus::Detected));

        assert_eq!(tracker.by_hw_category(HardwareCategory::Network).len(), 1);
        assert_eq!(tracker.by_hw_category(HardwareCategory::Audio).len(), 1);
    }
}
