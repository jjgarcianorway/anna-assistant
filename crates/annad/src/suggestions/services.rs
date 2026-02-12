//! Service-related suggestion checks.

use chrono::Utc;
use super::types::{Suggestion, SuggestionPriority};

/// Check for services that fail repeatedly
pub async fn check_recurring_failures() -> Option<Suggestion> {
    // Check systemd journal for recurring service failures
    let output = std::process::Command::new("journalctl")
        .args(["-p", "err", "-n", "100", "--no-pager", "-o", "json"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let logs = String::from_utf8_lossy(&output.stdout);
    let mut failure_counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

    for line in logs.lines() {
        if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(unit) = entry.get("UNIT").and_then(|v| v.as_str()) {
                if let Some(msg) = entry.get("MESSAGE").and_then(|v| v.as_str()) {
                    if msg.contains("Failed") || msg.contains("failed") {
                        *failure_counts.entry(unit.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    // Find service with most failures
    let max_failures = failure_counts.iter().max_by_key(|(_, &count)| count)?;

    if *max_failures.1 > 3 {
        Some(Suggestion {
            id: format!("recurring-failure-{}", max_failures.0),
            priority: SuggestionPriority::High,
            title: format!("{} failing repeatedly", max_failures.0),
            description: format!(
                "{} has failed {} times in recent logs. This suggests an underlying issue.",
                max_failures.0, max_failures.1
            ),
            reasoning: "Recurring failures indicate a misconfiguration or dependency issue.".to_string(),
            action: Some(format!("I can investigate. Just ask: 'why is {} failing?'", max_failures.0)),
            created_at: Utc::now().to_rfc3339(),
            shown_count: 0,
            dismissed: false,
        })
    } else {
        None
    }
}
