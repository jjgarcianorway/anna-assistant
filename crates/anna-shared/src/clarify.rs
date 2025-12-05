//! Clarification questions with verification probes (v0.0.32).
//!
//! When information is missing to answer a query, we ask concrete clarification
//! questions with associated probes to verify the answer.

use crate::facts::{FactKey, FactsStore};
use serde::{Deserialize, Serialize};

/// What kind of clarification is needed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClarifyKind {
    /// Which text editor they prefer
    PreferredEditor,
    /// Which service they're asking about
    ServiceName,
    /// Which mount point or disk
    MountPoint,
    /// Which network interface
    NetworkInterface,
    /// Which process/application
    ProcessName,
    /// Custom clarification
    Custom(String),
}

impl std::fmt::Display for ClarifyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PreferredEditor => write!(f, "preferred_editor"),
            Self::ServiceName => write!(f, "service_name"),
            Self::MountPoint => write!(f, "mount_point"),
            Self::NetworkInterface => write!(f, "network_interface"),
            Self::ProcessName => write!(f, "process_name"),
            Self::Custom(s) => write!(f, "custom:{}", s),
        }
    }
}

/// A clarification question with verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyQuestion {
    /// What kind of clarification this is
    pub kind: ClarifyKind,
    /// The question to ask the user
    pub question: String,
    /// Probe command to verify the answer (if applicable)
    pub verify_probe: Option<String>,
    /// Hint about what values are valid (for UI)
    pub hint: Option<String>,
    /// Default value if known from facts
    pub default: Option<String>,
}

impl ClarifyQuestion {
    /// Create a new clarification question
    pub fn new(kind: ClarifyKind, question: impl Into<String>) -> Self {
        Self {
            kind,
            question: question.into(),
            verify_probe: None,
            hint: None,
            default: None,
        }
    }

    /// Add verification probe
    pub fn with_verify(mut self, probe: impl Into<String>) -> Self {
        self.verify_probe = Some(probe.into());
        self
    }

    /// Add hint
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Add default from facts
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }
}

/// An option in a clarification question (v0.0.34)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyOption {
    /// Key to select this option (e.g., "1", "vim")
    pub key: String,
    /// Display label
    pub label: String,
    /// Evidence strings for this option (e.g., "installed: true", "recently used: 12 times")
    pub evidence: Vec<String>,
}

impl ClarifyOption {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            evidence: vec![],
        }
    }

    pub fn with_evidence(mut self, ev: impl Into<String>) -> Self {
        self.evidence.push(ev.into());
        self
    }
}

/// User's answer to a clarification question (v0.0.34)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyAnswer {
    /// ID of the question being answered
    pub question_id: String,
    /// Key of the selected option
    pub selected_key: String,
}

/// Result of a clarification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClarifyResult {
    /// Answer was verified, can learn fact
    Verified {
        kind: ClarifyKind,
        value: String,
        probe_output: Option<String>,
    },
    /// Answer could not be verified
    Unverified {
        kind: ClarifyKind,
        value: String,
        reason: String,
    },
    /// User declined to answer
    Declined,
}

/// Generate clarification question for a given kind
pub fn generate_question(kind: ClarifyKind, facts: &FactsStore) -> ClarifyQuestion {
    match &kind {
        ClarifyKind::PreferredEditor => {
            let default = facts.get_verified(&FactKey::PreferredEditor)
                .map(|s| s.to_string());
            ClarifyQuestion::new(kind, "What text editor do you prefer? (e.g., vim, nano, code)")
                .with_verify("which {}")
                .with_hint("vim, nano, emacs, code, nvim")
                .with_default(default.unwrap_or_default())
        }
        ClarifyKind::ServiceName => {
            ClarifyQuestion::new(kind, "Which service are you asking about?")
                .with_verify("systemctl is-active {}")
                .with_hint("nginx, docker, sshd")
        }
        ClarifyKind::MountPoint => {
            ClarifyQuestion::new(kind, "Which mount point or disk are you asking about?")
                .with_verify("df {}")
                .with_hint("/, /home, /var")
        }
        ClarifyKind::NetworkInterface => {
            let default = facts.get_verified(&FactKey::NetworkPrimaryInterface)
                .map(|s| s.to_string());
            ClarifyQuestion::new(kind, "Which network interface?")
                .with_verify("ip addr show {}")
                .with_hint("eth0, wlan0, enp3s0")
                .with_default(default.unwrap_or_default())
        }
        ClarifyKind::ProcessName => {
            ClarifyQuestion::new(kind, "Which process or application?")
                .with_verify("pgrep -x {}")
                .with_hint("firefox, chrome, code")
        }
        ClarifyKind::Custom(desc) => {
            ClarifyQuestion::new(kind.clone(), format!("Please specify: {}", desc))
        }
    }
}

