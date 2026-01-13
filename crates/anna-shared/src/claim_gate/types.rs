//! Type definitions for ClaimGate.

use serde::{Deserialize, Serialize};

/// Types of evidence that can back a claim
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EvidenceType {
    /// Command output from a probe
    ProbeResult {
        command: String,
        output: String,
        exit_code: i32,
        timestamp: String,
        /// v0.3.28: Whether output was empty on success (Phase 3 F15)
        #[serde(default)]
        output_empty: bool,
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
