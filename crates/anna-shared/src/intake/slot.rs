//! Clarification slot types (v0.0.180).

use serde::{Deserialize, Serialize};

use crate::facts::FactKey;

use super::question::ClarificationQuestion;
use super::verify_plan::VerifyPlan;

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
