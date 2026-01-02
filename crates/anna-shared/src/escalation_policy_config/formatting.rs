// v0.0.545: Escalation Policy Config - Formatting (Phase 121)
// Formatting and utility functions for escalation policy

use super::config::EscalationPolicyConfig;

/// Format escalation policy config
pub fn format_escalation_policy(config: &EscalationPolicyConfig) -> String {
    let mut output = String::new();
    output.push_str("=== Escalation Policy Configuration ===\n\n");

    output.push_str(&format!("Mode: {}\n", config.mode));
    output.push_str(&format!("Notifications: {}\n", config.notify));
    output.push_str(&format!(
        "Confidence Threshold: {}%\n",
        config.confidence_threshold
    ));
    output.push_str(&format!("Timeout: {}s\n", config.timeout_seconds));
    output.push_str(&format!(
        "Max Retries Before Escalate: {}\n",
        config.max_retries_before_escalate
    ));
    output.push_str(&format!(
        "Auto-Escalate Security: {}\n",
        config.auto_escalate_security
    ));
    output.push_str(&format!(
        "Auto-Escalate High Risk: {}\n",
        config.auto_escalate_high_risk
    ));
    output.push_str(&format!(
        "Show Escalation Reason: {}\n",
        config.show_escalation_reason
    ));
    output.push_str(&format!("Allow User Override: {}\n", config.allow_user_override));

    output
}

/// Check if query is escalation policy related
pub fn is_escalation_policy_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("escalation policy")
        || lower.contains("escalation mode")
        || lower.contains("when to escalate")
        || lower.contains("escalate setting")
}

/// Fun fact about escalation policy
pub fn escalation_policy_fun_fact() -> &'static str {
    "Teams with clear escalation policies resolve critical issues 30% faster than those without!"
}
