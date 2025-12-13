// v0.0.545: Escalation Policy Config (Phase 121)
// Configurable escalation policy per VISION.md - when to escalate junior to senior

use serde::{Deserialize, Serialize};

/// Escalation trigger condition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EscalationTrigger {
    LowConfidence,
    HighRisk,
    SecurityRelated,
    MultiDepartment,
    UserRequest,
    TimeoutExceeded,
    RepeatedFailure,
    ComplexQuery,
}

impl std::fmt::Display for EscalationTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LowConfidence => write!(f, "Low Confidence"),
            Self::HighRisk => write!(f, "High Risk"),
            Self::SecurityRelated => write!(f, "Security Related"),
            Self::MultiDepartment => write!(f, "Multi-Department"),
            Self::UserRequest => write!(f, "User Request"),
            Self::TimeoutExceeded => write!(f, "Timeout Exceeded"),
            Self::RepeatedFailure => write!(f, "Repeated Failure"),
            Self::ComplexQuery => write!(f, "Complex Query"),
        }
    }
}

/// Escalation priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum EscalationPriority {
    #[default]
    Normal,
    High,
    Critical,
    Immediate,
}

impl std::fmt::Display for EscalationPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "Normal"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
            Self::Immediate => write!(f, "Immediate"),
        }
    }
}

/// Escalation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum EscalationMode {
    #[default]
    Automatic,
    SemiAutomatic,
    Manual,
    Disabled,
}

impl std::fmt::Display for EscalationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Automatic => write!(f, "Automatic"),
            Self::SemiAutomatic => write!(f, "Semi-Automatic (ask first)"),
            Self::Manual => write!(f, "Manual (user decides)"),
            Self::Disabled => write!(f, "Disabled"),
        }
    }
}

/// Notification preference for escalations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum EscalationNotify {
    #[default]
    Always,
    OnlyHighPriority,
    OnlyImmediate,
    Never,
}

impl std::fmt::Display for EscalationNotify {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Always => write!(f, "Always Notify"),
            Self::OnlyHighPriority => write!(f, "High Priority Only"),
            Self::OnlyImmediate => write!(f, "Immediate Only"),
            Self::Never => write!(f, "Never Notify"),
        }
    }
}

/// Escalation policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationPolicyConfig {
    pub mode: EscalationMode,
    pub notify: EscalationNotify,
    pub confidence_threshold: u8,
    pub timeout_seconds: u64,
    pub max_retries_before_escalate: u8,
    pub auto_escalate_security: bool,
    pub auto_escalate_high_risk: bool,
    pub show_escalation_reason: bool,
    pub allow_user_override: bool,
}

impl Default for EscalationPolicyConfig {
    fn default() -> Self {
        Self {
            mode: EscalationMode::Automatic,
            notify: EscalationNotify::Always,
            confidence_threshold: 70,
            timeout_seconds: 30,
            max_retries_before_escalate: 2,
            auto_escalate_security: true,
            auto_escalate_high_risk: true,
            show_escalation_reason: true,
            allow_user_override: true,
        }
    }
}

impl EscalationPolicyConfig {
    /// Create new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Lenient policy - less escalations
    pub fn lenient() -> Self {
        Self {
            mode: EscalationMode::SemiAutomatic,
            notify: EscalationNotify::OnlyHighPriority,
            confidence_threshold: 50,
            timeout_seconds: 60,
            max_retries_before_escalate: 3,
            auto_escalate_security: true,
            auto_escalate_high_risk: false,
            show_escalation_reason: true,
            allow_user_override: true,
        }
    }

    /// Strict policy - more escalations
    pub fn strict() -> Self {
        Self {
            mode: EscalationMode::Automatic,
            notify: EscalationNotify::Always,
            confidence_threshold: 85,
            timeout_seconds: 20,
            max_retries_before_escalate: 1,
            auto_escalate_security: true,
            auto_escalate_high_risk: true,
            show_escalation_reason: true,
            allow_user_override: false,
        }
    }

