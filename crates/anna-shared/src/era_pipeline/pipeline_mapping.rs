//! Fact → Probe mapping table.

use super::pipeline_types::ProbeMapping;

/// Fact → Probe mapping table.
pub struct FactProbeTable {
    /// Mappings.
    mappings: Vec<ProbeMapping>,
}

impl FactProbeTable {
    /// Create with default mappings.
    pub fn new() -> Self {
        let mappings = vec![
            // Memory facts
            ProbeMapping {
                fact_name: "memory.free_gib".to_string(),
                probe_id: "free_h".to_string(),
                extractor: "extract_memory_free".to_string(),
            },
            ProbeMapping {
                fact_name: "memory.total_gib".to_string(),
                probe_id: "free_h".to_string(),
                extractor: "extract_memory_total".to_string(),
            },
            ProbeMapping {
                fact_name: "memory.used_pct".to_string(),
                probe_id: "free_h".to_string(),
                extractor: "extract_memory_used_pct".to_string(),
            },
            // Boot facts
            ProbeMapping {
                fact_name: "boot.total_time_s".to_string(),
                probe_id: "systemd_analyze".to_string(),
                extractor: "extract_boot_time".to_string(),
            },
            ProbeMapping {
                fact_name: "boot.blame".to_string(),
                probe_id: "systemd_blame".to_string(),
                extractor: "extract_blame_list".to_string(),
            },
            ProbeMapping {
                fact_name: "boot.slowest_service".to_string(),
                probe_id: "systemd_blame".to_string(),
                extractor: "extract_slowest_service".to_string(),
            },
            // CPU facts
            ProbeMapping {
                fact_name: "cpu.model".to_string(),
                probe_id: "lscpu".to_string(),
                extractor: "extract_cpu_model".to_string(),
            },
            ProbeMapping {
                fact_name: "cpu.cores".to_string(),
                probe_id: "lscpu".to_string(),
                extractor: "extract_cpu_cores".to_string(),
            },
            ProbeMapping {
                fact_name: "cpu.temp_c".to_string(),
                probe_id: "sensors".to_string(),
                extractor: "extract_cpu_temp".to_string(),
            },
            ProbeMapping {
                fact_name: "cpu.load_1m".to_string(),
                probe_id: "uptime".to_string(),
                extractor: "extract_load_1m".to_string(),
            },
            // Disk facts
            ProbeMapping {
                fact_name: "disk.root_free_gib".to_string(),
                probe_id: "df_h".to_string(),
                extractor: "extract_root_free".to_string(),
            },
            ProbeMapping {
                fact_name: "disk.root_used_pct".to_string(),
                probe_id: "df_h".to_string(),
                extractor: "extract_root_used_pct".to_string(),
            },
            ProbeMapping {
                fact_name: "disk.trim_enabled".to_string(),
                probe_id: "fstrim_status".to_string(),
                extractor: "extract_trim_status".to_string(),
            },
            // GPU facts
            ProbeMapping {
                fact_name: "gpu.model".to_string(),
                probe_id: "lspci_gpu".to_string(),
                extractor: "extract_gpu_model".to_string(),
            },
            ProbeMapping {
                fact_name: "gpu.driver".to_string(),
                probe_id: "lspci_k_gpu".to_string(),
                extractor: "extract_gpu_driver".to_string(),
            },
            // Service facts
            ProbeMapping {
                fact_name: "services.failed_count".to_string(),
                probe_id: "systemctl_failed".to_string(),
                extractor: "extract_failed_count".to_string(),
            },
            ProbeMapping {
                fact_name: "services.failed_list".to_string(),
                probe_id: "systemctl_failed".to_string(),
                extractor: "extract_failed_list".to_string(),
            },
        ];

        Self { mappings }
    }

    /// Get probe for a fact.
    pub fn get_probe(&self, fact_name: &str) -> Option<&ProbeMapping> {
        self.mappings.iter().find(|m| m.fact_name == fact_name)
    }

    /// Get all probes needed for a set of facts.
    pub fn get_probes_for_facts(&self, facts: &[String]) -> Vec<&ProbeMapping> {
        facts.iter().filter_map(|f| self.get_probe(f)).collect()
    }

    /// Get unique probe IDs needed.
    pub fn unique_probe_ids(&self, facts: &[String]) -> Vec<String> {
        let mut ids: Vec<String> = self
            .get_probes_for_facts(facts)
            .iter()
            .map(|m| m.probe_id.clone())
            .collect();
        ids.sort();
        ids.dedup();
        ids
    }
}

impl Default for FactProbeTable {
    fn default() -> Self {
        Self::new()
    }
}
