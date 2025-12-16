//! Hardware routes: CPU, GPU, audio, sensors (v0.0.805).
//!
//! v0.0.321: Added HardwareAcceleration route for browser/video queries.

use anna_shared::probe_spine::{EvidenceKind, ProbeId, RouteCapability};
use anna_shared::rpc::{QueryIntent, SpecialistDomain};

use super::{DeterministicRoute, QueryClass};

/// Build route for hardware queries
pub fn build_hardware_route(class: QueryClass) -> Option<DeterministicRoute> {
    match class {
        QueryClass::CpuInfo => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["lscpu".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: false,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Cpu],
                spine_probes: vec![ProbeId::Lscpu],
            },
        }),

        QueryClass::CpuCores => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["lscpu".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Cpu],
                spine_probes: vec![ProbeId::Lscpu],
            },
        }),

        QueryClass::CpuTemp => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["sensors".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: false,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::CpuTemperature],
                spine_probes: vec![ProbeId::Sensors],
            },
        }),

        QueryClass::RamInfo => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["free".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: false,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Memory],
                spine_probes: vec![ProbeId::Free],
            },
        }),

        QueryClass::GpuInfo => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["lspci_gpu".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: false,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Gpu],
                spine_probes: vec![],
            },
        }),

        QueryClass::HardwareAudio => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["lspci_audio".to_string(), "pactl_cards".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Audio],
                spine_probes: vec![ProbeId::LspciAudio, ProbeId::PactlCards],
            },
        }),

        QueryClass::CpuFrequency => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["cpu_frequency".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Cpu],
                spine_probes: vec![],
            },
        }),

        QueryClass::MemorySlots => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["memory_slots".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Memory],
                spine_probes: vec![],
            },
        }),

        QueryClass::SensorsTemp => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["sensors_temp".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::GpuMemory => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["gpu_memory".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::PciDevices => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["pci_devices".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::UsbDevices => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["lsusb".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::BluetoothDevices => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["bluetooth_devices".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::AudioDevices => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["audio_devices".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::PrinterStatus => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["printer_status".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::CpuGovernor => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["cpu_governor".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Cpu],
                spine_probes: vec![],
            },
        }),

        QueryClass::LoadedFirmware => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["loaded_firmware".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        // v0.0.802: Webcam/camera status
        QueryClass::WebcamStatus => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["webcam_devices".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        // v0.0.805: Screen/display/monitor resolution
        QueryClass::ScreenResolution => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["xrandr".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        _ => None,
    }
}
