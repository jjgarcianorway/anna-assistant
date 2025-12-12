//! Render formatting utilities (v0.0.203).
//! v0.0.452: Case ID format updated to CN-XXXX-DDMMYYYY per VISION.md.

use chrono::{Duration, Utc};

use super::types::RiskLevel;

/// Case ID generator for consistent formatting
/// v0.0.452: Format is now CN-XXXX-DDMMYYYY per VISION.md
pub fn generate_case_id(seq: u32) -> String {
    let now = Utc::now();
    format!("CN-{:04}-{}", seq, now.format("%d%m%Y"))
}

/// Format time delta in human terms
pub fn format_time_delta(duration: Duration) -> String {
    let secs = duration.num_seconds();
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        let mins = secs / 60;
        format!("{} minute{}", mins, if mins == 1 { "" } else { "s" })
    } else if secs < 86400 {
        let hours = secs / 3600;
        format!("{} hour{}", hours, if hours == 1 { "" } else { "s" })
    } else {
        let days = secs / 86400;
        format!("{} day{}", days, if days == 1 { "" } else { "s" })
    }
}

/// Determine risk level from answer content
pub fn determine_risk_level(answer: &str) -> RiskLevel {
    let lower = answer.to_lowercase();

    // High risk indicators
    if lower.contains("install")
        || lower.contains("remove")
        || lower.contains("pacman")
        || lower.contains("systemctl enable")
        || lower.contains("systemctl disable")
    {
        return RiskLevel::High;
    }

    // Medium risk indicators
    if lower.contains("edit")
        || lower.contains("modify")
        || lower.contains("config")
        || lower.contains("~/.")
        || lower.contains("/etc/")
    {
        return RiskLevel::Medium;
    }

    // Default to low (read-only)
    RiskLevel::Low
}
