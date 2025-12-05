//! Execution trace for auditable request processing.
//!
//! Provides structured, deterministic trace of which stages ran,
//! which failed, and which path produced the final answer.
//! No timestamps - only enums and counts for reproducibility.

use serde::{Deserialize, Serialize};

/// Outcome of the specialist stage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecialistOutcome {
    /// Specialist LLM produced an answer
    Ok,
    /// Specialist LLM timed out
    Timeout,
    /// Specialist exceeded budget before completing
    BudgetExceeded,
    /// Specialist was skipped (deterministic route answered directly)
    Skipped,
    /// Specialist returned an error
    Error,
}

impl std::fmt::Display for SpecialistOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Ok => "ok",
            Self::Timeout => "timeout",
            Self::BudgetExceeded => "budget_exceeded",
            Self::Skipped => "skipped",
            Self::Error => "error",
        };
        write!(f, "{}", s)
    }
}

/// What fallback was used when specialist failed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FallbackUsed {
    /// No fallback needed - specialist succeeded or was skipped
    None,
    /// Deterministic answerer produced the response
    Deterministic {
        /// The query class that was used for deterministic routing
        route_class: String,
    },
}

impl std::fmt::Display for FallbackUsed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Deterministic { route_class } => {
                write!(f, "deterministic ({})", route_class)
            }
        }
    }
}

/// Probe execution summary
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProbeStats {
    /// Number of probes planned by router/translator
    pub planned: usize,
    /// Number of probes that succeeded (exit_code == 0)
    pub succeeded: usize,
    /// Number of probes that failed (exit_code != 0, not timeout)
    pub failed: usize,
    /// Number of probes that timed out
    pub timed_out: usize,
}

impl ProbeStats {
    /// Create stats from probe results
    pub fn from_results(planned: usize, results: &[crate::rpc::ProbeResult]) -> Self {
        let succeeded = results.iter().filter(|p| p.exit_code == 0).count();
        let timed_out = results
            .iter()
            .filter(|p| p.stderr.to_lowercase().contains("timeout"))
            .count();
        let failed = results
            .iter()
            .filter(|p| p.exit_code != 0 && !p.stderr.to_lowercase().contains("timeout"))
            .count();

        Self { planned, succeeded, failed, timed_out }
    }
}

impl std::fmt::Display for ProbeStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{} probes succeeded",
            self.succeeded, self.planned
        )?;
        if self.failed > 0 {
            write!(f, ", {} failed", self.failed)?;
        }
        if self.timed_out > 0 {
            write!(f, ", {} timed out", self.timed_out)?;
        }
        Ok(())
    }
}

/// Parsed evidence kinds present in the response
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Memory,
    Disk,
    BlockDevices,
    Cpu,
    Services,
}

impl std::fmt::Display for EvidenceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Memory => "memory",
            Self::Disk => "disk",
            Self::BlockDevices => "block_devices",
            Self::Cpu => "cpu",
            Self::Services => "services",
        };
        write!(f, "{}", s)
    }
}

/// Derive evidence kinds from route class name
pub fn evidence_kinds_from_route(route_class: &str) -> Vec<EvidenceKind> {
    match route_class {
        "memory_usage" | "ram_info" => vec![EvidenceKind::Memory],
        "disk_usage" | "disk_space" => vec![EvidenceKind::Disk],
        "cpu_info" => vec![EvidenceKind::Cpu],
        "service_status" => vec![EvidenceKind::Services],
        "system_health_summary" => vec![
            EvidenceKind::Memory,
            EvidenceKind::Disk,
            EvidenceKind::BlockDevices,
            EvidenceKind::Cpu,
        ],
        _ => vec![],
    }
}

/// Full execution trace for a request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionTrace {
    /// Outcome of the specialist stage
    pub specialist_outcome: SpecialistOutcome,
    /// What fallback was used (if any)
    pub fallback_used: FallbackUsed,
    /// Probe execution statistics
    pub probe_stats: ProbeStats,
    /// Evidence kinds parsed from probe data
    pub evidence_kinds: Vec<EvidenceKind>,
    /// Whether the final answer came from deterministic path
    pub answer_is_deterministic: bool,
}

impl ExecutionTrace {
    /// Create a trace for successful specialist response
    pub fn specialist_ok(probe_stats: ProbeStats) -> Self {
        Self {
            specialist_outcome: SpecialistOutcome::Ok,
            fallback_used: FallbackUsed::None,
            probe_stats,
            evidence_kinds: vec![],
            answer_is_deterministic: false,
        }
    }

    /// Create a trace for skipped specialist (deterministic route)
    pub fn deterministic_route(_route_class: &str, probe_stats: ProbeStats, evidence_kinds: Vec<EvidenceKind>) -> Self {
        Self {
            specialist_outcome: SpecialistOutcome::Skipped,
            fallback_used: FallbackUsed::None,
            probe_stats,
            evidence_kinds,
            answer_is_deterministic: true,
        }
    }

