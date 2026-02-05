//! Natural Language Query Detection.
//!
//! v0.3.120: Detect when users are asking about system health naturally.
//! Instead of requiring explicit commands like `annactl issues`, users
//! can just ask "how is my system?" or "any problems?".

use crate::dashboard::dashboard_summary;
use crate::health_report::health_summary;
use crate::live_state::LiveState;
use crate::proactive::{scan_for_issues, format_issues, IssueSeverity};

/// Types of natural queries we detect.
#[derive(Debug, Clone, PartialEq)]
pub enum NaturalQuery {
    /// User asking about overall system health
    SystemHealth,
    /// User asking about problems/issues
    Problems,
    /// User asking about resources (disk, memory, cpu)
    Resources,
    /// User asking about what needs attention
    NeedsAttention,
    /// User greeting/checking in
    Greeting,
    /// Not a system query - pass to LLM
    Other,
}

/// Detect if the question is a natural system query.
pub fn detect_natural_query(question: &str) -> NaturalQuery {
    let q = question.to_lowercase();

    // System health queries
    if matches_health_query(&q) {
        return NaturalQuery::SystemHealth;
    }

    // Problem/issue queries
    if matches_problem_query(&q) {
        return NaturalQuery::Problems;
    }

    // Resource queries
    if matches_resource_query(&q) {
        return NaturalQuery::Resources;
    }

    // Attention/todo queries
    if matches_attention_query(&q) {
        return NaturalQuery::NeedsAttention;
    }

    // Greeting/check-in
    if matches_greeting(&q) {
        return NaturalQuery::Greeting;
    }

    NaturalQuery::Other
}

/// Handle a natural query and return a response.
/// Returns None if this should be passed to the LLM.
pub fn handle_natural_query(question: &str) -> Option<String> {
    let query_type = detect_natural_query(question);

    match query_type {
        NaturalQuery::SystemHealth => Some(generate_health_response()),
        NaturalQuery::Problems => Some(generate_problems_response()),
        NaturalQuery::Resources => Some(generate_resources_response()),
        NaturalQuery::NeedsAttention => Some(generate_attention_response()),
        NaturalQuery::Greeting => Some(generate_greeting_response()),
        NaturalQuery::Other => None,
    }
}

// === Pattern matchers ===

fn matches_health_query(q: &str) -> bool {
    let patterns = [
        "how is my system",
        "how's my system",
        "how is the system",
        "how's the system",
        "system status",
        "system health",
        "how am i doing",
        "how are things",
        "how's everything",
        "status report",
        "give me a status",
        "what's the status",
        "what is the status",
    ];
    patterns.iter().any(|p| q.contains(p))
}

fn matches_problem_query(q: &str) -> bool {
    let patterns = [
        "any problems",
        "any issues",
        "anything wrong",
        "is something wrong",
        "is anything broken",
        "what's wrong",
        "what is wrong",
        "any errors",
        "any warnings",
        "check for problems",
        "scan for issues",
        "diagnose",
    ];
    patterns.iter().any(|p| q.contains(p))
}

fn matches_resource_query(q: &str) -> bool {
    let patterns = [
        "disk space",
        "disk usage",
        "how much disk",
        "how much space",
        "memory usage",
        "ram usage",
        "how much memory",
        "how much ram",
        "cpu usage",
        "cpu load",
        "system load",
        "resources",
    ];
    patterns.iter().any(|p| q.contains(p))
}

fn matches_attention_query(q: &str) -> bool {
    let patterns = [
        "anything i should know",
        "anything to worry about",
        "anything needing attention",
        "what needs attention",
        "what should i do",
        "any recommendations",
        "any suggestions",
        "what do you recommend",
        "heads up",
    ];
    patterns.iter().any(|p| q.contains(p))
}

fn matches_greeting(q: &str) -> bool {
    let q_trimmed = q.trim();
    let patterns = [
        "hi", "hello", "hey", "good morning", "good afternoon",
        "good evening", "what's up", "sup", "yo",
    ];
    // Only match if it's just the greeting (short message)
    q_trimmed.len() < 20 && patterns.iter().any(|p| q_trimmed.starts_with(p))
}

// === Response generators ===

