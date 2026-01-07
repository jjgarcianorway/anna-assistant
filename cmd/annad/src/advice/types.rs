use crate::persona::types::Persona;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AdviceSeverity {
    #[default]
    Info,
    Warn,
    Action,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvicePlan {
    #[serde(default)]
    pub dry_run_cmds: Vec<String>,
    #[serde(default)]
    pub apply_cmds: Vec<String>,
    #[serde(default)]
    pub undo_cmds: Vec<String>,
}

impl AdvicePlan {
    pub fn dry_run_only(cmds: Vec<String>) -> Self {
        Self {
            dry_run_cmds: cmds,
            apply_cmds: Vec::new(),
            undo_cmds: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Advice {
    pub id: String,
    pub kind: String,
    pub persona_hint: Persona,
    pub reason: String,
    pub created_at: String,
    #[serde(default)]
    pub severity: AdviceSeverity,
    pub plan: AdvicePlan,
}

/// AdviceRecord is the serialized format (persona_hint as string)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdviceRecord {
    pub id: String,
    pub kind: String,
    pub persona_hint: String,
    pub reason: String,
    pub created_at: String,
    #[serde(default)]
    pub severity: AdviceSeverity,
    pub plan: AdvicePlan,
}