    /// Create a trace for specialist timeout with fallback
    pub fn specialist_timeout_with_fallback(route_class: &str, probe_stats: ProbeStats, evidence_kinds: Vec<EvidenceKind>) -> Self {
        Self {
            specialist_outcome: SpecialistOutcome::Timeout,
            fallback_used: FallbackUsed::Deterministic { route_class: route_class.to_string() },
            probe_stats,
            evidence_kinds,
            answer_is_deterministic: true,
        }
    }

    /// Create a trace for specialist error with fallback
    pub fn specialist_error_with_fallback(route_class: &str, probe_stats: ProbeStats, evidence_kinds: Vec<EvidenceKind>) -> Self {
        Self {
            specialist_outcome: SpecialistOutcome::Error,
            fallback_used: FallbackUsed::Deterministic { route_class: route_class.to_string() },
            probe_stats,
            evidence_kinds,
            answer_is_deterministic: true,
        }
    }

    /// Create a trace for specialist timeout without successful fallback
    pub fn specialist_timeout_no_fallback(probe_stats: ProbeStats) -> Self {
        Self {
            specialist_outcome: SpecialistOutcome::Timeout,
            fallback_used: FallbackUsed::None,
            probe_stats,
            evidence_kinds: vec![],
            answer_is_deterministic: false,
        }
    }
}

impl std::fmt::Display for ExecutionTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "path: ")?;
        if self.answer_is_deterministic {
            match &self.fallback_used {
                FallbackUsed::None => write!(f, "deterministic route")?,
                FallbackUsed::Deterministic { route_class } => {
                    write!(f, "deterministic fallback ({})", route_class)?
                }
            }
        } else {
            write!(f, "specialist")?;
        }

        write!(f, ", specialist: {}", self.specialist_outcome)?;

        if !self.evidence_kinds.is_empty() {
            let kinds: Vec<_> = self.evidence_kinds.iter().map(|k| k.to_string()).collect();
            write!(f, ", evidence: [{}]", kinds.join(", "))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specialist_ok_display() {
        let trace = ExecutionTrace::specialist_ok(ProbeStats {
            planned: 2,
            succeeded: 2,
            failed: 0,
            timed_out: 0,
        });
        assert_eq!(trace.to_string(), "path: specialist, specialist: ok");
    }

    #[test]
    fn test_deterministic_route_display() {
        let trace = ExecutionTrace::deterministic_route(
            "memory_usage",
            ProbeStats { planned: 1, succeeded: 1, failed: 0, timed_out: 0 },
            vec![EvidenceKind::Memory],
        );
        assert_eq!(
            trace.to_string(),
            "path: deterministic route, specialist: skipped, evidence: [memory]"
        );
    }

    #[test]
    fn test_timeout_with_fallback_display() {
        let trace = ExecutionTrace::specialist_timeout_with_fallback(
            "system_health_summary",
            ProbeStats { planned: 4, succeeded: 3, failed: 0, timed_out: 1 },
            vec![EvidenceKind::Memory, EvidenceKind::Disk, EvidenceKind::Cpu],
        );
        assert_eq!(
            trace.to_string(),
            "path: deterministic fallback (system_health_summary), specialist: timeout, evidence: [memory, disk, cpu]"
        );
    }

    #[test]
    fn test_probe_stats_display() {
        let stats = ProbeStats { planned: 4, succeeded: 3, failed: 0, timed_out: 1 };
        assert_eq!(stats.to_string(), "3/4 probes succeeded, 1 timed out");
    }

    #[test]
    fn test_probe_stats_with_failures() {
        let stats = ProbeStats { planned: 5, succeeded: 2, failed: 2, timed_out: 1 };
        assert_eq!(stats.to_string(), "2/5 probes succeeded, 2 failed, 1 timed out");
    }

    #[test]
    fn test_execution_trace_serialization() {
        let trace = ExecutionTrace::specialist_timeout_with_fallback(
            "disk_usage",
            ProbeStats { planned: 1, succeeded: 1, failed: 0, timed_out: 0 },
            vec![EvidenceKind::Disk],
        );
        let json = serde_json::to_string(&trace).unwrap();
        let parsed: ExecutionTrace = serde_json::from_str(&json).unwrap();
        assert_eq!(trace, parsed);
    }

    #[test]
    fn test_fallback_used_serialization() {
        let fallback = FallbackUsed::Deterministic {
            route_class: "memory_usage".to_string(),
        };
        let json = serde_json::to_string(&fallback).unwrap();
        assert!(json.contains("deterministic"));
        assert!(json.contains("memory_usage"));
    }
}
