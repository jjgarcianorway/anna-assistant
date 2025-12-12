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

use super::evidence::{EvidenceBundle, FactValue};
use super::reasoning::{ReasoningOutput, ReasoningRequest};

/// Pipeline stage identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PipelineStage {
    /// Extract intent and required facts.
    Evidence,
    /// Reason over evidence only.
    Reasoning,
    /// Convert reasoning to user answer.
    Answer,
}

impl PipelineStage {
    /// Get stage label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Evidence => "EVIDENCE",
            Self::Reasoning => "REASONING",
            Self::Answer => "ANSWER",
        }
    }

    /// Get next stage.
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Evidence => Some(Self::Reasoning),
            Self::Reasoning => Some(Self::Answer),
            Self::Answer => None,
        }
    }
}

/// Intent extracted from user question.
#[derive(Debug, Clone)]
pub struct ExtractedIntent {
    /// Case ID.
    pub case_id: String,
    /// Raw question from user.
    pub raw_question: String,
    /// Canonical intent (e.g., "memory.free", "boot.slow_service").
    pub intent: String,
    /// Required facts to answer this question.
    pub required_facts: Vec<String>,
    /// Expected answer type.
    pub answer_type: AnswerType,
}

/// Expected answer type for translator precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswerType {
    /// Numeric value (e.g., "17.0 GiB").
    Numeric,
    /// Yes/No + 1 sentence max.
    Boolean,
    /// List of items.
    List,
    /// Single entity name.
    Entity,
    /// Brief explanation (2-3 sentences max).
    Brief,
}

impl AnswerType {
    /// Parse from intent pattern.
    pub fn from_intent(intent: &str) -> Self {
        let lower = intent.to_lowercase();

        // Numeric patterns
        if lower.contains("how much")
            || lower.contains("how many")
            || lower.contains("size")
            || lower.contains("free")
            || lower.contains("usage")
            || lower.contains("time")
            || lower.contains("temp")
        {
            return Self::Numeric;
        }

        // Boolean patterns
        if lower.contains("is_")
            || lower.contains("has_")
            || lower.contains("enabled")
            || lower.contains("running")
            || lower.contains("installed")
        {
            return Self::Boolean;
        }

        // List patterns
        if lower.contains("which")
            || lower.contains("list")
            || lower.contains("all_")
            || lower.contains("failed")
        {
            return Self::List;
        }

        // Entity patterns
        if lower.contains("what_")
            || lower.contains("model")
            || lower.contains("version")
            || lower.contains("name")
        {
            return Self::Entity;
        }

        Self::Brief
    }

    /// Get label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Numeric => "numeric",
            Self::Boolean => "boolean",
            Self::List => "list",
            Self::Entity => "entity",
            Self::Brief => "brief",
        }
    }
}

/// Probe mapping for required facts.
#[derive(Debug, Clone)]
pub struct ProbeMapping {
    /// Fact name (e.g., "memory.free_gib").
    pub fact_name: String,
    /// Probe ID to run.
    pub probe_id: String,
    /// Extraction function name.
    pub extractor: String,
}

/// ERA Pipeline state machine.
#[derive(Debug, Clone)]
pub struct EraPipeline {
    /// Case ID.
    pub case_id: String,
    /// Current stage.
    pub stage: PipelineStage,
    /// Extracted intent (set after Evidence stage).
    pub intent: Option<ExtractedIntent>,
    /// Evidence bundle (set after Evidence stage).
    pub evidence: Option<EvidenceBundle>,
    /// Reasoning output (set after Reasoning stage).
    pub reasoning: Option<ReasoningOutput>,
    /// Final answer (set after Answer stage).
    pub answer: Option<String>,
    /// Pipeline errors.
    pub errors: Vec<PipelineError>,
}

/// Pipeline error.
#[derive(Debug, Clone)]
pub struct PipelineError {
    /// Stage where error occurred.
    pub stage: PipelineStage,
    /// Error message.
    pub message: String,
    /// Whether pipeline can continue.
    pub recoverable: bool,
}

impl EraPipeline {
    /// Create new pipeline.
    pub fn new(case_id: &str) -> Self {
        Self {
            case_id: case_id.to_string(),
            stage: PipelineStage::Evidence,
            intent: None,
            evidence: None,
            reasoning: None,
            answer: None,
            errors: Vec::new(),
        }
    }

    /// Check if pipeline can proceed to next stage.
    pub fn can_proceed(&self) -> bool {
        match self.stage {
            PipelineStage::Evidence => self.intent.is_some() && self.evidence.is_some(),
            PipelineStage::Reasoning => self.reasoning.is_some(),
            PipelineStage::Answer => false, // Terminal stage
        }
    }

    /// Advance to next stage.
    pub fn advance(&mut self) -> Result<(), PipelineError> {
        if !self.can_proceed() {
            return Err(PipelineError {
                stage: self.stage,
                message: format!(
                    "Cannot proceed from {} - prerequisites not met",
                    self.stage.label()
                ),
                recoverable: false,
            });
        }

        if let Some(next) = self.stage.next() {
            self.stage = next;
            Ok(())
        } else {
            Err(PipelineError {
                stage: self.stage,
                message: "Pipeline already complete".to_string(),
                recoverable: false,
            })
        }
    }

    /// Set evidence stage results.
    pub fn set_evidence(&mut self, intent: ExtractedIntent, evidence: EvidenceBundle) {
        self.intent = Some(intent);
        self.evidence = Some(evidence);
    }

