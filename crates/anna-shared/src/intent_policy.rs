//! Intent Policy Enforcement (v0.0.414).
//!
//! Enforces Anna's core design principle: NO HARDCODED NATURAL LANGUAGE CASES.
//!
//! FORBIDDEN:
//! - if user_question.contains("why is my fan loud") { do_fan_diagnostic() }
//! - match user_question { "wifi keeps disconnecting" => ... }
//!
//! ALLOWED:
//! - Route by intent: diagnose_boot_time, check_failed_services
//! - Generic recipes keyed by intent, not by specific phrasing
//! - Probe-based evidence gathering
//! - Doc-first reasoning
//!
//! This module provides validation to catch policy violations at compile/test time.

use serde::{Deserialize, Serialize};

/// Allowed intent categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntentCategory {
    // Diagnostic intents
    DiagnoseBootTime,
    DiagnosePerformance,
    DiagnoseNetworkConnectivity,
    DiagnoseAudioIssue,
    DiagnoseServiceFailure,
    DiagnoseStorageIssue,
    DiagnoseDisplayIssue,

    // Inspection intents
    InspectSystemHealth,
    InspectDiskUsage,
    InspectMemoryUsage,
    InspectNetworkStatus,
    InspectServiceStatus,
    InspectPackages,
    InspectProcesses,

    // Configuration intents
    ConfigureEditor,
    ConfigureShell,
    ConfigureDesktop,
    ConfigureNetwork,
    ConfigureAudio,
    ConfigureService,

    // Action intents
    InstallPackage,
    RemovePackage,
    UpdateSystem,
    EnableService,
    DisableService,
    RestartService,

    // Explanation intents
    ExplainCommand,
    ExplainConfig,
    ExplainError,
    ExplainConcept,

    // Generic fallback
    GeneralQuery,
}

impl IntentCategory {
    /// Map from domain + intent string to category
    pub fn from_domain_intent(domain: &str, intent: &str) -> Self {
        let domain_lower = domain.to_lowercase();
        let intent_lower = intent.to_lowercase();

        match (domain_lower.as_str(), intent_lower.as_str()) {
            // Boot domain
            ("boot", "diagnose") | ("boot", "debug") => Self::DiagnoseBootTime,
            ("boot", _) => Self::DiagnoseBootTime,

            // Performance domain
            ("performance", "diagnose") => Self::DiagnosePerformance,
            ("performance", "stats") | ("performance", "inspect") => Self::InspectProcesses,
            ("performance", _) => Self::DiagnosePerformance,

            // Network domain
            ("network", "diagnose") | ("network", "debug") => Self::DiagnoseNetworkConnectivity,
            ("network", "inspect") | ("network", "check") => Self::InspectNetworkStatus,
            ("network", "configure") | ("network", "setup") => Self::ConfigureNetwork,
            ("network", _) => Self::InspectNetworkStatus,

            // Audio domain
            ("audio", "diagnose") => Self::DiagnoseAudioIssue,
            ("audio", "configure") => Self::ConfigureAudio,
            ("audio", _) => Self::DiagnoseAudioIssue,

            // Services domain
            ("services", "diagnose") => Self::DiagnoseServiceFailure,
            ("services", "inspect") | ("services", "check") => Self::InspectServiceStatus,
            ("services", "enable") => Self::EnableService,
            ("services", "disable") => Self::DisableService,
            ("services", "restart") => Self::RestartService,
            ("services", "configure") => Self::ConfigureService,
            ("services", _) => Self::InspectServiceStatus,

            // Storage domain
            ("storage", "diagnose") => Self::DiagnoseStorageIssue,
            ("storage", "inspect") | ("storage", "check") => Self::InspectDiskUsage,
            ("storage", _) => Self::InspectDiskUsage,

            // Display domain
            ("display", "diagnose") => Self::DiagnoseDisplayIssue,
            ("display", "configure") => Self::ConfigureDesktop,
            ("display", _) => Self::DiagnoseDisplayIssue,

            // Desktop domain
            ("desktop", "configure") => Self::ConfigureDesktop,
            ("desktop", _) => Self::ConfigureDesktop,

            // Packages domain
            ("packages", "install") => Self::InstallPackage,
            ("packages", "remove") => Self::RemovePackage,
            ("packages", "update") => Self::UpdateSystem,
            ("packages", "inspect") | ("packages", "check") => Self::InspectPackages,
            ("packages", _) => Self::InspectPackages,

            // System domain
            ("system", "diagnose") => Self::InspectSystemHealth,
            ("system", "inspect") | ("system", "check") => Self::InspectSystemHealth,
            ("system", "update") => Self::UpdateSystem,
            ("system", _) => Self::InspectSystemHealth,

            // Generic
            (_, "explain") => Self::ExplainConcept,
            _ => Self::GeneralQuery,
        }
    }

