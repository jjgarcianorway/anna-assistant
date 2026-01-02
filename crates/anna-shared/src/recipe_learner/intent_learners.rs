//! Intent-specific recipe learning functions.

use super::observation::TicketObservation;
use super::utils::current_secs;
use crate::canonical_intents::CanonicalIntent;
use crate::learned_recipes::{
    AnswerTemplate, CompareOp, LearnedRecipe, RecipeComputeStep, RecipeStats,
};

pub fn learn_disk_usage_recipe(
    observations: &[&TicketObservation],
    _common_probes: &[String],
) -> Option<LearnedRecipe> {
    Some(LearnedRecipe {
        id: "disk_usage_v1".to_string(),
        name: "Disk Usage Check".to_string(),
        version: 1,
        intent: CanonicalIntent::CheckDiskUsage,
        domain: "storage".to_string(),
        required_probes: vec!["disk_usage".to_string()],
        optional_probes: vec!["block_devices".to_string()],
        steps: vec![
            RecipeComputeStep::Extract {
                probe: "disk_usage".to_string(),
                pattern: r"(\d+)%".to_string(),
                variable: "root_percent".to_string(),
            },
            RecipeComputeStep::ParseNumber {
                source_var: "root_percent".to_string(),
                target_var: "root_percent_num".to_string(),
            },
            RecipeComputeStep::Compare {
                variable: "root_percent_num".to_string(),
                operator: CompareOp::Ge,
                threshold: 90.0,
                result_var: "is_critical".to_string(),
            },
        ],
        answer_ok: AnswerTemplate {
            summary: "Root filesystem at {root_percent}% used".to_string(),
            details: vec![],
            evidence: vec!["disk_usage".to_string()],
        },
        answer_critical: Some(AnswerTemplate {
            summary: "[WARNING] Root filesystem at {root_percent}% used - running low on space"
                .to_string(),
            details: vec!["Consider cleaning package cache: pacman -Sc".to_string()],
            evidence: vec!["disk_usage".to_string()],
        }),
        answer_partial: None,
        knowledge_topics: vec!["df_command".to_string(), "disk_usage".to_string()],
        source_tickets: observations.iter().map(|o| o.ticket_id.clone()).collect(),
        stats: RecipeStats::default(),
        created_at: current_secs(),
        last_used_at: 0,
        deprecated: false,
    })
}

pub fn learn_memory_recipe(
    observations: &[&TicketObservation],
    _common_probes: &[String],
) -> Option<LearnedRecipe> {
    Some(LearnedRecipe {
        id: "memory_check_v1".to_string(),
        name: "Memory Usage Check".to_string(),
        version: 1,
        intent: CanonicalIntent::CheckFreeRam,
        domain: "system".to_string(),
        required_probes: vec!["memory_info".to_string()],
        optional_probes: vec![],
        steps: vec![
            RecipeComputeStep::Extract {
                probe: "memory_info".to_string(),
                pattern: r"Mem:.*?(\d+)".to_string(),
                variable: "total_mb".to_string(),
            },
            RecipeComputeStep::Extract {
                probe: "memory_info".to_string(),
                pattern: r"available:\s*(\d+)".to_string(),
                variable: "available_mb".to_string(),
            },
        ],
        answer_ok: AnswerTemplate {
            summary: "Available RAM: {available_mb} MiB".to_string(),
            details: vec![],
            evidence: vec!["memory_info".to_string()],
        },
        answer_critical: None,
        answer_partial: None,
        knowledge_topics: vec!["free_command".to_string(), "proc_meminfo".to_string()],
        source_tickets: observations.iter().map(|o| o.ticket_id.clone()).collect(),
        stats: RecipeStats::default(),
        created_at: current_secs(),
        last_used_at: 0,
        deprecated: false,
    })
}

pub fn learn_swap_recipe(
    observations: &[&TicketObservation],
    _common_probes: &[String],
) -> Option<LearnedRecipe> {
    Some(LearnedRecipe {
        id: "swap_check_v1".to_string(),
        name: "Swap Presence Check".to_string(),
        version: 1,
        intent: CanonicalIntent::CheckSwapPresence,
        domain: "system".to_string(),
        required_probes: vec!["swap_files".to_string()],
        optional_probes: vec![],
        steps: vec![RecipeComputeStep::IsEmpty {
            probe: "swap_files".to_string(),
            variable: "no_swap".to_string(),
        }],
        answer_ok: AnswerTemplate {
            summary: "Swap is configured on this system".to_string(),
            details: vec![],
            evidence: vec!["swap_files".to_string()],
        },
        answer_critical: Some(AnswerTemplate {
            summary: "No swap configured on this system".to_string(),
            details: vec![],
            evidence: vec!["swap_files".to_string()],
        }),
        answer_partial: None,
        knowledge_topics: vec!["swap_configuration".to_string(), "proc_swaps".to_string()],
        source_tickets: observations.iter().map(|o| o.ticket_id.clone()).collect(),
        stats: RecipeStats::default(),
        created_at: current_secs(),
        last_used_at: 0,
        deprecated: false,
    })
}

