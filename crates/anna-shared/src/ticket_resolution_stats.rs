//! Ticket Resolution Stats - Phase 86
//!
//! Tracks tickets closed by Anna vs specialists.
//! VISION.md: "Track amount of tickets closed by Anna vs specialists"
//! "Anna's ticket count should increase over time"

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Who resolved the ticket
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Resolver {
    #[default]
    Anna,
    Junior,
    Senior,
    Escalated,
    User,
    Unknown,
}

impl Resolver {
    pub fn name(&self) -> &'static str {
        match self {
            Resolver::Anna => "Anna",
            Resolver::Junior => "Junior",
            Resolver::Senior => "Senior",
            Resolver::Escalated => "Escalated",
            Resolver::User => "User",
            Resolver::Unknown => "Unknown",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Resolver::Anna => "A",
            Resolver::Junior => "J",
            Resolver::Senior => "S",
            Resolver::Escalated => "E",
            Resolver::User => "U",
            Resolver::Unknown => "?",
        }
    }

    pub fn is_specialist(&self) -> bool {
        matches!(self, Resolver::Junior | Resolver::Senior | Resolver::Escalated)
    }
}

/// Resolution method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ResolutionMethod {
    #[default]
    Recipe,
    Specialist,
    DirectAnswer,
    UserSelfHelp,
    Escalation,
    Timeout,
}

impl ResolutionMethod {
    pub fn name(&self) -> &'static str {
        match self {
            ResolutionMethod::Recipe => "Recipe",
            ResolutionMethod::Specialist => "Specialist",
            ResolutionMethod::DirectAnswer => "Direct Answer",
            ResolutionMethod::UserSelfHelp => "User Self-Help",
            ResolutionMethod::Escalation => "Escalation",
            ResolutionMethod::Timeout => "Timeout",
        }
    }
}

/// A resolution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionRecord {
    /// Ticket ID
    pub ticket_id: String,
    /// Who resolved it
    pub resolver: Resolver,
    /// Method used
    pub method: ResolutionMethod,
    /// Department/team involved
    pub department: Option<String>,
    /// Specialist name (if specialist)
    pub specialist_name: Option<String>,
    /// Resolution timestamp
    pub resolved_at: u64,
    /// Time to resolution (seconds)
    pub resolution_time_secs: u64,
    /// Was a recipe learned from this?
    pub recipe_learned: bool,
    /// Confidence score
    pub confidence: Option<u8>,
}

/// Ticket resolution statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TicketResolutionStats {
    /// All resolution records
    pub records: Vec<ResolutionRecord>,
    /// Count by resolver
    pub by_resolver: HashMap<String, u64>,
    /// Count by method
    pub by_method: HashMap<String, u64>,
    /// Count by department
    pub by_department: HashMap<String, u64>,
    /// Total Anna resolutions
    pub anna_count: u64,
    /// Total specialist resolutions
    pub specialist_count: u64,
    /// Recipes learned
    pub recipes_learned: u64,
}

