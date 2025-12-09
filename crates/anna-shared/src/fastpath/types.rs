//! Fast path types (v0.0.185).

use crate::facts::FactsStore;
use crate::snapshot::SystemSnapshot;
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
