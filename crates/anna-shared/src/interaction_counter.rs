//! Interaction Counter (v0.0.488).
//!
//! Tracks interactions between Anna and specialists.
//! Provides detailed statistics on communication patterns.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single interaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionRecord {
    /// Unix timestamp
    pub timestamp: u64,
    /// Source (who initiated)
    pub from: String,
    /// Target (who received)
    pub to: String,
    /// Type of interaction
    pub interaction_type: InteractionType,
    /// Ticket ID if applicable
    pub ticket_id: Option<String>,
    /// Duration in ms if applicable
    pub duration_ms: Option<u64>,
}

/// Type of interaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionType {
    /// Initial dispatch to specialist
    Dispatch,
    /// Response from specialist
    Response,
    /// Escalation to senior
    Escalation,
    /// Clarification request
    Clarification,
    /// Follow-up question
    FollowUp,
    /// Final resolution
    Resolution,
}

impl InteractionType {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Dispatch => "Dispatch",
            Self::Response => "Response",
            Self::Escalation => "Escalation",
            Self::Clarification => "Clarification",
            Self::FollowUp => "Follow-up",
            Self::Resolution => "Resolution",
        }
    }
}

impl InteractionRecord {
    /// Create a new interaction record
    pub fn new(from: &str, to: &str, interaction_type: InteractionType, timestamp: u64) -> Self {
        Self {
            timestamp,
            from: from.to_string(),
            to: to.to_string(),
            interaction_type,
            ticket_id: None,
            duration_ms: None,
        }
    }

    /// Set ticket ID
    pub fn with_ticket(mut self, ticket_id: &str) -> Self {
        self.ticket_id = Some(ticket_id.to_string());
        self
    }

    /// Set duration
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }
}

/// Statistics for a specific specialist
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpecialistStats {
    /// Total interactions
    pub total_interactions: u64,
    /// Dispatches received
    pub dispatches: u64,
    /// Responses sent
    pub responses: u64,
    /// Escalations made
    pub escalations: u64,
    /// Clarifications requested
    pub clarifications: u64,
    /// Average response time (ms)
    pub avg_response_ms: f64,
    /// Total response time (ms) for averaging
    total_response_ms: u64,
    response_count: u64,
}

impl SpecialistStats {
    /// Record an interaction
    pub fn record(&mut self, interaction_type: InteractionType, duration_ms: Option<u64>) {
        self.total_interactions += 1;

        match interaction_type {
            InteractionType::Dispatch => self.dispatches += 1,
            InteractionType::Response => {
                self.responses += 1;
                if let Some(ms) = duration_ms {
                    self.total_response_ms += ms;
                    self.response_count += 1;
                    self.avg_response_ms =
                        self.total_response_ms as f64 / self.response_count as f64;
                }
            }
            InteractionType::Escalation => self.escalations += 1,
            InteractionType::Clarification => self.clarifications += 1,
            _ => {}
        }
    }

    /// Get escalation rate
    pub fn escalation_rate(&self) -> f64 {
        if self.dispatches == 0 {
            0.0
        } else {
            self.escalations as f64 / self.dispatches as f64 * 100.0
        }
    }
}

/// Interaction counter tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InteractionCounter {
    /// Total interactions
    pub total_interactions: u64,
    /// Interactions per ticket (for averaging)
    pub interactions_per_ticket: HashMap<String, u64>,
    /// Stats per specialist
    pub by_specialist: HashMap<String, SpecialistStats>,
    /// Stats per interaction type
    pub by_type: HashMap<String, u64>,
    /// Anna solo resolutions (no specialist)
    pub anna_solo_count: u64,
    /// Recent interactions (last 50)
    pub recent: Vec<InteractionRecord>,
}

