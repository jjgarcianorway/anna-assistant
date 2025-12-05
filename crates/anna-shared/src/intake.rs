//! Intake module for query analysis and clarification planning.
//!
//! Determines what clarifications are needed and how to verify them.
//! Supports the "research first, ask second, verify always" philosophy.

use crate::facts::{FactKey, FactsStore, FactStatus};
use crate::rpc::{QueryIntent, SpecialistDomain};
use serde::{Deserialize, Serialize};

/// Plan for verifying a user's clarification answer
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerifyPlan {
    /// No verification needed
    None,
    /// Verify binary exists: `command -v <binary>` or `which <binary>`
    BinaryExists { binary: String },
    /// Verify systemd unit exists
    UnitExists { unit: String },
    /// Verify mount point exists (check df/lsblk output)
    MountExists { mount: String },
    /// Verify network interface exists (check ip link output)
    InterfaceExists { iface: String },
    /// Verify file exists
    FileExists { path: String },
    /// Verify directory exists
    DirectoryExists { path: String },
    /// Verify from existing probe evidence (key in evidence map)
    FromEvidence { key: String },
}

impl VerifyPlan {
    /// Get the probe command for this verification plan
    pub fn probe_command(&self) -> Option<String> {
        match self {
            Self::None => None,
            Self::BinaryExists { binary } => Some(format!("command -v {}", binary)),
            Self::UnitExists { unit } => Some(format!("systemctl list-unit-files {}", unit)),
            Self::MountExists { .. } => Some("df -h".to_string()), // Parse output
            Self::InterfaceExists { .. } => Some("ip link show".to_string()), // Parse output
            Self::FileExists { path } => Some(format!("test -f {} && echo exists", path)),
            Self::DirectoryExists { path } => Some(format!("test -d {} && echo exists", path)),
            Self::FromEvidence { .. } => None, // Use existing evidence
        }
    }

    /// Check if this plan requires running a probe
    pub fn needs_probe(&self) -> bool {
        !matches!(self, Self::None | Self::FromEvidence { .. })
    }
}

/// A clarification question with verification plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationQuestion {
    /// Unique ID for this question type
    pub id: String,
    /// The question to ask the user
    pub prompt: String,
    /// Optional choices to present
    pub choices: Vec<String>,
    /// Why this clarification is needed
    pub reason: String,
    /// How to verify the user's answer
    pub verify: VerifyPlan,
    /// Which fact key this clarification will populate
    pub populates: Option<FactKey>,
    /// Priority (lower = ask first)
    pub priority: u8,
}

impl ClarificationQuestion {
    /// Create a new clarification question
    pub fn new(id: &str, prompt: &str, reason: &str) -> Self {
        Self {
            id: id.to_string(),
            prompt: prompt.to_string(),
            choices: vec![],
            reason: reason.to_string(),
            verify: VerifyPlan::None,
            populates: None,
            priority: 50,
        }
    }

    /// Add choices
    pub fn with_choices(mut self, choices: Vec<&str>) -> Self {
        self.choices = choices.into_iter().map(String::from).collect();
        self
    }

    /// Set verification plan
    pub fn with_verify(mut self, verify: VerifyPlan) -> Self {
        self.verify = verify;
        self
    }

    /// Set fact key this populates
    pub fn populates_fact(mut self, key: FactKey) -> Self {
        self.populates = Some(key);
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
}

/// Result of intake analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntakeResult {
    /// The classified intent
    pub intent: QueryIntent,
    /// The target domain
    pub domain: SpecialistDomain,
    /// Clarifications needed before proceeding
    pub clarifications_needed: Vec<ClarificationQuestion>,
    /// Facts that were already known (from FactsStore)
    pub facts_used: Vec<FactKey>,
    /// Whether intake can proceed without clarification
    pub can_proceed: bool,
    /// Confidence in the classification (0.0-1.0)
    pub confidence: f32,
}

