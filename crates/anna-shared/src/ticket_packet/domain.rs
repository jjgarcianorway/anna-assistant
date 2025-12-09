//! Domain-specific probe and evidence recommendations (v0.0.216).

use crate::rpc::SpecialistDomain;
use crate::trace::EvidenceKind;

/// Recommended probes for each domain (v0.0.36)
pub fn recommended_probes_for_domain(domain: SpecialistDomain) -> Vec<&'static str> {
    match domain {
        SpecialistDomain::System => vec!["memory_info", "cpu_info", "failed_services"],
        SpecialistDomain::Storage => vec!["disk_usage", "block_devices"],
        SpecialistDomain::Network => vec!["network_addrs", "network_routes", "listening_ports"],
        SpecialistDomain::Security => vec!["failed_services", "listening_ports"],
        SpecialistDomain::Packages => vec![], // Uses package manager commands
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
        SpecialistDomain::Storage => vec![EvidenceKind::Disk, EvidenceKind::BlockDevices],
        SpecialistDomain::Network => vec![], // Network evidence kind not defined yet
        SpecialistDomain::Security => vec![EvidenceKind::Services],
        SpecialistDomain::Packages => vec![],
    }
}
