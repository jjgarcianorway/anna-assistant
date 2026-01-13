//! ClaimGate - Blocks user-visible factual claims unless backed by evidence.
//!
//! This is a code-level enforcement mechanism, not just a prompt.
//! Claims must be backed by:
//! - Probe results (structured command output)
//! - Trusted doc citations (Arch Wiki, man pages, --help)
//! - Validated skill artifacts with evidence chains
//!
//! If evidence is missing, ClaimGate switches to Investigator mode.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of evidence that can back a claim
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EvidenceType {
    /// Command output from a probe
    ProbeResult {
        command: String,
        output: String,
        exit_code: i32,
        timestamp: String,
    },
    /// Citation from trusted documentation
    TrustedDoc {
        source: TrustedDocSource,
        article: String,
        section: Option<String>,
        quote: String,
    },
    /// Validated skill artifact
    ValidatedSkill {
        skill_id: String,
        validation_timestamp: String,
        evidence_chain: Vec<String>,
    },
    /// User provided information (lower trust)
    UserProvided {
        content: String,
        timestamp: String,
    },
}

/// Sources of trusted documentation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrustedDocSource {
    /// Local Arch Wiki mirror
    ArchWiki,
    /// Man pages
    ManPage,
    /// --help output from commands
    HelpOutput,
    /// Official Arch documentation
    ArchDocs,
}

impl TrustedDocSource {
    pub fn reliability_weight(&self) -> f32 {
        match self {
            TrustedDocSource::ArchWiki => 0.95,
            TrustedDocSource::ManPage => 0.98,
            TrustedDocSource::HelpOutput => 0.90,
            TrustedDocSource::ArchDocs => 0.95,
        }
    }
}

/// v0.3.25: Sentence type classification for evidence requirements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SentenceType {
    /// Factual claim - requires evidence
    Fact,
    /// Suggestion or recommendation - no evidence required
    Suggestion,
    /// Question to user - no evidence required
    Question,
    /// Narrative/explanatory text - no evidence required
    Narrative,
}

impl SentenceType {
    /// Classify a sentence by its type
    pub fn classify(sentence: &str) -> Self {
        let trimmed = sentence.trim();
        let lower = trimmed.to_lowercase();

        // Questions end with ? or start with question words
        if trimmed.ends_with('?') {
            return SentenceType::Question;
        }
        if lower.starts_with("what ")
            || lower.starts_with("how ")
            || lower.starts_with("why ")
            || lower.starts_with("when ")
            || lower.starts_with("where ")
            || lower.starts_with("which ")
            || lower.starts_with("can you ")
            || lower.starts_with("could you ")
            || lower.starts_with("would you ")
        {
            return SentenceType::Question;
        }

        // Suggestions start with recommendation language
        if lower.starts_with("try ")
            || lower.starts_with("consider ")
            || lower.starts_with("you could ")
            || lower.starts_with("you can ")
            || lower.starts_with("you might ")
            || lower.starts_with("you may ")
            || lower.starts_with("i suggest ")
            || lower.starts_with("i recommend ")
            || lower.starts_with("it's recommended ")
            || lower.starts_with("one option ")
            || lower.starts_with("another option ")
        {
            return SentenceType::Suggestion;
        }

        // Facts contain state assertions with "is/are/has/have"
        // Look for patterns like "X is Y", "X are Y", "X has Y"
        let fact_patterns = [
            " is running",
            " is stopped",
            " is active",
            " is inactive",
            " is enabled",
            " is disabled",
            " is installed",
            " is not installed",
            " is missing",
            " is present",
            " is available",
            " is using ",
            " is listening",
            " is open",
            " is closed",
            " are running",
            " are active",
            " are installed",
            " are available",
            " has ",
            " have ",
            " contains ",
            " exists",
            " does not exist",
            "% of ",
            "% used",
            "% free",
            " gb ",
            " mb ",
            " tb ",
            " bytes",
        ];

        for pattern in &fact_patterns {
            if lower.contains(pattern) {
                return SentenceType::Fact;
            }
        }

        // Default to narrative for explanatory text
        SentenceType::Narrative
    }

    /// Returns true if this sentence type requires evidence
    pub fn requires_evidence(&self) -> bool {
        matches!(self, SentenceType::Fact)
    }
}

/// A factual claim that needs evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    /// The claim being made
    pub statement: String,
    /// Category of the claim
    pub category: ClaimCategory,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Evidence backing this claim
    pub evidence: Vec<EvidenceType>,
    /// Whether this claim has passed the gate
    pub verified: bool,
}