fn generate_health_response() -> String {
    let state = LiveState::capture();
    let issues = scan_for_issues();

    let critical = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Critical)).count();
    let warnings = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Warning)).count();

    let mut response = String::new();

    // Overall status
    if critical > 0 {
        response.push_str("Your system needs attention.\n\n");
    } else if warnings > 0 {
        response.push_str("Your system is mostly healthy with a few things to check.\n\n");
    } else if !state.failed_units.is_empty() {
        response.push_str("Your system is running but some services have issues.\n\n");
    } else {
        response.push_str("Your system is healthy.\n\n");
    }

    // Quick stats
    let cpu_pct = (state.load_avg.0 / num_cpus() as f32 * 100.0).min(100.0);
    response.push_str(&format!(
        "CPU: {:.0}% | Memory: {:.0}% | Disk: {:.0}%\n",
        cpu_pct,
        state.memory.percent_used(),
        state.disk.percent_used()
    ));

    // Uptime
    if state.uptime_hours > 24.0 {
        response.push_str(&format!("Uptime: {:.0} days\n", state.uptime_hours / 24.0));
    } else {
        response.push_str(&format!("Uptime: {:.1} hours\n", state.uptime_hours));
    }

    // Issues summary
    if critical > 0 || warnings > 0 {
        response.push_str(&format!("\n{} critical, {} warnings found.\n", critical, warnings));

        // Show top issues
        for issue in issues.iter().filter(|i| !matches!(i.severity, IssueSeverity::Info)).take(3) {
            let prefix = match issue.severity {
                IssueSeverity::Critical => "[!]",
                IssueSeverity::Warning => "[?]",
                IssueSeverity::Info => "[i]",
            };
            response.push_str(&format!("{} {}\n", prefix, issue.title));
        }
    }

    // Failed services
    if !state.failed_units.is_empty() {
        response.push_str(&format!("\n{} failed service(s): ", state.failed_units.len()));
        response.push_str(&state.failed_units.iter().take(3).cloned().collect::<Vec<_>>().join(", "));
        if state.failed_units.len() > 3 {
            response.push_str(&format!(" (+{})", state.failed_units.len() - 3));
        }
        response.push('\n');
    }

    response
}

fn generate_problems_response() -> String {
    let issues = scan_for_issues();

    if issues.is_empty() {
        return "No problems detected. Your system looks good!".to_string();
    }

    let critical: Vec<_> = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Critical)).collect();
    let warnings: Vec<_> = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Warning)).collect();
    let info: Vec<_> = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Info)).collect();

    let mut response = String::new();

    if !critical.is_empty() {
        response.push_str(&format!("Found {} critical issue(s):\n", critical.len()));
        for issue in &critical {
            response.push_str(&format!("  - {}: {}\n", issue.title, issue.description));
            if !issue.suggestion.is_empty() {
                response.push_str(&format!("    Suggestion: {}\n", issue.suggestion));
            }
        }
        response.push('\n');
    }

    if !warnings.is_empty() {
        response.push_str(&format!("Found {} warning(s):\n", warnings.len()));
        for issue in warnings.iter().take(5) {
            response.push_str(&format!("  - {}\n", issue.title));
        }
        if warnings.len() > 5 {
            response.push_str(&format!("  ... and {} more\n", warnings.len() - 5));
        }
        response.push('\n');
    }

    if !info.is_empty() && critical.is_empty() && warnings.is_empty() {
        response.push_str(&format!("{} informational item(s) - nothing urgent.\n", info.len()));
    }

    response
}

