//! Proactive Suggestions System
//! Anna analyzes system state and learned patterns to suggest improvements

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, info};

const SUGGESTIONS_FILE: &str = "/var/lib/anna/suggestions.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionPriority {
    Low,      // Nice to have
    Medium,   // Should consider
    High,     // Important
    Critical, // Needs attention
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub id: String,
    pub priority: SuggestionPriority,
    pub title: String,
    pub description: String,
    pub reasoning: String,
    pub action: Option<String>, // Optional action user can take
    pub created_at: String,
    pub shown_count: u32,
    pub dismissed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionsState {
    pub suggestions: Vec<Suggestion>,
    pub last_scan: String,
}

impl Default for SuggestionsState {
    fn default() -> Self {
        Self {
            suggestions: Vec::new(),
            last_scan: Utc::now().to_rfc3339(),
        }
    }
}

impl SuggestionsState {
    /// Load suggestions from disk
    pub fn load() -> Self {
        let path = PathBuf::from(SUGGESTIONS_FILE);
        if !path.exists() {
            return Self::default();
        }

        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save suggestions to disk
    pub fn save(&self) -> Result<()> {
        let path = PathBuf::from(SUGGESTIONS_FILE);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Add a new suggestion if it doesn't already exist
    pub fn add(&mut self, suggestion: Suggestion) {
        // Don't add if already exists
        if !self.suggestions.iter().any(|s| s.id == suggestion.id) {
            info!("New suggestion: {}", suggestion.title);
            self.suggestions.push(suggestion);
        }
    }

    /// Get active (non-dismissed) suggestions
    pub fn active_suggestions(&self) -> Vec<&Suggestion> {
        self.suggestions
            .iter()
            .filter(|s| !s.dismissed)
            .collect()
    }

    /// Mark suggestion as shown
    pub fn mark_shown(&mut self, id: &str) {
        if let Some(s) = self.suggestions.iter_mut().find(|s| s.id == id) {
            s.shown_count += 1;
        }
    }

    /// Dismiss a suggestion
    pub fn dismiss(&mut self, id: &str) {
        if let Some(s) = self.suggestions.iter_mut().find(|s| s.id == id) {
            s.dismissed = true;
        }
    }
}

/// Scan system for proactive suggestions
pub async fn scan_for_suggestions() -> Result<Vec<Suggestion>> {
    let mut suggestions = Vec::new();

    debug!("Scanning for proactive suggestions");

    // Check pacman cache size
    if let Some(s) = check_pacman_cache().await {
        suggestions.push(s);
    }

    // Check for orphaned packages
    if let Some(s) = check_orphaned_packages().await {
        suggestions.push(s);
    }

    // Check for failed services that keep failing
    if let Some(s) = check_recurring_failures().await {
        suggestions.push(s);
    }

    // Check disk usage trends
    if let Some(s) = check_disk_trends().await {
        suggestions.push(s);
    }

    // Check if Telegram not configured
    if let Some(s) = check_telegram_setup().await {
        suggestions.push(s);
    }

    info!("Found {} proactive suggestions", suggestions.len());
    Ok(suggestions)
}

/// Check if pacman cache is large
async fn check_pacman_cache() -> Option<Suggestion> {
    let output = std::process::Command::new("du")
        .args(["-sh", "/var/cache/pacman/pkg"])
        .output()
        .ok()?;

    let size_str = String::from_utf8_lossy(&output.stdout);
    let size = size_str.split_whitespace().next()?;

    // Parse size (e.g., "4.5G")
    let size_value: f32 = size.trim_end_matches(|c: char| !c.is_ascii_digit() && c != '.')
        .parse()
        .ok()?;

    let size_unit = size.chars().rev().next()?;

    let size_gb = match size_unit {
        'G' => size_value,
        'M' => size_value / 1024.0,
        'K' => size_value / (1024.0 * 1024.0),
        _ => return None,
    };

    if size_gb > 3.0 {
        Some(Suggestion {
            id: "pacman-cache-large".to_string(),
            priority: if size_gb > 5.0 {
                SuggestionPriority::High
            } else {
                SuggestionPriority::Medium
            },
            title: format!("Pacman cache is {:.1}GB", size_gb),
            description: format!(
                "Your package cache has grown to {:.1}GB. Cleaning it won't affect installed packages.",
                size_gb
            ),
            reasoning: "Large cache wastes disk space. Keeping last 3 versions is sufficient.".to_string(),
            action: Some("I can clean it for you, keeping the last 3 package versions. Just ask: 'clean pacman cache'".to_string()),
            created_at: Utc::now().to_rfc3339(),
            shown_count: 0,
            dismissed: false,
        })
    } else {
        None
    }
}

/// Check for orphaned packages
async fn check_orphaned_packages() -> Option<Suggestion> {
    let output = std::process::Command::new("pacman")
        .args(["-Qdtq"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let orphans = String::from_utf8_lossy(&output.stdout);
    let count = orphans.lines().filter(|l| !l.is_empty()).count();

    if count > 5 {
        // Get total size
        let size_output = std::process::Command::new("sh")
            .args(["-c", "pacman -Qdtq | xargs -r pacman -Qi | grep 'Installed Size' | awk '{sum+=$4} END {print sum}'"])
            .output()
            .ok()?;

        let size_kb: f32 = String::from_utf8_lossy(&size_output.stdout)
            .trim()
            .parse()
            .unwrap_or(0.0);

        let size_mb = size_kb / 1024.0;

        Some(Suggestion {
            id: "orphaned-packages".to_string(),
            priority: if count > 20 {
                SuggestionPriority::Medium
            } else {
                SuggestionPriority::Low
            },
            title: format!("{} orphaned packages found", count),
            description: format!(
                "You have {} packages that were installed as dependencies but are no longer needed ({:.0}MB).",
                count, size_mb
            ),
            reasoning: "Orphaned packages waste disk space and can accumulate over time.".to_string(),
            action: Some("I can remove them safely. Just ask: 'remove orphaned packages'".to_string()),
            created_at: Utc::now().to_rfc3339(),
            shown_count: 0,
            dismissed: false,
        })
    } else {
        None
    }
}

/// Check for services that fail repeatedly
async fn check_recurring_failures() -> Option<Suggestion> {
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

/// Check disk usage trends
async fn check_disk_trends() -> Option<Suggestion> {
    use crate::briefing::get_disk_usage_percentage;

    let current_usage = get_disk_usage_percentage();

    if current_usage > 85.0 {
        Some(Suggestion {
            id: "disk-usage-high".to_string(),
            priority: if current_usage > 95.0 {
                SuggestionPriority::Critical
            } else {
                SuggestionPriority::High
            },
            title: format!("Disk usage at {:.0}%", current_usage),
            description: "Your disk is filling up. This can cause system instability and prevent updates.".to_string(),
            reasoning: "Systems slow down significantly when disk exceeds 90% full.".to_string(),
            action: Some("I can help you find what's using space. Ask: 'what's using my disk space?'".to_string()),
            created_at: Utc::now().to_rfc3339(),
            shown_count: 0,
            dismissed: false,
        })
    } else {
        None
    }
}

/// Check if Telegram is configured
async fn check_telegram_setup() -> Option<Suggestion> {
    let telegram_config = std::path::Path::new("/etc/anna/telegram.env");

    if telegram_config.exists() {
        return None; // Already configured
    }

    Some(Suggestion {
        id: "telegram-not-configured".to_string(),
        priority: SuggestionPriority::Low,
        title: "Enable remote access via Telegram".to_string(),
        description: "You can control Anna from your phone and receive morning briefings with system health charts.".to_string(),
        reasoning: "Telegram provides convenient mobile access and proactive notifications.".to_string(),
        action: Some("Just ask: 'setup telegram bot'".to_string()),
        created_at: Utc::now().to_rfc3339(),
        shown_count: 0,
        dismissed: false,
    })
}

/// Format suggestions for user display
pub fn format_suggestions(suggestions: &[&Suggestion]) -> String {
    if suggestions.is_empty() {
        return String::new();
    }

    let mut output = String::new();
    output.push_str("\n💡 Proactive Suggestions:\n");

    for (i, suggestion) in suggestions.iter().enumerate() {
        let priority_icon = match suggestion.priority {
            SuggestionPriority::Critical => "🔴",
            SuggestionPriority::High => "🟠",
            SuggestionPriority::Medium => "🟡",
            SuggestionPriority::Low => "🔵",
        };

        output.push_str(&format!("\n{} {}. {}\n", priority_icon, i + 1, suggestion.title));
        output.push_str(&format!("   {}\n", suggestion.description));

        if let Some(action) = &suggestion.action {
            output.push_str(&format!("   → {}\n", action));
        }
    }

    output.push('\n');
    output
}
