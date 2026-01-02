//! Hardware capability formatting and display functions

use super::tracker::HardwareCapabilityTracker;

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
    use super::super::capability::HardwareCapability;
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
