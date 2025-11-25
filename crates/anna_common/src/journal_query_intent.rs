//! Journal Query Intent Parser - Natural language to structured queries
//!
//! v6.51.0: Parse user questions about change history into structured filters

use crate::change_journal::{EpisodeDomain, EpisodeFilter, EpisodeRiskFilter, TimeWindow};

/// Parsed intent from a natural language journal query
#[derive(Debug, Clone, PartialEq)]
pub struct JournalQueryIntent {
    pub time_window: Option<TimeWindow>,
    pub domain_hint: Option<EpisodeDomain>,
    pub risk_filter: Option<EpisodeRiskFilter>,
    pub tag_hints: Vec<String>,
    pub want_details: bool,
    pub executed_only: bool,
    pub rolled_back_only: bool,
}

impl JournalQueryIntent {
    /// Convert intent into an EpisodeFilter
    pub fn to_filter(&self) -> EpisodeFilter {
        EpisodeFilter {
            time_window: self.time_window,
            domain: self.domain_hint,
            risk: self.risk_filter,
            tags: self.tag_hints.clone(),
            executed_only: self.executed_only,
            rolled_back_only: self.rolled_back_only,
        }
    }
}

/// Parse a natural language journal query
///
/// Examples:
/// - "what did you change recently?" → Last24h, Any
/// - "show me vim changes last week" → Last7d, Config, tags:["vim"]
/// - "what did you do to ssh?" → All, Network, tags:["ssh"]
/// - "show high risk changes this month" → Last30d, HighOnly
pub fn parse_journal_query(query: &str) -> Option<JournalQueryIntent> {
    let query_lower = query.to_lowercase();

    // Quick rejection: if it doesn't look like a journal query, bail
    if !is_journal_query(&query_lower) {
        return None;
    }

    let mut intent = JournalQueryIntent {
        time_window: None,
        domain_hint: None,
        risk_filter: None,
        tag_hints: Vec::new(),
        want_details: false,
        executed_only: false,
        rolled_back_only: false,
    };

    // Parse time window
    intent.time_window = parse_time_window(&query_lower);

    // Parse domain hints
    intent.domain_hint = parse_domain_hint(&query_lower);

    // Parse risk filter
    intent.risk_filter = parse_risk_filter(&query_lower);

    // Parse tag hints (specific tools/services mentioned)
    intent.tag_hints = parse_tag_hints(&query_lower);

    // Check if user wants detailed view
    intent.want_details = query_lower.contains("details")
        || query_lower.contains("full")
        || query_lower.contains("verbose")
        || query_lower.contains("show me everything");

    // Check for execution filters
    intent.executed_only = query_lower.contains("executed")
        || query_lower.contains("ran")
        || query_lower.contains("did");

    intent.rolled_back_only = query_lower.contains("rolled back")
        || query_lower.contains("reverted")
        || query_lower.contains("undone");

    Some(intent)
}

/// Check if query looks like a journal/history question
fn is_journal_query(query: &str) -> bool {
    // Journal queries ask about past actions
    query.contains("what did")
        || query.contains("did you")
        || query.contains("show")
        || query.contains("list")
        || query.contains("history")
        || query.contains("journal")
        || query.contains("changes")
        || query.contains("recent")
        || query.contains("what have you")
        || query.contains("what you")
        || query.contains("what packages")
        || query.contains("what services")
}

/// Parse time window from query
fn parse_time_window(query: &str) -> Option<TimeWindow> {
    if query.contains("today") || query.contains("last 24") || query.contains("past day") {
        Some(TimeWindow::Last24h)
    } else if query.contains("last week")
        || query.contains("past week")
        || query.contains("last 7 days")
    {
        Some(TimeWindow::Last7d)
    } else if query.contains("last month")
        || query.contains("past month")
        || query.contains("last 30 days")
        || query.contains("this month")
    {
        Some(TimeWindow::Last30d)
    } else if query.contains("recently") || query.contains("recent") {
        // "Recently" defaults to 7 days
        Some(TimeWindow::Last7d)
    } else if query.contains("all time") || query.contains("everything") {
        Some(TimeWindow::All)
    } else {
        // Default: no time filter (storage layer will handle default)
        None
    }
}