    /// Manual policy - user controls everything
    pub fn manual() -> Self {
        Self {
            mode: EscalationMode::Manual,
            notify: EscalationNotify::Always,
            confidence_threshold: 70,
            timeout_seconds: 60,
            max_retries_before_escalate: 5,
            auto_escalate_security: false,
            auto_escalate_high_risk: false,
            show_escalation_reason: true,
            allow_user_override: true,
        }
    }

    /// Should escalate for this confidence?
    pub fn should_escalate_confidence(&self, confidence: u8) -> bool {
        if self.mode == EscalationMode::Disabled {
            return false;
        }
        confidence < self.confidence_threshold
    }

    /// Should escalate for security issue?
    pub fn should_escalate_security(&self) -> bool {
        if self.mode == EscalationMode::Disabled {
            return false;
        }
        self.auto_escalate_security
    }

    /// Should escalate for high risk?
    pub fn should_escalate_high_risk(&self) -> bool {
        if self.mode == EscalationMode::Disabled {
            return false;
        }
        self.auto_escalate_high_risk
    }

    /// Should ask user before escalating?
    pub fn needs_user_confirmation(&self) -> bool {
        matches!(
            self.mode,
            EscalationMode::SemiAutomatic | EscalationMode::Manual
        )
    }

    /// Is automatic escalation enabled?
    pub fn is_automatic(&self) -> bool {
        self.mode == EscalationMode::Automatic
    }

    /// Get priority for trigger
    pub fn priority_for(&self, trigger: EscalationTrigger) -> EscalationPriority {
        match trigger {
            EscalationTrigger::SecurityRelated => EscalationPriority::Critical,
            EscalationTrigger::HighRisk => EscalationPriority::High,
            EscalationTrigger::UserRequest => EscalationPriority::Immediate,
            EscalationTrigger::TimeoutExceeded => EscalationPriority::High,
            _ => EscalationPriority::Normal,
        }
    }

    /// Should notify for priority?
    pub fn should_notify(&self, priority: EscalationPriority) -> bool {
        match self.notify {
            EscalationNotify::Always => true,
            EscalationNotify::Never => false,
            EscalationNotify::OnlyHighPriority => {
                matches!(
                    priority,
                    EscalationPriority::High
                        | EscalationPriority::Critical
                        | EscalationPriority::Immediate
                )
            }
            EscalationNotify::OnlyImmediate => priority == EscalationPriority::Immediate,
        }
    }

