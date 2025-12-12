//! Strict Claim/Evidence Model (v0.0.445).
//!
//! Every claim must have matching evidence. No exceptions.
//!
//! Mapping rules:
//! - Metric claim → numeric evidence
//! - Boolean claim → explicit yes/no evidence
//! - List claim → list evidence
//! - Diagnosis claim → ≥2 independent evidence sources

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Claim types with strict evidence requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimType {
    /// Single numeric value (e.g., "17 GiB free RAM")
    Metric,
    /// Yes/No answer (e.g., "swap is enabled")
    Boolean,
    /// List of items (e.g., "installed packages: vim, nano")
    List,
    /// Explanation with cause (e.g., "slow because X")
    Diagnosis,
    /// File path or location
    Path,
    /// Status/state (e.g., "nginx is running")
    Status,
}

impl ClaimType {
    /// Get required evidence type for this claim.
    pub fn required_evidence(&self) -> EvidenceType {
        match self {
            Self::Metric => EvidenceType::Numeric,
            Self::Boolean => EvidenceType::Boolean,
            Self::List => EvidenceType::List,
            Self::Diagnosis => EvidenceType::MultiSource,
            Self::Path => EvidenceType::Path,
            Self::Status => EvidenceType::Status,
        }
    }

    /// Minimum evidence items required.
    pub fn min_evidence_count(&self) -> usize {
        match self {
            Self::Diagnosis => 2, // Must have ≥2 independent sources
            _ => 1,
        }
    }
}

/// Evidence types that can support claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    /// Numeric value from probe
    Numeric,
    /// Boolean (yes/no) from probe
    Boolean,
    /// List of items from probe
    List,
    /// Multiple independent sources
    MultiSource,
    /// File path existence check
    Path,
    /// Service/process status
    Status,
}

/// A strict claim that requires evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrictClaim {
    /// Unique claim ID (e.g., "C1", "C2")
    pub id: String,
    /// Human-readable claim text
    pub text: String,
    /// Claim type (determines evidence requirements)
    pub claim_type: ClaimType,
    /// Domain (e.g., "system", "storage", "network")
    pub domain: String,
    /// Bound evidence IDs (must be non-empty after gate)
    pub evidence_ids: Vec<String>,
}

impl StrictClaim {
    /// Create a new claim.
    pub fn new(id: &str, text: &str, claim_type: ClaimType, domain: &str) -> Self {
        Self {
            id: id.to_string(),
            text: text.to_string(),
            claim_type,
            domain: domain.to_string(),
            evidence_ids: Vec::new(),
        }
    }

    /// Bind evidence to this claim.
    pub fn bind_evidence(&mut self, evidence_id: &str) {
        if !self.evidence_ids.contains(&evidence_id.to_string()) {
            self.evidence_ids.push(evidence_id.to_string());
        }
    }

    /// Check if claim has sufficient evidence.
    pub fn has_sufficient_evidence(&self) -> bool {
        self.evidence_ids.len() >= self.claim_type.min_evidence_count()
    }

    /// Check if claim is properly bound.
    pub fn is_valid(&self) -> bool {
        !self.text.is_empty() && self.has_sufficient_evidence()
    }
}

/// Evidence that supports a claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrictEvidence {
    /// Unique evidence ID (e.g., "E1", "E2")
    pub id: String,
    /// Source (e.g., "probe:memory_info", "probe:df")
    pub source: String,
    /// Command that produced this evidence
    pub command: String,
    /// Extracted value (the actual data)
    pub extracted_value: String,
    /// Evidence type
    pub evidence_type: EvidenceType,
    /// Timestamp (must be from current request)
    pub timestamp: u64,
    /// Request ID (for freshness validation)
    pub request_id: String,
}

impl StrictEvidence {
    /// Create new evidence.
    pub fn new(
        id: &str,
        source: &str,
        command: &str,
        value: &str,
        evidence_type: EvidenceType,
        request_id: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            source: source.to_string(),
            command: command.to_string(),
            extracted_value: value.to_string(),
            evidence_type,
            timestamp: current_millis(),
            request_id: request_id.to_string(),
        }
    }

    /// Check if evidence is fresh (from this request).
    pub fn is_fresh(&self, current_request_id: &str) -> bool {
        self.request_id == current_request_id
    }

    /// Check if evidence type matches claim type.
    pub fn matches_claim_type(&self, claim_type: ClaimType) -> bool {
        let required = claim_type.required_evidence();
        self.evidence_type == required || required == EvidenceType::MultiSource
    }
}

/// Binding between claims and evidence.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvidenceBinding {
    /// All claims in the answer
    pub claims: Vec<StrictClaim>,
    /// All available evidence
    pub evidence: Vec<StrictEvidence>,
    /// Current request ID
    pub request_id: String,
}

impl EvidenceBinding {
    /// Create new binding for a request.
    pub fn new(request_id: &str) -> Self {
        Self {
            claims: Vec::new(),
            evidence: Vec::new(),
            request_id: request_id.to_string(),
        }
    }