/// Parse domain hint from query
fn parse_domain_hint(query: &str) -> Option<EpisodeDomain> {
    // Config domain
    if query.contains("vim")
        || query.contains("config")
        || query.contains("editor")
        || query.contains("neovim")
        || query.contains("emacs")
    {
        return Some(EpisodeDomain::Config);
    }

    // Services domain
    if query.contains("service")
        || query.contains("systemctl")
        || query.contains("daemon")
        || query.contains("sshd")
    {
        return Some(EpisodeDomain::Services);
    }

    // Packages domain
    if query.contains("package")
        || query.contains("install")
        || query.contains("remove")
        || query.contains("pacman")
    {
        return Some(EpisodeDomain::Packages);
    }

    // Network domain
    if query.contains("network")
        || query.contains("ssh")
        || query.contains("firewall")
        || query.contains("dns")
    {
        return Some(EpisodeDomain::Network);
    }

    None
}

/// Parse risk filter from query
fn parse_risk_filter(query: &str) -> Option<EpisodeRiskFilter> {
    if query.contains("high risk") || query.contains("dangerous") {
        Some(EpisodeRiskFilter::HighOnly)
    } else if query.contains("risky") || query.contains("non-safe") {
        Some(EpisodeRiskFilter::NonSafe)
    } else {
        // Default: show all risk levels
        Some(EpisodeRiskFilter::Any)
    }
}

/// Parse specific tag hints from query
fn parse_tag_hints(query: &str) -> Vec<String> {
    let mut tags = Vec::new();

    // Common tools and services
    let keywords = [
        "vim", "neovim", "emacs", "ssh", "sshd", "systemctl", "pacman", "yay", "config",
        "network", "firewall", "nginx", "apache", "docker", "tlp", "grub", "kernel",
    ];

    for keyword in &keywords {
        if query.contains(keyword) {
            tags.push(keyword.to_string());
        }
    }

    tags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_recent_changes() {
        let intent = parse_journal_query("what did you change recently?").unwrap();
        assert_eq!(intent.time_window, Some(TimeWindow::Last7d));
        assert_eq!(intent.domain_hint, None);
    }

    #[test]
    fn test_parse_vim_changes() {
        let intent = parse_journal_query("show me vim changes last week").unwrap();
        assert_eq!(intent.time_window, Some(TimeWindow::Last7d));
        assert_eq!(intent.domain_hint, Some(EpisodeDomain::Config));
        assert!(intent.tag_hints.contains(&"vim".to_string()));
    }

    #[test]
    fn test_parse_ssh_changes() {
        let intent = parse_journal_query("what did you do to ssh?").unwrap();
        assert_eq!(intent.domain_hint, Some(EpisodeDomain::Network));
        assert!(intent.tag_hints.contains(&"ssh".to_string()));
    }

    #[test]
    fn test_parse_high_risk() {
        let intent = parse_journal_query("show high risk changes this month").unwrap();
        assert_eq!(intent.time_window, Some(TimeWindow::Last30d));
        assert_eq!(intent.risk_filter, Some(EpisodeRiskFilter::HighOnly));
    }

    #[test]
    fn test_parse_service_changes() {
        let intent = parse_journal_query("what services did you restart?").unwrap();
        assert_eq!(intent.domain_hint, Some(EpisodeDomain::Services));
        assert!(intent.executed_only);
    }

    #[test]
    fn test_parse_rolled_back() {
        let intent = parse_journal_query("show me what you rolled back").unwrap();
        assert!(intent.rolled_back_only);
    }

    #[test]
    fn test_parse_details_request() {
        let intent = parse_journal_query("show me details of recent changes").unwrap();
        assert!(intent.want_details);
        assert_eq!(intent.time_window, Some(TimeWindow::Last7d));
    }

    #[test]
    fn test_reject_non_journal_query() {
        let intent = parse_journal_query("how do I install vim?");
        assert!(intent.is_none());
    }

    #[test]
    fn test_parse_package_changes() {
        let intent = parse_journal_query("what packages did you install last month?").unwrap();
        assert_eq!(intent.time_window, Some(TimeWindow::Last30d));
        assert_eq!(intent.domain_hint, Some(EpisodeDomain::Packages));
        assert!(intent.executed_only);
    }

    #[test]
    fn test_intent_to_filter() {
        let intent = JournalQueryIntent {
            time_window: Some(TimeWindow::Last7d),
            domain_hint: Some(EpisodeDomain::Config),
            risk_filter: Some(EpisodeRiskFilter::HighOnly),
            tag_hints: vec!["vim".to_string()],
            want_details: false,
            executed_only: true,
            rolled_back_only: false,
        };

        let filter = intent.to_filter();
        assert_eq!(filter.time_window, Some(TimeWindow::Last7d));
        assert_eq!(filter.domain, Some(EpisodeDomain::Config));
        assert_eq!(filter.risk, Some(EpisodeRiskFilter::HighOnly));
        assert_eq!(filter.tags, vec!["vim".to_string()]);
        assert!(filter.executed_only);
        assert!(!filter.rolled_back_only);
    }
}
