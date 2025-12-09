//! Legacy clarification types (v0.0.191).
//!
//! v0.0.32-v0.0.39 types for backwards compatibility.

use serde::{Deserialize, Serialize};

use crate::facts::{FactKey, FactsStore};

/// Clarification kind enum (legacy)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClarifyKind {
    PreferredEditor,
    ServiceName,
    MountPoint,
    NetworkInterface,
    ProcessName,
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

/// Clarification question (legacy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyQuestion {
    pub kind: ClarifyKind,
    pub question: String,
    pub verify_probe: Option<String>,
    pub hint: Option<String>,
    pub default: Option<String>,
}

impl ClarifyQuestion {
    pub fn new(kind: ClarifyKind, question: impl Into<String>) -> Self {
        Self {
            kind,
            question: question.into(),
            verify_probe: None,
            hint: None,
            default: None,
        }
    }
    pub fn with_verify(mut self, probe: impl Into<String>) -> Self {
        self.verify_probe = Some(probe.into());
        self
    }
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }
}

/// Clarification option (legacy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyOption {
    pub key: String,
    pub label: String,
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

/// Clarification answer (legacy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyAnswer {
    pub question_id: String,
    pub selected_key: String,
}

/// Clarification result (legacy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClarifyResultLegacy {
    Verified {
        kind: ClarifyKind,
        value: String,
        probe_output: Option<String>,
    },
    Unverified {
        kind: ClarifyKind,
        value: String,
        reason: String,
    },
    Declined,
}

/// Generate question based on kind (legacy)
pub fn generate_question(kind: ClarifyKind, facts: &FactsStore) -> ClarifyQuestion {
    match &kind {
        ClarifyKind::PreferredEditor => {
            let default = facts
                .get_verified(&FactKey::PreferredEditor)
                .map(|s| s.to_string());
            ClarifyQuestion::new(kind, "What text editor do you prefer?")
                .with_verify("which {}")
                .with_hint("vim, nano, emacs, code, nvim")
                .with_default(default.unwrap_or_default())
        }
        ClarifyKind::ServiceName => ClarifyQuestion::new(kind, "Which service?")
            .with_verify("systemctl is-active {}")
            .with_hint("nginx, docker, sshd"),
        ClarifyKind::MountPoint => ClarifyQuestion::new(kind, "Which mount point?")
            .with_verify("df {}")
            .with_hint("/, /home, /var"),
        ClarifyKind::NetworkInterface => {
            let default = facts
                .get_verified(&FactKey::NetworkPrimaryInterface)
                .map(|s| s.to_string());
            ClarifyQuestion::new(kind, "Which network interface?")
                .with_verify("ip addr show {}")
                .with_hint("eth0, wlan0")
                .with_default(default.unwrap_or_default())
        }
        ClarifyKind::ProcessName => ClarifyQuestion::new(kind, "Which process?")
            .with_verify("pgrep -x {}")
            .with_hint("firefox, chrome"),
        ClarifyKind::Custom(desc) => {
            ClarifyQuestion::new(kind.clone(), format!("Please specify: {}", desc))
        }
    }
}

/// Map clarify kind to fact key (legacy)
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

/// Build verify command from template (legacy)
pub fn build_verify_command(template: &str, value: &str) -> String {
    template.replace("{}", value)
}
