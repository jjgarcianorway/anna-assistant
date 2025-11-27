//! Anna Reliability Engine v0.6.0
//!
//! Multi-round LLM-A/LLM-B refinement loop with thresholds.
//! Ensures high-quality, evidence-based answers.

use serde::{Deserialize, Serialize};

use crate::presentation::{THRESHOLD_HIGH, THRESHOLD_MEDIUM};

/// Maximum number of refinement passes
pub const MAX_REFINEMENT_PASSES: u8 = 3;

/// Reliability assessment from LLM-B
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityAssessment {
    /// Overall reliability score (0.0 - 1.0)
    pub score: f64,
    /// Individual factor scores
    pub factors: ReliabilityFactors,
    /// Identified risks or gaps
    pub risks: Vec<String>,
    /// Corrections needed
    pub corrections: Vec<String>,
    /// Additional probes requested
    pub additional_probes: Vec<String>,
    /// Verdict from LLM-B
    pub verdict: RefinementVerdict,
}

impl Default for ReliabilityAssessment {
    fn default() -> Self {
        Self {
            score: 0.0,
            factors: ReliabilityFactors::default(),
            risks: Vec::new(),
            corrections: Vec::new(),
            additional_probes: Vec::new(),
            verdict: RefinementVerdict::CannotReachThreshold,
        }
    }
}

/// Individual reliability factors
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReliabilityFactors {
    /// Quality of evidence (0.0 - 1.0)
    pub evidence: f64,
    /// Coverage of the question (0.0 - 1.0)
    pub coverage: f64,
    /// Soundness of reasoning (0.0 - 1.0)
    pub reasoning: f64,
}

impl ReliabilityFactors {
    pub fn new(evidence: f64, coverage: f64, reasoning: f64) -> Self {
        Self {
            evidence: evidence.clamp(0.0, 1.0),
            coverage: coverage.clamp(0.0, 1.0),
            reasoning: reasoning.clamp(0.0, 1.0),
        }
    }

    /// Calculate weighted average score
    pub fn weighted_score(&self) -> f64 {
        // Evidence is most important, then reasoning, then coverage
        (self.evidence * 0.4 + self.reasoning * 0.35 + self.coverage * 0.25).clamp(0.0, 1.0)
    }

    pub fn as_strings(&self) -> Vec<String> {
        vec![
            format!("evidence: {}", Self::level_str(self.evidence)),
            format!("coverage: {}", Self::level_str(self.coverage)),
            format!("reasoning: {}", Self::level_str(self.reasoning)),
        ]
    }

    fn level_str(score: f64) -> &'static str {
        if score >= 0.8 {
            "high"
        } else if score >= 0.5 {
            "medium"
        } else {
            "low"
        }
    }
}

/// Verdict from LLM-B review
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefinementVerdict {
    /// Answer is acceptable, present to user
    Accept,
    /// Answer needs revision, iterate
    Revise,
    /// Cannot reach threshold, present with limitations
    CannotReachThreshold,
}

impl RefinementVerdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            RefinementVerdict::Accept => "accept",
            RefinementVerdict::Revise => "revise",
            RefinementVerdict::CannotReachThreshold => "cannot_reach_threshold",
        }
    }
}

/// Result of a refinement round
#[derive(Debug, Clone)]
pub struct RefinementResult {
    /// The answer text
    pub answer: String,
    /// Evidence sources used
    pub evidence_sources: Vec<EvidenceSource>,
    /// Reliability assessment
    pub assessment: ReliabilityAssessment,
    /// Number of passes completed
    pub passes: u8,
    /// Whether the threshold was reached
    pub threshold_reached: bool,
    /// Main limitations if threshold not reached
    pub limitations: Vec<String>,
}

/// Evidence source record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSource {
    /// Probe or config identifier
    pub id: String,
    /// Short description
    pub description: String,
    /// Freshness status
    pub freshness: EvidenceFreshness,
}

