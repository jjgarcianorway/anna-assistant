//! ERA Pipeline (Part A) - v0.0.441.
//!
//! Universal Evidence → Reasoning → Answer pipeline.
//!
//! Every request MUST follow this pipeline:
//! 1) EVIDENCE STAGE (deterministic, no LLM creativity)
//! 2) REASONING STAGE (LLM, constrained to evidence only)
//! 3) ANSWER STAGE (translator, exact response)
//!
//! No stage may skip the previous one.

// Re-export types from submodules
pub use super::pipeline_types::{AnswerType, ExtractedIntent, PipelineStage, ProbeMapping};
pub use super::pipeline_state::{EraPipeline, PipelineError, PipelineStatus};
pub use super::pipeline_mapping::FactProbeTable;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::era_pipeline::evidence::EvidenceBundle;

    #[test]
    fn test_pipeline_stages() {
        assert_eq!(
            PipelineStage::Evidence.next(),
            Some(PipelineStage::Reasoning)
        );
        assert_eq!(PipelineStage::Reasoning.next(), Some(PipelineStage::Answer));
        assert_eq!(PipelineStage::Answer.next(), None);
    }

    #[test]
    fn test_answer_type_detection() {
        assert_eq!(AnswerType::from_intent("memory.free"), AnswerType::Numeric);
        assert_eq!(
            AnswerType::from_intent("service.is_running"),
            AnswerType::Boolean
        );
        assert_eq!(AnswerType::from_intent("boot.which_slow"), AnswerType::List);
        assert_eq!(AnswerType::from_intent("gpu.model"), AnswerType::Entity);
    }

    #[test]
    fn test_fact_probe_table() {
        let table = FactProbeTable::new();

        let probe = table.get_probe("memory.free_gib");
        assert!(probe.is_some());
        assert_eq!(probe.unwrap().probe_id, "free_h");

        let facts = vec![
            "memory.free_gib".to_string(),
            "boot.total_time_s".to_string(),
        ];
        let ids = table.unique_probe_ids(&facts);
        assert!(ids.contains(&"free_h".to_string()));
        assert!(ids.contains(&"systemd_analyze".to_string()));
    }

    #[test]
    fn test_pipeline_state_machine() {
        let mut pipeline = EraPipeline::new("DSK-0127");
        assert_eq!(pipeline.stage, PipelineStage::Evidence);
        assert!(!pipeline.can_proceed());

        // Set evidence
        let intent = ExtractedIntent {
            case_id: "DSK-0127".to_string(),
            raw_question: "How much RAM?".to_string(),
            intent: "memory.free".to_string(),
            required_facts: vec!["memory.free_gib".to_string()],
            answer_type: AnswerType::Numeric,
        };
        let evidence = EvidenceBundle::new("DSK-0127");
        pipeline.set_evidence(intent, evidence);

        assert!(pipeline.can_proceed());
        pipeline.advance().unwrap();
        assert_eq!(pipeline.stage, PipelineStage::Reasoning);
    }
}
