//! Tests for probe registry
//!
//! Extracted from probe_registry.rs for modularization.

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_probe_id_to_command() {
        assert_eq!(
            probe_id_to_command("top_memory"),
            Some("ps aux --sort=-%mem")
        );
        assert_eq!(probe_id_to_command("invalid"), None);
    }

    #[test]
    fn test_filter_valid_probes() {
        let probes = vec![
            "top_memory".to_string(),
            "invalid".to_string(),
            "cpu_info".to_string(),
        ];
        let filtered = filter_valid_probes(probes);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains(&"top_memory".to_string()));
        assert!(!filtered.contains(&"invalid".to_string()));
    }

    #[test]
    fn test_probe_ids_list() {
        // Ensure PROBE_IDS contains the core probes
        assert!(PROBE_IDS.contains(&"top_memory"));
        assert!(PROBE_IDS.contains(&"cpu_info"));
        assert!(PROBE_IDS.contains(&"memory_info"));
        assert!(PROBE_IDS.contains(&"disk_usage"));
    }
}
