// v0.0.545: Escalation Policy Config - Configuration (Phase 121)
// Main configuration struct and policy presets

use serde::{Deserialize, Serialize};

use super::types::{EscalationMode, EscalationNotify, EscalationPriority, EscalationTrigger};

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