impl IntakeResult {
    /// Create a result that can proceed without clarification
    pub fn proceed(intent: QueryIntent, domain: SpecialistDomain) -> Self {
        Self {
            intent,
            domain,
            clarifications_needed: vec![],
            facts_used: vec![],
            can_proceed: true,
            confidence: 1.0,
        }
    }

    /// Create a result that needs clarification
    pub fn needs_clarification(
        intent: QueryIntent,
        domain: SpecialistDomain,
        clarifications: Vec<ClarificationQuestion>,
    ) -> Self {
        Self {
            intent,
            domain,
            clarifications_needed: clarifications,
            facts_used: vec![],
            can_proceed: false,
            confidence: 0.5,
        }
    }
}

/// Slot types that may need clarification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClarificationSlot {
    /// Which editor to use
    EditorName,
    /// Config file path
    ConfigPath,
    /// Network interface
    NetworkInterface,
    /// Service/unit name
    ServiceName,
    /// Mount point
    MountPoint,
    /// Package name
    PackageName,
}

impl ClarificationSlot {
    /// Get the fact key this slot maps to
    pub fn to_fact_key(&self) -> Option<FactKey> {
        match self {
            Self::EditorName => Some(FactKey::PreferredEditor),
            Self::NetworkInterface => Some(FactKey::NetworkPrimaryInterface),
            _ => None,
        }
    }
}

/// Generate clarification question for a slot
pub fn generate_clarification(slot: ClarificationSlot, context: &str) -> ClarificationQuestion {
    match slot {
        ClarificationSlot::EditorName => ClarificationQuestion::new(
            "editor_selection",
            "Which text editor would you like me to configure?",
            context,
        )
        .with_choices(vec!["vim", "nvim", "nano", "vi", "emacs"])
        .with_verify(VerifyPlan::BinaryExists {
            binary: "PLACEHOLDER".to_string(), // Will be replaced with user's answer
        })
        .populates_fact(FactKey::PreferredEditor)
        .with_priority(10),

        ClarificationSlot::ConfigPath => ClarificationQuestion::new(
            "config_path",
            "Which configuration file should I modify?",
            context,
        )
        .with_verify(VerifyPlan::FileExists {
            path: "PLACEHOLDER".to_string(),
        })
        .with_priority(20),

        ClarificationSlot::NetworkInterface => ClarificationQuestion::new(
            "network_interface",
            "Which network connection are you having trouble with?",
            context,
        )
        .with_choices(vec!["wifi", "ethernet", "both"])
        .with_verify(VerifyPlan::FromEvidence {
            key: "network_interfaces".to_string(),
        })
        .populates_fact(FactKey::NetworkPreference)
        .with_priority(15),

        ClarificationSlot::ServiceName => ClarificationQuestion::new(
            "service_name",
            "Which service are you asking about?",
            context,
        )
        .with_verify(VerifyPlan::UnitExists {
            unit: "PLACEHOLDER".to_string(),
        })
        .with_priority(10),

        ClarificationSlot::MountPoint => ClarificationQuestion::new(
            "mount_point",
            "Which disk or partition are you asking about?",
            context,
        )
        .with_verify(VerifyPlan::MountExists {
            mount: "PLACEHOLDER".to_string(),
        })
        .with_priority(20),

        ClarificationSlot::PackageName => ClarificationQuestion::new(
            "package_name",
            "Which package should I help you with?",
            context,
        )
        .with_priority(10),
    }
}

/// Check if a clarification is already satisfied by known facts
pub fn check_slot_satisfied(slot: ClarificationSlot, facts: &FactsStore) -> Option<String> {
    let key = slot.to_fact_key()?;
    facts.get_verified(&key).map(String::from)
}

