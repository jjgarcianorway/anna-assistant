// v0.0.548: Timeout Config (Phase 124)
// Configurable timeout settings per VISION.md

use serde::{Deserialize, Serialize};

/// Timeout scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeoutScope {
    Command,
    LlmCall,
    Confirmation,
    Research,
    TotalTicket,
    IdleDetection,
}

impl std::fmt::Display for TimeoutScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Command => write!(f, "Command Execution"),
            Self::LlmCall => write!(f, "LLM Call"),
            Self::Confirmation => write!(f, "User Confirmation"),
            Self::Research => write!(f, "Research Phase"),
            Self::TotalTicket => write!(f, "Total Ticket"),
            Self::IdleDetection => write!(f, "Idle Detection"),
        }
    }
}

/// Timeout action when exceeded
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum TimeoutAction {
    #[default]
    Cancel,
    Warn,
    Extend,
    Escalate,
    Background,
}

impl std::fmt::Display for TimeoutAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cancel => write!(f, "Cancel"),
            Self::Warn => write!(f, "Warn and continue"),
            Self::Extend => write!(f, "Auto-extend"),
            Self::Escalate => write!(f, "Escalate"),
            Self::Background => write!(f, "Move to background"),
        }
    }
}

/// Timeout profile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum TimeoutProfile {
    Fast,
    #[default]
    Normal,
    Patient,
    Unlimited,
}

impl std::fmt::Display for TimeoutProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fast => write!(f, "Fast (strict timeouts)"),
            Self::Normal => write!(f, "Normal"),
            Self::Patient => write!(f, "Patient (extended timeouts)"),
            Self::Unlimited => write!(f, "Unlimited (no timeouts)"),
        }
    }
}

/// Timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    pub profile: TimeoutProfile,
    pub command_timeout_ms: u64,
    pub llm_timeout_ms: u64,
    pub confirmation_timeout_ms: u64,
    pub research_timeout_ms: u64,
    pub total_ticket_timeout_ms: u64,
    pub idle_detection_ms: u64,
    pub action_on_command_timeout: TimeoutAction,
    pub action_on_llm_timeout: TimeoutAction,
    pub show_countdown: bool,
    pub auto_extend_on_activity: bool,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            profile: TimeoutProfile::Normal,
            command_timeout_ms: 30_000,
            llm_timeout_ms: 60_000,
            confirmation_timeout_ms: 30_000,
            research_timeout_ms: 120_000,
            total_ticket_timeout_ms: 300_000,
            idle_detection_ms: 300_000,
            action_on_command_timeout: TimeoutAction::Cancel,
            action_on_llm_timeout: TimeoutAction::Escalate,
            show_countdown: true,
            auto_extend_on_activity: true,
        }
    }
}

impl TimeoutConfig {
    /// Create new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Fast profile - strict timeouts
    pub fn fast() -> Self {
        Self {
            profile: TimeoutProfile::Fast,
            command_timeout_ms: 10_000,
            llm_timeout_ms: 20_000,
            confirmation_timeout_ms: 10_000,
            research_timeout_ms: 30_000,
            total_ticket_timeout_ms: 60_000,
            idle_detection_ms: 60_000,
            action_on_command_timeout: TimeoutAction::Cancel,
            action_on_llm_timeout: TimeoutAction::Cancel,
            show_countdown: true,
            auto_extend_on_activity: false,
        }
    }

    /// Patient profile - extended timeouts
    pub fn patient() -> Self {
        Self {
            profile: TimeoutProfile::Patient,
            command_timeout_ms: 120_000,
            llm_timeout_ms: 180_000,
            confirmation_timeout_ms: 120_000,
            research_timeout_ms: 600_000,
            total_ticket_timeout_ms: 1_800_000,
            idle_detection_ms: 600_000,
            action_on_command_timeout: TimeoutAction::Warn,
            action_on_llm_timeout: TimeoutAction::Extend,
            show_countdown: false,
            auto_extend_on_activity: true,
        }
    }

    /// Unlimited profile - no timeouts
    pub fn unlimited() -> Self {
        Self {
            profile: TimeoutProfile::Unlimited,
            command_timeout_ms: u64::MAX,
            llm_timeout_ms: u64::MAX,
            confirmation_timeout_ms: u64::MAX,
            research_timeout_ms: u64::MAX,
            total_ticket_timeout_ms: u64::MAX,
            idle_detection_ms: u64::MAX,
            action_on_command_timeout: TimeoutAction::Warn,
            action_on_llm_timeout: TimeoutAction::Warn,
            show_countdown: false,
            auto_extend_on_activity: true,
        }
    }

    /// Get timeout for scope in ms
    pub fn timeout_for(&self, scope: TimeoutScope) -> u64 {
        match scope {
            TimeoutScope::Command => self.command_timeout_ms,
            TimeoutScope::LlmCall => self.llm_timeout_ms,
            TimeoutScope::Confirmation => self.confirmation_timeout_ms,
            TimeoutScope::Research => self.research_timeout_ms,
            TimeoutScope::TotalTicket => self.total_ticket_timeout_ms,
            TimeoutScope::IdleDetection => self.idle_detection_ms,
        }
    }

    /// Get timeout in seconds for scope
    pub fn timeout_seconds(&self, scope: TimeoutScope) -> u64 {
        self.timeout_for(scope) / 1000
    }

    /// Is timeout unlimited?
    pub fn is_unlimited(&self) -> bool {
        self.profile == TimeoutProfile::Unlimited
    }

    /// Is fast mode?
    pub fn is_fast(&self) -> bool {
        self.profile == TimeoutProfile::Fast
    }

