//! Interesting facts for greeting personalization (v0.0.289).
//!
//! Generates data-driven facts about system, hardware, user patterns,
//! and performance for the LLM translator to naturalize into greetings.
//!
//! Key principle: All facts derived from actual data, no hardcoded content.

use crate::event_log::{AggregatedEvents, EventLog};
use crate::learning_progress::{compute_learning_progress, LearningProgress};
use crate::snapshot::SystemSnapshot;
use crate::system_telemetry::TelemetryStore;
use serde::{Deserialize, Serialize};

/// Categories of interesting facts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactCategory {
    /// System performance (CPU, memory, disk trends)
    Performance,
    /// Hardware information (uptime, specs)
    Hardware,
    /// User patterns (usage times, favorite topics)
    UserPattern,
    /// Anna's growth (recipes learned, success rates)
    Growth,
    /// Historical milestones
    Milestone,
}

/// A single interesting fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterestingFact {
    /// Category of the fact
    pub category: FactCategory,
    /// The fact text (for LLM to naturalize)
    pub fact: String,
    /// Priority (1=most interesting, 5=least)
    pub priority: u8,
}

/// Collection of interesting facts
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InterestingFacts {
    pub facts: Vec<InterestingFact>,
}

impl InterestingFacts {
    /// Generate facts from all available data sources
    pub fn generate(
        snapshot: Option<&SystemSnapshot>,
        telemetry: Option<&TelemetryStore>,
        events: Option<&AggregatedEvents>,
        progress: Option<&LearningProgress>,
    ) -> Self {
        let mut facts = Vec::new();

        // Hardware/uptime facts
        if let Some(snap) = snapshot {
            facts.extend(hardware_facts(snap));
        }

        // Performance trend facts
        if let Some(tel) = telemetry {
            facts.extend(performance_facts(tel));
        }

        // User pattern facts
        if let Some(agg) = events {
            facts.extend(user_pattern_facts(agg));
        }

        // Anna's growth facts
        if let Some(prog) = progress {
            facts.extend(growth_facts(prog));
        }

        // Sort by priority
        facts.sort_by_key(|f| f.priority);

        Self { facts }
    }

    /// Load all data and generate facts
    pub fn from_current_state(snapshot: &SystemSnapshot) -> Self {
        let event_log = EventLog::new(EventLog::default_path(), 10000);
        let events = event_log.aggregate().ok();
        let telemetry = TelemetryStore::load_if_exists();
        let progress = compute_learning_progress();

        Self::generate(
            Some(snapshot),
            telemetry.as_ref(),
            events.as_ref(),
            Some(&progress),
        )
    }

    /// Get top N facts for greeting
    pub fn top(&self, n: usize) -> Vec<&InterestingFact> {
        self.facts.iter().take(n).collect()
    }

    /// Get facts as strings for LLM context
    pub fn as_strings(&self, max: usize) -> Vec<String> {
        self.facts
            .iter()
            .take(max)
            .map(|f| f.fact.clone())
            .collect()
    }
}

/// Generate hardware-related facts from system snapshot
fn hardware_facts(snapshot: &SystemSnapshot) -> Vec<InterestingFact> {
    let mut facts = Vec::new();

    // Uptime fact
    if snapshot.boot_time_secs > 0 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let uptime_secs = now.saturating_sub(snapshot.boot_time_secs);
        let uptime_days = uptime_secs / 86400;
        let uptime_hours = (uptime_secs % 86400) / 3600;

        if uptime_days >= 30 {
            facts.push(InterestingFact {
                category: FactCategory::Hardware,
                fact: format!(
                    "System uptime: {} days - impressive stability",
                    uptime_days
                ),
                priority: 2,
            });
        } else if uptime_days >= 7 {
            facts.push(InterestingFact {
                category: FactCategory::Hardware,
                fact: format!("System running for {} days, {} hours", uptime_days, uptime_hours),
                priority: 3,
            });
        } else if uptime_days >= 1 {
            facts.push(InterestingFact {
                category: FactCategory::Hardware,
                fact: format!("System uptime: {} day(s)", uptime_days),
                priority: 4,
            });
        }
    }

    // Memory usage fact
    if snapshot.memory_total_bytes > 0 {
        let total_gb = snapshot.memory_total_bytes as f64 / 1_073_741_824.0;
        let used_pct = snapshot.memory_percent();

        if used_pct < 50 {
            facts.push(InterestingFact {
                category: FactCategory::Hardware,
                fact: format!(
                    "Memory usage healthy at {}% of {:.1}GB",
                    used_pct, total_gb
                ),
                priority: 4,
            });
        }
    }

    // Load average fact
    if snapshot.load_1min > 0.0 {
        if snapshot.load_1min < 1.0 {
            facts.push(InterestingFact {
                category: FactCategory::Performance,
                fact: format!("System load low at {:.2}", snapshot.load_1min),
                priority: 5,
            });
        } else if snapshot.load_5min < snapshot.load_1min * 0.7 {
            facts.push(InterestingFact {
                category: FactCategory::Performance,
                fact: "Load decreasing - system settling down".to_string(),
                priority: 3,
            });
        }
    }

    // Network status
    if snapshot.network_connected && !snapshot.ip_addresses.is_empty() {
        facts.push(InterestingFact {
            category: FactCategory::Hardware,
            fact: format!(
                "Network connected with {} IP address(es)",
                snapshot.ip_addresses.len()
            ),
            priority: 5,
        });
    }

    facts
}

