//! Performance department intent mappings.

use super::intent_mapping::IntentMapping;
use super::intent_schema::{CanonicalIntent, Department};
use std::collections::HashMap;

pub(super) fn register_performance_mappings(
    mappings: &mut HashMap<CanonicalIntent, IntentMapping>,
) {
    mappings.insert(
        CanonicalIntent::BootPerf,
        IntentMapping {
            intent: CanonicalIntent::BootPerf,
            department: Department::Performance,
            required_probes: vec!["systemd_analyze", "systemd_blame"],
            optional_probes: vec!["systemd_critical_chain", "journalctl_boot_errors"],
            can_answer_from_probes: true, // Boot time is a fact from systemd-analyze
            description: "Boot performance analysis",
        },
    );

    mappings.insert(
        CanonicalIntent::MemStatus,
        IntentMapping {
            intent: CanonicalIntent::MemStatus,
            department: Department::Performance,
            required_probes: vec!["free_h"],
            optional_probes: vec!["meminfo", "vmstat"],
            can_answer_from_probes: true, // RAM available is a direct fact
            description: "Memory status and usage",
        },
    );

    mappings.insert(
        CanonicalIntent::CpuLoad,
        IntentMapping {
            intent: CanonicalIntent::CpuLoad,
            department: Department::Performance,
            required_probes: vec!["uptime", "top_cpu"],
            optional_probes: vec!["mpstat", "ps_aux_cpu"],
            can_answer_from_probes: true, // Load average is a direct fact
            description: "CPU load and top consumers",
        },
    );

    mappings.insert(
        CanonicalIntent::IoWait,
        IntentMapping {
            intent: CanonicalIntent::IoWait,
            department: Department::Performance,
            required_probes: vec!["iostat", "vmstat"],
            optional_probes: vec!["iotop_snapshot"],
            can_answer_from_probes: true,
            description: "I/O wait analysis",
        },
    );
}
