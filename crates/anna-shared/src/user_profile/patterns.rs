//! User pattern analysis and trend detection (v0.0.236).
//!
//! Tracks tool usage trends over time to detect shifts like:
//! - "You're using vim more than nano lately"
//! - "Your network questions have increased"

use chrono::{DateTime, Datelike, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Time-windowed usage tracking for trend detection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatternHistory {
    /// Tool usage by week (week_key -> tool -> count)
    #[serde(default)]
    pub weekly_tool_usage: HashMap<String, HashMap<String, u32>>,
    /// Topic usage by week
    #[serde(default)]
    pub weekly_topic_usage: HashMap<String, HashMap<String, u32>>,
    /// Previous preferred editor (for trend detection)
    #[serde(default)]
    pub previous_editor: Option<String>,
    /// When the editor preference changed
    #[serde(default)]
    pub editor_changed_at: Option<DateTime<Utc>>,
}

impl PatternHistory {
    /// Get current week key (YYYY-WW format)
    pub fn current_week_key() -> String {
        let now = Utc::now();
        format!("{}-W{:02}", now.format("%Y"), now.iso_week().week())
    }

    /// Record tool usage for current week
    pub fn record_tool(&mut self, tool: &str) {
        let week = Self::current_week_key();
        let week_data = self.weekly_tool_usage.entry(week).or_default();
        *week_data.entry(tool.to_string()).or_insert(0) += 1;
    }

    /// Record topic for current week
    pub fn record_topic(&mut self, topic: &str) {
        let week = Self::current_week_key();
        let week_data = self.weekly_topic_usage.entry(week).or_default();
        *week_data.entry(topic.to_string()).or_insert(0) += 1;
    }

    /// Record editor preference change
    pub fn record_editor_change(&mut self, old: Option<&str>, new: &str) {
        if old.map(|o| o != new).unwrap_or(true) {
            self.previous_editor = old.map(String::from);
            self.editor_changed_at = Some(Utc::now());
        }
    }

    /// Detect editor trend shift (returns insight if trend changed)
    pub fn editor_trend_insight(
        &self,
        current_editor: Option<&str>,
        tool_usage: &HashMap<String, u32>,
    ) -> Option<EditorTrendInsight> {
        let editors = ["vim", "nvim", "nano", "emacs", "helix", "micro", "code"];

        // Get editor usage counts
        let mut editor_counts: Vec<(&str, u32)> = editors
            .iter()
            .filter_map(|e| tool_usage.get(*e).map(|c| (*e, *c)))
            .collect();

        // Sort by count descending
        editor_counts.sort_by(|a, b| b.1.cmp(&a.1));

        if editor_counts.len() < 2 {
            return None;
        }

        let (top_editor, top_count) = editor_counts[0];
        let (second_editor, second_count) = editor_counts[1];

        // Check if preference recently changed
        if let Some(ref prev) = self.previous_editor {
            if current_editor.map(|c| c != prev).unwrap_or(false) {
                // Recent change detected
                if let Some(changed_at) = self.editor_changed_at {
                    let days_since = (Utc::now() - changed_at).num_days();
                    if days_since < 14 {
                        return Some(EditorTrendInsight::RecentSwitch {
                            from: prev.clone(),
                            to: current_editor.unwrap_or("unknown").to_string(),
                            days_ago: days_since as u32,
                        });
                    }
                }
            }
        }

        // Check if user is shifting preferences (close race)
        if top_count > 2 && second_count > 2 {
            let ratio = top_count as f32 / second_count as f32;
            if ratio < 1.5 {
                // Close race - might be learning new editor
                return Some(EditorTrendInsight::LearningNew {
                    current: top_editor.to_string(),
                    emerging: second_editor.to_string(),
                    current_count: top_count,
                    emerging_count: second_count,
                });
            }
        }

        None
    }

    /// Get topic trend insights
    pub fn topic_trend_insight(
        &self,
        topic_interests: &HashMap<String, u32>,
    ) -> Option<TopicTrendInsight> {
        // Get recent week data
        let current_week = Self::current_week_key();
        let last_week = {
            let now = Utc::now();
            let last = now - Duration::weeks(1);
            format!("{}-W{:02}", last.format("%Y"), last.iso_week().week())
        };

        let current_topics = self.weekly_topic_usage.get(&current_week);
        let last_topics = self.weekly_topic_usage.get(&last_week);

        // Compare if we have data
        if let (Some(curr), Some(last)) = (current_topics, last_topics) {
            // Find topics that increased significantly
            for (topic, curr_count) in curr {
                let last_count = last.get(topic).copied().unwrap_or(0);
                if *curr_count > last_count + 2 && *curr_count >= 3 {
                    return Some(TopicTrendInsight::Increasing {
                        topic: topic.clone(),
                        this_week: *curr_count,
                        last_week: last_count,
                    });
                }
            }
        }

        // Find dominant topic overall
        if let Some((topic, count)) = topic_interests.iter().max_by_key(|(_, v)| *v) {
            if *count >= 5 {
                return Some(TopicTrendInsight::Dominant {
                    topic: topic.clone(),
                    count: *count,
                });
            }
        }

        None
    }

    /// Clean up old data (keep last 12 weeks)
    pub fn cleanup_old_data(&mut self) {
        let cutoff = {
            let now = Utc::now();
            let old = now - Duration::weeks(12);
            format!("{}-W{:02}", old.format("%Y"), old.iso_week().week())
        };

        self.weekly_tool_usage.retain(|k, _| k >= &cutoff);
        self.weekly_topic_usage.retain(|k, _| k >= &cutoff);
    }
}

/// Insight about editor usage trends
#[derive(Debug, Clone)]
pub enum EditorTrendInsight {
    /// User recently switched editors
    RecentSwitch {
        from: String,
        to: String,
        days_ago: u32,
    },
    /// User appears to be learning a new editor
    LearningNew {
        current: String,
        emerging: String,
        current_count: u32,
        emerging_count: u32,
    },
}

impl EditorTrendInsight {
    /// Format as conversational message
    pub fn to_message(&self) -> String {
        match self {
            EditorTrendInsight::RecentSwitch { from, to, days_ago } => {
                if *days_ago <= 3 {
                    format!(
                        "I noticed you switched from {} to {} recently. Learning the new hotness?",
                        from, to
                    )
                } else {
                    format!(
                        "You've been using {} more than {} lately (switched {} days ago).",
                        to, from, days_ago
                    )
                }
            }
            EditorTrendInsight::LearningNew {
                current,
                emerging,
                current_count,
                emerging_count,
            } => {
                format!(
                    "I see you're using both {} ({} times) and {} ({} times). Maybe learning {}?",
                    current, current_count, emerging, emerging_count, emerging
                )
            }
        }
    }
}

/// Insight about topic usage trends
#[derive(Debug, Clone)]
pub enum TopicTrendInsight {
    /// Topic usage increasing this week
    Increasing {
        topic: String,
        this_week: u32,
        last_week: u32,
    },
    /// Topic is dominant overall
    Dominant { topic: String, count: u32 },
}

impl TopicTrendInsight {
    /// Format as conversational message
    pub fn to_message(&self) -> String {
        match self {
            TopicTrendInsight::Increasing {
                topic,
                this_week,
                last_week,
            } => {
                format!(
                    "You're asking about {} more this week ({} vs {} last week).",
                    topic, this_week, last_week
                )
            }
            TopicTrendInsight::Dominant { topic, count } => {
                format!(
                    "You ask about {} a lot ({} times). Want some tips?",
                    topic, count
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_week_key_format() {
        let key = PatternHistory::current_week_key();
        assert!(key.contains("-W"), "Week key should contain -W: {}", key);
    }

    #[test]
    fn test_record_tool() {
        let mut history = PatternHistory::default();
        history.record_tool("vim");
        history.record_tool("vim");
        history.record_tool("nano");

        let week = PatternHistory::current_week_key();
        let week_data = history.weekly_tool_usage.get(&week).unwrap();
        assert_eq!(week_data.get("vim"), Some(&2));
        assert_eq!(week_data.get("nano"), Some(&1));
    }

    #[test]
    fn test_editor_trend_learning() {
        let history = PatternHistory::default();
        let mut usage = HashMap::new();
        usage.insert("vim".to_string(), 5);
        usage.insert("nano".to_string(), 4);

        let insight = history.editor_trend_insight(Some("vim"), &usage);
        assert!(insight.is_some());
        match insight.unwrap() {
            EditorTrendInsight::LearningNew { current, emerging, .. } => {
                assert_eq!(current, "vim");
                assert_eq!(emerging, "nano");
            }
            _ => panic!("Expected LearningNew insight"),
        }
    }
}