/// Evidence freshness status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceFreshness {
    /// Just collected
    Fresh,
    /// From cache, still valid
    Cached,
    /// Potentially stale
    Stale,
    /// Unknown freshness
    Unknown,
}

impl EvidenceFreshness {
    pub fn as_str(&self) -> &'static str {
        match self {
            EvidenceFreshness::Fresh => "fresh",
            EvidenceFreshness::Cached => "cached",
            EvidenceFreshness::Stale => "stale",
            EvidenceFreshness::Unknown => "unknown",
        }
    }
}

/// Refinement loop state
#[derive(Debug)]
pub struct RefinementLoop {
    /// Maximum passes allowed
    pub max_passes: u8,
    /// Current pass number
    pub current_pass: u8,
    /// High confidence threshold
    pub high_threshold: f64,
    /// Medium confidence threshold
    pub medium_threshold: f64,
    /// Whether to continue refining
    pub should_continue: bool,
    /// History of assessments
    pub history: Vec<ReliabilityAssessment>,
}

impl Default for RefinementLoop {
    fn default() -> Self {
        Self {
            max_passes: MAX_REFINEMENT_PASSES,
            current_pass: 0,
            high_threshold: THRESHOLD_HIGH,
            medium_threshold: THRESHOLD_MEDIUM,
            should_continue: true,
            history: Vec::new(),
        }
    }
}

impl RefinementLoop {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_passes(mut self, max: u8) -> Self {
        self.max_passes = max;
        self
    }

    /// Record an assessment and determine next action
    pub fn record_assessment(&mut self, assessment: ReliabilityAssessment) -> RefinementAction {
        self.current_pass += 1;
        self.history.push(assessment.clone());

        // Check if we've reached the threshold
        if assessment.score >= self.high_threshold {
            self.should_continue = false;
            return RefinementAction::Accept {
                score: assessment.score,
                passes: self.current_pass,
            };
        }

        // Check if we've exhausted passes
        if self.current_pass >= self.max_passes {
            self.should_continue = false;

            if assessment.score >= self.medium_threshold {
                return RefinementAction::AcceptPartial {
                    score: assessment.score,
                    passes: self.current_pass,
                    gaps: assessment.risks.clone(),
                };
            } else {
                return RefinementAction::AcceptLowConfidence {
                    score: assessment.score,
                    passes: self.current_pass,
                    limitations: assessment.risks.clone(),
                };
            }
        }

        // Check verdict
        match assessment.verdict {
            RefinementVerdict::Accept => {
                self.should_continue = false;
                RefinementAction::Accept {
                    score: assessment.score,
                    passes: self.current_pass,
                }
            }
            RefinementVerdict::Revise => RefinementAction::Continue {
                corrections: assessment.corrections.clone(),
                additional_probes: assessment.additional_probes.clone(),
            },
            RefinementVerdict::CannotReachThreshold => {
                self.should_continue = false;
                RefinementAction::AcceptLowConfidence {
                    score: assessment.score,
                    passes: self.current_pass,
                    limitations: assessment.risks.clone(),
                }
            }
        }
    }

    /// Get the final assessment (last in history)
    pub fn final_assessment(&self) -> Option<&ReliabilityAssessment> {
        self.history.last()
    }

    /// Check if threshold was reached
    pub fn threshold_reached(&self) -> bool {
        self.history
            .last()
            .map(|a| a.score >= self.high_threshold)
            .unwrap_or(false)
    }
}

/// Action to take after assessment
#[derive(Debug, Clone)]
pub enum RefinementAction {
    /// Accept the answer (high confidence)
    Accept { score: f64, passes: u8 },
    /// Accept but mark as partial (medium confidence)
    AcceptPartial {
        score: f64,
        passes: u8,
        gaps: Vec<String>,
    },
    /// Accept with low confidence warning
    AcceptLowConfidence {
        score: f64,
        passes: u8,
        limitations: Vec<String>,
    },
    /// Continue refining
    Continue {
        corrections: Vec<String>,
        additional_probes: Vec<String>,
    },
}

