//! Junior Rubric v0.0.65 - Evidence-Based Answer Verification
//!
//! Junior's job is to verify answers against evidence and penalize:
//! - Wrong evidence (CPU info for disk question) -> max 20%
//! - Missing evidence (no disk fields at all) -> max 50%
//! - Answer doesn't match question -> max 60%
//! - Irrelevant tools (hw_snapshot for specific query) -> penalty
//! - Uncited claims -> penalty
//! - Low coverage -> penalty
//!
//! v0.0.65: Added relevance checks with tool_satisfies_topic
//! This rubric is deterministic - no LLM needed for basic verification.

use serde::{Deserialize, Serialize};

use crate::evidence_coverage::{
    analyze_coverage, check_evidence_mismatch, get_max_score_for_coverage,
    EvidenceCoverage, COVERAGE_SUFFICIENT_THRESHOLD,
};
use crate::evidence_router::tool_satisfies_topic;
use crate::evidence_topic::EvidenceTopic;
use crate::system_query_router::{validate_answer_for_target, QueryTarget};
use crate::tools::ToolResult;

// ============================================================================
// Verification Result
// ============================================================================

/// Result of Junior's verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Final reliability score (0-100)
    pub reliability_score: u8,
    /// Base score before penalties
    pub base_score: u8,
    /// Evidence coverage analysis
    pub coverage: EvidenceCoverage,
    /// Whether evidence mismatches the target
    pub has_mismatch: bool,
    /// Mismatch description if any
    pub mismatch_reason: Option<String>,
    /// Whether the answer addresses the question
    pub answer_matches_question: bool,
    /// Answer critique if it doesn't match
    pub answer_critique: Option<String>,
    /// Penalties applied
    pub penalties: Vec<Penalty>,
    /// Whether to ship this answer
    pub ship_it: bool,
    /// Junior's verdict message
    pub verdict: String,
}

/// A penalty applied during verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Penalty {
    pub reason: String,
    pub points: i32,
}

// ============================================================================
// Score Thresholds
// ============================================================================

/// Score threshold for "ship it"
pub const SHIP_IT_THRESHOLD: u8 = 75;

/// Base score for a correctly structured answer
pub const BASE_SCORE: u8 = 85;

/// Penalty for wrong evidence type
pub const WRONG_EVIDENCE_PENALTY: i32 = -65; // Caps at 20%

/// Penalty for missing evidence fields
pub const MISSING_EVIDENCE_PENALTY: i32 = -35; // Caps at 50%

/// Penalty for answer not matching question
pub const ANSWER_MISMATCH_PENALTY: i32 = -25; // Caps at 60%

/// Penalty per uncited claim
pub const UNCITED_CLAIM_PENALTY: i32 = -10;

/// Bonus for high evidence coverage
pub const HIGH_COVERAGE_BONUS: i32 = 10;

// ============================================================================
// Verification Functions
// ============================================================================

/// Verify an answer against evidence and target
pub fn verify_answer(
    target: QueryTarget,
    answer: &str,
    evidence: &[ToolResult],
) -> VerificationResult {
    let mut penalties = Vec::new();
    let mut base_score = BASE_SCORE;

    // 1. Analyze evidence coverage
    let coverage = analyze_coverage(target, evidence);

    // 2. Check for evidence mismatch (wrong evidence entirely)
    let mismatch_reason = check_evidence_mismatch(target, evidence);
    let has_mismatch = mismatch_reason.is_some();

    if has_mismatch {
        penalties.push(Penalty {
            reason: mismatch_reason.clone().unwrap(),
            points: WRONG_EVIDENCE_PENALTY,
        });
    }

    // 3. Check coverage level
    if !coverage.is_sufficient && !has_mismatch {
        let missing_str = coverage.missing_fields.join(", ");
        penalties.push(Penalty {
            reason: format!("Evidence missing required fields: {}", missing_str),
            points: MISSING_EVIDENCE_PENALTY,
        });
    }

    // 4. Validate answer matches the target
    let (answer_matches, answer_critique_str) = validate_answer_for_target(target, answer);
    let answer_critique = if answer_matches {
        None
    } else {
        Some(answer_critique_str.clone())
    };

    if !answer_matches {
        penalties.push(Penalty {
            reason: answer_critique_str,
            points: ANSWER_MISMATCH_PENALTY,
        });
    }

    // 5. Check for uncited claims (answers without [E#] citations)
    let uncited_claims = count_uncited_claims(answer, evidence);
    if uncited_claims > 0 {
        penalties.push(Penalty {
            reason: format!("{} claim(s) without evidence citation", uncited_claims),
            points: UNCITED_CLAIM_PENALTY * uncited_claims,
        });
    }

    // 6. Bonus for high coverage
    if coverage.coverage_percent >= 95 && !has_mismatch {
        base_score = base_score.saturating_add(HIGH_COVERAGE_BONUS as u8);
    }

    // Calculate final score
    let total_penalty: i32 = penalties.iter().map(|p| p.points).sum();
    let raw_score = (base_score as i32 + total_penalty).max(0) as u8;

    // Apply coverage cap
    let max_allowed = get_max_score_for_coverage(&coverage, has_mismatch);
    let reliability_score = raw_score.min(max_allowed);

    // Determine if we ship
    let ship_it = reliability_score >= SHIP_IT_THRESHOLD;

    // Generate verdict
    let verdict = generate_verdict(reliability_score, &coverage, has_mismatch, ship_it);

    VerificationResult {
        reliability_score,
        base_score,
        coverage,
        has_mismatch,
        mismatch_reason,
        answer_matches_question: answer_matches,
        answer_critique,
        penalties,
        ship_it,
        verdict,
    }
}