/// Categories of claims
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ClaimCategory {
    /// State of a service (running, stopped, etc.)
    ServiceState,
    /// File or directory existence/contents
    FileSystem,
    /// Package installation status
    PackageStatus,
    /// System configuration
    Configuration,
    /// Network state
    NetworkState,
    /// Hardware/resource status
    HardwareStatus,
    /// Process/runtime state
    ProcessState,
    /// General factual claim
    General,
}

impl ClaimCategory {
    /// Minimum confidence required for this category
    pub fn min_confidence(&self) -> f32 {
        match self {
            ClaimCategory::ServiceState => 0.8,
            ClaimCategory::FileSystem => 0.85,
            ClaimCategory::PackageStatus => 0.8,
            ClaimCategory::Configuration => 0.75,
            ClaimCategory::NetworkState => 0.7,
            ClaimCategory::HardwareStatus => 0.8,
            ClaimCategory::ProcessState => 0.75,
            ClaimCategory::General => 0.6,
        }
    }

    /// Probe commands that can verify this category
    pub fn verification_probes(&self) -> Vec<&'static str> {
        match self {
            ClaimCategory::ServiceState => vec![
                "systemctl status {service}",
                "systemctl is-active {service}",
                "systemctl is-enabled {service}",
            ],
            ClaimCategory::FileSystem => vec![
                "test -e {path} && echo exists || echo missing",
                "ls -la {path}",
                "cat {path}",
                "stat {path}",
            ],
            ClaimCategory::PackageStatus => vec![
                "pacman -Q {package}",
                "pacman -Qi {package}",
                "which {binary}",
            ],
            ClaimCategory::Configuration => vec![
                "cat {config_file}",
                "grep {pattern} {config_file}",
            ],
            ClaimCategory::NetworkState => vec![
                "ip addr",
                "ip route",
                "ss -tuln",
                "ping -c1 {host}",
            ],
            ClaimCategory::HardwareStatus => vec![
                "lspci",
                "lsusb",
                "cat /proc/cpuinfo",
                "free -h",
                "df -h",
            ],
            ClaimCategory::ProcessState => vec![
                "ps aux | grep {process}",
                "pgrep {process}",
                "top -bn1 | head -20",
            ],
            ClaimCategory::General => vec![],
        }
    }
}

/// Result of passing a claim through the gate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GateResult {
    /// Claim is verified, can be presented to user
    Verified {
        claim: Claim,
        evidence_summary: String,
    },
    /// Insufficient evidence, need investigation
    NeedsInvestigation {
        claim: Claim,
        missing_evidence: Vec<String>,
        suggested_probes: Vec<String>,
    },
    /// Cannot verify, must admit uncertainty
    CannotVerify {
        claim: Claim,
        reason: String,
        alternative_statement: String,
    },
}

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

/// Configuration for ClaimGate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimGateConfig {
    /// Require at least this many evidence items
    pub min_evidence_count: usize,
    /// Minimum confidence for any claim
    pub min_confidence: f32,
    /// Allow user-provided evidence to count
    pub allow_user_evidence: bool,
    /// Maximum age of cached evidence (seconds)
    pub evidence_cache_ttl: u64,
}