    /// Get the probes recommended for this intent
    pub fn recommended_probes(&self) -> Vec<&'static str> {
        match self {
            Self::DiagnoseBootTime => vec!["systemd_analyze", "systemd_blame", "journal_boot"],
            Self::DiagnosePerformance => vec!["top", "ps_mem", "cpu_usage", "load_avg"],
            Self::DiagnoseNetworkConnectivity => vec!["ip_addr", "ip_route", "ping", "dns_resolve"],
            Self::DiagnoseAudioIssue => vec!["pactl_info", "wpctl_status", "aplay_list"],
            Self::DiagnoseServiceFailure => vec!["systemctl_failed", "journal_errors"],
            Self::DiagnoseStorageIssue => vec!["df_root", "lsblk", "mount_info"],
            Self::DiagnoseDisplayIssue => vec!["xrandr", "wayland_info"],

            Self::InspectSystemHealth => vec!["uptime", "df_root", "free_mem", "systemctl_failed"],
            Self::InspectDiskUsage => vec!["df_all", "lsblk", "du_home"],
            Self::InspectMemoryUsage => vec!["free_mem", "meminfo", "ps_mem"],
            Self::InspectNetworkStatus => vec!["ip_addr", "ss_listen", "networkctl"],
            Self::InspectServiceStatus => vec!["systemctl_status", "systemctl_list"],
            Self::InspectPackages => vec!["pacman_count", "pacman_recent"],
            Self::InspectProcesses => vec!["ps_aux", "top_cpu", "top_mem"],

            Self::ConfigureEditor => vec!["which_editor", "config_check"],
            Self::ConfigureShell => vec!["shell_info", "shell_config"],
            Self::ConfigureDesktop => vec!["desktop_env", "wm_info"],
            Self::ConfigureNetwork => vec!["networkctl", "nmcli_status"],
            Self::ConfigureAudio => vec!["pactl_info", "audio_config"],
            Self::ConfigureService => vec!["systemctl_status", "service_config"],

            Self::InstallPackage => vec!["pacman_search"],
            Self::RemovePackage => vec!["pacman_query"],
            Self::UpdateSystem => vec!["checkupdates"],
            Self::EnableService | Self::DisableService | Self::RestartService => {
                vec!["systemctl_status"]
            }

            Self::ExplainCommand | Self::ExplainConfig | Self::ExplainError | Self::ExplainConcept => {
                vec![] // Explanation intents rely on docs, not probes
            }

            Self::GeneralQuery => vec![],
        }
    }

    /// Get the knowledge domains to search for this intent
    pub fn knowledge_domains(&self) -> Vec<&'static str> {
        match self {
            Self::DiagnoseBootTime => vec!["boot", "systemd"],
            Self::DiagnosePerformance => vec!["performance", "system"],
            Self::DiagnoseNetworkConnectivity => vec!["network"],
            Self::DiagnoseAudioIssue => vec!["audio"],
            Self::DiagnoseServiceFailure => vec!["services", "systemd"],
            Self::DiagnoseStorageIssue => vec!["storage"],
            Self::DiagnoseDisplayIssue => vec!["display", "desktop"],

            Self::InspectSystemHealth => vec!["system"],
            Self::InspectDiskUsage => vec!["storage"],
            Self::InspectMemoryUsage => vec!["performance", "system"],
            Self::InspectNetworkStatus => vec!["network"],
            Self::InspectServiceStatus => vec!["services"],
            Self::InspectPackages => vec!["packages"],
            Self::InspectProcesses => vec!["performance"],

            Self::ConfigureEditor => vec!["desktop"],
            Self::ConfigureShell => vec!["system"],
            Self::ConfigureDesktop => vec!["desktop"],
            Self::ConfigureNetwork => vec!["network"],
            Self::ConfigureAudio => vec!["audio"],
            Self::ConfigureService => vec!["services"],

            Self::InstallPackage | Self::RemovePackage | Self::UpdateSystem => vec!["packages"],
            Self::EnableService | Self::DisableService | Self::RestartService => vec!["services"],

            _ => vec!["system"],
        }
    }
}

impl std::fmt::Display for IntentCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::DiagnoseBootTime => "diagnose_boot_time",
            Self::DiagnosePerformance => "diagnose_performance",
            Self::DiagnoseNetworkConnectivity => "diagnose_network",
            Self::DiagnoseAudioIssue => "diagnose_audio",
            Self::DiagnoseServiceFailure => "diagnose_service_failure",
            Self::DiagnoseStorageIssue => "diagnose_storage",
            Self::DiagnoseDisplayIssue => "diagnose_display",
            Self::InspectSystemHealth => "inspect_system_health",
            Self::InspectDiskUsage => "inspect_disk_usage",
            Self::InspectMemoryUsage => "inspect_memory_usage",
            Self::InspectNetworkStatus => "inspect_network_status",
            Self::InspectServiceStatus => "inspect_service_status",
            Self::InspectPackages => "inspect_packages",
            Self::InspectProcesses => "inspect_processes",
            Self::ConfigureEditor => "configure_editor",
            Self::ConfigureShell => "configure_shell",
            Self::ConfigureDesktop => "configure_desktop",
            Self::ConfigureNetwork => "configure_network",
            Self::ConfigureAudio => "configure_audio",
            Self::ConfigureService => "configure_service",
            Self::InstallPackage => "install_package",
            Self::RemovePackage => "remove_package",
            Self::UpdateSystem => "update_system",
            Self::EnableService => "enable_service",
            Self::DisableService => "disable_service",
            Self::RestartService => "restart_service",
            Self::ExplainCommand => "explain_command",
            Self::ExplainConfig => "explain_config",
            Self::ExplainError => "explain_error",
            Self::ExplainConcept => "explain_concept",
            Self::GeneralQuery => "general_query",
        };
        write!(f, "{}", name)
    }
}

