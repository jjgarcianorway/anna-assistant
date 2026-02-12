//! Pattern Learning - Anna learns user habits and proposes automation.
//!
//! Philosophy: If the user asks the same thing regularly, automate it!

use anyhow::Result;
use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info};

const PATTERNS_DB: &str = "/var/lib/anna/patterns.json";
const MIN_OCCURRENCES: usize = 3; // Need 3+ occurrences to suggest automation

/// A detected usage pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    /// Pattern ID (hash of normalized question)
    pub id: String,
    /// Normalized question pattern
    pub pattern: String,
    /// Occurrences with timestamps
    pub occurrences: Vec<DateTime<Utc>>,
    /// Day of week pattern (if any)
    pub day_pattern: Option<Weekday>,
    /// Time of day pattern (hour, if any)
    pub time_pattern: Option<u32>,
    /// Whether user was offered automation
    pub automation_offered: bool,
    /// User's response to automation offer
    pub user_response: Option<AutomationResponse>,
}

/// User's response to automation suggestion.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AutomationResponse {
    Accepted,
    Declined,
    Later,
}

/// Pattern database.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatternDatabase {
    pub patterns: HashMap<String, Pattern>,
    pub last_cleanup: Option<DateTime<Utc>>,
}

impl PatternDatabase {
    /// Load from disk.
    pub fn load() -> Self {
        let path = PathBuf::from(PATTERNS_DB);
        if !path.exists() {
            return Self::default();
        }

        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save to disk.
    pub fn save(&self) -> Result<()> {
        let path = PathBuf::from(PATTERNS_DB);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    /// Record a question occurrence.
    pub fn record(&mut self, question: &str) {
        let normalized = normalize_question(question);
        let pattern_id = format!("{:x}", md5::compute(&normalized));

        let now = Utc::now();

        if let Some(pattern) = self.patterns.get_mut(&pattern_id) {
            pattern.occurrences.push(now);
        } else {
            let pattern = Pattern {
                id: pattern_id.clone(),
                pattern: normalized,
                occurrences: vec![now],
                day_pattern: None,
                time_pattern: None,
                automation_offered: false,
                user_response: None,
            };
            self.patterns.insert(pattern_id, pattern);
        }
    }

    /// Analyze patterns and detect recurring questions.
    pub fn analyze_patterns(&mut self) {
        for pattern in self.patterns.values_mut() {
            if pattern.occurrences.len() < MIN_OCCURRENCES {
                continue;
            }

            // Detect day-of-week pattern
            let days: Vec<Weekday> = pattern
                .occurrences
                .iter()
                .map(|dt| dt.weekday())
                .collect();

            if let Some(common_day) = most_common_day(&days) {
                let same_day_count = days.iter().filter(|&&d| d == common_day).count();
                if same_day_count >= (days.len() * 2) / 3 {
                    // 2/3+ on same day
                    pattern.day_pattern = Some(common_day);
                }
            }

            // Detect time-of-day pattern (within 2-hour window)
            let hours: Vec<u32> = pattern.occurrences.iter().map(|dt| dt.hour()).collect();

            if let Some(avg_hour) = average_hour(&hours) {
                let within_window = hours
                    .iter()
                    .filter(|&&h| (h as i32 - avg_hour as i32).abs() <= 2)
                    .count();

                if within_window >= (hours.len() * 2) / 3 {
                    pattern.time_pattern = Some(avg_hour);
                }
            }
        }
    }

    /// Get patterns ready for automation suggestions.
    pub fn patterns_for_automation(&self) -> Vec<&Pattern> {
        self.patterns
            .values()
            .filter(|p| {
                p.occurrences.len() >= MIN_OCCURRENCES
                    && !p.automation_offered
                    && p.user_response != Some(AutomationResponse::Declined)
            })
            .collect()
    }

    /// Mark automation as offered.
    pub fn mark_offered(&mut self, pattern_id: &str) {
        if let Some(pattern) = self.patterns.get_mut(pattern_id) {
            pattern.automation_offered = true;
        }
    }

    /// Record user response to automation offer.
    pub fn record_response(&mut self, pattern_id: &str, response: AutomationResponse) {
        if let Some(pattern) = self.patterns.get_mut(pattern_id) {
            pattern.user_response = Some(response);
        }
    }

    /// Cleanup old patterns (>90 days, no recent occurrences).
    pub fn cleanup(&mut self) {
        let cutoff = Utc::now() - chrono::Duration::days(90);

        self.patterns.retain(|_, pattern| {
            pattern
                .occurrences
                .last()
                .map(|&dt| dt > cutoff)
                .unwrap_or(false)
        });

        self.last_cleanup = Some(Utc::now());
    }
}

/// Normalize question for pattern matching.
fn normalize_question(question: &str) -> String {
    let q = question.to_lowercase();

    // Remove dates, times, specific numbers
    let q = regex::Regex::new(r"\d{4}-\d{2}-\d{2}")
        .unwrap()
        .replace_all(&q, "DATE");
    let q = regex::Regex::new(r"\d+:\d+").unwrap().replace_all(&q, "TIME");
    let q = regex::Regex::new(r"\d+\.\d+|\d+")
        .unwrap()
        .replace_all(&q, "NUM");

    // Remove "please", "can you", etc.
    let q = q
        .replace("please", "")
        .replace("can you", "")
        .replace("could you", "")
        .replace("would you", "");

    // Normalize whitespace
    q.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Find most common day of week.
fn most_common_day(days: &[Weekday]) -> Option<Weekday> {
    let mut counts: HashMap<Weekday, usize> = HashMap::new();
    for &day in days {
        *counts.entry(day).or_insert(0) += 1;
    }

    counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(day, _)| day)
}

/// Calculate average hour (handling wrap-around).
fn average_hour(hours: &[u32]) -> Option<u32> {
    if hours.is_empty() {
        return None;
    }

    let sum: u32 = hours.iter().sum();
    Some(sum / hours.len() as u32)
}

/// Record a question for pattern learning.
pub fn record_question(question: &str) {
    let mut db = PatternDatabase::load();
    db.record(question);
    db.analyze_patterns();

    if let Err(e) = db.save() {
        debug!("Failed to save pattern database: {}", e);
    }
}

/// Check if this question matches a pattern that needs automation suggestion.
pub fn check_for_automation_opportunity(question: &str) -> Option<AutomationSuggestion> {
    let mut db = PatternDatabase::load();
    let normalized = normalize_question(question);
    let pattern_id = format!("{:x}", md5::compute(&normalized));

    if let Some(pattern) = db.patterns.get(&pattern_id) {
        if pattern.occurrences.len() >= MIN_OCCURRENCES && !pattern.automation_offered {
            let suggestion = generate_automation_suggestion(pattern);

            // Mark as offered
            db.mark_offered(&pattern_id);
            let _ = db.save();

            return Some(suggestion);
        }
    }

    None
}

/// Automation suggestion for user.
#[derive(Debug, Clone)]
pub struct AutomationSuggestion {
    pub pattern_id: String,
    pub message: String,
    pub options: Vec<AutomationOption>,
}

#[derive(Debug, Clone)]
pub struct AutomationOption {
    pub id: String,
    pub description: String,
    pub action: AutomationAction,
}

#[derive(Debug, Clone)]
pub enum AutomationAction {
    AddToBriefing,
    CreateScheduledTask { day: Option<Weekday>, hour: Option<u32> },
    CreateAlert { threshold: String },
    DoNothing,
}

/// Generate automation suggestion from pattern.
fn generate_automation_suggestion(pattern: &Pattern) -> AutomationSuggestion {
    let count = pattern.occurrences.len();

    let mut message = format!(
        "I noticed you've asked about this {} times",
        count
    );

    if let Some(day) = pattern.day_pattern {
        message.push_str(&format!(" (usually on {}s)", day_name(day)));
    }

    if let Some(hour) = pattern.time_pattern {
        message.push_str(&format!(" around {}:00", hour));
    }

    message.push_str(". Would you like me to:\n");

    let mut options = Vec::new();

    // Option 1: Add to weekly briefing
    options.push(AutomationOption {
        id: "briefing".to_string(),
        description: "Add this to your morning briefing automatically".to_string(),
        action: AutomationAction::AddToBriefing,
    });

    // Option 2: Create scheduled task (if time pattern detected)
    if let (Some(day), Some(hour)) = (pattern.day_pattern, pattern.time_pattern) {
        options.push(AutomationOption {
            id: "schedule".to_string(),
            description: format!("Send automatic report every {} at {}:00", day_name(day), hour),
            action: AutomationAction::CreateScheduledTask {
                day: Some(day),
                hour: Some(hour),
            },
        });
    } else if let Some(day) = pattern.day_pattern {
        options.push(AutomationOption {
            id: "schedule".to_string(),
            description: format!("Send automatic report every {}", day_name(day)),
            action: AutomationAction::CreateScheduledTask {
                day: Some(day),
                hour: pattern.time_pattern,
            },
        });
    }

    // Option 3: Alert-based (only if >80%, critical, etc.)
    if pattern.pattern.contains("disk") || pattern.pattern.contains("memory") {
        options.push(AutomationOption {
            id: "alert".to_string(),
            description: "Alert me only if usage exceeds 80%".to_string(),
            action: AutomationAction::CreateAlert {
                threshold: "80%".to_string(),
            },
        });
    }

    // Option 4: Do nothing
    options.push(AutomationOption {
        id: "nothing".to_string(),
        description: "Keep asking manually (don't automate)".to_string(),
        action: AutomationAction::DoNothing,
    });

    AutomationSuggestion {
        pattern_id: pattern.id.clone(),
        message,
        options,
    }
}

fn day_name(day: Weekday) -> &'static str {
    match day {
        Weekday::Mon => "Monday",
        Weekday::Tue => "Tuesday",
        Weekday::Wed => "Wednesday",
        Weekday::Thu => "Thursday",
        Weekday::Fri => "Friday",
        Weekday::Sat => "Saturday",
        Weekday::Sun => "Sunday",
    }
}

/// Format automation suggestion for display.
pub fn format_automation_suggestion(suggestion: &AutomationSuggestion) -> String {
    let mut response = format!("{}\n\n", suggestion.message);

    for (i, option) in suggestion.options.iter().enumerate() {
        response.push_str(&format!("{}. {}\n", i + 1, option.description));
    }

    response.push_str("\nWhich option would you like?");
    response
}