/// Analyze query and determine needed clarifications
pub fn analyze_intake(
    query: &str,
    intent: QueryIntent,
    domain: SpecialistDomain,
    facts: &FactsStore,
    entities: &[String],
) -> IntakeResult {
    let q = query.to_lowercase();
    let mut clarifications = Vec::new();
    let mut facts_used = Vec::new();

    // Check for editor-related queries
    if needs_editor_clarification(&q, &intent, entities) {
        match facts.status(&FactKey::PreferredEditor) {
            FactStatus::Known(editor) => {
                facts_used.push(FactKey::PreferredEditor);
                // Also check if the editor binary is still available
                let binary_key = FactKey::BinaryAvailable(editor.clone());
                if facts.has_verified(&binary_key) {
                    facts_used.push(binary_key);
                }
            }
            _ => {
                clarifications.push(generate_clarification(
                    ClarificationSlot::EditorName,
                    "I need to know which editor to configure",
                ));
            }
        }
    }

    // Check for network-related queries
    if needs_network_clarification(&q, &intent) {
        match facts.status(&FactKey::NetworkPrimaryInterface) {
            FactStatus::Known(_iface) => {
                facts_used.push(FactKey::NetworkPrimaryInterface);
                facts_used.push(FactKey::NetworkPreference);
            }
            _ => {
                clarifications.push(generate_clarification(
                    ClarificationSlot::NetworkInterface,
                    "I need to know which network connection you're asking about",
                ));
            }
        }
    }

    // Check for service-related queries that need a specific service
    if needs_service_clarification(&q, entities) {
        clarifications.push(generate_clarification(
            ClarificationSlot::ServiceName,
            "I need to know which service you're asking about",
        ));
    }

    // Sort by priority
    clarifications.sort_by_key(|c| c.priority);

    IntakeResult {
        intent,
        domain,
        clarifications_needed: clarifications.clone(),
        facts_used,
        can_proceed: clarifications.is_empty(),
        confidence: if clarifications.is_empty() { 1.0 } else { 0.5 },
    }
}

/// Check if query needs editor clarification
fn needs_editor_clarification(query: &str, intent: &QueryIntent, entities: &[String]) -> bool {
    // Editor config requests need clarification
    if query.contains("editor") || query.contains("syntax") || query.contains("highlight") {
        if matches!(intent, QueryIntent::Request) {
            return true;
        }
    }

    // Check entities for editor-related terms
    let editor_terms = ["vim", "nvim", "nano", "emacs", "vimrc", "config"];
    for entity in entities {
        let e = entity.to_lowercase();
        for term in editor_terms {
            if e.contains(term) {
                return matches!(intent, QueryIntent::Request);
            }
        }
    }

    false
}

/// Check if query needs network clarification
fn needs_network_clarification(query: &str, intent: &QueryIntent) -> bool {
    let network_terms = ["internet", "connection", "network", "wifi", "ethernet", "broken"];
    let matches_network = network_terms.iter().any(|t| query.contains(t));

    if !matches_network {
        return false;
    }

    // Investigation queries about network need clarification
    matches!(intent, QueryIntent::Investigate)
}

/// Check if query needs service clarification (when no specific service mentioned)
fn needs_service_clarification(query: &str, entities: &[String]) -> bool {
    if !query.contains("service") && !query.contains("restart") && !query.contains("status") {
        return false;
    }

    // If entities already contain a service name, no clarification needed
    for entity in entities {
        if entity.ends_with(".service") || entity.contains("systemd") {
            return false;
        }
    }

    // Check for common service names in query
    let common_services = ["nginx", "apache", "docker", "ssh", "postgres", "mysql", "redis"];
    for svc in common_services {
        if query.contains(svc) {
            return false;
        }
    }

    // Generic service question needs clarification
    query.contains("service") && !query.contains("all service") && !query.contains("failed service")
}

/// Verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether verification passed
    pub verified: bool,
    /// The verified value (if successful)
    pub value: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Alternative options found (if verification failed)
    pub alternatives: Vec<String>,
    /// Source of verification
    pub source: String,
}

impl VerificationResult {
    /// Create a successful verification
    pub fn success(value: String, source: &str) -> Self {
        Self {
            verified: true,
            value: Some(value),
            error: None,
            alternatives: vec![],
            source: source.to_string(),
        }
    }

