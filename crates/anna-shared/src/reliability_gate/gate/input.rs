//! Input for reliability gate evaluation.

use crate::reliability_gate::answer_contract::AnswerContract;
use crate::reliability_gate::claim_evidence::EvidenceBinding;

/// Input for reliability gate evaluation.
#[derive(Debug, Clone, Default)]
pub struct GateInput {
    /// Request ID
    pub request_id: String,
    /// Evidence binding (claims + evidence)
    pub binding: EvidenceBinding,
    /// Answer contract
    pub contract: Option<AnswerContract>,
    /// Did timeout occur?
    pub timeout_occurred: bool,
    /// Did parse error occur?
    pub parse_error_occurred: bool,
    /// Is this a generic/fallback answer?
    pub is_generic_answer: bool,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Original question text
    pub question: String,
    /// Detected domain
    pub domain: String,
    /// Probe domains actually used
    pub probe_domains: Vec<String>,
    /// Did any probe fail or timeout?
    pub probe_failed: bool,
    /// Did any probe return empty?
    pub probe_empty: bool,
    /// Entities in answer that need verification
    pub answer_entities: Vec<String>,
    /// Entities found in probe output
    pub evidence_entities: Vec<String>,
}

impl GateInput {
    /// Create new gate input.
    pub fn new(request_id: &str) -> Self {
        Self {
            request_id: request_id.to_string(),
            binding: EvidenceBinding::new(request_id),
            contract: None,
            timeout_occurred: false,
            parse_error_occurred: false,
            is_generic_answer: false,
            confidence: 1.0,
            question: String::new(),
            domain: String::new(),
            probe_domains: Vec::new(),
            probe_failed: false,
            probe_empty: false,
            answer_entities: Vec::new(),
            evidence_entities: Vec::new(),
        }
    }

    /// Set evidence binding.
    pub fn with_binding(mut self, binding: EvidenceBinding) -> Self {
        self.binding = binding;
        self
    }

    /// Set answer contract.
    pub fn with_contract(mut self, contract: AnswerContract) -> Self {
        self.contract = Some(contract);
        self
    }

    /// Mark timeout occurred.
    pub fn with_timeout(mut self) -> Self {
        self.timeout_occurred = true;
        self
    }

    /// Mark parse error occurred.
    pub fn with_parse_error(mut self) -> Self {
        self.parse_error_occurred = true;
        self
    }

    /// Mark as generic answer.
    pub fn with_generic_answer(mut self) -> Self {
        self.is_generic_answer = true;
        self
    }

    /// Set confidence.
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }

    /// Set original question.
    pub fn with_question(mut self, question: &str) -> Self {
        self.question = question.to_string();
        self
    }

    /// Set domain.
    pub fn with_domain(mut self, domain: &str) -> Self {
        self.domain = domain.to_string();
        self
    }

    /// Set probe domains.
    pub fn with_probe_domains(mut self, domains: Vec<String>) -> Self {
        self.probe_domains = domains;
        self
    }

    /// Mark probe failed.
    pub fn with_probe_failed(mut self) -> Self {
        self.probe_failed = true;
        self
    }

    /// Mark probe returned empty.
    pub fn with_probe_empty(mut self) -> Self {
        self.probe_empty = true;
        self
    }

    /// Set answer entities for hallucination check.
    pub fn with_answer_entities(mut self, entities: Vec<String>) -> Self {
        self.answer_entities = entities;
        self
    }

    /// Set evidence entities for hallucination check.
    pub fn with_evidence_entities(mut self, entities: Vec<String>) -> Self {
        self.evidence_entities = entities;
        self
    }
}