/// Count claims in the answer that aren't backed by evidence citations
fn count_uncited_claims(answer: &str, evidence: &[ToolResult]) -> i32 {
    // If no evidence was collected, we can't cite anything
    if evidence.is_empty() {
        // Check if answer makes specific claims
        let claim_indicators = [
            "is", "has", "are", "running", "using", "have", "total",
            "free", "available", "connected",
        ];
        let has_claims = claim_indicators.iter().any(|&word|
            answer.to_lowercase().contains(word)
        );

        return if has_claims { 1 } else { 0 };
    }

    // Check if answer has any evidence citations
    let has_citations = evidence.iter().any(|e|
        answer.contains(&format!("[{}]", e.evidence_id))
    );

    // If the answer has claims but no citations, that's a problem
    if !has_citations && !answer.trim().is_empty() {
        return 1;
    }

    0
}

/// Generate Junior's verdict message
fn generate_verdict(
    score: u8,
    coverage: &EvidenceCoverage,
    has_mismatch: bool,
    ship_it: bool,
) -> String {
    if has_mismatch {
        format!(
            "Reliability {}%. Wrong evidence type. Evidence doesn't include {}.",
            score,
            coverage.target.as_str()
        )
    } else if coverage.coverage_percent < 50 {
        format!(
            "Reliability {}%. Coverage {}%. Missing: {}. Don't guess.",
            score,
            coverage.coverage_percent,
            coverage.missing_fields.join(", ")
        )
    } else if !ship_it {
        format!(
            "Reliability {}%. Not good enough. Coverage {}%.",
            score,
            coverage.coverage_percent
        )
    } else if score >= 90 {
        format!(
            "Reliability {}%. Solid evidence. Ship it.",
            score
        )
    } else {
        format!(
            "Reliability {}%. Acceptable. Ship it.",
            score
        )
    }
}

// ============================================================================
// Quick Verification for Common Cases
// ============================================================================

/// Quick check if evidence is clearly wrong for target
pub fn is_clearly_wrong_evidence(target: QueryTarget, evidence: &[ToolResult]) -> bool {
    check_evidence_mismatch(target, evidence).is_some()
}

// ============================================================================
// v0.0.65: Topic-Based Relevance Verification
// ============================================================================

/// Penalty for using irrelevant tools
pub const IRRELEVANT_TOOL_PENALTY: i32 = -40; // Caps at 45%

/// Check if tools used are relevant for the topic
/// Returns (has_irrelevant, irrelevant_tools)
pub fn check_tool_relevance(topic: EvidenceTopic, evidence: &[ToolResult]) -> (bool, Vec<String>) {
    let mut irrelevant = Vec::new();

    for result in evidence {
        if result.success && !tool_satisfies_topic(&result.tool_name, topic) {
            irrelevant.push(result.tool_name.clone());
        }
    }

    (!irrelevant.is_empty(), irrelevant)
}

/// v0.0.65: Verify answer with topic-based relevance checking
pub fn verify_answer_with_topic(
    target: QueryTarget,
    topic: EvidenceTopic,
    answer: &str,
    evidence: &[ToolResult],
) -> VerificationResult {
    // Start with standard verification
    let mut result = verify_answer(target, answer, evidence);

    // Additional check: tool relevance for the topic
    let (has_irrelevant, irrelevant_tools) = check_tool_relevance(topic, evidence);

    if has_irrelevant && topic != EvidenceTopic::Unknown {
        // Add penalty for irrelevant tools
        result.penalties.push(Penalty {
            reason: format!(
                "Irrelevant tool(s) for {}: {}",
                topic.human_label(),
                irrelevant_tools.join(", ")
            ),
            points: IRRELEVANT_TOOL_PENALTY,
        });

        // Recalculate score
        let total_penalty: i32 = result.penalties.iter().map(|p| p.points).sum();
        let raw_score = (result.base_score as i32 + total_penalty).max(0) as u8;
        let max_allowed = get_max_score_for_coverage(&result.coverage, result.has_mismatch);
        result.reliability_score = raw_score.min(max_allowed).min(45); // Cap at 45% for irrelevant tools

        // Update verdict
        result.verdict = format!(
            "Reliability {}%. Wrong tools for {}. Need: {}.",
            result.reliability_score,
            topic.human_label(),
            crate::evidence_router::get_tool_for_topic(topic)
        );
        result.ship_it = result.reliability_score >= SHIP_IT_THRESHOLD;
    }

    result
}