fn generate_resources_response() -> String {
    let state = LiveState::capture();

    let cpu_pct = (state.load_avg.0 / num_cpus() as f32 * 100.0).min(100.0);

    let mut response = String::new();

    // CPU
    let cpu_status = if cpu_pct > 90.0 { "high" } else if cpu_pct > 70.0 { "moderate" } else { "normal" };
    response.push_str(&format!("CPU: {:.0}% ({})\n", cpu_pct, cpu_status));
    response.push_str(&format!("  Load: {:.2} / {:.2} / {:.2}\n", state.load_avg.0, state.load_avg.1, state.load_avg.2));

    // Memory
    let mem_pct = state.memory.percent_used();
    let mem_status = if mem_pct > 90.0 { "critical" } else if mem_pct > 80.0 { "high" } else { "normal" };
    response.push_str(&format!("\nMemory: {:.0}% ({}) - {:.1} GB / {:.1} GB\n", mem_pct, mem_status, state.memory.used_gb, state.memory.total_gb));

    // Swap
    if state.memory.swap_used_gb > 0.1 {
        response.push_str(&format!("  Swap: {:.1} GB / {:.1} GB\n", state.memory.swap_used_gb, state.memory.swap_total_gb));
    }

    // Disk
    let disk_pct = state.disk.percent_used();
    let disk_status = if disk_pct > 90.0 { "critical" } else if disk_pct > 80.0 { "getting full" } else { "normal" };
    response.push_str(&format!("\nDisk: {:.0}% ({}) - {:.0} GB / {:.0} GB\n", disk_pct, disk_status, state.disk.used_gb, state.disk.total_gb));

    // Network
    match &state.network_status {
        crate::live_state::NetworkStatus::Connected { interface, ip } => {
            response.push_str(&format!("\nNetwork: connected via {} ({})\n", interface, ip));
        }
        crate::live_state::NetworkStatus::Disconnected => {
            response.push_str("\nNetwork: disconnected\n");
        }
        _ => {}
    }

    response
}

fn generate_attention_response() -> String {
    let issues = scan_for_issues();
    let state = LiveState::capture();

    let critical: Vec<_> = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Critical)).collect();
    let warnings: Vec<_> = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Warning)).collect();

    if critical.is_empty() && warnings.is_empty() && state.failed_units.is_empty() {
        return "Nothing needs your attention right now. System is running smoothly.".to_string();
    }

    let mut response = String::new();

    if !critical.is_empty() {
        response.push_str("Things that need attention:\n\n");
        for issue in &critical {
            response.push_str(&format!("1. {} - {}\n", issue.title, issue.suggestion));
        }
    }

    if !warnings.is_empty() {
        if critical.is_empty() {
            response.push_str("A few things to consider:\n\n");
        }
        for (i, issue) in warnings.iter().take(3).enumerate() {
            response.push_str(&format!("{}. {}\n", i + 1 + critical.len(), issue.title));
        }
    }

    if !state.failed_units.is_empty() {
        response.push_str(&format!("\nAlso: {} service(s) have failed.\n", state.failed_units.len()));
    }

    response
}

fn generate_greeting_response() -> String {
    let state = LiveState::capture();
    let issues = scan_for_issues();

    let critical = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Critical)).count();

    if critical > 0 {
        format!("Hi! Heads up - there are {} issue(s) that need attention. Ask me 'any problems?' for details.", critical)
    } else if !state.failed_units.is_empty() {
        format!("Hello! Your system is mostly fine, but {} service(s) have issues. How can I help?", state.failed_units.len())
    } else {
        "Hi! Your system is running well. What would you like to know?".to_string()
    }
}

fn num_cpus() -> usize {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .map(|s| s.matches("processor").count())
        .unwrap_or(1)
        .max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_health_query() {
        assert_eq!(detect_natural_query("how is my system"), NaturalQuery::SystemHealth);
        assert_eq!(detect_natural_query("How's my system?"), NaturalQuery::SystemHealth);
        assert_eq!(detect_natural_query("give me a status report"), NaturalQuery::SystemHealth);
    }

    #[test]
    fn test_detect_problem_query() {
        assert_eq!(detect_natural_query("any problems?"), NaturalQuery::Problems);
        assert_eq!(detect_natural_query("is anything wrong"), NaturalQuery::Problems);
    }

    #[test]
    fn test_detect_greeting() {
        assert_eq!(detect_natural_query("hi"), NaturalQuery::Greeting);
        assert_eq!(detect_natural_query("hello"), NaturalQuery::Greeting);
        // Long messages with hi shouldn't match
        assert_eq!(detect_natural_query("hi can you help me with this long question about linux"), NaturalQuery::Other);
    }

    #[test]
    fn test_detect_other() {
        assert_eq!(detect_natural_query("how do I install firefox"), NaturalQuery::Other);
        assert_eq!(detect_natural_query("what kernel am I running"), NaturalQuery::Other);
    }
}
