//! Tests for ticket log module.

#[cfg(test)]
mod tests {
    use crate::rpc::{ProbeResult, SpecialistDomain};
    use crate::ticket_log::{ProbeLog, TicketLog};

    #[test]
    fn test_ticket_log_creation() {
        let log = TicketLog::new(
            "SVC-0001",
            SpecialistDomain::Services,
            "diagnose",
            "why is sshd failing",
        )
        .with_solver("llm:junior")
        .with_handler("llm:junior")
        .with_metrics(500, 85);

        assert_eq!(log.id, "SVC-0001");
        assert_eq!(log.domain, "services");
        assert_eq!(log.handled_by, "llm:junior");
        assert_eq!(log.reliability_score, 85);
    }

    #[test]
    fn test_probe_log_from() {
        let probe = ProbeResult {
            command: "systemctl --failed".to_string(),
            stdout: "0 failed units".to_string(),
            stderr: String::new(),
            exit_code: 0,
            timing_ms: 50,
        };

        let log = ProbeLog::from(&probe);
        assert_eq!(log.exit_code, 0);
        assert_eq!(log.duration_ms, 50);
    }

    #[test]
    fn test_truncate_output() {
        use super::super::types::truncate_output;
        
        let short = "hello";
        assert_eq!(truncate_output(short, 10), short);

        let long = "a".repeat(100);
        let truncated = truncate_output(&long, 20);
        assert!(truncated.contains("...[truncated]"));
        assert!(truncated.len() < 50);
    }
}