/// Get suggestions when evidence is insufficient
pub fn get_evidence_suggestions(coverage: &EvidenceCoverage) -> Vec<String> {
    let mut suggestions = Vec::new();

    if !coverage.is_sufficient {
        suggestions.push(format!(
            "Need evidence for: {}",
            coverage.missing_fields.join(", ")
        ));

        if !coverage.suggested_tools.is_empty() {
            suggestions.push(format!(
                "Suggested tools: {}",
                coverage.suggested_tools.join(", ")
            ));
        }
    }

    suggestions
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_evidence(tool: &str, summary: &str, id: &str) -> ToolResult {
        ToolResult {
            tool_name: tool.to_string(),
            evidence_id: id.to_string(),
            data: serde_json::json!({}),
            human_summary: summary.to_string(),
            success: true,
            error: None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    #[test]
    fn test_correct_disk_answer() {
        let evidence = vec![
            make_evidence("mount_usage", "Disk /: 433 GiB free of 500 GiB total", "E1"),
        ];
        let answer = "You have 433 GiB free on / [E1].";

        let result = verify_answer(QueryTarget::DiskFree, answer, &evidence);
        assert!(result.ship_it);
        assert!(result.reliability_score >= 75);
    }

    #[test]
    fn test_wrong_evidence_for_disk() {
        let evidence = vec![
            make_evidence("hw_snapshot_cpu", "CPU: AMD Ryzen 7 5800X, 8 cores", "E1"),
        ];
        let answer = "Your CPU is AMD Ryzen 7 [E1].";

        let result = verify_answer(QueryTarget::DiskFree, answer, &evidence);
        assert!(!result.ship_it);
        assert!(result.has_mismatch);
        assert!(result.reliability_score <= 20);
    }

    #[test]
    fn test_memory_answer_correct() {
        let evidence = vec![
            make_evidence("memory_info", "Memory: 32 GiB total, 24 GiB available", "E1"),
        ];
        let answer = "You have 32 GiB of RAM [E1], with 24 GiB available.";

        let result = verify_answer(QueryTarget::Memory, answer, &evidence);
        assert!(result.ship_it);
        assert!(result.reliability_score >= 85);
    }

    #[test]
    fn test_kernel_answer_correct() {
        let evidence = vec![
            make_evidence("kernel_version", "Linux 6.7.1-arch1-1", "E1"),
        ];
        let answer = "You are running Linux kernel 6.7.1-arch1-1 [E1].";

        let result = verify_answer(QueryTarget::KernelVersion, answer, &evidence);
        assert!(result.ship_it);
    }

    #[test]
    fn test_answer_without_citation() {
        let evidence = vec![
            make_evidence("memory_info", "Memory: 32 GiB total", "E1"),
        ];
        let answer = "You have 32 GiB of RAM."; // Missing [E1]

        let result = verify_answer(QueryTarget::Memory, answer, &evidence);
        // Should have penalty for uncited claim
        assert!(result.penalties.iter().any(|p| p.reason.contains("citation")));
    }

    #[test]
    fn test_no_evidence_low_score() {
        let evidence: Vec<ToolResult> = vec![];
        let answer = "You have 32 GiB of RAM.";

        let result = verify_answer(QueryTarget::Memory, answer, &evidence);
        assert!(!result.ship_it);
        assert!(result.reliability_score < 50);
    }

    #[test]
    fn test_verdict_messages() {
        // High score
        let coverage = EvidenceCoverage {
            target: QueryTarget::Memory,
            coverage_percent: 100,
            present_fields: vec!["mem_total".to_string()],
            missing_fields: vec![],
            optional_present: vec![],
            tools_used: vec!["memory_info".to_string()],
            is_sufficient: true,
            suggested_tools: vec![],
        };
        let verdict = generate_verdict(92, &coverage, false, true);
        assert!(verdict.contains("Ship it"));

        // Low coverage
        let low_coverage = EvidenceCoverage {
            target: QueryTarget::DiskFree,
            coverage_percent: 30,
            present_fields: vec![],
            missing_fields: vec!["root_fs_free".to_string()],
            optional_present: vec![],
            tools_used: vec![],
            is_sufficient: false,
            suggested_tools: vec!["mount_usage".to_string()],
        };
        let verdict = generate_verdict(45, &low_coverage, false, false);
        assert!(verdict.contains("Coverage 30%"));
    }
}