    /// Create a failed verification with alternatives
    pub fn failed_with_alternatives(error: &str, alternatives: Vec<String>, source: &str) -> Self {
        Self {
            verified: false,
            value: None,
            error: Some(error.to_string()),
            alternatives,
            source: source.to_string(),
        }
    }

    /// Create a simple failure
    pub fn failed(error: &str, source: &str) -> Self {
        Self {
            verified: false,
            value: None,
            error: Some(error.to_string()),
            alternatives: vec![],
            source: source.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_plan_probe_command() {
        let plan = VerifyPlan::BinaryExists { binary: "vim".to_string() };
        assert_eq!(plan.probe_command(), Some("command -v vim".to_string()));

        let plan = VerifyPlan::None;
        assert_eq!(plan.probe_command(), None);
    }

    #[test]
    fn test_clarification_question_builder() {
        let q = ClarificationQuestion::new("test", "Test question?", "testing")
            .with_choices(vec!["a", "b"])
            .with_verify(VerifyPlan::BinaryExists { binary: "test".to_string() })
            .with_priority(5);

        assert_eq!(q.id, "test");
        assert_eq!(q.choices, vec!["a", "b"]);
        assert_eq!(q.priority, 5);
    }

    #[test]
    fn test_analyze_intake_editor_with_known_fact() {
        let mut facts = FactsStore::new();
        facts.set_verified(FactKey::PreferredEditor, "vim".to_string(), "test".to_string());
        facts.set_verified(
            FactKey::BinaryAvailable("vim".to_string()),
            "/usr/bin/vim".to_string(),
            "test".to_string(),
        );

        let result = analyze_intake(
            "enable syntax highlighting in my editor",
            QueryIntent::Request,
            SpecialistDomain::System,
            &facts,
            &[],
        );

        assert!(result.can_proceed);
        assert!(result.clarifications_needed.is_empty());
        assert!(result.facts_used.contains(&FactKey::PreferredEditor));
    }

    #[test]
    fn test_analyze_intake_editor_without_fact() {
        let facts = FactsStore::new();

        let result = analyze_intake(
            "enable syntax highlighting in my editor",
            QueryIntent::Request,
            SpecialistDomain::System,
            &facts,
            &[],
        );

        assert!(!result.can_proceed);
        assert!(!result.clarifications_needed.is_empty());
        assert_eq!(result.clarifications_needed[0].id, "editor_selection");
    }

    #[test]
    fn test_analyze_intake_network_clarification() {
        let facts = FactsStore::new();

        let result = analyze_intake(
            "my internet connection is broken",
            QueryIntent::Investigate,
            SpecialistDomain::Network,
            &facts,
            &[],
        );

        assert!(!result.can_proceed);
        assert!(result.clarifications_needed.iter().any(|c| c.id == "network_interface"));
    }

    #[test]
    fn test_verification_result_success() {
        let result = VerificationResult::success("/usr/bin/vim".to_string(), "probe:which vim");
        assert!(result.verified);
        assert_eq!(result.value, Some("/usr/bin/vim".to_string()));
    }

    #[test]
    fn test_verification_result_failed_with_alternatives() {
        let result = VerificationResult::failed_with_alternatives(
            "vim not found",
            vec!["vi".to_string(), "nvim".to_string()],
            "probe:which vim",
        );
        assert!(!result.verified);
        assert_eq!(result.alternatives, vec!["vi", "nvim"]);
    }

    #[test]
    fn test_check_slot_satisfied() {
        let mut facts = FactsStore::new();
        facts.set_verified(FactKey::PreferredEditor, "vim".to_string(), "test".to_string());

        let result = check_slot_satisfied(ClarificationSlot::EditorName, &facts);
        assert_eq!(result, Some("vim".to_string()));

        let result = check_slot_satisfied(ClarificationSlot::NetworkInterface, &facts);
        assert_eq!(result, None);
    }
}