    /// Should show countdown?
    pub fn should_show_countdown(&self) -> bool {
        self.show_countdown && !self.is_unlimited()
    }

    /// Apply natural language change
    pub fn apply_change(&mut self, request: &str) -> Option<String> {
        let lower = request.to_lowercase();

        // Profile changes
        if lower.contains("fast timeout") || lower.contains("strict timeout") || lower.contains("quick") {
            *self = Self::fast();
            return Some("Fast timeout profile - strict time limits.".to_string());
        }
        if lower.contains("patient") || lower.contains("take your time") || lower.contains("no rush") {
            *self = Self::patient();
            return Some("Patient timeout profile - extended time limits.".to_string());
        }
        if lower.contains("unlimited") || lower.contains("no timeout") || lower.contains("wait forever") {
            *self = Self::unlimited();
            return Some("Unlimited profile - no timeouts.".to_string());
        }
        if lower.contains("normal timeout") || lower.contains("default timeout") {
            *self = Self::default();
            return Some("Normal timeout profile restored.".to_string());
        }

        // Individual toggles
        if lower.contains("show countdown") || lower.contains("show timer") {
            self.show_countdown = true;
            return Some("Countdown timer will be shown.".to_string());
        }
        if lower.contains("hide countdown") || lower.contains("no countdown") {
            self.show_countdown = false;
            return Some("Countdown timer hidden.".to_string());
        }
        if lower.contains("auto extend") || lower.contains("extend on activity") {
            self.auto_extend_on_activity = true;
            return Some("Timeouts will auto-extend on activity.".to_string());
        }
        if lower.contains("no auto extend") || lower.contains("strict timer") {
            self.auto_extend_on_activity = false;
            return Some("Strict timeouts - no auto-extension.".to_string());
        }

        None
    }
}

/// Format timeout config
pub fn format_timeout_config(config: &TimeoutConfig) -> String {
    let mut output = String::new();
    output.push_str("=== Timeout Configuration ===\n\n");

    output.push_str(&format!("Profile: {}\n", config.profile));
    output.push_str(&format!(
        "Command Timeout: {}s\n",
        config.command_timeout_ms / 1000
    ));
    output.push_str(&format!(
        "LLM Timeout: {}s\n",
        config.llm_timeout_ms / 1000
    ));
    output.push_str(&format!(
        "Confirmation Timeout: {}s\n",
        config.confirmation_timeout_ms / 1000
    ));
    output.push_str(&format!(
        "Research Timeout: {}s\n",
        config.research_timeout_ms / 1000
    ));
    output.push_str(&format!(
        "Total Ticket Timeout: {}s\n",
        config.total_ticket_timeout_ms / 1000
    ));
    output.push_str(&format!(
        "Action on Command Timeout: {}\n",
        config.action_on_command_timeout
    ));
    output.push_str(&format!(
        "Action on LLM Timeout: {}\n",
        config.action_on_llm_timeout
    ));
    output.push_str(&format!("Show Countdown: {}\n", config.show_countdown));
    output.push_str(&format!(
        "Auto-Extend on Activity: {}\n",
        config.auto_extend_on_activity
    ));

    output
}

/// Check if query is timeout-related
pub fn is_timeout_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("timeout")
        || lower.contains("time limit")
        || lower.contains("how long")
        || lower.contains("wait time")
}

/// Fun fact about timeouts
pub fn timeout_fun_fact() -> &'static str {
    "The TCP timeout was originally set to 2 minutes in the 1980s - some things never change!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_display() {
        assert_eq!(format!("{}", TimeoutScope::Command), "Command Execution");
        assert_eq!(format!("{}", TimeoutScope::LlmCall), "LLM Call");
    }

    #[test]
    fn test_default_config() {
        let config = TimeoutConfig::default();
        assert_eq!(config.profile, TimeoutProfile::Normal);
        assert_eq!(config.command_timeout_ms, 30_000);
    }

    #[test]
    fn test_fast_preset() {
        let config = TimeoutConfig::fast();
        assert_eq!(config.profile, TimeoutProfile::Fast);
        assert_eq!(config.command_timeout_ms, 10_000);
    }

    #[test]
    fn test_patient_preset() {
        let config = TimeoutConfig::patient();
        assert_eq!(config.profile, TimeoutProfile::Patient);
        assert_eq!(config.command_timeout_ms, 120_000);
    }

    #[test]
    fn test_unlimited_preset() {
        let config = TimeoutConfig::unlimited();
        assert!(config.is_unlimited());
        assert_eq!(config.command_timeout_ms, u64::MAX);
    }

    #[test]
    fn test_timeout_for_scope() {
        let config = TimeoutConfig::default();
        assert_eq!(config.timeout_for(TimeoutScope::Command), 30_000);
        assert_eq!(config.timeout_seconds(TimeoutScope::Command), 30);
    }

    #[test]
    fn test_should_show_countdown() {
        let config = TimeoutConfig::default();
        assert!(config.should_show_countdown());
        let unlimited = TimeoutConfig::unlimited();
        assert!(!unlimited.should_show_countdown());
    }

    #[test]
    fn test_apply_fast() {
        let mut config = TimeoutConfig::default();
        let result = config.apply_change("use fast timeouts");
        assert!(result.is_some());
        assert!(config.is_fast());
    }

    #[test]
    fn test_apply_countdown() {
        let mut config = TimeoutConfig::default();
        config.apply_change("hide countdown");
        assert!(!config.show_countdown);
    }

    #[test]
    fn test_is_timeout_query() {
        assert!(is_timeout_query("Change timeout settings"));
        assert!(is_timeout_query("What's the time limit?"));
        assert!(!is_timeout_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = timeout_fun_fact();
        assert!(fact.contains("TCP"));
    }
}