    /// Add a claim.
    pub fn add_claim(&mut self, claim: StrictClaim) {
        self.claims.push(claim);
    }

    /// Add evidence.
    pub fn add_evidence(&mut self, evidence: StrictEvidence) {
        self.evidence.push(evidence);
    }

    /// Get evidence by ID.
    pub fn get_evidence(&self, id: &str) -> Option<&StrictEvidence> {
        self.evidence.iter().find(|e| e.id == id)
    }

    /// Bind evidence to claim by IDs.
    pub fn bind(&mut self, claim_id: &str, evidence_id: &str) -> bool {
        // Verify evidence exists and is fresh
        let evidence = match self.evidence.iter().find(|e| e.id == evidence_id) {
            Some(e) if e.is_fresh(&self.request_id) => e,
            _ => return false,
        };

        // Find claim and verify type match
        if let Some(claim) = self.claims.iter_mut().find(|c| c.id == claim_id) {
            if evidence.matches_claim_type(claim.claim_type) {
                claim.bind_evidence(evidence_id);
                return true;
            }
        }
        false
    }

    /// Check if all claims have sufficient evidence.
    pub fn all_claims_bound(&self) -> bool {
        !self.claims.is_empty() && self.claims.iter().all(|c| c.has_sufficient_evidence())
    }

    /// Get unbound claims.
    pub fn unbound_claims(&self) -> Vec<&StrictClaim> {
        self.claims
            .iter()
            .filter(|c| !c.has_sufficient_evidence())
            .collect()
    }

    /// Get claims without any evidence.
    pub fn claims_without_evidence(&self) -> Vec<&StrictClaim> {
        self.claims
            .iter()
            .filter(|c| c.evidence_ids.is_empty())
            .collect()
    }

    /// Calculate evidence coverage (0.0 to 1.0).
    pub fn coverage(&self) -> f32 {
        if self.claims.is_empty() {
            return 0.0;
        }
        let bound = self
            .claims
            .iter()
            .filter(|c| c.has_sufficient_evidence())
            .count();
        bound as f32 / self.claims.len() as f32
    }
}

/// Get current time in milliseconds.
fn current_millis() -> u64 {
    SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claim_evidence_binding() {
        let mut binding = EvidenceBinding::new("REQ-001");

        let claim = StrictClaim::new("C1", "Free memory is 17 GiB", ClaimType::Metric, "system");
        binding.add_claim(claim);

        let evidence = StrictEvidence::new(
            "E1",
            "probe:memory_info",
            "cat /proc/meminfo",
            "MemAvailable: 17848320 kB",
            EvidenceType::Numeric,
            "REQ-001",
        );
        binding.add_evidence(evidence);

        assert!(!binding.all_claims_bound());
        assert!(binding.bind("C1", "E1"));
        assert!(binding.all_claims_bound());
    }

    #[test]
    fn test_stale_evidence_rejected() {
        let mut binding = EvidenceBinding::new("REQ-002");

        let claim = StrictClaim::new("C1", "Swap is enabled", ClaimType::Boolean, "system");
        binding.add_claim(claim);

        // Evidence from different request (stale)
        let evidence = StrictEvidence::new(
            "E1",
            "probe:swap",
            "swapon --show",
            "yes",
            EvidenceType::Boolean,
            "REQ-001", // Wrong request ID
        );
        binding.add_evidence(evidence);

        assert!(!binding.bind("C1", "E1")); // Should fail - stale
    }

    #[test]
    fn test_type_mismatch_rejected() {
        let mut binding = EvidenceBinding::new("REQ-001");

        let claim = StrictClaim::new("C1", "Free memory is 17 GiB", ClaimType::Metric, "system");
        binding.add_claim(claim);

        // Wrong evidence type (boolean instead of numeric)
        let evidence = StrictEvidence::new(
            "E1",
            "probe:swap",
            "swapon --show",
            "yes",
            EvidenceType::Boolean, // Wrong type for metric claim
            "REQ-001",
        );
        binding.add_evidence(evidence);

        assert!(!binding.bind("C1", "E1")); // Should fail - type mismatch
    }

    #[test]
    fn test_diagnosis_requires_multiple_sources() {
        let mut binding = EvidenceBinding::new("REQ-001");

        let claim = StrictClaim::new(
            "C1",
            "System is slow due to high memory usage",
            ClaimType::Diagnosis,
            "system",
        );
        binding.add_claim(claim);

        let e1 = StrictEvidence::new(
            "E1",
            "probe:memory",
            "free -h",
            "Mem: 16G, Used: 15G",
            EvidenceType::MultiSource,
            "REQ-001",
        );
        let e2 = StrictEvidence::new(
            "E2",
            "probe:top",
            "top -bn1",
            "load average: 5.2",
            EvidenceType::MultiSource,
            "REQ-001",
        );
        binding.add_evidence(e1);
        binding.add_evidence(e2);

        binding.bind("C1", "E1");
        assert!(!binding.all_claims_bound()); // Need 2 sources

        binding.bind("C1", "E2");
        assert!(binding.all_claims_bound()); // Now satisfied
    }
}
