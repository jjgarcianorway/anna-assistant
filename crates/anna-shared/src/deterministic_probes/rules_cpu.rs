//! CPU-related probe rules.

use crate::deterministic_probes::types::ProbeRule;

pub fn cpu_rules() -> Vec<ProbeRule> {
    vec![
        ProbeRule {
            intent_id: "cpu.top_process",
            keywords: &["cpu", "most"],
            negative_keywords: &["info", "what cpu", "which cpu"],
            probes: &["top_cpu"],
            description: "Which process uses most CPU",
        },
        ProbeRule {
            intent_id: "cpu.top_process_alt",
            keywords: &["using", "cpu"],
            negative_keywords: &["info"],
            probes: &["top_cpu"],
            description: "What is using CPU",
        },
        ProbeRule {
            intent_id: "cpu.info",
            keywords: &["what", "cpu"],
            negative_keywords: &["using", "most"],
            probes: &["cpu_info"],
            description: "CPU hardware info",
        },
    ]
}