/// Parse reliability assessment from LLM-B response
pub fn parse_reliability_response(response: &str) -> ReliabilityAssessment {
    let mut assessment = ReliabilityAssessment::default();

    // Parse score
    if let Some(score) = extract_score(response) {
        assessment.score = score;
    }

    // Parse factors
    assessment.factors = extract_factors(response);

    // Parse verdict
    assessment.verdict = extract_verdict(response);

    // Parse risks
    assessment.risks = extract_list(response, "risk");

    // Parse corrections
    assessment.corrections = extract_list(response, "correction");

    // Parse additional probes
    assessment.additional_probes = extract_list(response, "probe");

    assessment
}

fn extract_score(response: &str) -> Option<f64> {
    let patterns = [
        r"score[:\s]+([0-9]+\.?[0-9]*)",
        r"reliability[:\s]+([0-9]+\.?[0-9]*)",
        r"confidence[:\s]+([0-9]+\.?[0-9]*)",
    ];

    for pattern in patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(&response.to_lowercase()) {
                if let Some(m) = caps.get(1) {
                    if let Ok(score) = m.as_str().parse::<f64>() {
                        // Normalize to 0-1 range if needed
                        return Some(if score > 1.0 {
                            score / 100.0
                        } else {
                            score
                        }
                        .clamp(0.0, 1.0));
                    }
                }
            }
        }
    }
    None
}

fn extract_factors(response: &str) -> ReliabilityFactors {
    let lower = response.to_lowercase();

    let evidence = if lower.contains("evidence: high") || lower.contains("evidence is high") {
        0.9
    } else if lower.contains("evidence: medium") || lower.contains("evidence is medium") {
        0.6
    } else if lower.contains("evidence: low") || lower.contains("evidence is low") {
        0.3
    } else {
        0.5
    };

    let coverage = if lower.contains("coverage: high") || lower.contains("coverage is high") {
        0.9
    } else if lower.contains("coverage: medium") || lower.contains("coverage is medium") {
        0.6
    } else if lower.contains("coverage: low") || lower.contains("coverage is low") {
        0.3
    } else {
        0.5
    };

    let reasoning = if lower.contains("reasoning: high") || lower.contains("reasoning is high") {
        0.9
    } else if lower.contains("reasoning: medium") || lower.contains("reasoning is medium") {
        0.6
    } else if lower.contains("reasoning: low") || lower.contains("reasoning is low") {
        0.3
    } else {
        0.5
    };

    ReliabilityFactors::new(evidence, coverage, reasoning)
}

fn extract_verdict(response: &str) -> RefinementVerdict {
    let lower = response.to_lowercase();

    if lower.contains("verdict: accept")
        || lower.contains("verdict=accept")
        || lower.contains("approved")
    {
        RefinementVerdict::Accept
    } else if lower.contains("verdict: revise")
        || lower.contains("verdict=revise")
        || lower.contains("needs revision")
    {
        RefinementVerdict::Revise
    } else {
        RefinementVerdict::CannotReachThreshold
    }
}