/// Convert ClarifyKind to FactKey for storage (if applicable)
pub fn kind_to_fact_key(kind: &ClarifyKind, value: &str) -> Option<FactKey> {
    match kind {
        ClarifyKind::PreferredEditor => Some(FactKey::PreferredEditor),
        ClarifyKind::ServiceName => Some(FactKey::UnitExists(value.to_string())),
        ClarifyKind::MountPoint => Some(FactKey::MountExists(value.to_string())),
        ClarifyKind::NetworkInterface => Some(FactKey::NetworkPrimaryInterface),
        ClarifyKind::ProcessName => Some(FactKey::BinaryAvailable(value.to_string())),
        ClarifyKind::Custom(_) => None,
    }
}

/// Build verification command from template
pub fn build_verify_command(template: &str, value: &str) -> String {
    template.replace("{}", value)
}

/// Check if clarification is needed based on query and facts
pub fn needs_clarification(query: &str, facts: &FactsStore) -> Option<ClarifyKind> {
    let q = query.to_lowercase();

    // Check for editor-related queries without specified editor
    if (q.contains("edit") || q.contains("open") || q.contains("editor"))
        && !q.contains("vim")
        && !q.contains("nano")
        && !q.contains("emacs")
        && !q.contains("code")
        && !facts.has_verified(&FactKey::PreferredEditor)
    {
        return Some(ClarifyKind::PreferredEditor);
    }

    // Check for service queries without specific service
    if (q.contains("service") || q.contains("systemctl"))
        && !q.contains("--failed")
        && !extract_service_name(&q).is_some()
    {
        return Some(ClarifyKind::ServiceName);
    }

    // Check for mount/disk queries without specific path
    if (q.contains("mount") || q.contains("partition"))
        && !q.contains("/")
    {
        return Some(ClarifyKind::MountPoint);
    }

    None
}

/// Try to extract service name from query
fn extract_service_name(query: &str) -> Option<String> {
    // Look for common service name patterns
    let patterns = ["nginx", "docker", "sshd", "apache", "mysql", "postgresql", "redis"];
    for p in patterns {
        if query.contains(p) {
            return Some(p.to_string());
        }
    }
    // Look for ".service" pattern
    if let Some(idx) = query.find(".service") {
        let before = &query[..idx];
        if let Some(start) = before.rfind(' ') {
            return Some(before[start+1..].to_string());
        }
    }
    None
}

/// Known editors to check for (v0.0.35)
pub const KNOWN_EDITORS: &[&str] = &["vim", "vi", "nvim", "nano", "emacs", "code", "micro", "hx"];

/// Generate editor options based on installed editors (v0.0.35)
/// Probes `command -v <editor>` to check availability.
/// Returns ClarifyOptions with evidence for installed status.
pub fn generate_editor_options_sync() -> Vec<ClarifyOption> {
    let mut options = Vec::new();

    for editor in KNOWN_EDITORS {
        let installed = std::process::Command::new("command")
            .args(["-v", editor])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        // Also try `which` as fallback (command -v may not work in all shells)
        let installed = installed || std::process::Command::new("which")
            .arg(editor)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if installed {
            options.push(ClarifyOption::new(*editor, *editor)
                .with_evidence("installed: true"));
        }
    }

    // Always add "other" option
    if !options.is_empty() {
        options.push(ClarifyOption::new("other", "Other (specify)")
            .with_evidence("custom input"));
    }

    options
}

