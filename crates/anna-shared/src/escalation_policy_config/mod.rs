// v0.0.545: Escalation Policy Config (Phase 121)
// Configurable escalation policy per VISION.md - when to escalate junior to senior

mod config;
mod formatting;
#[cfg(test)]
mod tests;
mod types;

// Re-export all public types to preserve the original API
pub use config::EscalationPolicyConfig;
pub use formatting::{
    escalation_policy_fun_fact, format_escalation_policy, is_escalation_policy_query,
};
pub use types::{EscalationMode, EscalationNotify, EscalationPriority, EscalationTrigger};
