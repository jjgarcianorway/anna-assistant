//! Query Type Router (v0.0.481).
//!
//! Central hub for detecting query types across all display modules.
//! Consolidates is_*_query functions into a unified routing system.

use crate::capabilities_display::{is_capabilities_query, CapabilityCategory};
use crate::fun_stats_display::is_fun_stats_query;
use crate::session_display::is_session_query;
use crate::settings_display::is_all_settings_query;
use crate::xp_display::is_xp_query;

/// Types of informational queries Anna can answer directly
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryType {
    /// "what can you do?" - capabilities display
    Capabilities(Option<CapabilityCategory>),
    /// "show my stats", "fun facts" - fun statistics
    FunStats,
    /// "what is my XP", "my level" - XP/level display
    XpLevel,
    /// "show session", "what did we do" - session summary
    Session,
    /// "show settings", "my preferences" - settings display
    Settings,
    /// "status", "system status" - status display
    Status,
    /// "help" with context - contextual help
    ContextualHelp(String),
    /// Not an informational query - needs specialist processing
    Other,
}

impl QueryType {
    /// Detect query type from natural language
    pub fn detect(query: &str) -> Self {
        let lower = query.to_lowercase();

        // Check capabilities first (handles "help" too)
        if is_capabilities_query(query) {
            // Try to detect specific category
            let category = crate::capabilities_display::parse_capability_category(query);
            return Self::Capabilities(category);
        }

        // Check for XP/level queries
        if is_xp_query(query) {
            return Self::XpLevel;
        }

        // Check for fun stats queries
        if is_fun_stats_query(query) {
            return Self::FunStats;
        }

        // Check for session queries
        if is_session_query(query) {
            return Self::Session;
        }

        // Check for settings queries
        if is_all_settings_query(query) {
            return Self::Settings;
        }

        // Check for status queries
        if is_status_query(&lower) {
            return Self::Status;
        }

        // Check for contextual help ("help with X")
        if let Some(context) = extract_help_context(&lower) {
            return Self::ContextualHelp(context);
        }

        Self::Other
    }

    /// Whether this query type can be answered without specialists
    pub fn is_informational(&self) -> bool {
        !matches!(self, Self::Other | Self::ContextualHelp(_))
    }

    /// Get a description of this query type
    pub fn description(&self) -> &'static str {
        match self {
            Self::Capabilities(_) => "Capabilities display",
            Self::FunStats => "Fun statistics",
            Self::XpLevel => "XP and level display",
            Self::Session => "Session summary",
            Self::Settings => "Settings display",
            Self::Status => "System status",
            Self::ContextualHelp(_) => "Contextual help",
            Self::Other => "Specialist query",
        }
    }
}

/// Check if query is asking for status
fn is_status_query(query: &str) -> bool {
    let patterns = [
        "status",
        "system status",
        "anna status",
        "daemon status",
        "show status",
    ];

    for pattern in patterns {
        if query.contains(pattern) {
            return true;
        }
    }

    // Exact match for "status"
    query.trim() == "status"
}

/// Extract context from "help with X" type queries
fn extract_help_context(query: &str) -> Option<String> {
    // "help with X" patterns
    let prefixes = ["help with ", "help me with ", "i need help with "];

    for prefix in prefixes {
        if let Some(rest) = query.strip_prefix(prefix) {
            let context = rest.trim();
            if !context.is_empty() {
                return Some(context.to_string());
            }
        }
    }

    // "X help" patterns
    let suffixes = [" help", " assistance"];
    for suffix in suffixes {
        if query.ends_with(suffix) {
            let context = query.strip_suffix(suffix)?.trim();
            if !context.is_empty() && context != "need" && context != "some" {
                return Some(context.to_string());
            }
        }
    }

    None
}

/// Route a query and return what to display
pub fn route_query(query: &str) -> QueryType {
    QueryType::detect(query)
}

/// Check if query should skip specialist routing
pub fn should_handle_locally(query: &str) -> bool {
    let query_type = QueryType::detect(query);
    query_type.is_informational()
}