    /// Set reasoning stage results.
    pub fn set_reasoning(&mut self, reasoning: ReasoningOutput) {
        self.reasoning = Some(reasoning);
    }

    /// Set answer stage results.
    pub fn set_answer(&mut self, answer: String) {
        self.answer = Some(answer);
    }

    /// Record an error.
    pub fn record_error(&mut self, error: PipelineError) {
        self.errors.push(error);
    }

    /// Check if pipeline completed successfully.
    pub fn is_complete(&self) -> bool {
        self.stage == PipelineStage::Answer && self.answer.is_some()
    }

    /// Check if pipeline failed.
    pub fn is_failed(&self) -> bool {
        self.errors.iter().any(|e| !e.recoverable)
    }

    /// Get completion status.
    pub fn status(&self) -> PipelineStatus {
        if self.is_complete() {
            PipelineStatus::Complete
        } else if self.is_failed() {
            PipelineStatus::Failed
        } else {
            PipelineStatus::InProgress(self.stage)
        }
    }
}

/// Pipeline completion status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineStatus {
    /// Pipeline in progress at stage.
    InProgress(PipelineStage),
    /// Pipeline completed successfully.
    Complete,
    /// Pipeline failed.
    Failed,
}

/// Fact → Probe mapping table.
pub struct FactProbeTable {
    /// Mappings.
    mappings: Vec<ProbeMapping>,
}

impl FactProbeTable {
    /// Create with default mappings.
    pub fn new() -> Self {
        let mappings = vec![
            // Memory facts
            ProbeMapping {
                fact_name: "memory.free_gib".to_string(),
                probe_id: "free_h".to_string(),
                extractor: "extract_memory_free".to_string(),
            },
            ProbeMapping {
                fact_name: "memory.total_gib".to_string(),
                probe_id: "free_h".to_string(),
                extractor: "extract_memory_total".to_string(),
            },
            ProbeMapping {
                fact_name: "memory.used_pct".to_string(),
                probe_id: "free_h".to_string(),
                extractor: "extract_memory_used_pct".to_string(),
            },
            // Boot facts
            ProbeMapping {
                fact_name: "boot.total_time_s".to_string(),
                probe_id: "systemd_analyze".to_string(),
                extractor: "extract_boot_time".to_string(),
            },
            ProbeMapping {
                fact_name: "boot.blame".to_string(),
                probe_id: "systemd_blame".to_string(),
                extractor: "extract_blame_list".to_string(),
            },
            ProbeMapping {
                fact_name: "boot.slowest_service".to_string(),
                probe_id: "systemd_blame".to_string(),
                extractor: "extract_slowest_service".to_string(),
            },
            // CPU facts
            ProbeMapping {
                fact_name: "cpu.model".to_string(),
                probe_id: "lscpu".to_string(),
                extractor: "extract_cpu_model".to_string(),
            },
            ProbeMapping {
                fact_name: "cpu.cores".to_string(),
                probe_id: "lscpu".to_string(),
                extractor: "extract_cpu_cores".to_string(),
            },
            ProbeMapping {
                fact_name: "cpu.temp_c".to_string(),
                probe_id: "sensors".to_string(),
                extractor: "extract_cpu_temp".to_string(),
            },
            ProbeMapping {
                fact_name: "cpu.load_1m".to_string(),
                probe_id: "uptime".to_string(),
                extractor: "extract_load_1m".to_string(),
            },
            // Disk facts
            ProbeMapping {
                fact_name: "disk.root_free_gib".to_string(),
                probe_id: "df_h".to_string(),
                extractor: "extract_root_free".to_string(),
            },
            ProbeMapping {
                fact_name: "disk.root_used_pct".to_string(),
                probe_id: "df_h".to_string(),
                extractor: "extract_root_used_pct".to_string(),
            },
            ProbeMapping {
                fact_name: "disk.trim_enabled".to_string(),
                probe_id: "fstrim_status".to_string(),
                extractor: "extract_trim_status".to_string(),
            },
            // GPU facts
            ProbeMapping {
                fact_name: "gpu.model".to_string(),
                probe_id: "lspci_gpu".to_string(),
                extractor: "extract_gpu_model".to_string(),
            },
            ProbeMapping {
                fact_name: "gpu.driver".to_string(),
                probe_id: "lspci_k_gpu".to_string(),
                extractor: "extract_gpu_driver".to_string(),
            },
            // Service facts
            ProbeMapping {
                fact_name: "services.failed_count".to_string(),
                probe_id: "systemctl_failed".to_string(),
                extractor: "extract_failed_count".to_string(),
            },
            ProbeMapping {
                fact_name: "services.failed_list".to_string(),
                probe_id: "systemctl_failed".to_string(),
                extractor: "extract_failed_list".to_string(),
            },
        ];

        Self { mappings }
    }

    /// Get probe for a fact.
    pub fn get_probe(&self, fact_name: &str) -> Option<&ProbeMapping> {
        self.mappings.iter().find(|m| m.fact_name == fact_name)
    }

    /// Get all probes needed for a set of facts.
    pub fn get_probes_for_facts(&self, facts: &[String]) -> Vec<&ProbeMapping> {
        facts.iter().filter_map(|f| self.get_probe(f)).collect()
    }

    /// Get unique probe IDs needed.
    pub fn unique_probe_ids(&self, facts: &[String]) -> Vec<String> {
        let mut ids: Vec<String> = self
            .get_probes_for_facts(facts)
            .iter()
            .map(|m| m.probe_id.clone())
            .collect();
        ids.sort();
        ids.dedup();
        ids
    }
}

impl Default for FactProbeTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