impl TicketResolutionStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a resolution
    pub fn record(&mut self, resolution: ResolutionRecord) {
        *self.by_resolver.entry(resolution.resolver.name().to_string()).or_insert(0) += 1;
        *self.by_method.entry(resolution.method.name().to_string()).or_insert(0) += 1;

        if let Some(dept) = &resolution.department {
            *self.by_department.entry(dept.clone()).or_insert(0) += 1;
        }

        if resolution.resolver == Resolver::Anna {
            self.anna_count += 1;
        } else if resolution.resolver.is_specialist() {
            self.specialist_count += 1;
        }

        if resolution.recipe_learned {
            self.recipes_learned += 1;
        }

        self.records.push(resolution);
    }

    /// Get Anna's resolution rate
    pub fn anna_rate(&self) -> f64 {
        let total = self.anna_count + self.specialist_count;
        if total == 0 {
            0.0
        } else {
            (self.anna_count as f64 / total as f64) * 100.0
        }
    }

    /// Get resolutions by resolver
    pub fn by_res(&self, resolver: Resolver) -> Vec<&ResolutionRecord> {
        self.records.iter().filter(|r| r.resolver == resolver).collect()
    }

    /// Get resolutions by method
    pub fn by_res_method(&self, method: ResolutionMethod) -> Vec<&ResolutionRecord> {
        self.records.iter().filter(|r| r.method == method).collect()
    }

    /// Get recent resolutions
    pub fn recent(&self, limit: usize) -> Vec<&ResolutionRecord> {
        self.records.iter().rev().take(limit).collect()
    }

    /// Get recipe resolutions
    pub fn recipe_resolutions(&self) -> Vec<&ResolutionRecord> {
        self.records.iter().filter(|r| r.method == ResolutionMethod::Recipe).collect()
    }

    /// Average resolution time (seconds)
    pub fn avg_resolution_time(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        let total: u64 = self.records.iter().map(|r| r.resolution_time_secs).sum();
        total as f64 / self.records.len() as f64
    }

    /// Fastest resolution
    pub fn fastest_resolution(&self) -> Option<u64> {
        self.records.iter().map(|r| r.resolution_time_secs).min()
    }

    /// Slowest resolution
    pub fn slowest_resolution(&self) -> Option<u64> {
        self.records.iter().map(|r| r.resolution_time_secs).max()
    }

    /// Most active department
    pub fn most_active_department(&self) -> Option<(&str, u64)> {
        self.by_department
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, v)| (k.as_str(), *v))
    }

    /// Total count
    pub fn total_count(&self) -> usize {
        self.records.len()
    }

    /// Anna is improving (higher rate over time)
    pub fn anna_improving(&self) -> bool {
        if self.records.len() < 20 {
            return false;
        }

        let mid = self.records.len() / 2;
        let first_half: Vec<_> = self.records[..mid].iter().collect();
        let second_half: Vec<_> = self.records[mid..].iter().collect();

        let first_anna = first_half.iter().filter(|r| r.resolver == Resolver::Anna).count();
        let second_anna = second_half.iter().filter(|r| r.resolver == Resolver::Anna).count();

        let first_rate = first_anna as f64 / first_half.len() as f64;
        let second_rate = second_anna as f64 / second_half.len() as f64;

        second_rate > first_rate
    }
}

/// Format resolution stats for display
pub fn format_resolution_stats(stats: &TicketResolutionStats) -> String {
    let mut lines = vec!["=== Ticket Resolution Stats ===".to_string()];
    lines.push(String::new());

    if stats.records.is_empty() {
        lines.push("No resolutions yet.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total resolutions: {}", stats.total_count()));
    lines.push(format!("Anna: {} ({:.1}%)", stats.anna_count, stats.anna_rate()));
    lines.push(format!("Specialists: {}", stats.specialist_count));
    lines.push(format!("Recipes learned: {}", stats.recipes_learned));

    // Improvement
    if stats.anna_improving() {
        lines.push("Anna is improving over time!".to_string());
    }

    // Times
    lines.push(String::new());
    lines.push(format!("Avg resolution: {:.1} sec", stats.avg_resolution_time()));
    if let Some(fastest) = stats.fastest_resolution() {
        lines.push(format!("Fastest: {} sec", fastest));
    }
    if let Some(slowest) = stats.slowest_resolution() {
        lines.push(format!("Slowest: {} sec", slowest));
    }

    // By resolver
    if !stats.by_resolver.is_empty() {
        lines.push(String::new());
        lines.push("By resolver:".to_string());
        for (resolver, count) in &stats.by_resolver {
            lines.push(format!("  {}: {}", resolver, count));
        }
    }

    lines.join("\n")
}

/// Format resolution stats compact
pub fn format_resolution_stats_compact(stats: &TicketResolutionStats) -> String {
    format!(
        "Resolutions: {} total | Anna: {:.0}% | {} recipes learned",
        stats.total_count(),
        stats.anna_rate(),
        stats.recipes_learned
    )
}

/// Format resolution stats one-line
pub fn format_resolution_stats_oneline(stats: &TicketResolutionStats) -> String {
    format!(
        "{} resolved (Anna: {})",
        stats.total_count(),
        stats.anna_count
    )
}

/// Check if query is about resolution stats
pub fn is_resolution_stats_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "resolution stats",
        "tickets resolved",
        "who resolved",
        "anna vs specialist",
        "ticket stats",
        "resolution rate",
        "tickets closed",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about resolution stats
