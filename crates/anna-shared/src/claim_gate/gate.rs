//! ClaimGate implementation - enforces evidence requirements.

use std::collections::HashMap;

use super::config::ClaimGateConfig;
use super::types::{Claim, ClaimCategory, EvidenceType, GateResult, TrustedDocSource};

/// The ClaimGate - enforces evidence requirements
#[derive(Debug, Clone, Default)]
pub struct ClaimGate {
    /// Pending claims awaiting verification
    pending_claims: Vec<Claim>,
    /// Verified claims
    verified_claims: Vec<Claim>,
    /// Evidence cache (command -> output)
    evidence_cache: HashMap<String, EvidenceType>,
    /// Configuration
    config: ClaimGateConfig,
}

impl ClaimGate {
    /// Create a new ClaimGate
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom config
    pub fn with_config(config: ClaimGateConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Submit a claim for verification
    pub fn submit_claim(&mut self, statement: &str, category: ClaimCategory) -> Claim {
        Claim {
            statement: statement.to_string(),
            category,
            confidence: 0.0,
            evidence: Vec::new(),
            verified: false,
        }
    }

    /// Add evidence to a claim
    pub fn add_evidence(&mut self, claim: &mut Claim, evidence: EvidenceType) {
        claim.evidence.push(evidence.clone());
        claim.confidence = self.calculate_confidence(claim);
    }

    /// Calculate confidence based on evidence
    pub(crate) fn calculate_confidence(&self, claim: &Claim) -> f32 {
        if claim.evidence.is_empty() {
            return 0.0;
        }

        let mut total_weight = 0.0;
        let mut weighted_confidence = 0.0;

        for evidence in &claim.evidence {
            let (weight, confidence) = match evidence {
                EvidenceType::ProbeResult { exit_code, .. } => {
                    let base = if *exit_code == 0 { 1.0 } else { 0.7 };
                    (1.0, base)
                }
                EvidenceType::TrustedDoc { source, .. } => {
                    (0.9, source.reliability_weight())
                }
                EvidenceType::ValidatedSkill { .. } => {
                    (0.85, 0.85)
                }
                EvidenceType::UserProvided { .. } => {
                    if self.config.allow_user_evidence {
                        (0.3, 0.5)
                    } else {
                        (0.0, 0.0)
                    }
                }
            };
            total_weight += weight;
            weighted_confidence += weight * confidence;
        }

        if total_weight == 0.0 {
            0.0
        } else {
            weighted_confidence / total_weight
        }
    }

    /// Pass a claim through the gate
    pub fn verify(&self, claim: &Claim) -> GateResult {
        let min_confidence = claim.category.min_confidence().max(self.config.min_confidence);

        // Check evidence count
        let valid_evidence_count = claim.evidence.iter()
            .filter(|e| {
                !matches!(e, EvidenceType::UserProvided { .. }) || self.config.allow_user_evidence
            })
            .count();

        if valid_evidence_count < self.config.min_evidence_count {
            let suggested_probes = claim.category.verification_probes()
                .iter()
                .map(|s| s.to_string())
                .collect();

            return GateResult::NeedsInvestigation {
                claim: claim.clone(),
                missing_evidence: vec![format!(
                    "Need at least {} evidence items, have {}",
                    self.config.min_evidence_count,
                    valid_evidence_count
                )],
                suggested_probes,
            };
        }

        // Check confidence
        if claim.confidence < min_confidence {
            let suggested_probes = claim.category.verification_probes()
                .iter()
                .map(|s| s.to_string())
                .collect();

            return GateResult::NeedsInvestigation {
                claim: claim.clone(),
                missing_evidence: vec![format!(
                    "Confidence {:.0}% below threshold {:.0}%",
                    claim.confidence * 100.0,
                    min_confidence * 100.0
                )],
                suggested_probes,
            };
        }

        // Claim passes the gate
        let evidence_summary = self.summarize_evidence(&claim.evidence);
        let mut verified_claim = claim.clone();
        verified_claim.verified = true;

        GateResult::Verified {
            claim: verified_claim,
            evidence_summary,
        }
    }

    /// Summarize evidence for display
    fn summarize_evidence(&self, evidence: &[EvidenceType]) -> String {
        let mut parts = Vec::new();

        for ev in evidence {
            match ev {
                EvidenceType::ProbeResult { command, .. } => {
                    parts.push(format!("probe: {}", command));
                }
                EvidenceType::TrustedDoc { source, article, section, .. } => {
                    let sec = section.as_ref().map(|s| format!(" - {}", s)).unwrap_or_default();
                    parts.push(format!("{:?}: {}{}", source, article, sec));
                }
                EvidenceType::ValidatedSkill { skill_id, .. } => {
                    parts.push(format!("skill: {}", skill_id));
                }
                EvidenceType::UserProvided { .. } => {
                    parts.push("user provided".to_string());
                }
            }
        }

        if parts.is_empty() {
            "no evidence".to_string()
        } else {
            parts.join(", ")
        }
    }

    /// Cache evidence for reuse
    pub fn cache_evidence(&mut self, key: &str, evidence: EvidenceType) {
        self.evidence_cache.insert(key.to_string(), evidence);
    }

    /// Get cached evidence
    pub fn get_cached_evidence(&self, key: &str) -> Option<&EvidenceType> {
        self.evidence_cache.get(key)
    }

    /// Create an "unverified" disclaimer for claims that can't be verified
    pub fn create_unverified_statement(claim: &str) -> String {
        format!("[Cannot verify with current evidence] {}", claim)
    }

    /// Create evidence from command output
    /// v0.3.28: Detects empty output on success (Phase 3 F15)
    pub fn evidence_from_probe(command: &str, output: &str, exit_code: i32) -> EvidenceType {
        // v0.3.28: Phase 3 F15 - detect empty output on successful probe
        let output_empty = exit_code == 0 && output.trim().is_empty();
        EvidenceType::ProbeResult {
            command: command.to_string(),
            output: output.to_string(),
            exit_code,
            timestamp: chrono::Utc::now().to_rfc3339(),
            output_empty,
        }
    }

    /// Create evidence from wiki search
    pub fn evidence_from_wiki(article: &str, section: Option<&str>, quote: &str) -> EvidenceType {
        EvidenceType::TrustedDoc {
            source: TrustedDocSource::ArchWiki,
            article: article.to_string(),
            section: section.map(|s| s.to_string()),
            quote: quote.to_string(),
        }
    }

    /// v0.3.26: Create evidence from man page
    pub fn evidence_from_man(command: &str, section: u8, excerpt: &str) -> EvidenceType {
        EvidenceType::TrustedDoc {
            source: TrustedDocSource::ManPage,
            article: format!("{}({})", command, section),
            section: None,
            quote: excerpt.to_string(),
        }
    }

    /// v0.3.26: Create evidence from --help output
    pub fn evidence_from_help(command: &str, excerpt: &str) -> EvidenceType {
        EvidenceType::TrustedDoc {
            source: TrustedDocSource::HelpOutput,
            article: command.to_string(),
            section: None,
            quote: excerpt.to_string(),
        }
    }

    /// v0.3.26: Create evidence from DocCitation
    pub fn evidence_from_doc_citation(citation: &crate::docs::DocCitation) -> EvidenceType {
        let source = match &citation.source {
            crate::docs::DocSource::ArchWiki => TrustedDocSource::ArchWiki,
            crate::docs::DocSource::ManPage { .. } => TrustedDocSource::ManPage,
            crate::docs::DocSource::HelpOutput { .. } => TrustedDocSource::HelpOutput,
        };

        EvidenceType::TrustedDoc {
            source,
            article: citation.title.clone(),
            section: citation.section.clone(),
            quote: citation.excerpt.clone(),
        }
    }

    /// v0.3.26: Check if a claim requires documentation (not just probes)
    pub fn claim_requires_docs(claim_text: &str) -> bool {
        let doc_patterns = [
            r"(?i)how\s+(does|do|to|can)\b",
            r"(?i)what\s+(does|is|are)\b",
            r"(?i)explain\b",
            r"(?i)meaning\s+of\b",
            r"(?i)purpose\s+of\b",
            r"(?i)default\s+(value|setting|behavior)\b",
            r"(?i)(configure|configuration)\b",
            r"(?i)(option|flag|parameter)\b",
            r"(?i)syntax\b",
            r"(?i)behavior\b",
        ];

        for pattern in &doc_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(claim_text) {
                    return true;
                }
            }
        }
        false
    }

    /// v0.3.26: Check if evidence includes any documentation
    pub fn has_doc_evidence(evidence: &[EvidenceType]) -> bool {
        evidence.iter().any(|e| matches!(e, EvidenceType::TrustedDoc { .. }))
    }

    /// Extract claims from LLM output text
    pub fn extract_claims(text: &str) -> Vec<(String, ClaimCategory)> {
        let mut claims = Vec::new();

        // Patterns that indicate factual claims
        let patterns = [
            // Service state claims (formal)
            (r"(?i)(service|daemon|systemd unit)\s+\w+\s+(is|are)\s+(running|stopped|active|inactive|enabled|disabled)", ClaimCategory::ServiceState),
            // Service state claims (direct: "nginx is running", "sshd is active")
            (r"(?i)\b(\w+)\s+(is|are)\s+(running|stopped|active|inactive|enabled|disabled)\b", ClaimCategory::ServiceState),
            // File system claims
            (r"(?i)(file|directory|path)\s+[\w/.-]+\s+(exists|is present|is missing|contains)", ClaimCategory::FileSystem),
            // Package claims
            (r"(?i)(package|program|command)\s+\w+\s+(is|are)\s+(installed|not installed|available)", ClaimCategory::PackageStatus),
            // Network claims (formal)
            (r"(?i)(port|interface|connection|network)\s+\w+\s+(is|are)\s+(open|closed|up|down|listening)", ClaimCategory::NetworkState),
            // Network claims (direct: "port 80 is open")
            (r"(?i)port\s+\d+\s+(is|are)\s+(open|closed|listening)", ClaimCategory::NetworkState),
            // Resource claims
            (r"(?i)(disk|memory|cpu|ram)\s+(usage|space|available)\s+(is|are)\s+[\d.]+", ClaimCategory::HardwareStatus),
        ];

        for (pattern, category) in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                for cap in re.find_iter(text) {
                    claims.push((cap.as_str().to_string(), *category));
                }
            }
        }

        claims
    }

    /// v0.3.28: Detect conflicting probes (Phase 3 F4)
    /// Returns (conflicts_detected, conflict_descriptions)
    pub(crate) fn detect_probe_conflicts(evidence: &[EvidenceType]) -> (bool, Vec<String>) {
        let mut conflicts = Vec::new();

        // Extract probe results
        let probes: Vec<_> = evidence.iter()
            .filter_map(|e| {
                if let EvidenceType::ProbeResult { command, output, exit_code, .. } = e {
                    Some((command.as_str(), output.as_str(), *exit_code))
                } else {
                    None
                }
            })
            .collect();

        // Look for conflicting probes about the same subject
        let subjects: Vec<&str> = vec!["nginx", "systemd", "service", "package", "port"];

        for subject in subjects {
            let related: Vec<_> = probes.iter()
                .filter(|(cmd, _, _)| cmd.to_lowercase().contains(subject))
                .collect();

            if related.len() >= 2 {
                let successes: Vec<_> = related.iter().filter(|(_, _, exit)| *exit == 0).collect();
                let failures: Vec<_> = related.iter().filter(|(_, _, exit)| *exit != 0).collect();

                if !successes.is_empty() && !failures.is_empty() {
                    let success_cmds: Vec<_> = successes.iter().map(|(cmd, _, _)| *cmd).collect();
                    let failure_cmds: Vec<_> = failures.iter().map(|(cmd, _, _)| *cmd).collect();
                    conflicts.push(format!(
                        "Conflict for '{}': {} succeeded but {} failed",
                        subject,
                        success_cmds.join(", "),
                        failure_cmds.join(", ")
                    ));
                }
            }
        }

        (!conflicts.is_empty(), conflicts)
    }
}
