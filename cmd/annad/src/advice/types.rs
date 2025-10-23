use crate::persona::types::Persona;
use serde::{Deserialize, Serialize};

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
    pub plan: AdvicePlan,
}
