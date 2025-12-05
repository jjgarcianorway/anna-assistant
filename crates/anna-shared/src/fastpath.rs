//! Fast path engine for answering health/status queries without LLM (v0.0.39).
//!
//! Handles common "how is my computer" queries deterministically using:
//! - Cached snapshot (if fresh)
//! - Minimal probes (free + df + systemctl --failed) when snapshot is stale
//! - Known facts and recipes index
//!
//! Never calls specialist LLM for these query classes.
//!
//! v0.0.40: Uses RelevantHealthSummary for minimal, actionable responses.

use crate::facts::FactsStore;
use crate::health_view::{build_health_summary, has_health_issues};
use crate::recipe::{search_recipes_by_keywords, RecipeMatch};
use crate::snapshot::{
    diff_snapshots, format_deltas_text, has_actionable_deltas, load_last_snapshot, DeltaItem,
    SystemSnapshot, DISK_CRITICAL_THRESHOLD, DISK_WARN_THRESHOLD, MEMORY_HIGH_THRESHOLD,
};
use crate::trace::EvidenceKind;
use serde::{Deserialize, Serialize};

/// Default max age for snapshot freshness (seconds)
pub const DEFAULT_SNAPSHOT_MAX_AGE: u64 = 300;

/// Fast path query classes (subset of router::QueryClass for fast path)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FastPathClass {
    /// "how is my computer", "any errors", "any warnings", "status"
    SystemHealth,
    /// "disk usage", "how much disk space"
    DiskUsage,
    /// "memory usage", "how much memory"
    MemoryUsage,
    /// "failed services", "any failed units"
    FailedServices,
    /// "what changed since last time"
    WhatChanged,
    /// Not a fast path query
    NotFastPath,
}

impl std::fmt::Display for FastPathClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::SystemHealth => "system_health",
            Self::DiskUsage => "disk_usage",
            Self::MemoryUsage => "memory_usage",
            Self::FailedServices => "failed_services",
            Self::WhatChanged => "what_changed",
            Self::NotFastPath => "not_fast_path",
        };
        write!(f, "{}", s)
    }
}

/// Fast path answer result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastPathAnswer {
    /// Whether the fast path handled this query
    pub handled: bool,
    /// The answer text (empty if not handled)
    pub answer_text: String,
    /// Evidence kinds used to produce answer
    pub evidence_used: Vec<EvidenceKind>,
    /// Trace note for debug mode (e.g., "snapshot fresh, no probes needed")
    pub trace_note: String,
    /// Reliability hint (0-100, deterministic baseline)
    pub reliability_hint: u8,
    /// Fast path class that matched
    pub class: FastPathClass,
    /// Whether probes were run (false = answered from cache)
    pub probes_run: bool,
}

impl FastPathAnswer {
    /// Create a "not handled" result
    pub fn not_handled(reason: &str) -> Self {
        Self {
            handled: false,
            answer_text: String::new(),
            evidence_used: Vec::new(),
            trace_note: format!("fast path declined: {}", reason),
            reliability_hint: 0,
            class: FastPathClass::NotFastPath,
            probes_run: false,
        }
    }

    /// Create a handled result
    pub fn handled(
        class: FastPathClass,
        answer: String,
        evidence: Vec<EvidenceKind>,
        note: &str,
        reliability: u8,
        probes_run: bool,
    ) -> Self {
        Self {
            handled: true,
            answer_text: answer,
            evidence_used: evidence,
            trace_note: note.to_string(),
            reliability_hint: reliability,
            class,
            probes_run,
        }
    }
}

/// Fast path policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastPathPolicy {
    /// Maximum age of snapshot to consider fresh (seconds)
    pub snapshot_max_age_secs: u64,
    /// Whether to enable fast path at all
    pub enabled: bool,
    /// Minimum reliability score for fast path answers
    pub min_reliability: u8,
}

impl Default for FastPathPolicy {
    fn default() -> Self {
        Self {
            snapshot_max_age_secs: DEFAULT_SNAPSHOT_MAX_AGE,
            enabled: true,
            min_reliability: 70,
        }
    }
}

/// Input for fast path evaluation
pub struct FastPathInput<'a> {
    /// The user's request text
    pub request: &'a str,
    /// Last snapshot (if available)
    pub snapshot: Option<&'a SystemSnapshot>,
    /// Facts store for known facts
    pub facts: Option<&'a FactsStore>,
    /// Policy configuration
    pub policy: &'a FastPathPolicy,
}

