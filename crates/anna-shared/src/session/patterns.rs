//! Cross-session pattern mining and analysis.
//!
//! v0.0.889: Patterns mined from user behavior across sessions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::SessionStore;

/// Cross-session patterns mined from user behavior (v0.0.889)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrossSessionPatterns {
    /// Common topic sequences (e.g., "network" -> "dns" -> "firewall")
    pub topic_flows: Vec<TopicFlow>,
    /// Frequently asked question patterns
    pub frequent_patterns: Vec<FrequentPattern>,
    /// Time-of-day patterns
    pub time_patterns: HashMap<String, Vec<String>>,
    /// Recurring issues the user has encountered
    pub recurring_issues: Vec<RecurringIssue>,
    /// Last pattern mining timestamp
    pub last_mined: Option<String>,
}

/// A common sequence of topics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicFlow {
    /// Sequence of topics (e.g., ["network", "dns", "firewall"])
    pub sequence: Vec<String>,
    /// How often this flow has been observed
    pub count: u32,
    /// Confidence score (0-1)
    pub confidence: f32,
}

/// A frequently asked question pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequentPattern {
    /// Keywords that define this pattern
    pub keywords: Vec<String>,
    /// Canonical question form
    pub canonical: String,
    /// How many times this pattern has been asked
    pub count: u32,
    /// Commands that typically answer this
    pub typical_commands: Vec<String>,
}

/// A recurring issue the user encounters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurringIssue {
    /// Description of the issue
    pub description: String,
    /// Keywords associated with this issue
    pub keywords: Vec<String>,
    /// How many times it's occurred
    pub occurrences: u32,
    /// Last occurrence timestamp
    pub last_seen: String,
}

impl SessionStore {
    /// Mine patterns from all sessions (v0.0.889)
    pub fn mine_patterns(&mut self) {
        self.mine_topic_flows();
        self.mine_frequent_patterns();
        self.mine_recurring_issues();
        self.patterns.last_mined = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Mine topic flow patterns from session histories
    pub(super) fn mine_topic_flows(&mut self) {
        let mut flow_counts: HashMap<Vec<String>, u32> = HashMap::new();

        for session in self.sessions.values() {
            let topics: Vec<String> = session.context.explored_topics.clone();

            let mut full_topics = topics;
            if let Some(ref current) = session.context.current_topic {
                full_topics.push(current.clone());
            }

            if full_topics.len() >= 2 {
                for window in full_topics.windows(2) {
                    let seq = window.to_vec();
                    *flow_counts.entry(seq).or_insert(0) += 1;
                }
            }
            if full_topics.len() >= 3 {
                for window in full_topics.windows(3) {
                    let seq = window.to_vec();
                    *flow_counts.entry(seq).or_insert(0) += 1;
                }
            }
        }

        let total_sessions = self.sessions.len().max(1) as f32;
        self.patterns.topic_flows = flow_counts
            .into_iter()
            .filter(|(_, count)| *count >= 2)
            .map(|(sequence, count)| TopicFlow {
                sequence,
                count,
                confidence: count as f32 / total_sessions,
            })
            .collect();

        self.patterns
            .topic_flows
            .sort_by(|a, b| b.count.cmp(&a.count));
        self.patterns.topic_flows.truncate(20);
    }

    /// Mine frequently asked question patterns
    pub(super) fn mine_frequent_patterns(&mut self) {
        let mut keyword_counts: HashMap<String, (u32, Vec<String>)> = HashMap::new();

        for session in self.sessions.values() {
            for turn in &session.history {
                let keywords: Vec<String> = turn
                    .question
                    .to_lowercase()
                    .split_whitespace()
                    .filter(|w| w.len() > 3)
                    .map(String::from)
                    .collect();

                for kw in &keywords {
                    let entry = keyword_counts.entry(kw.clone()).or_insert((0, Vec::new()));
                    entry.0 += 1;
                    for cmd in &turn.commands {
                        if !entry.1.contains(cmd) && entry.1.len() < 5 {
                            entry.1.push(cmd.clone());
                        }
                    }
                }
            }
        }

        self.patterns.frequent_patterns = keyword_counts
            .into_iter()
            .filter(|(_, (count, _))| *count >= 3)
            .map(|(keyword, (count, commands))| FrequentPattern {
                keywords: vec![keyword.clone()],
                canonical: keyword,
                count,
                typical_commands: commands,
            })
            .collect();

        self.patterns
            .frequent_patterns
            .sort_by(|a, b| b.count.cmp(&a.count));
        self.patterns.frequent_patterns.truncate(30);
    }

    /// Mine recurring issues from session histories
    pub(super) fn mine_recurring_issues(&mut self) {
        let issue_keywords = [
            "error", "fail", "broken", "not working", "issue", "problem",
        ];
        let mut issues: HashMap<String, (u32, String)> = HashMap::new();

        for session in self.sessions.values() {
            for turn in &session.history {
                let q_lower = turn.question.to_lowercase();

                if issue_keywords.iter().any(|k| q_lower.contains(k)) {
                    if let Some(topic) = &session.context.current_topic {
                        let entry = issues
                            .entry(topic.clone())
                            .or_insert((0, turn.timestamp.clone()));
                        entry.0 += 1;
                        entry.1 = turn.timestamp.clone();
                    }
                }
            }
        }

        self.patterns.recurring_issues = issues
            .into_iter()
            .filter(|(_, (count, _))| *count >= 2)
            .map(|(topic, (occurrences, last_seen))| RecurringIssue {
                description: format!("{} issues", topic),
                keywords: vec![topic],
                occurrences,
                last_seen,
            })
            .collect();

        self.patterns
            .recurring_issues
            .sort_by(|a, b| b.occurrences.cmp(&a.occurrences));
    }

    /// Get suggested next topics based on current topic (v0.0.889)
    pub fn suggest_next_topics(&self, current_topic: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        for flow in &self.patterns.topic_flows {
            if flow.sequence.first() == Some(&current_topic.to_string()) {
                if let Some(next) = flow.sequence.get(1) {
                    if !suggestions.contains(next) {
                        suggestions.push(next.clone());
                    }
                }
            }
        }

        suggestions.truncate(3);
        suggestions
    }

    /// Check if this looks like a recurring issue (v0.0.889)
    pub fn is_recurring_issue(&self, question: &str) -> Option<&RecurringIssue> {
        let q_lower = question.to_lowercase();

        for issue in &self.patterns.recurring_issues {
            if issue.keywords.iter().any(|k| q_lower.contains(k)) {
                return Some(issue);
            }
        }

        None
    }
}