pub fn resolution_fun_fact(stats: &TicketResolutionStats) -> String {
    if stats.records.is_empty() {
        return "No ticket resolutions yet!".to_string();
    }

    let facts = [
        format!(
            "Anna has resolved {} tickets on her own!",
            stats.anna_count
        ),
        format!(
            "Anna's resolution rate is {:.1}%.",
            stats.anna_rate()
        ),
        format!(
            "{} recipes were learned from resolutions.",
            stats.recipes_learned
        ),
        format!(
            "Average resolution time is {:.1} seconds.",
            stats.avg_resolution_time()
        ),
        {
            if stats.anna_improving() {
                "Anna is getting better over time!".to_string()
            } else {
                format!("Specialists resolved {} tickets.", stats.specialist_count)
            }
        },
    ];

    facts[stats.total_count() % facts.len()].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_resolution(resolver: Resolver, method: ResolutionMethod) -> ResolutionRecord {
        ResolutionRecord {
            ticket_id: format!("TKT-{:?}", resolver),
            resolver,
            method,
            department: Some("Desktop".to_string()),
            specialist_name: None,
            resolved_at: 1234567890,
            resolution_time_secs: 60,
            recipe_learned: false,
            confidence: Some(85),
        }
    }

    #[test]
    fn test_resolver() {
        assert_eq!(Resolver::Anna.name(), "Anna");
        assert_eq!(Resolver::Junior.symbol(), "J");
        assert!(Resolver::Senior.is_specialist());
        assert!(!Resolver::Anna.is_specialist());
    }

    #[test]
    fn test_resolution_method() {
        assert_eq!(ResolutionMethod::Recipe.name(), "Recipe");
        assert_eq!(ResolutionMethod::Specialist.name(), "Specialist");
    }

    #[test]
    fn test_record_resolution() {
        let mut stats = TicketResolutionStats::new();
        stats.record(make_resolution(Resolver::Anna, ResolutionMethod::Recipe));

        assert_eq!(stats.total_count(), 1);
        assert_eq!(stats.anna_count, 1);
    }

    #[test]
    fn test_anna_rate() {
        let mut stats = TicketResolutionStats::new();
        stats.record(make_resolution(Resolver::Anna, ResolutionMethod::Recipe));
        stats.record(make_resolution(Resolver::Anna, ResolutionMethod::Recipe));
        stats.record(make_resolution(Resolver::Junior, ResolutionMethod::Specialist));

        assert!((stats.anna_rate() - 66.66).abs() < 1.0);
    }

    #[test]
    fn test_by_resolver() {
        let mut stats = TicketResolutionStats::new();
        stats.record(make_resolution(Resolver::Anna, ResolutionMethod::Recipe));
        stats.record(make_resolution(Resolver::Junior, ResolutionMethod::Specialist));

        assert_eq!(stats.by_res(Resolver::Anna).len(), 1);
        assert_eq!(stats.by_res(Resolver::Junior).len(), 1);
    }

    #[test]
    fn test_recipe_learned() {
        let mut stats = TicketResolutionStats::new();
        let mut resolution = make_resolution(Resolver::Anna, ResolutionMethod::Recipe);
        resolution.recipe_learned = true;
        stats.record(resolution);

        assert_eq!(stats.recipes_learned, 1);
    }

    #[test]
    fn test_avg_resolution_time() {
        let mut stats = TicketResolutionStats::new();
        let mut r1 = make_resolution(Resolver::Anna, ResolutionMethod::Recipe);
        r1.resolution_time_secs = 30;
        let mut r2 = make_resolution(Resolver::Anna, ResolutionMethod::Recipe);
        r2.resolution_time_secs = 90;

        stats.record(r1);
        stats.record(r2);

        assert_eq!(stats.avg_resolution_time(), 60.0);
    }

    #[test]
    fn test_format_resolution_stats() {
        let mut stats = TicketResolutionStats::new();
        stats.record(make_resolution(Resolver::Anna, ResolutionMethod::Recipe));

        let output = format_resolution_stats(&stats);
        assert!(output.contains("Resolution Stats"));
        assert!(output.contains("Anna: 1"));
    }

    #[test]
    fn test_is_resolution_stats_query() {
        assert!(is_resolution_stats_query("show resolution stats"));
        assert!(is_resolution_stats_query("how many tickets resolved?"));
        assert!(is_resolution_stats_query("anna vs specialist stats"));
        assert!(!is_resolution_stats_query("what is the weather?"));
    }

    #[test]
    fn test_resolution_fun_fact() {
        let mut stats = TicketResolutionStats::new();
        stats.record(make_resolution(Resolver::Anna, ResolutionMethod::Recipe));

        let fact = resolution_fun_fact(&stats);
        assert!(!fact.is_empty());
    }
}
