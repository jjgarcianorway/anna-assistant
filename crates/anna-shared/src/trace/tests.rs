//! Tests for trace module (v0.0.184).

#[cfg(test)]
mod tests {
    use crate::trace::{
        EvidenceKind, ExecutionTrace, FallbackUsed, ProbeStats,
    };

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
            ProbeStats {
                planned: 1,
                succeeded: 1,
                failed: 0,
                timed_out: 0,
            },
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
            ProbeStats {
                planned: 4,
                succeeded: 3,
                failed: 0,
                timed_out: 1,
            },
            vec![EvidenceKind::Memory, EvidenceKind::Disk, EvidenceKind::Cpu],
        );
        assert_eq!(
            trace.to_string(),
            "path: deterministic fallback (system_health_summary), specialist: timeout, evidence: [memory, disk, cpu]"
        );
    }

    #[test]
    fn test_probe_stats_display() {
        let stats = ProbeStats {
            planned: 4,
            succeeded: 3,
            failed: 0,
            timed_out: 1,
        };
        assert_eq!(stats.to_string(), "3/4 probes succeeded, 1 timed out");
    }

    #[test]
    fn test_probe_stats_with_failures() {
        let stats = ProbeStats {
            planned: 5,
            succeeded: 2,
            failed: 2,
            timed_out: 1,
        };
        assert_eq!(
            stats.to_string(),
            "2/5 probes succeeded, 2 failed, 1 timed out"
        );
    }

    #[test]
    fn test_execution_trace_serialization() {
        let trace = ExecutionTrace::specialist_timeout_with_fallback(
            "disk_usage",
            ProbeStats {
                planned: 1,
                succeeded: 1,
                failed: 0,
                timed_out: 0,
            },
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
