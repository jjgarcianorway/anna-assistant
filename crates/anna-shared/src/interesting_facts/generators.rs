//! Fact generators (v0.0.379).
//!
//! Functions that generate facts from various data sources.
//!
//! v0.0.379: Added boot time comparison facts.

use crate::event_log::AggregatedEvents;
use crate::learning_progress::LearningProgress;
use crate::snapshot::SystemSnapshot;
use crate::system_telemetry::TelemetryStore;
use crate::telemetry::TelemetrySnapshot;

use super::types::{FactCategory, InterestingFact};

/// Generate hardware-related facts from system snapshot
pub fn hardware_facts(snapshot: &SystemSnapshot) -> Vec<InterestingFact> {
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
                fact: format!("System uptime: {} days - impressive stability", uptime_days),
                priority: 2,
            });
        } else if uptime_days >= 7 {
            facts.push(InterestingFact {
                category: FactCategory::Hardware,
                fact: format!(
                    "System running for {} days, {} hours",
                    uptime_days, uptime_hours
                ),
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
                fact: format!("Memory usage healthy at {}% of {:.1}GB", used_pct, total_gb),
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
pub fn performance_facts(telemetry: &TelemetryStore) -> Vec<InterestingFact> {
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
            fact: format!("CPU usage trending up {:.0}% recently", trends.cpu_trend),
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
            fact: format!("Disk usage grew {:.1}% recently", trends.disk_trend),
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
pub fn user_pattern_facts(events: &AggregatedEvents) -> Vec<InterestingFact> {
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
            fact: format!("On a {}-day streak - consistent usage", events.current_streak),
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
                fact: format!(
                    "Average response: {:.1}s",
                    events.avg_duration_ms / 1000.0
                ),
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
pub fn growth_facts(progress: &LearningProgress) -> Vec<InterestingFact> {
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
            fact: format!("Recipe reliability at {}%", progress.avg_reliability),
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

/// Generate boot time comparison facts from telemetry
/// v0.0.379: Compare current vs previous boot time
pub fn boot_time_facts(telemetry: &TelemetrySnapshot) -> Vec<InterestingFact> {
    let mut facts = Vec::new();

    if let Some(delta_ms) = telemetry.boot_delta_ms {
        let delta_secs = delta_ms.abs() as f64 / 1000.0;

        // Only report meaningful changes (>0.5s)
        if delta_secs >= 0.5 {
            let fact = if delta_ms < 0 {
                // Faster boot (negative delta = improvement)
                if delta_secs >= 5.0 {
                    format!(
                        "Boot time improved by {:.1}s - nice optimization!",
                        delta_secs
                    )
                } else {
                    format!("Boot time {:.1}s faster than last session", delta_secs)
                }
            } else {
                // Slower boot (positive delta = regression)
                if delta_secs >= 5.0 {
                    format!(
                        "Boot time increased by {:.1}s since last session",
                        delta_secs
                    )
                } else {
                    format!("Boot {:.1}s slower than before", delta_secs)
                }
            };

            // High priority for significant changes
            let priority = if delta_secs >= 3.0 { 2 } else { 4 };

            facts.push(InterestingFact {
                category: FactCategory::Performance,
                fact,
                priority,
            });
        }
    }

    facts
}