    /// Apply natural language change
    pub fn apply_change(&mut self, request: &str) -> Option<String> {
        let lower = request.to_lowercase();

        // Mode changes
        if lower.contains("automatic escalat") || lower.contains("auto escalate") {
            self.mode = EscalationMode::Automatic;
            return Some("Escalation mode set to automatic.".to_string());
        }
        if lower.contains("ask before escalat") || lower.contains("semi automatic") {
            self.mode = EscalationMode::SemiAutomatic;
            return Some("Escalation mode set to semi-automatic (will ask before).".to_string());
        }
        if lower.contains("manual escalat") || lower.contains("i decide when to escalate") {
            self.mode = EscalationMode::Manual;
            return Some("Escalation mode set to manual (you decide).".to_string());
        }
        if lower.contains("disable escalat") || lower.contains("no escalation") {
            self.mode = EscalationMode::Disabled;
            return Some("Escalations disabled.".to_string());
        }

        // Policy presets
        if lower.contains("lenient") || lower.contains("less escalat") {
            *self = Self::lenient();
            return Some("Lenient escalation policy applied.".to_string());
        }
        if lower.contains("strict") || lower.contains("more escalat") {
            *self = Self::strict();
            return Some("Strict escalation policy applied.".to_string());
        }

        // Security/risk toggles
        if lower.contains("always escalate security") {
            self.auto_escalate_security = true;
            return Some("Security issues will always escalate.".to_string());
        }
        if lower.contains("don't escalate security") {
            self.auto_escalate_security = false;
            return Some("Security auto-escalation disabled.".to_string());
        }
        if lower.contains("always escalate high risk") {
            self.auto_escalate_high_risk = true;
            return Some("High risk issues will always escalate.".to_string());
        }
        if lower.contains("don't escalate high risk") {
            self.auto_escalate_high_risk = false;
            return Some("High risk auto-escalation disabled.".to_string());
        }

        // Notification changes
        if lower.contains("always notify") || lower.contains("notify me") {
            self.notify = EscalationNotify::Always;
            return Some("You'll be notified of all escalations.".to_string());
        }
        if lower.contains("quiet escalat") || lower.contains("don't notify") {
            self.notify = EscalationNotify::Never;
            return Some("Escalation notifications disabled.".to_string());
        }

        None
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_display() {
        assert_eq!(
            format!("{}", EscalationTrigger::LowConfidence),
            "Low Confidence"
        );
        assert_eq!(
            format!("{}", EscalationTrigger::SecurityRelated),
            "Security Related"
        );
    }

    #[test]
    fn test_default_config() {
        let config = EscalationPolicyConfig::default();
        assert_eq!(config.mode, EscalationMode::Automatic);
        assert_eq!(config.confidence_threshold, 70);
    }

    #[test]
    fn test_lenient_preset() {
        let config = EscalationPolicyConfig::lenient();
        assert_eq!(config.mode, EscalationMode::SemiAutomatic);
        assert_eq!(config.confidence_threshold, 50);
        assert!(!config.auto_escalate_high_risk);
    }

    #[test]
    fn test_strict_preset() {
        let config = EscalationPolicyConfig::strict();
        assert_eq!(config.confidence_threshold, 85);
        assert!(config.auto_escalate_high_risk);
    }

    #[test]
    fn test_should_escalate_confidence() {
        let config = EscalationPolicyConfig::default();
        assert!(config.should_escalate_confidence(50));
        assert!(!config.should_escalate_confidence(80));
    }

    #[test]
    fn test_disabled_mode() {
        let mut config = EscalationPolicyConfig::default();
        config.mode = EscalationMode::Disabled;
        assert!(!config.should_escalate_confidence(10));
        assert!(!config.should_escalate_security());
    }

    #[test]
    fn test_priority_for_trigger() {
        let config = EscalationPolicyConfig::default();
        assert_eq!(
            config.priority_for(EscalationTrigger::SecurityRelated),
            EscalationPriority::Critical
        );
        assert_eq!(
            config.priority_for(EscalationTrigger::LowConfidence),
            EscalationPriority::Normal
        );
    }

    #[test]
    fn test_should_notify() {
        let mut config = EscalationPolicyConfig::default();
        assert!(config.should_notify(EscalationPriority::Normal));

        config.notify = EscalationNotify::OnlyHighPriority;
        assert!(!config.should_notify(EscalationPriority::Normal));
        assert!(config.should_notify(EscalationPriority::High));
    }

    #[test]
    fn test_apply_automatic() {
        let mut config = EscalationPolicyConfig::manual();
        let result = config.apply_change("use automatic escalation");
        assert!(result.is_some());
        assert_eq!(config.mode, EscalationMode::Automatic);
    }

    #[test]
    fn test_apply_strict() {
        let mut config = EscalationPolicyConfig::default();
        config.apply_change("use strict escalation policy");
        assert_eq!(config.confidence_threshold, 85);
    }

    #[test]
    fn test_is_policy_query() {
        assert!(is_escalation_policy_query("Show escalation policy"));
        assert!(is_escalation_policy_query("When to escalate?"));
        assert!(!is_escalation_policy_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = escalation_policy_fun_fact();
        assert!(fact.contains("30%"));
    }
}
