//! Common hardware capabilities and helper functions

use super::types::HardwareCategory;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_relevant_helpers() {
        let helpers = get_relevant_helpers("ethernet");
        assert!(helpers.contains(&"ethtool"));
    }
}
