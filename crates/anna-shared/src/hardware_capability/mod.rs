//! Hardware Capability Detector - Phase 92
//!
//! Detects what hardware exists before installing helpers.
//! VISION.md: "Never install useless helpers (no ethtool if no ethernet)"

mod types;
mod capability;
mod tracker;
mod helpers;
mod format;

// Re-export all public types and functions to maintain the same API
pub use types::{HardwareCategory, HardwareStatus};
pub use capability::HardwareCapability;
pub use tracker::HardwareCapabilityTracker;
pub use helpers::{COMMON_CAPABILITIES, get_relevant_helpers};
pub use format::{
    format_hardware_tracker,
    format_hardware_tracker_compact,
    format_hardware_tracker_oneline,
    is_hardware_query,
    hardware_fun_fact,
};

#[cfg(test)]
mod tests {
    use super::*;

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
}
