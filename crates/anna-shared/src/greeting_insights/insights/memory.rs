//! Memory-related greeting insights.

use crate::greeting_insights::types::GreetingInsight;
use crate::roster::{person_for, Tier};
use crate::snapshot::SystemSnapshot;
use crate::teams::Team;

/// Add memory-related insights to the collection
pub fn add_memory_insights(snapshot: &SystemSnapshot, insights: &mut Vec<GreetingInsight>) {
    if snapshot.memory_total_bytes == 0 {
        return;
    }

    let used_percent =
        (snapshot.memory_used_bytes as f64 / snapshot.memory_total_bytes as f64 * 100.0) as u8;

    if used_percent >= 90 {
        let person = person_for(Team::Performance, Tier::Senior);
        insights.push(GreetingInsight {
            staff_name: person.display_name,
            team: Team::Performance,
            message: format!(
                "memory at {}% - that's 90% efficiency... wait",
                used_percent
            ),
            priority: 90,
            positive: false,
        });
    } else if used_percent >= 80 {
        let person = person_for(Team::Performance, Tier::Junior);
        insights.push(GreetingInsight {
            staff_name: person.display_name,
            team: Team::Performance,
            message: format!("*glances at htop* memory at {}%", used_percent),
            priority: 60,
            positive: false,
        });
    } else if used_percent < 30 {
        let person = person_for(Team::Performance, Tier::Junior);
        insights.push(GreetingInsight {
            staff_name: person.display_name,
            team: Team::Performance,
            message: "numbers look good, no bottlenecks!".to_string(),
            priority: 15,
            positive: true,
        });
    }
}