/// Classify a query as fast path or not
pub fn classify_fast_path(query: &str) -> FastPathClass {
    let q = query.to_lowercase();

    // Strip common greetings
    let stripped = strip_greetings(&q);

    // SystemHealth: "how is my computer", "any errors", "status", "health"
    if stripped.contains("how is my computer")
        || stripped.contains("how's my computer")
        || stripped.contains("computer doing")
        || stripped.contains("any errors")
        || stripped.contains("any problems")
        || stripped.contains("any issues")
        || stripped.contains("any warnings")
        || stripped.contains("errors so far")
        || stripped.contains("problems so far")
        || stripped.contains("is everything ok")
        || stripped.contains("is everything okay")
        || q.contains("health")
        || q.trim() == "status"
        || q.trim() == "errors"
        || q.trim() == "warnings"
        || q.trim() == "problems"
    {
        return FastPathClass::SystemHealth;
    }

    // WhatChanged: "what changed", "changes since", "since last time"
    if stripped.contains("what changed")
        || stripped.contains("changes since")
        || stripped.contains("since last time")
        || stripped.contains("what's new")
        || stripped.contains("what's different")
    {
        return FastPathClass::WhatChanged;
    }

    // DiskUsage: "disk usage", "disk space", "how much disk"
    if stripped.contains("disk usage")
        || stripped.contains("disk space")
        || stripped.contains("how much disk")
        || stripped.contains("storage space")
    {
        return FastPathClass::DiskUsage;
    }

    // MemoryUsage: "memory usage", "how much memory", "ram usage"
    if stripped.contains("memory usage")
        || stripped.contains("how much memory")
        || stripped.contains("ram usage")
        || stripped.contains("how much ram")
    {
        return FastPathClass::MemoryUsage;
    }

    // FailedServices: "failed services", "failed units"
    if stripped.contains("failed service")
        || stripped.contains("failed unit")
        || stripped.contains("service failures")
    {
        return FastPathClass::FailedServices;
    }

    FastPathClass::NotFastPath
}

/// Strip common greetings from query for better classification
fn strip_greetings(query: &str) -> String {
    let q = query.to_lowercase();
    let patterns = [
        "hello", "hi ", "hey ", "good morning", "good afternoon", "good evening",
        "anna", ":)", ":(", ";)", ":d", ":p", "!", "?", "…", "...",
    ];
    let mut result = q;
    for p in patterns {
        result = result.replace(p, " ");
    }
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

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
        FastPathClass::NotFastPath => FastPathAnswer::not_handled("not fast path"),
    }
}

/// Answer system health from snapshot using RelevantHealthSummary (v0.0.40)
/// Only shows actionable issues - short "no issues" when healthy.
fn answer_system_health(snapshot: &SystemSnapshot, is_fresh: bool) -> FastPathAnswer {
    if !is_fresh {
        return FastPathAnswer::not_handled("snapshot stale, probes needed");
    }

    // Use the new RelevantHealthSummary for minimal, actionable output
    let summary = build_health_summary(snapshot, None);

    // Build evidence list based on what was checked
    let mut evidence = vec![EvidenceKind::Memory];
    if !snapshot.disk.is_empty() {
        evidence.push(EvidenceKind::Disk);
    }
    if !snapshot.failed_services.is_empty() || has_health_issues(snapshot) {
        evidence.push(EvidenceKind::FailedUnits);
    }

    let reliability = if summary.nothing_to_report { 90 } else { 85 };

    FastPathAnswer::handled(
        FastPathClass::SystemHealth,
        summary.format(),
        evidence,
        "answered from fresh snapshot (relevant health)",
        reliability,
        false, // no probes run
    )
}

/// Answer disk usage from snapshot
fn answer_disk_usage(snapshot: &SystemSnapshot, is_fresh: bool) -> FastPathAnswer {
    if !is_fresh {
        return FastPathAnswer::not_handled("snapshot stale, probes needed");
    }

    if snapshot.disk.is_empty() {
        return FastPathAnswer::not_handled("no disk data in snapshot");
    }

    let mut lines = vec!["**Disk Usage:**".to_string()];
    for (mount, &pct) in &snapshot.disk {
        let status = if pct >= DISK_CRITICAL_THRESHOLD {
            "CRITICAL"
        } else if pct >= DISK_WARN_THRESHOLD {
            "WARNING"
        } else {
            "OK"
        };
        lines.push(format!("  {} - {}% used [{}]", mount, pct, status));
    }

    FastPathAnswer::handled(
        FastPathClass::DiskUsage,
        lines.join("\n"),
        vec![EvidenceKind::Disk],
        "answered from fresh snapshot",
        88,
        false,
    )
}

