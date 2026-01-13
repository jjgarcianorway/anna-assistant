//! ClaimVerifier trait and VerifiedResponse struct.

use serde::{Deserialize, Serialize};

use super::gate::ClaimGate;
use super::types::{Claim, EvidenceType, GateResult, TrustedDocSource};

/// Interface for claims verification in the decision loop
pub trait ClaimVerifier {
    /// Verify all claims in a response before showing to user
    fn verify_response(&self, response: &str, evidence: &[EvidenceType]) -> VerifiedResponse;

    /// v0.3.26: Verify response with question context for doc requirements
    fn verify_response_with_context(
        &self,
        response: &str,
        question: &str,
        evidence: &[EvidenceType],
    ) -> VerifiedResponse;
}

/// A response that has been through claim verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedResponse {
    /// The original response
    pub original: String,
    /// Modified response with unverified claims BLOCKED (replaced with uncertainty statements)
    pub verified_text: String,
    /// Claims that were verified
    pub verified_claims: Vec<Claim>,
    /// Claims that could not be verified (these are BLOCKED, not emitted)
    pub unverified_claims: Vec<Claim>,
    /// Overall confidence in the response
    pub confidence: f32,
    /// Whether the response should switch to investigator mode
    pub needs_investigation: bool,
    /// Suggested probes if investigation needed
    pub suggested_probes: Vec<String>,
    /// v0.3.26: Whether docs are required for this response
    #[serde(default)]
    pub docs_required: bool,
    /// v0.3.26: Whether docs were found
    #[serde(default)]
    pub docs_found: bool,
    /// v0.3.26: Doc citations used
    #[serde(default)]
    pub doc_citations: Vec<String>,
    /// v0.3.27: Whether any claims were blocked due to lack of evidence
    #[serde(default)]
    pub claims_blocked: bool,
    /// v0.3.27: Reasons why claims were blocked
    #[serde(default)]
    pub block_reasons: Vec<String>,
    /// v0.3.27: Whether any probes failed (exit code != 0)
    #[serde(default)]
    pub probes_failed: bool,
    /// v0.3.27: List of failed probe commands
    #[serde(default)]
    pub failed_probes: Vec<String>,
    /// v0.3.28: Whether probes conflict on the same state (Phase 3 F4)
    #[serde(default)]
    pub conflicts_detected: bool,
    /// v0.3.28: Description of detected conflicts
    #[serde(default)]
    pub conflict_descriptions: Vec<String>,
}

impl ClaimVerifier for ClaimGate {
    fn verify_response(&self, response: &str, evidence: &[EvidenceType]) -> VerifiedResponse {
        let extracted = ClaimGate::extract_claims(response);
        let mut verified_claims = Vec::new();
        let mut unverified_claims = Vec::new();
        let mut suggested_probes = Vec::new();
        let mut modified_text = response.to_string();
        let mut block_reasons = Vec::new();
        let mut claims_blocked = false;

        // v0.3.27: Detect failed probes (exit code != 0)
        let failed_probes: Vec<String> = evidence.iter()
            .filter_map(|e| {
                if let EvidenceType::ProbeResult { command, exit_code, .. } = e {
                    if *exit_code != 0 {
                        Some(command.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        let probes_failed = !failed_probes.is_empty();

        for (statement, category) in extracted {
            let mut claim = Claim {
                statement: statement.clone(),
                category,
                confidence: 0.0,
                evidence: evidence.to_vec(),
                verified: false,
            };

            // Calculate confidence based on evidence
            claim.confidence = self.calculate_confidence(&claim);

            match self.verify(&claim) {
                GateResult::Verified { claim: verified, .. } => {
                    verified_claims.push(verified);
                }
                GateResult::NeedsInvestigation { claim: unverified, suggested_probes: probes, missing_evidence } => {
                    // v0.3.27: BLOCK unverified claims - replace with uncertainty statement
                    let reason = missing_evidence.first()
                        .map(|s| s.as_str())
                        .unwrap_or("no evidence");
                    let blocked_statement = format!(
                        "[I cannot verify: {} ({})]",
                        statement, reason
                    );
                    modified_text = modified_text.replace(&statement, &blocked_statement);
                    unverified_claims.push(unverified);
                    suggested_probes.extend(probes);
                    block_reasons.push(format!("Blocked '{}': {}", statement, reason));
                    claims_blocked = true;
                }
                GateResult::CannotVerify { claim: unverified, reason, .. } => {
                    // v0.3.27: Replace with explicit uncertainty
                    let blocked_statement = format!(
                        "[I cannot verify: {} ({})]",
                        statement, reason
                    );
                    modified_text = modified_text.replace(&statement, &blocked_statement);
                    unverified_claims.push(unverified);
                    block_reasons.push(format!("Blocked '{}': {}", statement, reason));
                    claims_blocked = true;
                }
            }
        }

        let total_claims = verified_claims.len() + unverified_claims.len();
        let confidence = if total_claims == 0 {
            1.0 // No factual claims = safe
        } else {
            verified_claims.len() as f32 / total_claims as f32
        };

        let needs_investigation = !unverified_claims.is_empty() && confidence < 0.7;

        // v0.3.26: Extract doc citations from evidence
        let doc_citations: Vec<String> = evidence.iter()
            .filter_map(|e| {
                if let EvidenceType::TrustedDoc { source, article, section, .. } = e {
                    let source_name = match source {
                        TrustedDocSource::ArchWiki => "Arch Wiki",
                        TrustedDocSource::ManPage => "man",
                        TrustedDocSource::HelpOutput => "--help",
                        TrustedDocSource::ArchDocs => "Arch Docs",
                    };
                    if let Some(s) = section {
                        Some(format!("[{}: {} - {}]", source_name, article, s))
                    } else {
                        Some(format!("[{}: {}]", source_name, article))
                    }
                } else {
                    None
                }
            })
            .collect();

        // v0.3.28: Detect conflicting probes (Phase 3 F4)
        let (conflicts_detected, conflict_descriptions) = ClaimGate::detect_probe_conflicts(evidence);

        VerifiedResponse {
            original: response.to_string(),
            verified_text: modified_text,
            verified_claims,
            unverified_claims,
            confidence,
            needs_investigation,
            suggested_probes,
            docs_required: false,
            docs_found: !doc_citations.is_empty(),
            doc_citations,
            claims_blocked,
            block_reasons,
            probes_failed,
            failed_probes,
            conflicts_detected,
            conflict_descriptions,
        }
    }

    /// v0.3.26: Verify response with question context for doc requirements
    fn verify_response_with_context(
        &self,
        response: &str,
        question: &str,
        evidence: &[EvidenceType],
    ) -> VerifiedResponse {
        let mut result = self.verify_response(response, evidence);

        // Check if docs are required based on question type
        result.docs_required = ClaimGate::claim_requires_docs(question);

        // If docs are required but not found, mark as needing investigation
        if result.docs_required && !result.docs_found {
            result.needs_investigation = true;
            // Add suggestion to search docs
            result.suggested_probes.push("Search Arch Wiki for relevant article".to_string());
            result.suggested_probes.push("Check man pages for command documentation".to_string());
        }

        result
    }
}
