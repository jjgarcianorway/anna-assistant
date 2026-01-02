//! Memory and swap-related probe rules.

use crate::deterministic_probes::types::ProbeRule;

pub fn memory_rules() -> Vec<ProbeRule> {
    vec![
        ProbeRule {
            intent_id: "memory.free",
            keywords: &["free", "ram"],
            negative_keywords: &[],
            probes: &["memory_info"],
            description: "Free RAM",
        },
        ProbeRule {
            intent_id: "memory.free_alt",
            keywords: &["memory", "available"],
            negative_keywords: &[],
            probes: &["memory_info"],
            description: "Available memory",
        },
        ProbeRule {
            intent_id: "memory.top_process",
            keywords: &["memory", "most"],
            negative_keywords: &[],
            probes: &["top_memory"],
            description: "Which process uses most memory",
        },
        ProbeRule {
            intent_id: "memory.top_process_alt",
            keywords: &["using", "memory"],
            negative_keywords: &[],
            probes: &["top_memory"],
            description: "What is using memory",
        },
        ProbeRule {
            intent_id: "swap.status",
            keywords: &["swap"],
            negative_keywords: &["install", "package"],
            probes: &["swap_files", "memory_info"],
            description: "Swap configuration status",
        },
        ProbeRule {
            intent_id: "swap.have",
            keywords: &["have", "swap"],
            negative_keywords: &["install"],
            probes: &["swap_files", "memory_info"],
            description: "Do I have swap",
        },
    ]
}