/// Answer memory usage from snapshot
fn answer_memory_usage(snapshot: &SystemSnapshot, is_fresh: bool) -> FastPathAnswer {
    if !is_fresh {
        return FastPathAnswer::not_handled("snapshot stale, probes needed");
    }

    if snapshot.memory_total_bytes == 0 {
        return FastPathAnswer::not_handled("no memory data in snapshot");
    }

    let total_gb = snapshot.memory_total_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
    let used_gb = snapshot.memory_used_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
    let pct = snapshot.memory_percent();

    let status = if pct >= MEMORY_HIGH_THRESHOLD {
        "HIGH"
    } else {
        "OK"
    };

    let answer = format!(
        "**Memory Usage:**\n  {:.1} GB / {:.1} GB ({}%) [{}]",
        used_gb, total_gb, pct, status
    );

    FastPathAnswer::handled(
        FastPathClass::MemoryUsage,
        answer,
        vec![EvidenceKind::Memory],
        "answered from fresh snapshot",
        88,
        false,
    )
}

/// Answer failed services from snapshot
fn answer_failed_services(snapshot: &SystemSnapshot, is_fresh: bool) -> FastPathAnswer {
    if !is_fresh {
        return FastPathAnswer::not_handled("snapshot stale, probes needed");
    }

    let answer = if snapshot.failed_services.is_empty() {
        "No failed services. All systemd units are running normally.".to_string()
    } else {
        let mut lines = vec![format!(
            "**{} Failed Service(s):**",
            snapshot.failed_services.len()
        )];
        for svc in &snapshot.failed_services {
            lines.push(format!("  🔴 {}", svc));
        }
        lines.join("\n")
    };

    FastPathAnswer::handled(
        FastPathClass::FailedServices,
        answer,
        vec![EvidenceKind::FailedUnits],
        "answered from fresh snapshot",
        90,
        false,
    )
}

/// Answer "what changed since last time"
fn answer_what_changed(current: &SystemSnapshot) -> FastPathAnswer {
    // Load previous snapshot for comparison
    let prev = match load_last_snapshot() {
        Some(s) => s,
        None => {
            return FastPathAnswer::handled(
                FastPathClass::WhatChanged,
                "No previous snapshot available for comparison. This is the first check."
                    .to_string(),
                vec![],
                "no previous snapshot",
                75,
                false,
            );
        }
    };

    let deltas = diff_snapshots(&prev, current);

    if deltas.is_empty() {
        return FastPathAnswer::handled(
            FastPathClass::WhatChanged,
            "No significant changes since last check.".to_string(),
            vec![],
            "no deltas detected",
            85,
            false,
        );
    }

    // Collect evidence kinds from deltas
    let mut evidence = Vec::new();
    for delta in &deltas {
        match delta {
            DeltaItem::DiskWarning { .. }
            | DeltaItem::DiskCritical { .. }
            | DeltaItem::DiskIncreased { .. } => {
                if !evidence.contains(&EvidenceKind::Disk) {
                    evidence.push(EvidenceKind::Disk);
                }
            }
            DeltaItem::NewFailedService { .. } | DeltaItem::ServiceRecovered { .. } => {
                if !evidence.contains(&EvidenceKind::FailedUnits) {
                    evidence.push(EvidenceKind::FailedUnits);
                }
            }
            DeltaItem::MemoryHigh { .. } | DeltaItem::MemoryIncreased { .. } => {
                if !evidence.contains(&EvidenceKind::Memory) {
                    evidence.push(EvidenceKind::Memory);
                }
            }
        }
    }

    let answer = format_deltas_text(&deltas);
    let reliability = if has_actionable_deltas(&deltas) { 85 } else { 80 };

    FastPathAnswer::handled(
        FastPathClass::WhatChanged,
        answer,
        evidence,
        &format!("{} changes detected", deltas.len()),
        reliability,
        false,
    )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_system_health() {
        assert_eq!(
            classify_fast_path("how is my computer"),
            FastPathClass::SystemHealth
        );
        assert_eq!(
            classify_fast_path("any errors"),
            FastPathClass::SystemHealth
        );
        assert_eq!(
            classify_fast_path("hello anna :) how is my computer? any errors or problems so far?"),
            FastPathClass::SystemHealth
        );
    }

    #[test]
    fn test_classify_what_changed() {
        assert_eq!(
            classify_fast_path("what changed since last time"),
            FastPathClass::WhatChanged
        );
    }

    #[test]
    fn test_classify_disk_usage() {
        assert_eq!(
            classify_fast_path("disk usage"),
            FastPathClass::DiskUsage
        );
    }

    #[test]
    fn test_classify_not_fast_path() {
        assert_eq!(
            classify_fast_path("install vim"),
            FastPathClass::NotFastPath
        );
        assert_eq!(
            classify_fast_path("edit my vimrc"),
            FastPathClass::NotFastPath
        );
    }

    #[test]
    fn test_fast_path_disabled() {
        let policy = FastPathPolicy {
            enabled: false,
            ..Default::default()
        };
        let input = FastPathInput {
            request: "how is my computer",
            snapshot: None,
            facts: None,
            policy: &policy,
        };
        let result = try_fast_path(&input);
        assert!(!result.handled);
        assert!(result.trace_note.contains("disabled"));
    }
}