impl InteractionCounter {
    /// Create new counter
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an interaction
    pub fn record(&mut self, record: InteractionRecord) {
        self.total_interactions += 1;

        // Track by ticket
        if let Some(ticket_id) = &record.ticket_id {
            *self.interactions_per_ticket.entry(ticket_id.clone()).or_default() += 1;
        }

        // Track by specialist (the 'to' field for dispatches, 'from' for responses)
        let specialist = match record.interaction_type {
            InteractionType::Dispatch => &record.to,
            _ => &record.from,
        };

        if specialist != "Anna" && specialist != "User" {
            let stats = self.by_specialist.entry(specialist.clone()).or_default();
            stats.record(record.interaction_type, record.duration_ms);
        }

        // Track by type
        let type_name = record.interaction_type.display_name().to_string();
        *self.by_type.entry(type_name).or_default() += 1;

        // Add to recent
        self.recent.push(record);
        if self.recent.len() > 50 {
            self.recent.remove(0);
        }
    }

    /// Record Anna solving without specialist
    pub fn record_anna_solo(&mut self) {
        self.anna_solo_count += 1;
    }

    /// Get average interactions per ticket
    pub fn average_per_ticket(&self) -> f64 {
        if self.interactions_per_ticket.is_empty() {
            0.0
        } else {
            let total: u64 = self.interactions_per_ticket.values().sum();
            total as f64 / self.interactions_per_ticket.len() as f64
        }
    }

    /// Get total tickets tracked
    pub fn total_tickets(&self) -> usize {
        self.interactions_per_ticket.len()
    }

    /// Get most consulted specialist
    pub fn most_consulted(&self) -> Option<(&str, u64)> {
        self.by_specialist
            .iter()
            .max_by_key(|(_, s)| s.total_interactions)
            .map(|(name, stats)| (name.as_str(), stats.total_interactions))
    }

    /// Get least consulted specialist
    pub fn least_consulted(&self) -> Option<(&str, u64)> {
        self.by_specialist
            .iter()
            .filter(|(_, s)| s.total_interactions > 0)
            .min_by_key(|(_, s)| s.total_interactions)
            .map(|(name, stats)| (name.as_str(), stats.total_interactions))
    }

    /// Get fastest responding specialist
    pub fn fastest_responder(&self) -> Option<(&str, f64)> {
        self.by_specialist
            .iter()
            .filter(|(_, s)| s.avg_response_ms > 0.0)
            .min_by(|a, b| {
                a.1.avg_response_ms
                    .partial_cmp(&b.1.avg_response_ms)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(name, stats)| (name.as_str(), stats.avg_response_ms))
    }

    /// Get Anna's solo resolution rate
    pub fn anna_solo_rate(&self) -> f64 {
        let total_resolutions = self.anna_solo_count + self.total_tickets() as u64;
        if total_resolutions == 0 {
            0.0
        } else {
            self.anna_solo_count as f64 / total_resolutions as f64 * 100.0
        }
    }
}

/// Interaction counter summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionSummary {
    /// Total interactions
    pub total: u64,
    /// Total tickets
    pub tickets: usize,
    /// Average per ticket
    pub avg_per_ticket: f64,
    /// Anna solo count
    pub anna_solo: u64,
    /// Anna solo rate
    pub anna_solo_rate: f64,
    /// Most consulted specialist
    pub most_consulted: Option<String>,
    /// Specialist count
    pub specialist_count: usize,
}