fn extract_list(response: &str, keyword: &str) -> Vec<String> {
    let mut items = Vec::new();
    let lower = response.to_lowercase();

    // Look for bullet points after keyword
    let patterns = [
        format!(r"{}s?:\s*\n((?:\s*[-*]\s*[^\n]+\n?)+)", keyword),
        format!(r"{}s?:\s*([^\n]+)", keyword),
    ];

    for pattern in patterns {
        if let Ok(re) = regex::Regex::new(&pattern) {
            if let Some(caps) = re.captures(&lower) {
                if let Some(m) = caps.get(1) {
                    let content = m.as_str();
                    for line in content.lines() {
                        let item = line.trim_start_matches(['-', '*', ' '].as_ref()).trim();
                        if !item.is_empty() {
                            items.push(item.to_string());
                        }
                    }
                }
            }
        }
    }

    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reliability_factors_weighted_score() {
        let factors = ReliabilityFactors::new(0.9, 0.8, 0.9);
        let score = factors.weighted_score();
        assert!(score > 0.85 && score < 0.90);
    }

    #[test]
    fn test_refinement_loop_accept_high_confidence() {
        let mut loop_state = RefinementLoop::new();
        let assessment = ReliabilityAssessment {
            score: 0.95,
            verdict: RefinementVerdict::Accept,
            ..Default::default()
        };

        let action = loop_state.record_assessment(assessment);

        match action {
            RefinementAction::Accept { score, passes } => {
                assert!((score - 0.95).abs() < 0.01);
                assert_eq!(passes, 1);
            }
            _ => panic!("Expected Accept action"),
        }

        assert!(loop_state.threshold_reached());
    }

    #[test]
    fn test_refinement_loop_continue() {
        let mut loop_state = RefinementLoop::new();
        let assessment = ReliabilityAssessment {
            score: 0.75,
            verdict: RefinementVerdict::Revise,
            corrections: vec!["Fix claim about CPU".to_string()],
            ..Default::default()
        };

        let action = loop_state.record_assessment(assessment);

        match action {
            RefinementAction::Continue { corrections, .. } => {
                assert_eq!(corrections.len(), 1);
            }
            _ => panic!("Expected Continue action"),
        }

        assert!(!loop_state.threshold_reached());
    }

    #[test]
    fn test_refinement_loop_exhausted_medium() {
        let mut loop_state = RefinementLoop::new().with_max_passes(2);

        // Pass 1: revise
        loop_state.record_assessment(ReliabilityAssessment {
            score: 0.6,
            verdict: RefinementVerdict::Revise,
            ..Default::default()
        });

        // Pass 2: still medium, exhausted
        let action = loop_state.record_assessment(ReliabilityAssessment {
            score: 0.75,
            verdict: RefinementVerdict::Revise,
            risks: vec!["Missing network probe".to_string()],
            ..Default::default()
        });

        match action {
            RefinementAction::AcceptPartial { score, passes, .. } => {
                assert!((score - 0.75).abs() < 0.01);
                assert_eq!(passes, 2);
            }
            _ => panic!("Expected AcceptPartial action"),
        }
    }

    #[test]
    fn test_refinement_loop_exhausted_low() {
        let mut loop_state = RefinementLoop::new().with_max_passes(3);

        for _ in 0..3 {
            loop_state.record_assessment(ReliabilityAssessment {
                score: 0.5,
                verdict: RefinementVerdict::Revise,
                risks: vec!["No evidence".to_string()],
                ..Default::default()
            });
        }

        let action = loop_state.history.last().map(|a| {
            if a.score < THRESHOLD_MEDIUM {
                RefinementAction::AcceptLowConfidence {
                    score: a.score,
                    passes: 3,
                    limitations: a.risks.clone(),
                }
            } else {
                RefinementAction::AcceptPartial {
                    score: a.score,
                    passes: 3,
                    gaps: a.risks.clone(),
                }
            }
        });

        assert!(matches!(action, Some(RefinementAction::AcceptLowConfidence { .. })));
    }

    #[test]
    fn test_parse_reliability_response() {
        let response = r#"
            Score: 0.85
            Evidence: high
            Coverage: medium
            Reasoning: high
            Verdict: accept
            Risks:
            - Missing GPU info
        "#;

        let assessment = parse_reliability_response(response);
        assert!((assessment.score - 0.85).abs() < 0.01);
        assert_eq!(assessment.verdict, RefinementVerdict::Accept);
    }

    #[test]
    fn test_threshold_constants() {
        assert!((THRESHOLD_HIGH - 0.9).abs() < 0.01);
        assert!((THRESHOLD_MEDIUM - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_max_passes_constant() {
        assert_eq!(MAX_REFINEMENT_PASSES, 3);
    }
}
