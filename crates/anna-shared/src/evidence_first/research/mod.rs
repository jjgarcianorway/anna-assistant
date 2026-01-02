//! Research module - Evidence-First Research (v0.0.435).

mod research_helpers;
mod research_loop;
mod research_types;

pub use research_helpers::QuickResearch;
pub use research_loop::ResearchLoop;
pub use research_types::{Confidence, DocResult, Finding, ResearchPlan, ResearchResult};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence_first::probe_plan::ProbeOutput;

    #[test]
    fn test_research_plan_creation() {
        let plan = ResearchPlan::new("ticket-123")
            .with_keywords(vec!["boot".to_string(), "slow".to_string()])
            .with_commands(vec!["systemctl".to_string()]);

        assert_eq!(plan.ticket_id, "ticket-123");
        assert_eq!(plan.keywords.len(), 2);
        assert_eq!(plan.commands.len(), 1);
    }

    #[test]
    fn test_research_plan_iterations() {
        let mut plan = ResearchPlan::new("test");
        assert!(plan.can_iterate());

        plan.next_iteration();
        assert!(plan.can_iterate());

        plan.next_iteration();
        assert!(!plan.can_iterate());
    }

    #[test]
    fn test_finding_confidence() {
        let unsupported = Finding::new("claim", vec![]);
        assert!(matches!(unsupported.confidence, Confidence::Unsupported));

        let medium = Finding::new("claim", vec!["ev1".to_string()]);
        assert!(matches!(medium.confidence, Confidence::Medium));

        let high = Finding::new("claim", vec!["ev1".to_string(), "ev2".to_string()]);
        assert!(matches!(high.confidence, Confidence::High));
    }

    #[test]
    fn test_research_result_counts() {
        let mut result = ResearchResult::new("test");

        result.probe_outputs.push(ProbeOutput {
            primitive_id: "test".to_string(),
            raw_output: "output".to_string(),
            parsed: None,
            exit_code: Some(0),
            execution_time_ms: 100,
            error: None,
        });

        result.docs_retrieved.push(DocResult {
            source_type: "man".to_string(),
            name: "test".to_string(),
            success: true,
            excerpts: vec![],
        });

        assert_eq!(result.evidence_count(), 2);
    }

    #[test]
    fn test_research_loop_creation() {
        let _loop = ResearchLoop::new();
        // Just verify it can be created
        assert!(true);
    }
}