impl InteractionCounter {
    /// Generate summary
    pub fn summary(&self) -> InteractionSummary {
        InteractionSummary {
            total: self.total_interactions,
            tickets: self.total_tickets(),
            avg_per_ticket: self.average_per_ticket(),
            anna_solo: self.anna_solo_count,
            anna_solo_rate: self.anna_solo_rate(),
            most_consulted: self.most_consulted().map(|(n, _)| n.to_string()),
            specialist_count: self.by_specialist.len(),
        }
    }
}

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

    #[test]
    fn test_interaction_record_new() {
        let record = InteractionRecord::new("Anna", "Desktop Admin", InteractionType::Dispatch, 1000);
        assert_eq!(record.from, "Anna");
        assert_eq!(record.to, "Desktop Admin");
    }

    #[test]
    fn test_interaction_with_ticket() {
        let record = InteractionRecord::new("Anna", "Network", InteractionType::Dispatch, 1000)
            .with_ticket("TKT-001");
        assert_eq!(record.ticket_id, Some("TKT-001".to_string()));
    }

    #[test]
    fn test_specialist_stats_record() {
        let mut stats = SpecialistStats::default();

        stats.record(InteractionType::Dispatch, None);
        stats.record(InteractionType::Response, Some(500));
        stats.record(InteractionType::Escalation, None);

        assert_eq!(stats.dispatches, 1);
        assert_eq!(stats.responses, 1);
        assert_eq!(stats.escalations, 1);
        assert_eq!(stats.avg_response_ms, 500.0);
    }

    #[test]
    fn test_escalation_rate() {
        let mut stats = SpecialistStats::default();

        stats.record(InteractionType::Dispatch, None);
        stats.record(InteractionType::Dispatch, None);
        stats.record(InteractionType::Escalation, None);

        assert_eq!(stats.escalation_rate(), 50.0);
    }

    #[test]
    fn test_counter_record() {
        let mut counter = InteractionCounter::new();

        let record = InteractionRecord::new("Anna", "Desktop Admin", InteractionType::Dispatch, 1000)
            .with_ticket("TKT-001");

        counter.record(record);

        assert_eq!(counter.total_interactions, 1);
        assert_eq!(counter.total_tickets(), 1);
    }

    #[test]
    fn test_average_per_ticket() {
        let mut counter = InteractionCounter::new();

        // Ticket 1: 2 interactions
        counter.record(
            InteractionRecord::new("Anna", "Admin", InteractionType::Dispatch, 1000)
                .with_ticket("TKT-001"),
        );
        counter.record(
            InteractionRecord::new("Admin", "Anna", InteractionType::Response, 2000)
                .with_ticket("TKT-001"),
        );

        // Ticket 2: 4 interactions
        for i in 0..4 {
            counter.record(
                InteractionRecord::new("Anna", "Network", InteractionType::Dispatch, 3000 + i * 100)
                    .with_ticket("TKT-002"),
            );
        }

        // Average should be 3 (6 total / 2 tickets)
        assert_eq!(counter.average_per_ticket(), 3.0);
    }

    #[test]
    fn test_anna_solo() {
        let mut counter = InteractionCounter::new();

        counter.record_anna_solo();
        counter.record_anna_solo();

        // Add one ticket with specialist
        counter.record(
            InteractionRecord::new("Anna", "Admin", InteractionType::Dispatch, 1000)
                .with_ticket("TKT-001"),
        );

        // 2 solo / (2 solo + 1 ticket) = 66.67%
        assert!(counter.anna_solo_rate() > 60.0);
    }

    #[test]
    fn test_most_consulted() {
        let mut counter = InteractionCounter::new();

        // Desktop Admin: 3 interactions
        for i in 0..3 {
            counter.record(InteractionRecord::new(
                "Anna",
                "Desktop Admin",
                InteractionType::Dispatch,
                i * 1000,
            ));
        }

        // Network: 1 interaction
        counter.record(InteractionRecord::new(
            "Anna",
            "Network",
            InteractionType::Dispatch,
            5000,
        ));

        let (name, count) = counter.most_consulted().unwrap();
        assert_eq!(name, "Desktop Admin");
        assert_eq!(count, 3);
    }

    #[test]
    fn test_recent_limit() {
        let mut counter = InteractionCounter::new();

        for i in 0..60 {
            counter.record(InteractionRecord::new(
                "Anna",
                "Admin",
                InteractionType::Dispatch,
                i * 1000,
            ));
        }

        assert_eq!(counter.recent.len(), 50);
    }

    #[test]
    fn test_summary() {
        let mut counter = InteractionCounter::new();

        counter.record(InteractionRecord::new(
            "Anna",
            "Admin",
            InteractionType::Dispatch,
            1000,
        ));
        counter.record_anna_solo();

        let summary = counter.summary();
        assert_eq!(summary.total, 1);
        assert_eq!(summary.anna_solo, 1);
    }

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

    #[test]
    fn test_interaction_type_display() {
        assert_eq!(InteractionType::Dispatch.display_name(), "Dispatch");
        assert_eq!(InteractionType::Escalation.display_name(), "Escalation");
    }
}
