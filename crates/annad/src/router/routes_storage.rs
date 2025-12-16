//! Storage routes: filesystems, block devices, mounts (v0.0.806).

use anna_shared::probe_spine::{EvidenceKind, ProbeId, RouteCapability};
use anna_shared::rpc::{QueryIntent, SpecialistDomain};

use super::{DeterministicRoute, QueryClass};

/// Build route for storage queries
pub fn build_storage_route(class: QueryClass) -> Option<DeterministicRoute> {
    match class {
        QueryClass::MountedFilesystems => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Storage,
            intent: QueryIntent::Question,
            probes: vec!["findmnt".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Disk],
                spine_probes: vec![],
            },
        }),

        QueryClass::BlockDevices => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Storage,
            intent: QueryIntent::Question,
            probes: vec!["block_devices".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Disk],
                spine_probes: vec![ProbeId::Lsblk],
            },
        }),

        QueryClass::ZfsStatus => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Storage,
            intent: QueryIntent::Question,
            probes: vec!["zfs_status".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Disk],
                spine_probes: vec![],
            },
        }),

        QueryClass::LvmStatus => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Storage,
            intent: QueryIntent::Question,
            probes: vec!["lvm_status".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Disk],
                spine_probes: vec![],
            },
        }),

        QueryClass::RaidStatus => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Storage,
            intent: QueryIntent::Question,
            probes: vec!["raid_status".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Disk],
                spine_probes: vec![],
            },
        }),

        QueryClass::TmpFiles => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Storage,
            intent: QueryIntent::Question,
            probes: vec!["tmp_files".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Disk],
                spine_probes: vec![],
            },
        }),

        QueryClass::FstabEntries => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Storage,
            intent: QueryIntent::Question,
            probes: vec!["fstab_entries".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Disk],
                spine_probes: vec![],
            },
        }),

        QueryClass::SwapFiles => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Storage,
            intent: QueryIntent::Question,
            probes: vec!["swap_files".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Disk],
                spine_probes: vec![],
            },
        }),

        QueryClass::SystemdMounts => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Storage,
            intent: QueryIntent::Question,
            probes: vec!["systemd_mounts".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Disk],
                spine_probes: vec![],
            },
        }),

        // v0.0.390: Largest folders - "top folders taking space"
        // v0.0.809: Reverted to LLM path - du scan is inherently slow on large filesystems
        // The LLM will explain this and offer alternatives
        QueryClass::LargestFolders => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Storage,
            intent: QueryIntent::Question,
            probes: vec!["disk_usage".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: false, // Requires slow scan
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Disk],
                spine_probes: vec![],
            },
        }),

        _ => None,
    }
}
