//! Formatting and display functions for interaction statistics.

use super::counter::InteractionCounter;

/// Format interaction stats for display
pub fn format_interactions(counter: &InteractionCounter) -> String {
    let mut output = String::new();

    output.push_str("Interaction Statistics\n");
    output.push_str("══════════════════════════════════════\n\n");

    if counter.total_interactions == 0 {
        output.push_str("No interactions recorded yet.\n");
        return output;
    }

    output.push_str(&format!(
        "Total Interactions: {}\n",
        counter.total_interactions
    ));
    output.push_str(&format!(
        "Tickets Processed:  {}\n",
        counter.total_tickets()
    ));
    output.push_str(&format!(
        "Avg per Ticket:     {:.1}\n",
        counter.average_per_ticket()
    ));
    output.push_str(&format!(
        "Anna Solo Rate:     {:.1}%\n\n",
        counter.anna_solo_rate()
    ));

    if !counter.by_specialist.is_empty() {
        output.push_str("By Specialist:\n");
        let mut specialists: Vec<_> = counter.by_specialist.iter().collect();
        specialists.sort_by(|a, b| b.1.total_interactions.cmp(&a.1.total_interactions));

        for (name, stats) in specialists.iter().take(5) {
            output.push_str(&format!(
                "  {} - {} interactions ({} dispatches, {:.1}% escalation)\n",
                name,
                stats.total_interactions,
                stats.dispatches,
                stats.escalation_rate()
            ));
        }
    }

    output
}

/// Format compact interaction info
pub fn format_interactions_compact(counter: &InteractionCounter) -> String {
    if counter.total_interactions == 0 {
        return "No interactions yet".to_string();
    }

    let most = counter
        .most_consulted()
        .map(|(n, _)| n)
        .unwrap_or("none");

    format!(
        "{} interactions, {:.1} avg/ticket, {:.0}% Anna solo, top: {}",
        counter.total_interactions,
        counter.average_per_ticket(),
        counter.anna_solo_rate(),
        most
    )
}

/// Generate fun fact about interactions
pub fn interaction_fun_fact(counter: &InteractionCounter) -> Option<String> {
    if counter.total_interactions < 5 {
        return None;
    }

    let facts = vec![
        format!(
            "Anna handles {:.0}% of requests solo - {}!",
            counter.anna_solo_rate(),
            if counter.anna_solo_rate() > 50.0 {
                "she's learning fast"
            } else {
                "teamwork makes the dream work"
            }
        ),
        format!(
            "Average ticket needs {:.1} interactions - {}!",
            counter.average_per_ticket(),
            if counter.average_per_ticket() < 3.0 {
                "efficient communication"
            } else {
                "thorough investigation"
            }
        ),
        counter
            .most_consulted()
            .map(|(name, count)| {
                format!(
                    "{} is the go-to expert with {} consultations!",
                    name, count
                )
            })
            .unwrap_or_else(|| "The team works well together!".to_string()),
        format!(
            "{} specialists have been consulted across {} tickets!",
            counter.by_specialist.len(),
            counter.total_tickets()
        ),
    ];

    let index = (counter.total_interactions as usize) % facts.len();
    Some(facts[index].clone())
}

/// Check if query is asking about interactions
pub fn is_interaction_query(query: &str) -> bool {
    let lower = query.to_lowercase();

    let patterns = [
        "interaction",
        "specialist",
        "consulted",
        "team communication",
        "escalation",
        "anna solo",
        "who helped",
    ];

    patterns.iter().any(|p| lower.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interaction_counter::{InteractionRecord, InteractionType};

    #[test]
    fn test_format_compact() {
        let mut counter = InteractionCounter::new();

        counter.record(
            InteractionRecord::new("Anna", "Admin", InteractionType::Dispatch, 1000)
                .with_ticket("TKT-001"),
        );

        let output = format_interactions_compact(&counter);
        assert!(output.contains("1 interactions"));
    }

    #[test]
    fn test_fun_fact() {
        let mut counter = InteractionCounter::new();

        for i in 0..10 {
            counter.record(
                InteractionRecord::new("Anna", "Admin", InteractionType::Dispatch, i * 1000)
                    .with_ticket(&format!("TKT-{:03}", i)),
            );
        }

        let fact = interaction_fun_fact(&counter);
        assert!(fact.is_some());
    }

    #[test]
    fn test_is_interaction_query() {
        assert!(is_interaction_query("show interaction stats"));
        assert!(is_interaction_query("who is the most consulted specialist"));
        assert!(is_interaction_query("anna solo rate"));

        assert!(!is_interaction_query("install vim"));
        assert!(!is_interaction_query("status"));
    }
}