/// Generate performance trend facts from telemetry
fn performance_facts(telemetry: &TelemetryStore) -> Vec<InterestingFact> {
    let mut facts = Vec::new();

    let trends = &telemetry.trends;

    // CPU trend
    if trends.cpu_trend < -5.0 && trends.sample_count >= 10 {
        facts.push(InterestingFact {
            category: FactCategory::Performance,
            fact: format!(
                "CPU usage dropped {:.0}% over the last {:.1} hours",
                trends.cpu_trend.abs(),
                trends.window_hours
            ),
            priority: 2,
        });
    } else if trends.cpu_trend > 10.0 && trends.sample_count >= 10 {
        facts.push(InterestingFact {
            category: FactCategory::Performance,
            fact: format!(
                "CPU usage trending up {:.0}% recently",
                trends.cpu_trend
            ),
            priority: 2,
        });
    }

    // Memory trend
    if trends.memory_trend < -5.0 && trends.sample_count >= 10 {
        facts.push(InterestingFact {
            category: FactCategory::Performance,
            fact: "Memory usage trending down - good cleanup".to_string(),
            priority: 3,
        });
    }

    // Disk trend
    if trends.disk_trend > 2.0 && trends.sample_count >= 10 {
        facts.push(InterestingFact {
            category: FactCategory::Performance,
            fact: format!(
                "Disk usage grew {:.1}% recently",
                trends.disk_trend
            ),
            priority: 3,
        });
    }

    // Health score
    let score = telemetry.health_score();
    if score >= 90 {
        facts.push(InterestingFact {
            category: FactCategory::Performance,
            fact: format!("System health excellent at {}%", score),
            priority: 2,
        });
    }

    // Sample count (data richness)
    if telemetry.samples.len() >= 100 {
        facts.push(InterestingFact {
            category: FactCategory::Performance,
            fact: format!(
                "Tracking {} telemetry samples over {:.1} hours",
                telemetry.samples.len(),
                trends.window_hours
            ),
            priority: 5,
        });
    }

    facts
}

/// Generate user pattern facts from event aggregation
fn user_pattern_facts(events: &AggregatedEvents) -> Vec<InterestingFact> {
    let mut facts = Vec::new();

    // Total requests milestone
    let milestones = [10, 50, 100, 250, 500, 1000, 2500, 5000];
    for milestone in milestones {
        if events.total_requests >= milestone && events.total_requests < milestone + 10 {
            facts.push(InterestingFact {
                category: FactCategory::Milestone,
                fact: format!("Reached {} requests with Anna", milestone),
                priority: 1,
            });
            break;
        }
    }

    // Success rate
    if events.total_requests >= 10 {
        let success_rate = events.verified_count as f32 / events.total_requests as f32 * 100.0;
        if success_rate >= 90.0 {
            facts.push(InterestingFact {
                category: FactCategory::Growth,
                fact: format!(
                    "Success rate at {:.0}% across {} requests",
                    success_rate, events.total_requests
                ),
                priority: 2,
            });
        }
    }

    // Streak fact
    if events.current_streak >= 7 {
        facts.push(InterestingFact {
            category: FactCategory::UserPattern,
            fact: format!(
                "On a {}-day streak - consistent usage",
                events.current_streak
            ),
            priority: 2,
        });
    } else if events.current_streak >= 3 {
        facts.push(InterestingFact {
            category: FactCategory::UserPattern,
            fact: format!("{} days in a row", events.current_streak),
            priority: 4,
        });
    }

    // Best streak
    if events.best_streak >= 14 && events.best_streak > events.current_streak {
        facts.push(InterestingFact {
            category: FactCategory::Milestone,
            fact: format!("Best streak: {} consecutive days", events.best_streak),
            priority: 3,
        });
    }

    // Lucky team
    if let Some(ref team) = events.lucky_team {
        if events.lucky_team_rate >= 0.8 && events.by_team.get(team).unwrap_or(&0) >= &5 {
            facts.push(InterestingFact {
                category: FactCategory::UserPattern,
                fact: format!(
                    "{} questions have {:.0}% success rate",
                    team,
                    events.lucky_team_rate * 100.0
                ),
                priority: 3,
            });
        }
    }

    // Average response time
    if events.avg_duration_ms > 0.0 && events.total_requests >= 10 {
        if events.avg_duration_ms < 500.0 {
            facts.push(InterestingFact {
                category: FactCategory::Performance,
                fact: format!("Average response: {:.0}ms - fast", events.avg_duration_ms),
                priority: 4,
            });
        } else if events.avg_duration_ms < 2000.0 {
            facts.push(InterestingFact {
                category: FactCategory::Performance,
                fact: format!("Average response: {:.1}s", events.avg_duration_ms / 1000.0),
                priority: 5,
            });
        }
    }

    // Tenure
    if events.first_event_ts > 0 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let days_since = (now - events.first_event_ts) / 86400;
        if days_since >= 30 {
            facts.push(InterestingFact {
                category: FactCategory::Milestone,
                fact: format!("Using Anna for {} days", days_since),
                priority: 3,
            });
        }
    }

    facts
}

