//! Domain-specific probe and evidence recommendations (v0.0.405).
//! v0.0.388: Added package probes for Packages domain.
//! v0.0.405: Expanded for all domains per clean architecture roadmap.

use crate::rpc::SpecialistDomain;
use crate::trace::EvidenceKind;

/// Recommended probes for each domain
/// v0.0.405: Complete mapping for all domains
pub fn recommended_probes_for_domain(domain: SpecialistDomain) -> Vec<&'static str> {
    match domain {
        SpecialistDomain::System => vec!["memory_info", "cpu_info", "failed_services"],
        SpecialistDomain::Boot => vec!["boot_time", "boot_blame", "failed_services"],
        SpecialistDomain::Services => vec!["failed_services", "running_services"],
        SpecialistDomain::Network => vec!["network_addrs", "network_routes", "listening_ports"],
        SpecialistDomain::Storage => vec!["disk_usage", "block_devices"],
        SpecialistDomain::Packages => vec!["installed_packages", "package_count"],
        SpecialistDomain::Audio => vec!["audio_devices", "audio_server"],
        SpecialistDomain::Display => vec!["gpu_info", "display_info"],
        SpecialistDomain::Desktop => vec!["desktop_session", "display_server"],
        SpecialistDomain::Security => vec!["failed_services", "listening_ports"],
    }
}

/// Recommended evidence kinds for each domain
pub fn evidence_kinds_for_domain(domain: SpecialistDomain) -> Vec<EvidenceKind> {
    match domain {
        SpecialistDomain::System => vec![
            EvidenceKind::Memory,
            EvidenceKind::Cpu,
            EvidenceKind::Services,
        ],
        SpecialistDomain::Boot => vec![EvidenceKind::Services],
        SpecialistDomain::Services => vec![EvidenceKind::Services],
        SpecialistDomain::Network => vec![], // Network evidence kind not defined yet
        SpecialistDomain::Storage => vec![EvidenceKind::Disk, EvidenceKind::BlockDevices],
        SpecialistDomain::Packages => vec![EvidenceKind::Packages],
        SpecialistDomain::Audio => vec![],
        SpecialistDomain::Display => vec![],
        SpecialistDomain::Desktop => vec![],
        SpecialistDomain::Security => vec![EvidenceKind::Services],
    }
}