/// Get suggested display for a query type
pub fn suggest_display(query_type: &QueryType) -> &'static str {
    match query_type {
        QueryType::Capabilities(_) => "Use format_capabilities() or format_capability_category()",
        QueryType::FunStats => "Use format_fun_stats()",
        QueryType::XpLevel => "Use format_xp_display()",
        QueryType::Session => "Use format_current_session()",
        QueryType::Settings => "Use format_all_settings()",
        QueryType::Status => "Use daemon status display",
        QueryType::ContextualHelp(_) => "Route to appropriate team with help context",
        QueryType::Other => "Route to translator and specialists",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_capabilities() {
        assert!(matches!(
            QueryType::detect("what can you do?"),
            QueryType::Capabilities(_)
        ));
        assert!(matches!(
            QueryType::detect("help"),
            QueryType::Capabilities(_)
        ));
        assert!(matches!(
            QueryType::detect("show capabilities"),
            QueryType::Capabilities(_)
        ));
    }

    #[test]
    fn test_detect_xp() {
        assert_eq!(QueryType::detect("what is my xp?"), QueryType::XpLevel);
        assert_eq!(QueryType::detect("show my level"), QueryType::XpLevel);
    }

    #[test]
    fn test_detect_fun_stats() {
        assert_eq!(
            QueryType::detect("show me fun stats"),
            QueryType::FunStats
        );
        assert_eq!(
            QueryType::detect("any interesting facts?"),
            QueryType::FunStats
        );
    }

    #[test]
    fn test_detect_session() {
        assert_eq!(
            QueryType::detect("show session summary"),
            QueryType::Session
        );
        assert_eq!(
            QueryType::detect("what did we do today?"),
            QueryType::Session
        );
    }

    #[test]
    fn test_detect_settings() {
        assert_eq!(QueryType::detect("show all settings"), QueryType::Settings);
        assert_eq!(
            QueryType::detect("list all settings"),
            QueryType::Settings
        );
    }

    #[test]
    fn test_detect_status() {
        assert_eq!(QueryType::detect("status"), QueryType::Status);
        assert_eq!(QueryType::detect("anna status"), QueryType::Status);
        assert_eq!(QueryType::detect("system status"), QueryType::Status);
    }

    #[test]
    fn test_detect_contextual_help() {
        let result = QueryType::detect("help with vim");
        assert!(matches!(result, QueryType::ContextualHelp(ctx) if ctx == "vim"));

        let result2 = QueryType::detect("network help");
        assert!(matches!(result2, QueryType::ContextualHelp(ctx) if ctx == "network"));
    }

    #[test]
    fn test_detect_other() {
        assert_eq!(QueryType::detect("restart nginx"), QueryType::Other);
        assert_eq!(QueryType::detect("check disk space"), QueryType::Other);
        assert_eq!(QueryType::detect("install htop"), QueryType::Other);
    }

    #[test]
    fn test_is_informational() {
        assert!(QueryType::Capabilities(None).is_informational());
        assert!(QueryType::XpLevel.is_informational());
        assert!(QueryType::FunStats.is_informational());
        assert!(QueryType::Session.is_informational());
        assert!(QueryType::Settings.is_informational());
        assert!(QueryType::Status.is_informational());
        assert!(!QueryType::Other.is_informational());
        assert!(!QueryType::ContextualHelp("vim".to_string()).is_informational());
    }

    #[test]
    fn test_should_handle_locally() {
        assert!(should_handle_locally("what can you do?"));
        assert!(should_handle_locally("show my xp"));
        assert!(!should_handle_locally("restart docker"));
        assert!(!should_handle_locally("help with networking"));
    }

    #[test]
    fn test_extract_help_context() {
        assert_eq!(extract_help_context("help with vim"), Some("vim".to_string()));
        assert_eq!(
            extract_help_context("help me with docker"),
            Some("docker".to_string())
        );
        assert_eq!(
            extract_help_context("network help"),
            Some("network".to_string())
        );
        assert_eq!(extract_help_context("help"), None);
        assert_eq!(extract_help_context("need help"), None);
    }

    #[test]
    fn test_query_type_description() {
        assert_eq!(
            QueryType::Capabilities(None).description(),
            "Capabilities display"
        );
        assert_eq!(QueryType::Other.description(), "Specialist query");
    }

    #[test]
    fn test_suggest_display() {
        let suggestion = suggest_display(&QueryType::XpLevel);
        assert!(suggestion.contains("format_xp_display"));
    }
}
