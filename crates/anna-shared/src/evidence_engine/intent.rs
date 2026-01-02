//! Evidence intent classification

use serde::{Deserialize, Serialize};

/// Intent classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceIntent {
    /// User wants to understand something
    Diagnose,
    /// User wants an explanation
    Explain,
    /// User wants to change configuration
    Configure,
    /// User wants to inspect current state
    Inspect,
    /// User wants statistics/metrics
    Stats,
    /// User wants to fix a problem
    Fix,
}

impl EvidenceIntent {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "diagnose" | "debug" | "troubleshoot" => Some(Self::Diagnose),
            "explain" | "what" | "why" => Some(Self::Explain),
            "configure" | "setup" | "enable" | "disable" => Some(Self::Configure),
            "inspect" | "check" | "show" | "list" => Some(Self::Inspect),
            "stats" | "metrics" | "usage" | "count" => Some(Self::Stats),
            "fix" | "repair" | "solve" => Some(Self::Fix),
            _ => None,
        }
    }
}

impl std::fmt::Display for EvidenceIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Diagnose => write!(f, "diagnose"),
            Self::Explain => write!(f, "explain"),
            Self::Configure => write!(f, "configure"),
            Self::Inspect => write!(f, "inspect"),
            Self::Stats => write!(f, "stats"),
            Self::Fix => write!(f, "fix"),
        }
    }
}
