//! Fun fact generation from statistics (v0.0.479).

use super::types::FunStats;

/// Generate a fun fact about the stats
pub fn generate_fun_fact(stats: &FunStats) -> Option<String> {
    let mut facts = Vec::new();

    // Streak facts
    if stats.current_streak >= 7 {
        facts.push(format!(
            "You're on a {} day streak! Keep it up!",
            stats.current_streak
        ));
    }

    if stats.best_streak >= 30 {
        facts.push(format!(
            "Your record streak was {} days. Impressive dedication!",
            stats.best_streak
        ));
    }

    // Anna solo facts
    if stats.anna_solo_pct >= 50.0 {
        facts.push(format!(
            "Anna handled {:.0}% of requests solo - quite self-sufficient!",
            stats.anna_solo_pct
        ));
    }

    // Lucky team
    if let Some(team) = &stats.lucky_team {
        if stats.lucky_team_rate >= 0.9 {
            facts.push(format!(
                "The {} team has a {:.0}% success rate - your lucky team!",
                team,
                stats.lucky_team_rate * 100.0
            ));
        }
    }

    // Installation milestone
    if stats.days_active >= 365 {
        let years = stats.days_active / 365;
        facts.push(format!(
            "You've been using Anna for over {} year{}. Thank you for your loyalty!",
            years,
            if years > 1 { "s" } else { "" }
        ));
    } else if stats.days_active >= 30 {
        facts.push(format!(
            "Anna has been helping you for {} days now.",
            stats.days_active
        ));
    }

    // Recipes learned
    if stats.recipes_learned >= 50 {
        facts.push(format!(
            "Anna has learned {} recipes from you. She's becoming quite knowledgeable!",
            stats.recipes_learned
        ));
    }

    // Request milestones
    let milestone = match stats.total_requests {
        r if r >= 10000 => Some(("10,000", "power user")),
        r if r >= 1000 => Some(("1,000", "regular")),
        r if r >= 100 => Some(("100", "getting started")),
        _ => None,
    };

    if let Some((count, label)) = milestone {
        facts.push(format!(
            "You've reached {} requests - you're a {}!",
            count, label
        ));
    }

    if facts.is_empty() {
        None
    } else {
        // Return a pseudo-random fact based on current time
        let idx = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0) as usize)
            % facts.len();
        Some(facts[idx].clone())
    }
}