pub fn learn_failed_services_recipe(
    observations: &[&TicketObservation],
    _common_probes: &[String],
) -> Option<LearnedRecipe> {
    Some(LearnedRecipe {
        id: "failed_services_v1".to_string(),
        name: "Failed Services Check".to_string(),
        version: 1,
        intent: CanonicalIntent::CheckFailedServices,
        domain: "services".to_string(),
        required_probes: vec!["failed_services".to_string()],
        optional_probes: vec![],
        steps: vec![
            RecipeComputeStep::Count {
                probe: "failed_services".to_string(),
                pattern: r"(?m)^\s*\*".to_string(),
                variable: "failed_count".to_string(),
            },
            RecipeComputeStep::Compare {
                variable: "failed_count".to_string(),
                operator: CompareOp::Gt,
                threshold: 0.0,
                result_var: "has_failures".to_string(),
            },
        ],
        answer_ok: AnswerTemplate {
            summary: "No failed systemd units".to_string(),
            details: vec![],
            evidence: vec!["failed_services".to_string()],
        },
        answer_critical: Some(AnswerTemplate {
            summary: "{failed_count} systemd unit(s) failed".to_string(),
            details: vec!["Run 'systemctl --failed' for details".to_string()],
            evidence: vec!["failed_services".to_string()],
        }),
        answer_partial: None,
        knowledge_topics: vec!["systemctl".to_string(), "systemd_units".to_string()],
        source_tickets: observations.iter().map(|o| o.ticket_id.clone()).collect(),
        stats: RecipeStats::default(),
        created_at: current_secs(),
        last_used_at: 0,
        deprecated: false,
    })
}

pub fn learn_uptime_recipe(
    observations: &[&TicketObservation],
    _common_probes: &[String],
) -> Option<LearnedRecipe> {
    Some(LearnedRecipe {
        id: "uptime_v1".to_string(),
        name: "System Uptime".to_string(),
        version: 1,
        intent: CanonicalIntent::CheckUptime,
        domain: "system".to_string(),
        required_probes: vec!["uptime".to_string()],
        optional_probes: vec![],
        steps: vec![RecipeComputeStep::Extract {
            probe: "uptime".to_string(),
            pattern: r"up\s+(.+?),".to_string(),
            variable: "uptime_str".to_string(),
        }],
        answer_ok: AnswerTemplate {
            summary: "System uptime: {uptime_str}".to_string(),
            details: vec![],
            evidence: vec!["uptime".to_string()],
        },
        answer_critical: None,
        answer_partial: None,
        knowledge_topics: vec!["uptime".to_string()],
        source_tickets: observations.iter().map(|o| o.ticket_id.clone()).collect(),
        stats: RecipeStats::default(),
        created_at: current_secs(),
        last_used_at: 0,
        deprecated: false,
    })
}

pub fn learn_boot_time_recipe(
    observations: &[&TicketObservation],
    _common_probes: &[String],
) -> Option<LearnedRecipe> {
    Some(LearnedRecipe {
        id: "boot_time_v1".to_string(),
        name: "Boot Time Check".to_string(),
        version: 1,
        intent: CanonicalIntent::CheckBootTime,
        domain: "boot".to_string(),
        required_probes: vec!["boot_time".to_string()],
        optional_probes: vec!["boot_blame".to_string()],
        steps: vec![RecipeComputeStep::Extract {
            probe: "boot_time".to_string(),
            pattern: r"=\s*([\d.]+)s".to_string(),
            variable: "total_seconds".to_string(),
        }],
        answer_ok: AnswerTemplate {
            summary: "Boot time: {total_seconds}s".to_string(),
            details: vec![],
            evidence: vec!["boot_time".to_string()],
        },
        answer_critical: None,
        answer_partial: None,
        knowledge_topics: vec![
            "systemd_analyze".to_string(),
            "boot_performance".to_string(),
        ],
        source_tickets: observations.iter().map(|o| o.ticket_id.clone()).collect(),
        stats: RecipeStats::default(),
        created_at: current_secs(),
        last_used_at: 0,
        deprecated: false,
    })
}

pub fn learn_generic_recipe(
    intent: CanonicalIntent,
    observations: &[&TicketObservation],
    common_probes: &[String],
) -> Option<LearnedRecipe> {
    if common_probes.is_empty() {
        return None;
    }

    let domain = observations.first()?.domain.clone();

    Some(LearnedRecipe {
        id: format!("{:?}_v1", intent).to_lowercase(),
        name: format!("{}", intent.display()),
        version: 1,
        intent,
        domain,
        required_probes: common_probes.to_vec(),
        optional_probes: vec![],
        steps: vec![],
        answer_ok: AnswerTemplate {
            summary: format!("[{}] See probe output", intent.display()),
            details: vec![],
            evidence: common_probes.to_vec(),
        },
        answer_critical: None,
        answer_partial: None,
        knowledge_topics: intent
            .knowledge_topics()
            .iter()
            .map(|s| s.to_string())
            .collect(),
        source_tickets: observations.iter().map(|o| o.ticket_id.clone()).collect(),
        stats: RecipeStats::default(),
        created_at: current_secs(),
        last_used_at: 0,
        deprecated: false,
    })
}