/// Verify that an editor is installed (v0.0.35)
pub fn verify_editor_installed(editor: &str) -> bool {
    // Try `which <editor>`
    std::process::Command::new("which")
        .arg(editor)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Generate editor clarification with detected options (v0.0.35)
pub fn generate_editor_clarification(facts: &FactsStore) -> (ClarifyQuestion, Vec<ClarifyOption>) {
    let default = facts.get_verified(&FactKey::PreferredEditor)
        .map(|s| s.to_string());

    let question = ClarifyQuestion::new(
        ClarifyKind::PreferredEditor,
        "Which text editor do you prefer?"
    )
    .with_verify("which {}")
    .with_hint("Select from installed editors or specify another")
    .with_default(default.unwrap_or_default());

    let options = generate_editor_options_sync();

    (question, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_editor_question() {
        let facts = FactsStore::new();
        let q = generate_question(ClarifyKind::PreferredEditor, &facts);
        assert!(q.question.contains("editor"));
        assert!(q.verify_probe.is_some());
    }

    #[test]
    fn test_generate_editor_with_default() {
        let mut facts = FactsStore::new();
        facts.set_verified(FactKey::PreferredEditor, "vim".to_string(), "user".to_string());
        let q = generate_question(ClarifyKind::PreferredEditor, &facts);
        assert_eq!(q.default, Some("vim".to_string()));
    }

    #[test]
    fn test_build_verify_command() {
        let cmd = build_verify_command("which {}", "vim");
        assert_eq!(cmd, "which vim");

        let cmd = build_verify_command("systemctl is-active {}", "nginx");
        assert_eq!(cmd, "systemctl is-active nginx");
    }

    #[test]
    fn test_kind_to_fact_key() {
        assert_eq!(
            kind_to_fact_key(&ClarifyKind::PreferredEditor, "vim"),
            Some(FactKey::PreferredEditor)
        );
        assert_eq!(
            kind_to_fact_key(&ClarifyKind::ServiceName, "nginx"),
            Some(FactKey::UnitExists("nginx".to_string()))
        );
    }

    #[test]
    fn test_needs_clarification_editor() {
        let facts = FactsStore::new();

        // Needs clarification
        assert_eq!(
            needs_clarification("edit the config file", &facts),
            Some(ClarifyKind::PreferredEditor)
        );

        // Already specified editor
        assert_eq!(needs_clarification("open in vim", &facts), None);
    }

    #[test]
    fn test_needs_clarification_with_known_fact() {
        let mut facts = FactsStore::new();
        facts.set_verified(FactKey::PreferredEditor, "vim".to_string(), "user".to_string());

        // No clarification needed - we know their editor
        assert_eq!(needs_clarification("edit the config file", &facts), None);
    }

    #[test]
    fn test_extract_service_name() {
        assert_eq!(extract_service_name("is nginx running"), Some("nginx".to_string()));
        assert_eq!(extract_service_name("check foo.service status"), Some("foo".to_string()));
        assert_eq!(extract_service_name("show all services"), None);
    }

    #[test]
    fn test_known_editors_list() {
        assert!(KNOWN_EDITORS.contains(&"vim"));
        assert!(KNOWN_EDITORS.contains(&"nano"));
        assert!(KNOWN_EDITORS.contains(&"nvim"));
    }

    #[test]
    fn test_clarify_option_with_evidence() {
        let opt = ClarifyOption::new("vim", "Vim")
            .with_evidence("installed: true")
            .with_evidence("recently used: 5 times");
        assert_eq!(opt.evidence.len(), 2);
    }

    #[test]
    fn test_clarify_answer_structure() {
        let answer = ClarifyAnswer {
            question_id: "q1".to_string(),
            selected_key: "vim".to_string(),
        };
        assert_eq!(answer.question_id, "q1");
        assert_eq!(answer.selected_key, "vim");
    }

    #[test]
    fn test_generate_editor_clarification() {
        let facts = FactsStore::new();
        let (question, _options) = generate_editor_clarification(&facts);
        assert_eq!(question.kind, ClarifyKind::PreferredEditor);
        assert!(question.question.contains("editor"));
    }
}
