//! Specialist routing types (v0.0.405).
//! v0.0.405: Expanded domains and intents per clean architecture roadmap.

use serde::{Deserialize, Serialize};

/// Specialist domain for service desk routing
/// v0.0.405: Expanded to cover all system aspects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SpecialistDomain {
    #[default]
    System,
    Boot,
    Services,
    Network,
    Storage,
    Packages,
    Audio,
    Display,
    Desktop,
    Security,
}

impl SpecialistDomain {
    /// All domain variants for iteration
    pub const ALL: &'static [SpecialistDomain] = &[
        Self::System,
        Self::Boot,
        Self::Services,
        Self::Network,
        Self::Storage,
        Self::Packages,
        Self::Audio,
        Self::Display,
        Self::Desktop,
        Self::Security,
    ];

    /// Parse from string (case insensitive)
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "system" => Some(Self::System),
            "boot" => Some(Self::Boot),
            "services" | "service" | "systemd" => Some(Self::Services),
            "network" | "net" | "networking" => Some(Self::Network),
            "storage" | "disk" | "disks" => Some(Self::Storage),
            "packages" | "package" | "pacman" => Some(Self::Packages),
            "audio" | "sound" | "pulseaudio" | "pipewire" => Some(Self::Audio),
            "display" | "graphics" | "gpu" | "screen" => Some(Self::Display),
            "desktop" | "de" | "wm" | "hyprland" | "kde" | "gnome" => Some(Self::Desktop),
            "security" | "permissions" | "firewall" => Some(Self::Security),
            _ => None,
        }
    }
}

impl std::fmt::Display for SpecialistDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "system"),
            Self::Boot => write!(f, "boot"),
            Self::Services => write!(f, "services"),
            Self::Network => write!(f, "network"),
            Self::Storage => write!(f, "storage"),
            Self::Packages => write!(f, "packages"),
            Self::Audio => write!(f, "audio"),
            Self::Display => write!(f, "display"),
            Self::Desktop => write!(f, "desktop"),
            Self::Security => write!(f, "security"),
        }
    }
}

/// Intent classification from translator
/// v0.0.405: Expanded to cover all query types per roadmap
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum QueryIntent {
    /// Query a specific metric (how much RAM, disk space, etc.)
    #[default]
    QueryMetric,
    /// Diagnose a problem (why is X slow, what's wrong with Y)
    Diagnose,
    /// Configure something (enable X, set Y)
    Configure,
    /// List items (show services, list packages)
    List,
    /// Check status (is X running, is Y enabled)
    CheckStatus,
    /// Explain something (what is X, how does Y work)
    Explain,
    // Legacy variants for backward compatibility
    /// Generic question (maps to QueryMetric or Explain)
    #[serde(alias = "question")]
    Question,
    /// Generic request (maps to Configure)
    #[serde(alias = "request")]
    Request,
    /// Investigation (maps to Diagnose)
    #[serde(alias = "investigate")]
    Investigate,
}

impl QueryIntent {
    /// All intent variants for iteration
    pub const ALL: &'static [QueryIntent] = &[
        Self::QueryMetric,
        Self::Diagnose,
        Self::Configure,
        Self::List,
        Self::CheckStatus,
        Self::Explain,
    ];

    /// Parse from string (case insensitive)
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "query_metric" | "metric" | "query" => Some(Self::QueryMetric),
            "diagnose" | "diagnosis" | "investigate" | "debug" => Some(Self::Diagnose),
            "configure" | "config" | "setup" | "enable" | "disable" | "request" => {
                Some(Self::Configure)
            }
            "list" | "show" | "enumerate" => Some(Self::List),
            "check_status" | "status" | "check" | "is_running" => Some(Self::CheckStatus),
            "explain" | "what_is" | "how" | "why" | "question" => Some(Self::Explain),
            _ => None,
        }
    }

    /// Normalize legacy intents to new ones
    pub fn normalize(self) -> Self {
        match self {
            Self::Question => Self::QueryMetric,
            Self::Request => Self::Configure,
            Self::Investigate => Self::Diagnose,
            other => other,
        }
    }
}

impl std::fmt::Display for QueryIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::QueryMetric => write!(f, "query_metric"),
            Self::Diagnose => write!(f, "diagnose"),
            Self::Configure => write!(f, "configure"),
            Self::List => write!(f, "list"),
            Self::CheckStatus => write!(f, "check_status"),
            Self::Explain => write!(f, "explain"),
            Self::Question => write!(f, "question"),
            Self::Request => write!(f, "request"),
            Self::Investigate => write!(f, "investigate"),
        }
    }
}

/// Translator ticket - structured output from LLM translator
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TranslatorTicket {
    /// Query intent classification
    #[serde(default)]
    pub intent: QueryIntent,
    /// Target specialist domain
    #[serde(default)]
    pub domain: SpecialistDomain,
    /// Extracted entities (processes, services, mounts, etc.)
    #[serde(default)]
    pub entities: Vec<String>,
    /// Probe IDs needed from allowlist
    #[serde(default)]
    pub needs_probes: Vec<String>,
    /// Clarification question if query is ambiguous
    #[serde(default)]
    pub clarification_question: Option<String>,
    /// Translator confidence 0.0-1.0
    #[serde(default)]
    pub confidence: f32,
    /// v0.0.74: Answer contract defining what the answer should contain
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer_contract: Option<crate::answer_contract::AnswerContract>,
}