/// Generate Anna's growth facts from learning progress
fn growth_facts(progress: &LearningProgress) -> Vec<InterestingFact> {
    let mut facts = Vec::new();

    // Recipes learned milestones
    let milestones = [5, 10, 25, 50, 100, 200];
    for milestone in milestones {
        if progress.recipes_total >= milestone && progress.recipes_total < milestone + 3 {
            facts.push(InterestingFact {
                category: FactCategory::Growth,
                fact: format!("Anna learned {} recipes", milestone),
                priority: 1,
            });
            break;
        }
    }

    // Self-sufficiency
    if progress.self_sufficiency >= 0.5 {
        facts.push(InterestingFact {
            category: FactCategory::Growth,
            fact: format!(
                "Anna handles {:.0}% of requests from learned knowledge",
                progress.self_sufficiency * 100.0
            ),
            priority: 2,
        });
    } else if progress.self_sufficiency >= 0.2 && progress.recipes_total >= 10 {
        facts.push(InterestingFact {
            category: FactCategory::Growth,
            fact: format!(
                "Anna's self-sufficiency at {:.0}% and growing",
                progress.self_sufficiency * 100.0
            ),
            priority: 3,
        });
    }

    // Strong areas
    if !progress.strong_areas.is_empty() {
        let areas = progress.strong_areas.join(", ");
        facts.push(InterestingFact {
            category: FactCategory::Growth,
            fact: format!("Anna is strong in: {}", areas),
            priority: 3,
        });
    }

    // Average reliability
    if progress.avg_reliability >= 80 && progress.recipes_total >= 10 {
        facts.push(InterestingFact {
            category: FactCategory::Growth,
            fact: format!(
                "Recipe reliability at {}%",
                progress.avg_reliability
            ),
            priority: 4,
        });
    }

    // Category diversity
    let category_count = progress.by_category.len();
    if category_count >= 5 {
        facts.push(InterestingFact {
            category: FactCategory::Growth,
            fact: format!("Knowledge spans {} different areas", category_count),
            priority: 3,
        });
    }

    facts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_facts() {
        let facts = InterestingFacts::generate(None, None, None, None);
        assert!(facts.facts.is_empty());
    }

    #[test]
    fn test_hardware_uptime_fact() {
        let mut snapshot = SystemSnapshot::default();
        // Set boot time to 8 days ago
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        snapshot.boot_time_secs = now - (8 * 86400);

        let facts = hardware_facts(&snapshot);
        assert!(!facts.is_empty());
        assert!(facts.iter().any(|f| f.fact.contains("8 days")));
    }

    #[test]
    fn test_milestone_detection() {
        let mut events = AggregatedEvents::default();
        events.total_requests = 100;
        events.verified_count = 95;

        let facts = user_pattern_facts(&events);
        assert!(facts.iter().any(|f| f.fact.contains("100 requests")));
    }

    #[test]
    fn test_growth_facts() {
        let mut progress = LearningProgress::default();
        progress.recipes_total = 50;
        progress.self_sufficiency = 0.6;
        progress.strong_areas = vec!["storage".to_string(), "network".to_string()];

        let facts = growth_facts(&progress);
        assert!(!facts.is_empty());
        assert!(facts.iter().any(|f| f.fact.contains("50 recipes")));
    }

    #[test]
    fn test_facts_sorted_by_priority() {
        let mut snapshot = SystemSnapshot::default();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        snapshot.boot_time_secs = now - (8 * 86400);
        snapshot.memory_total_bytes = 16 * 1024 * 1024 * 1024;
        snapshot.memory_used_bytes = 4 * 1024 * 1024 * 1024;

        let facts = InterestingFacts::generate(Some(&snapshot), None, None, None);

        // Check facts are sorted by priority (ascending)
        for window in facts.facts.windows(2) {
            assert!(window[0].priority <= window[1].priority);
        }
    }
}
