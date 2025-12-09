//! Fast path engine (v0.0.261).
//!
//! v0.0.259: Added uptime, CPU usage, and network status fast paths.
//! v0.0.261: Added top processes fast path.

use crate::recipe::{search_recipes_by_keywords, RecipeMatch};
use crate::snapshot::load_last_snapshot;

use super::answers::{
    answer_cpu_usage, answer_disk_usage, answer_failed_services, answer_memory_usage,
    answer_network_status, answer_system_health, answer_top_processes, answer_uptime,
    answer_what_changed,
};
use super::classify::classify_fast_path;
use super::types::{FastPathAnswer, FastPathClass, FastPathInput};

/// Try to answer via fast path (no LLM)
/// Returns FastPathAnswer with handled=true if answered, false if needs LLM
pub fn try_fast_path(input: &FastPathInput) -> FastPathAnswer {
    if !input.policy.enabled {
        return FastPathAnswer::not_handled("fast path disabled");
    }

    let class = classify_fast_path(input.request);
    if class == FastPathClass::NotFastPath {
        return FastPathAnswer::not_handled("query not in fast path class");
    }

    // Get snapshot (load from disk if not provided)
    let loaded_snapshot;
    let snapshot = match input.snapshot {
        Some(s) => s,
        None => {
            loaded_snapshot = load_last_snapshot();
            match &loaded_snapshot {
                Some(s) => s,
                None => {
                    return FastPathAnswer::not_handled("no snapshot available, probes needed");
                }
            }
        }
    };

    // Check freshness
    let is_fresh = snapshot.is_fresh(input.policy.snapshot_max_age_secs);

    match class {
        FastPathClass::SystemHealth => answer_system_health(snapshot, is_fresh),
        FastPathClass::DiskUsage => answer_disk_usage(snapshot, is_fresh),
        FastPathClass::MemoryUsage => answer_memory_usage(snapshot, is_fresh),
        FastPathClass::FailedServices => answer_failed_services(snapshot, is_fresh),
        FastPathClass::WhatChanged => answer_what_changed(snapshot),
        FastPathClass::Uptime => answer_uptime(snapshot, is_fresh),
        FastPathClass::CpuUsage => answer_cpu_usage(snapshot, is_fresh),
        FastPathClass::NetworkStatus => answer_network_status(snapshot, is_fresh),
        FastPathClass::TopProcesses => answer_top_processes(snapshot, is_fresh),
        FastPathClass::NotFastPath => FastPathAnswer::not_handled("not fast path"),
    }
}

/// Check if any recipes match the query (for RAG hints)
pub fn find_matching_recipes(query: &str, limit: usize) -> Vec<RecipeMatch> {
    let keywords: Vec<&str> = query
        .split_whitespace()
        .filter(|w| w.len() > 2)
        .take(10)
        .collect();

    if keywords.is_empty() {
        return Vec::new();
    }

    search_recipes_by_keywords(&keywords, limit)
}
