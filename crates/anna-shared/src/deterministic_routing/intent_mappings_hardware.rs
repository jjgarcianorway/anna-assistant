//! Hardware department intent mappings.

use super::intent_mapping::IntentMapping;
use super::intent_schema::{CanonicalIntent, Department};
use std::collections::HashMap;

pub(super) fn register_hardware_mappings(mappings: &mut HashMap<CanonicalIntent, IntentMapping>) {
    mappings.insert(
        CanonicalIntent::GpuInfo,
        IntentMapping {
            intent: CanonicalIntent::GpuInfo,
            department: Department::Hardware,
            required_probes: vec!["lspci_gpu"],
            optional_probes: vec!["glxinfo", "nvidia_smi"],
            can_answer_from_probes: true,
            description: "GPU information",
        },
    );

    mappings.insert(
        CanonicalIntent::GpuDriver,
        IntentMapping {
            intent: CanonicalIntent::GpuDriver,
            department: Department::Hardware,
            required_probes: vec!["lspci_k_gpu", "lsmod_gpu"],
            optional_probes: vec!["modinfo_gpu", "dmesg_gpu"],
            can_answer_from_probes: true,
            description: "GPU driver status",
        },
    );

    mappings.insert(
        CanonicalIntent::HardwareSensors,
        IntentMapping {
            intent: CanonicalIntent::HardwareSensors,
            department: Department::Hardware,
            required_probes: vec!["sensors"],
            optional_probes: vec!["hwinfo_temps"],
            can_answer_from_probes: true,
            description: "Hardware temperature sensors",
        },
    );

    mappings.insert(
        CanonicalIntent::CpuInfo,
        IntentMapping {
            intent: CanonicalIntent::CpuInfo,
            department: Department::Hardware,
            required_probes: vec!["lscpu"],
            optional_probes: vec!["cpuinfo"],
            can_answer_from_probes: true,
            description: "CPU hardware information",
        },
    );

    mappings.insert(
        CanonicalIntent::AudioHealth,
        IntentMapping {
            intent: CanonicalIntent::AudioHealth,
            department: Department::Hardware,
            required_probes: vec!["pactl_info", "aplay_l"],
            optional_probes: vec!["pipewire_status", "alsa_info"],
            can_answer_from_probes: false, // Audio "health" needs synthesis
            description: "Audio subsystem health",
        },
    );

    mappings.insert(
        CanonicalIntent::UsbDevices,
        IntentMapping {
            intent: CanonicalIntent::UsbDevices,
            department: Department::Hardware,
            required_probes: vec!["lsusb"],
            optional_probes: vec!["usb_devices"],
            can_answer_from_probes: true,
            description: "USB device listing",
        },
    );

    mappings.insert(
        CanonicalIntent::PciDevices,
        IntentMapping {
            intent: CanonicalIntent::PciDevices,
            department: Department::Hardware,
            required_probes: vec!["lspci"],
            optional_probes: vec!["lspci_v"],
            can_answer_from_probes: true,
            description: "PCI device listing",
        },
    );
}
