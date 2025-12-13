//! Hardware Capability Detector - Phase 92
//!
//! Detects what hardware exists before installing helpers.
//! VISION.md: "Never install useless helpers (no ethtool if no ethernet)"

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Hardware category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HardwareCategory {
    #[default]
    Network,
    Audio,
    Video,
    Storage,
    Input,
    Usb,
    Bluetooth,
    Wireless,
    Power,
    Cpu,
    Memory,
    Other,
}

impl HardwareCategory {
    pub fn name(&self) -> &'static str {
        match self {
            HardwareCategory::Network => "Network",
            HardwareCategory::Audio => "Audio",
            HardwareCategory::Video => "Video",
            HardwareCategory::Storage => "Storage",
            HardwareCategory::Input => "Input",
            HardwareCategory::Usb => "USB",
            HardwareCategory::Bluetooth => "Bluetooth",
            HardwareCategory::Wireless => "Wireless",
            HardwareCategory::Power => "Power",
            HardwareCategory::Cpu => "CPU",
            HardwareCategory::Memory => "Memory",
            HardwareCategory::Other => "Other",
        }
    }
}

/// Hardware status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HardwareStatus {
    #[default]
    Detected,
    NotDetected,
    Disabled,
    Error,
    Unknown,
}

impl HardwareStatus {
    pub fn name(&self) -> &'static str {
        match self {
            HardwareStatus::Detected => "Detected",
            HardwareStatus::NotDetected => "Not Detected",
            HardwareStatus::Disabled => "Disabled",
            HardwareStatus::Error => "Error",
            HardwareStatus::Unknown => "Unknown",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            HardwareStatus::Detected => "✓",
            HardwareStatus::NotDetected => "-",
            HardwareStatus::Disabled => "x",
            HardwareStatus::Error => "!",
            HardwareStatus::Unknown => "?",
        }
    }
}

/// A hardware capability record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareCapability {
    /// Capability name (e.g., "ethernet", "wifi", "sound")
    pub name: String,
    /// Category
    pub category: HardwareCategory,
    /// Status
    pub status: HardwareStatus,
    /// Device name/model if detected
    pub device: Option<String>,
    /// When last checked
    pub last_check: u64,
    /// Relevant helpers for this capability
    pub relevant_helpers: Vec<String>,
}

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

/// Common hardware capability names
pub const COMMON_CAPABILITIES: &[(&str, HardwareCategory, &[&str])] = &[
    ("ethernet", HardwareCategory::Network, &["ethtool", "mii-tool"]),
    ("wifi", HardwareCategory::Wireless, &["iwconfig", "iw", "nmcli"]),
    ("bluetooth", HardwareCategory::Bluetooth, &["bluetoothctl", "hcitool"]),
    ("sound", HardwareCategory::Audio, &["alsamixer", "pulseaudio", "pipewire"]),
    ("nvidia_gpu", HardwareCategory::Video, &["nvidia-smi", "nvtop"]),
    ("amd_gpu", HardwareCategory::Video, &["radeontop"]),
    ("battery", HardwareCategory::Power, &["acpi", "upower"]),
    ("nvme", HardwareCategory::Storage, &["nvme-cli"]),
    ("sata", HardwareCategory::Storage, &["smartctl", "hdparm"]),
];

/// Get relevant helpers for a capability
pub fn get_relevant_helpers(capability: &str) -> Vec<&'static str> {
    for (name, _, helpers) in COMMON_CAPABILITIES {
        if *name == capability {
            return helpers.to_vec();
        }
    }
    vec![]
}

/// Format hardware tracker for display
pub fn format_hardware_tracker(tracker: &HardwareCapabilityTracker) -> String {
    let mut lines = vec!["=== Hardware Capabilities ===".to_string()];
    lines.push(String::new());

    if tracker.capabilities.is_empty() {
        lines.push("No capabilities detected yet.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total capabilities: {}", tracker.total_count()));
    lines.push(format!("Detected: {}", tracker.detected_count()));

    // By category
    if !tracker.by_category.is_empty() {
        lines.push(String::new());
        lines.push("By category:".to_string());
        for (cat, count) in &tracker.by_category {
            lines.push(format!("  {}: {}", cat, count));
        }
    }

    // Detected hardware
    let detected = tracker.detected();
    if !detected.is_empty() {
        lines.push(String::new());
        lines.push("Detected hardware:".to_string());
        for cap in detected.iter().take(10) {
            let device = cap.device.as_deref().unwrap_or("unknown");
            lines.push(format!("  [{}] {} - {}", cap.status.symbol(), cap.name, device));
        }
    }

    lines.join("\n")
}

/// Format hardware tracker compact
pub fn format_hardware_tracker_compact(tracker: &HardwareCapabilityTracker) -> String {
    format!(
        "Hardware: {} capabilities | {} detected",
        tracker.total_count(),
        tracker.detected_count()
    )
}

/// Format hardware tracker one-line
pub fn format_hardware_tracker_oneline(tracker: &HardwareCapabilityTracker) -> String {
    format!(
        "{} hardware ({} detected)",
        tracker.total_count(),
        tracker.detected_count()
    )
}

/// Check if query is about hardware
pub fn is_hardware_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "hardware",
        "capability",
        "capabilities",
        "what hardware",
        "detected hardware",
        "system info",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about hardware
pub fn hardware_fun_fact(tracker: &HardwareCapabilityTracker) -> String {
    if tracker.capabilities.is_empty() {
        return "No hardware detected yet!".to_string();
    }

    let facts = [
        format!(
            "Anna knows about {} hardware capabilities.",
            tracker.total_count()
        ),
        format!(
            "{} hardware capabilities are detected.",
            tracker.detected_count()
        ),
        format!(
            "{} capabilities are not detected.",
            tracker.not_detected().len()
        ),
    ];

    facts[tracker.total_count() % facts.len()].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_hardware_category() {
        assert_eq!(HardwareCategory::Network.name(), "Network");
        assert_eq!(HardwareCategory::Audio.name(), "Audio");
    }

    #[test]
    fn test_hardware_status() {
        assert_eq!(HardwareStatus::Detected.name(), "Detected");
        assert_eq!(HardwareStatus::Detected.symbol(), "✓");
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

    #[test]
    fn test_get_relevant_helpers() {
        let helpers = get_relevant_helpers("ethernet");
        assert!(helpers.contains(&"ethtool"));
    }

    #[test]
    fn test_format_hardware_tracker() {
        let mut tracker = HardwareCapabilityTracker::new();
        tracker.register(make_capability("ethernet", HardwareCategory::Network, HardwareStatus::Detected));

        let output = format_hardware_tracker(&tracker);
        assert!(output.contains("Hardware Capabilities"));
        assert!(output.contains("ethernet"));
    }

    #[test]
    fn test_is_hardware_query() {
        assert!(is_hardware_query("what hardware is detected?"));
        assert!(is_hardware_query("show capabilities"));
        assert!(is_hardware_query("system info"));
        assert!(!is_hardware_query("what is the weather?"));
    }

    #[test]
    fn test_hardware_fun_fact() {
        let mut tracker = HardwareCapabilityTracker::new();
        tracker.register(make_capability("ethernet", HardwareCategory::Network, HardwareStatus::Detected));

        let fact = hardware_fun_fact(&tracker);
        assert!(!fact.is_empty());
    }
}
