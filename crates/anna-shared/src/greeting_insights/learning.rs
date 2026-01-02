//! Learning progress insights (v0.0.335).
//!
//! v0.0.326: Initial implementation.
//! v0.0.335: Enhanced with health status and trends.

use crate::probe_learning::{LearningHealth, ProbeLearningStore, TrendDirection};
use crate::roster::{person_for, Tier};
use crate::teams::Team;

use super::types::GreetingInsight;

/// v0.0.326: Add learning progress insight
/// v0.0.335: Enhanced with health status and trends
pub fn add_learning_insights(insights: &mut Vec<GreetingInsight>) {
    let store = ProbeLearningStore::load();
    let stats = store.learning_stats();

    // Only show if we have meaningful learning data
    if stats.total_queries < 3 {
        return; // Too early to mention learning
    }

    let health = store.health_status();
    let trend = store.quality_trend();

    // v0.0.335: Priority message for declining health
    if health == LearningHealth::NeedsAttention {
        let person = person_for(Team::Desktop, Tier::Senior);
        insights.push(GreetingInsight {
            staff_name: person.display_name,
            team: Team::Desktop,
            message: "learning quality is declining - might need fresh data".to_string(),
            priority: 35, // Higher than normal learning, but lower than system issues
            positive: false,
        });
        return;
    }

    // v0.0.335: Celebrate improving trend
    if let Some(ref t) = trend {
        if t.trend == TrendDirection::Improving && t.change > 0.5 {
            let person = person_for(Team::Desktop, Tier::Junior);
            insights.push(GreetingInsight {
                staff_name: person.display_name,
                team: Team::Desktop,
                message: format!(
                    "answers improving! {:.1} → {:.1}/5",
                    t.previous_avg, t.current_avg
                ),
                priority: 28,
                positive: true,
            });
            return;
        }
    }

    // Show different messages based on learning stage
    let (message, priority) = match health {
        LearningHealth::Excellent => (
            format!(
                "operating at peak learning ({} patterns, {:.1}/5 quality)",
                stats.successful_patterns, stats.avg_quality
            ),
            25,
        ),
        LearningHealth::Good => (
            format!(
                "learning going well - {} keywords from {} queries",
                stats.keywords_learned, stats.total_queries
            ),
            20,
        ),
        LearningHealth::Developing => (
            format!("building knowledge from {} queries", stats.total_queries),
            15,
        ),
        LearningHealth::Insufficient | LearningHealth::NeedsAttention => {
            ("still getting to know your system".to_string(), 10)
        }
    };

    // Use Sofia (Desktop Jr) for learning insights - she's the learner
    let person = person_for(Team::Desktop, Tier::Junior);
    insights.push(GreetingInsight {
        staff_name: person.display_name,
        team: Team::Desktop,
        message,
        priority,
        positive: true,
    });
}
