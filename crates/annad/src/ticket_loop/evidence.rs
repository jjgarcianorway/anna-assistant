//! Evidence kind derivation from route class.

use anna_shared::trace::EvidenceKind;

/// Derive evidence kinds from route class (for ticket creation)
pub fn evidence_kinds_from_route(route_class: &str) -> Vec<EvidenceKind> {
    match route_class {
        "MemoryUsage" | "MemoryInfo" | "memory_usage" | "ram_info" => vec![EvidenceKind::Memory],
        "DiskUsage" | "DiskInfo" | "disk_usage" | "disk_space" => vec![EvidenceKind::Disk],
        "CpuInfo" | "CpuUsage" | "cpu_info" => vec![EvidenceKind::Cpu],
        "SystemServices" | "ServiceStatus" | "service_status" => vec![EvidenceKind::Services],
        "BlockDevices" | "lsblk" => vec![EvidenceKind::BlockDevices],
        "SystemHealth" | "system_health_summary" => {
            vec![EvidenceKind::Memory, EvidenceKind::Disk, EvidenceKind::Cpu]
        }
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_kinds_mapping() {
        assert_eq!(
            evidence_kinds_from_route("MemoryUsage"),
            vec![EvidenceKind::Memory]
        );
        assert_eq!(
            evidence_kinds_from_route("DiskUsage"),
            vec![EvidenceKind::Disk]
        );
        assert_eq!(
            evidence_kinds_from_route("service_status"),
            vec![EvidenceKind::Services]
        );
        assert!(evidence_kinds_from_route("Unknown").is_empty());
    }
}