/// Policy violation type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyViolation {
    /// Recipe is keyed by specific natural language
    RecipeKeyedByQuestion(String),
    /// Hardcoded question-response mapping
    HardcodedQuestionMapping(String),
    /// Recipe too narrow (matches only one phrasing)
    TooNarrowRecipe(String),
    /// Missing intent classification
    MissingIntentClassification,
}

impl std::fmt::Display for PolicyViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RecipeKeyedByQuestion(q) => {
                write!(f, "Recipe keyed by specific question: '{}'", q)
            }
            Self::HardcodedQuestionMapping(q) => {
                write!(f, "Hardcoded question mapping: '{}'", q)
            }
            Self::TooNarrowRecipe(name) => {
                write!(f, "Recipe '{}' is too narrow (single phrasing)", name)
            }
            Self::MissingIntentClassification => {
                write!(f, "Missing intent classification")
            }
        }
    }
}

/// Validate a recipe doesn't violate policy
pub fn validate_recipe_policy(
    recipe_id: &str,
    trigger_patterns: &[String],
) -> Result<(), PolicyViolation> {
    // Check for patterns that are too specific (look like natural language questions)
    for pattern in trigger_patterns {
        let lower = pattern.to_lowercase();

        // Forbidden: Full natural language questions
        if lower.contains("why is my")
            || lower.contains("how do i")
            || lower.contains("what is the")
            || lower.contains("can you")
        {
            return Err(PolicyViolation::RecipeKeyedByQuestion(pattern.clone()));
        }

        // Forbidden: Too specific phrasings
        if lower.split_whitespace().count() > 6 {
            return Err(PolicyViolation::TooNarrowRecipe(recipe_id.to_string()));
        }
    }

    // Must have at least 2 trigger patterns (not single-phrasing)
    if trigger_patterns.len() < 2 {
        return Err(PolicyViolation::TooNarrowRecipe(recipe_id.to_string()));
    }

    Ok(())
}

/// Check if a string looks like a hardcoded question match
pub fn is_hardcoded_question_match(pattern: &str) -> bool {
    let lower = pattern.to_lowercase();

    // Full sentence patterns are forbidden
    let forbidden_starts = [
        "why is",
        "how do",
        "what is",
        "can you",
        "could you",
        "please",
        "i want",
        "i need",
        "my ",
    ];

    forbidden_starts.iter().any(|s| lower.starts_with(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_from_domain() {
        assert_eq!(
            IntentCategory::from_domain_intent("boot", "diagnose"),
            IntentCategory::DiagnoseBootTime
        );
        assert_eq!(
            IntentCategory::from_domain_intent("services", "inspect"),
            IntentCategory::InspectServiceStatus
        );
        assert_eq!(
            IntentCategory::from_domain_intent("packages", "install"),
            IntentCategory::InstallPackage
        );
    }

    #[test]
    fn test_recommended_probes() {
        let probes = IntentCategory::DiagnoseServiceFailure.recommended_probes();
        assert!(probes.contains(&"systemctl_failed"));
    }

    #[test]
    fn test_validate_recipe_policy_good() {
        let result = validate_recipe_policy(
            "check_service_status",
            &["service status".to_string(), "is service running".to_string()],
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_recipe_policy_bad_question() {
        let result = validate_recipe_policy(
            "bad_recipe",
            &["why is my fan so loud".to_string()],
        );
        assert!(matches!(result, Err(PolicyViolation::RecipeKeyedByQuestion(_))));
    }

    #[test]
    fn test_validate_recipe_policy_too_narrow() {
        let result = validate_recipe_policy(
            "narrow_recipe",
            &["single trigger".to_string()], // Only one pattern
        );
        assert!(matches!(result, Err(PolicyViolation::TooNarrowRecipe(_))));
    }

    #[test]
    fn test_is_hardcoded_question_match() {
        assert!(is_hardcoded_question_match("why is my wifi slow"));
        assert!(is_hardcoded_question_match("how do I install vim"));
        assert!(!is_hardcoded_question_match("service status"));
        assert!(!is_hardcoded_question_match("disk usage"));
    }
}