impl Default for ClaimGateConfig {
    fn default() -> Self {
        Self {
            min_evidence_count: 1,
            min_confidence: 0.6,
            allow_user_evidence: false,
            evidence_cache_ttl: 60,
        }
    }
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
    fn calculate_confidence(&self, claim: &Claim) -> f32 {
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
    pub fn evidence_from_probe(command: &str, output: &str, exit_code: i32) -> EvidenceType {
        EvidenceType::ProbeResult {
            command: command.to_string(),
            output: output.to_string(),
            exit_code,
            timestamp: chrono::Utc::now().to_rfc3339(),
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
}

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
    /// Modified response with unverified claims marked
    pub verified_text: String,
    /// Claims that were verified
    pub verified_claims: Vec<Claim>,
    /// Claims that could not be verified
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
}

impl ClaimVerifier for ClaimGate {
    fn verify_response(&self, response: &str, evidence: &[EvidenceType]) -> VerifiedResponse {
        let extracted = Self::extract_claims(response);
        let mut verified_claims = Vec::new();
        let mut unverified_claims = Vec::new();
        let mut suggested_probes = Vec::new();
        let mut modified_text = response.to_string();

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
                GateResult::NeedsInvestigation { claim: unverified, suggested_probes: probes, .. } => {
                    // Mark unverified claim in text
                    let marked = format!("[unverified: {}]", statement);
                    modified_text = modified_text.replace(&statement, &marked);
                    unverified_claims.push(unverified);
                    suggested_probes.extend(probes);
                }
                GateResult::CannotVerify { claim: unverified, alternative_statement, .. } => {
                    modified_text = modified_text.replace(&statement, &alternative_statement);
                    unverified_claims.push(unverified);
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
        result.docs_required = Self::claim_requires_docs(question);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claim_gate_basic() {
        let mut gate = ClaimGate::new();
        let mut claim = gate.submit_claim("nginx is running", ClaimCategory::ServiceState);

        // Without evidence, should need investigation
        let result = gate.verify(&claim);
        assert!(matches!(result, GateResult::NeedsInvestigation { .. }));

        // Add evidence
        let evidence = ClaimGate::evidence_from_probe(
            "systemctl is-active nginx",
            "active",
            0,
        );
        gate.add_evidence(&mut claim, evidence);

        // Now should be verified
        let result = gate.verify(&claim);
        assert!(matches!(result, GateResult::Verified { .. }));
    }

    #[test]
    fn test_claim_extraction() {
        let text = "The service nginx is running and port 80 is open";
        let claims = ClaimGate::extract_claims(text);
        assert!(!claims.is_empty());
    }

    #[test]
    fn test_evidence_confidence() {
        let mut gate = ClaimGate::new();
        let mut claim = gate.submit_claim("test", ClaimCategory::General);

        // No evidence = 0 confidence
        assert_eq!(gate.calculate_confidence(&claim), 0.0);

        // Add probe evidence
        gate.add_evidence(&mut claim, ClaimGate::evidence_from_probe("test", "ok", 0));
        assert!(claim.confidence > 0.5);
    }

    #[test]
    fn test_unverified_statement() {
        let marked = ClaimGate::create_unverified_statement("nginx is running");
        assert!(marked.contains("Cannot verify"));
    }

    // v0.3.25: SentenceType classifier tests
    #[test]
    fn test_sentence_type_fact() {
        assert_eq!(SentenceType::classify("nginx is running"), SentenceType::Fact);
        assert_eq!(SentenceType::classify("The service is stopped"), SentenceType::Fact);
        assert_eq!(SentenceType::classify("Port 80 is open"), SentenceType::Fact);
        assert_eq!(SentenceType::classify("You have 16 GB of RAM"), SentenceType::Fact);
        assert_eq!(SentenceType::classify("The package is installed"), SentenceType::Fact);
        assert_eq!(SentenceType::classify("Disk usage is 47% used"), SentenceType::Fact);
    }

    #[test]
    fn test_sentence_type_suggestion() {
        assert_eq!(SentenceType::classify("Try restarting the service"), SentenceType::Suggestion);
        assert_eq!(SentenceType::classify("Consider using a different port"), SentenceType::Suggestion);
        assert_eq!(SentenceType::classify("You could install htop"), SentenceType::Suggestion);
        assert_eq!(SentenceType::classify("I recommend checking the logs"), SentenceType::Suggestion);
    }

    #[test]
    fn test_sentence_type_question() {
        assert_eq!(SentenceType::classify("Is nginx running?"), SentenceType::Question);
        assert_eq!(SentenceType::classify("What version are you using?"), SentenceType::Question);
        assert_eq!(SentenceType::classify("How much RAM do you have?"), SentenceType::Question);
        assert_eq!(SentenceType::classify("Can you check the config?"), SentenceType::Question);
    }

    #[test]
    fn test_sentence_type_narrative() {
        assert_eq!(SentenceType::classify("Let me check that for you"), SentenceType::Narrative);
        assert_eq!(SentenceType::classify("This configuration file controls nginx behavior"), SentenceType::Narrative);
        assert_eq!(SentenceType::classify("Looking at the output"), SentenceType::Narrative);
    }

    #[test]
    fn test_sentence_type_requires_evidence() {
        assert!(SentenceType::Fact.requires_evidence());
        assert!(!SentenceType::Suggestion.requires_evidence());
        assert!(!SentenceType::Question.requires_evidence());
        assert!(!SentenceType::Narrative.requires_evidence());
    }

    // v0.3.25: ClaimGate enforcement tests
    #[test]
    fn test_fact_without_evidence_blocked() {
        let gate = ClaimGate::new();
        let response = "nginx is running on port 80";
        let result = gate.verify_response(response, &[]); // No evidence
        assert!(result.needs_investigation);
        assert!(!result.unverified_claims.is_empty());
    }

    #[test]
    fn test_fact_with_probe_evidence_passes() {
        let gate = ClaimGate::new();
        let response = "The service nginx is running";
        let evidence = vec![ClaimGate::evidence_from_probe(
            "systemctl is-active nginx",
            "active",
            0,
        )];
        let result = gate.verify_response(response, &evidence);
        // With evidence, claims should be verified
        assert!(!result.verified_claims.is_empty() || result.unverified_claims.is_empty());
    }

    #[test]
    fn test_conflicting_evidence_forces_investigation() {
        let gate = ClaimGate::new();
        let response = "The service nginx is running";
        // Evidence shows nginx is NOT running (exit code 3 = inactive)
        let evidence = vec![ClaimGate::evidence_from_probe(
            "systemctl is-active nginx",
            "inactive",
            3,
        )];
        let result = gate.verify_response(response, &evidence);
        // Lower confidence from failed command
        assert!(result.confidence < 1.0);
    }

    // v0.3.26: Doc citation tests
    #[test]
    fn test_claim_requires_docs() {
        // "How X works" questions require docs
        assert!(ClaimGate::claim_requires_docs("how does systemctl mask work"));
        assert!(ClaimGate::claim_requires_docs("what does the -S flag mean"));
        assert!(ClaimGate::claim_requires_docs("explain TRIM"));
        assert!(ClaimGate::claim_requires_docs("configure ssh"));
        assert!(ClaimGate::claim_requires_docs("syntax for crontab"));

        // State queries don't require docs
        assert!(!ClaimGate::claim_requires_docs("is nginx running"));
        assert!(!ClaimGate::claim_requires_docs("list services"));
    }

    #[test]
    fn test_has_doc_evidence() {
        let probe_only = vec![
            ClaimGate::evidence_from_probe("free -h", "16GB", 0),
        ];
        assert!(!ClaimGate::has_doc_evidence(&probe_only));

        let with_wiki = vec![
            ClaimGate::evidence_from_probe("free -h", "16GB", 0),
            ClaimGate::evidence_from_wiki("Systemd", Some("User units"), "systemctl --user"),
        ];
        assert!(ClaimGate::has_doc_evidence(&with_wiki));

        let with_man = vec![
            ClaimGate::evidence_from_man("systemctl", 1, "mask - mask units"),
        ];
        assert!(ClaimGate::has_doc_evidence(&with_man));
    }

    #[test]
    fn test_verify_with_context_docs_required() {
        let gate = ClaimGate::new();
        let question = "how does systemctl mask work";
        let response = "systemctl mask prevents a unit from being started";

        // Without doc evidence, should need investigation
        let probe_evidence = vec![
            ClaimGate::evidence_from_probe("systemctl mask test", "Created symlink", 0),
        ];
        let result = gate.verify_response_with_context(response, question, &probe_evidence);
        assert!(result.docs_required);
        assert!(!result.docs_found);
        assert!(result.needs_investigation);

        // With doc evidence, should be fine
        let doc_evidence = vec![
            ClaimGate::evidence_from_man("systemctl", 1, "mask UNIT... Mask one or more units"),
        ];
        let result = gate.verify_response_with_context(response, question, &doc_evidence);
        assert!(result.docs_required);
        assert!(result.docs_found);
    }

    #[test]
    fn test_doc_citation_formatting() {
        let gate = ClaimGate::new();
        let response = "ok";
        let evidence = vec![
            ClaimGate::evidence_from_wiki("Systemd", Some("Timers"), "OnCalendar="),
            ClaimGate::evidence_from_man("systemctl", 1, "mask units"),
        ];
        let result = gate.verify_response(response, &evidence);
        assert_eq!(result.doc_citations.len(), 2);
        assert!(result.doc_citations[0].contains("Arch Wiki"));
        assert!(result.doc_citations[1].contains("man"));
    }

    #[test]
    fn test_probe_only_for_state_query() {
        let gate = ClaimGate::new();
        let question = "how much RAM is free";
        let response = "You have 8GB free";
        let evidence = vec![
            ClaimGate::evidence_from_probe("free -h", "Mem: 16G 8G 8G", 0),
        ];
        let result = gate.verify_response_with_context(response, question, &evidence);
        // "how much" is not a "how does" question, so no docs required
        assert!(!result.docs_required);
    }
}
