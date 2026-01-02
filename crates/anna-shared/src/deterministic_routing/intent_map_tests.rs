//! Tests for Intent Mapping and Intent Map Table.
//!
//! Part of the Deterministic Intent Map (v0.0.439).

#[cfg(test)]
mod tests {
    use crate::deterministic_routing::intent_map_table::IntentMapTable;
    use crate::deterministic_routing::intent_schema::{CanonicalIntent, Department};

    #[test]
    fn test_boot_perf_maps_to_performance() {
        let map = IntentMapTable::build();
        assert_eq!(
            map.get_department(CanonicalIntent::BootPerf),
            Department::Performance
        );
    }

    #[test]
    fn test_gpu_maps_to_hardware() {
        let map = IntentMapTable::build();
        assert_eq!(
            map.get_department(CanonicalIntent::GpuInfo),
            Department::Hardware
        );
        assert_eq!(
            map.get_department(CanonicalIntent::GpuDriver),
            Department::Hardware
        );
    }

    #[test]
    fn test_disk_maps_to_storage() {
        let map = IntentMapTable::build();
        assert_eq!(
            map.get_department(CanonicalIntent::DiskUsage),
            Department::Storage
        );
    }

    #[test]
    fn test_ram_maps_to_performance() {
        let map = IntentMapTable::build();
        assert_eq!(
            map.get_department(CanonicalIntent::MemStatus),
            Department::Performance
        );
    }

    #[test]
    fn test_required_probes_for_boot_perf() {
        let map = IntentMapTable::build();
        let probes = map.get_required_probes(CanonicalIntent::BootPerf);
        assert!(probes.contains(&"systemd_analyze"));
        assert!(probes.contains(&"systemd_blame"));
    }

    #[test]
    fn test_can_answer_directly() {
        let map = IntentMapTable::build();
        // Facts can be answered directly
        assert!(map.can_answer_directly(CanonicalIntent::MemStatus));
        assert!(map.can_answer_directly(CanonicalIntent::DiskUsage));
        // "Health" synthesis cannot
        assert!(!map.can_answer_directly(CanonicalIntent::SvcHealth));
        assert!(!map.can_answer_directly(CanonicalIntent::NetHealth));
    }

    #[test]
    fn test_intents_for_department() {
        let map = IntentMapTable::build();
        let perf_intents = map.intents_for_department(Department::Performance);
        assert!(perf_intents.contains(&CanonicalIntent::BootPerf));
        assert!(perf_intents.contains(&CanonicalIntent::MemStatus));
        assert!(!perf_intents.contains(&CanonicalIntent::DiskUsage)); // Storage
    }
}
