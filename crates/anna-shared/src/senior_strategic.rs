//! Senior strategic thinking during idle time (v0.0.458).
//!
//! During idle time, senior specialists think strategically:
//! - Analyze patterns in past tickets
//! - Identify recurring issues
//! - Suggest preventive measures
//! - Generate weekly insights
//!
//! v0.0.458: Initial implementation per VISION.md Phase 37.

use crate::teams::Team;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A strategic insight from a senior specialist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategicInsight {
    /// Unique insight ID
    pub id: String,
    /// Team the insight is from
    pub team: Team,
    /// Senior specialist who generated it
    pub specialist: String,
    /// Category of insight
    pub category: InsightCategory,
    /// Title/summary
    pub title: String,
    /// Detailed analysis
    pub analysis: String,
    /// Recommended actions
    pub recommendations: Vec<String>,
    /// Priority level
    pub priority: InsightPriority,
    /// When generated (unix timestamp)
    pub generated_at: u64,
    /// Based on how many tickets
    pub ticket_count: usize,
    /// Time period analyzed (days)
    pub period_days: u32,
}

/// Categories of strategic insights
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InsightCategory {
    /// Recurring pattern detected
    RecurringPattern,
    /// Performance degradation trend
    PerformanceTrend,
    /// Security concern
    SecurityConcern,
    /// Maintenance suggestion
    MaintenanceSuggestion,
    /// Configuration opportunity
    ConfigurationOpportunity,
    /// Capacity planning
    CapacityPlanning,
}

impl InsightCategory {
    pub fn display(&self) -> &'static str {
        match self {
            Self::RecurringPattern => "Recurring Pattern",
            Self::PerformanceTrend => "Performance Trend",
            Self::SecurityConcern => "Security Concern",
            Self::MaintenanceSuggestion => "Maintenance Suggestion",
            Self::ConfigurationOpportunity => "Configuration Opportunity",
            Self::CapacityPlanning => "Capacity Planning",
        }
    }
}

/// Priority of insights
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InsightPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl InsightPriority {
    pub fn display(&self) -> &'static str {
        match self {
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
            Self::Critical => "CRITICAL",
        }
    }
}

/// Strategic analysis session state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StrategicSession {
    /// Session ID
    pub session_id: String,
    /// Started at (unix timestamp)
    pub started_at: u64,
    /// Completed at (if finished)
    pub completed_at: Option<u64>,
    /// Insights generated
    pub insights: Vec<StrategicInsight>,
    /// Tickets analyzed
    pub tickets_analyzed: usize,
    /// Time range analyzed (days)
    pub days_analyzed: u32,
    /// Can be resumed if interrupted
    pub resumable: bool,
    /// Progress checkpoint (for resuming)
    pub checkpoint: Option<AnalysisCheckpoint>,
}

/// Checkpoint for resuming interrupted analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisCheckpoint {
    /// Last ticket ID processed
    pub last_ticket_id: String,
    /// Current team being analyzed
    pub current_team: Team,
    /// Partial insights collected
    pub partial_insights: Vec<StrategicInsight>,
}

impl StrategicSession {
    /// Create new session
    pub fn new() -> Self {
        Self {
            session_id: generate_session_id(),
            started_at: now_timestamp(),
            completed_at: None,
            insights: vec![],
            tickets_analyzed: 0,
            days_analyzed: 7, // Default to 7 days
            resumable: true,
            checkpoint: None,
        }
    }

    /// Mark as completed
    pub fn complete(&mut self) {
        self.completed_at = Some(now_timestamp());
        self.checkpoint = None;
    }

    /// Create checkpoint for resuming
    pub fn create_checkpoint(&mut self, last_ticket: &str, team: Team) {
        self.checkpoint = Some(AnalysisCheckpoint {
            last_ticket_id: last_ticket.to_string(),
            current_team: team,
            partial_insights: self.insights.clone(),
        });
    }

    /// Resume from checkpoint
    pub fn resume(&mut self) -> Option<&AnalysisCheckpoint> {
        self.checkpoint.as_ref()
    }

    /// Add insight
    pub fn add_insight(&mut self, insight: StrategicInsight) {
        self.insights.push(insight);
    }

    /// Is session complete?
    pub fn is_complete(&self) -> bool {
        self.completed_at.is_some()
    }

    /// Get duration in seconds
    pub fn duration_secs(&self) -> u64 {
        let end = self.completed_at.unwrap_or_else(now_timestamp);
        end.saturating_sub(self.started_at)
    }
}

/// Pattern detection for strategic analysis
pub struct PatternDetector;

impl PatternDetector {
    /// Detect recurring patterns in ticket data
    pub fn detect_patterns(ticket_summaries: &[TicketSummary]) -> Vec<DetectedPattern> {
        let mut patterns = vec![];

        // Group by team
        let by_team = group_by_team(ticket_summaries);

        for (team, tickets) in by_team {
            // Look for repeated query patterns
            let query_freq = query_frequency(&tickets);
            for (query_type, count) in query_freq {
                if count >= 3 {
                    patterns.push(DetectedPattern {
                        team,
                        pattern_type: query_type,
                        occurrence_count: count,
                        first_seen: tickets.first().map(|t| t.created_at).unwrap_or(0),
                        last_seen: tickets.last().map(|t| t.created_at).unwrap_or(0),
                    });
                }
            }
        }

        patterns
    }

    /// Generate insights from patterns
    pub fn patterns_to_insights(
        patterns: Vec<DetectedPattern>,
        specialist: &str,
    ) -> Vec<StrategicInsight> {
        patterns
            .into_iter()
            .map(|p| {
                let (category, title, analysis, recommendations) = insight_for_pattern(&p);

                StrategicInsight {
                    id: generate_insight_id(),
                    team: p.team,
                    specialist: specialist.to_string(),
                    category,
                    title,
                    analysis,
                    recommendations,
                    priority: priority_for_count(p.occurrence_count),
                    generated_at: now_timestamp(),
                    ticket_count: p.occurrence_count,
                    period_days: days_between(p.first_seen, p.last_seen),
                }
            })
            .collect()
    }
}

/// Summary of a ticket for analysis
#[derive(Debug, Clone)]
pub struct TicketSummary {
    pub ticket_id: String,
    pub team: Team,
    pub query_type: String,
    pub resolution_success: bool,
    pub created_at: u64,
    pub resolved_at: Option<u64>,
}

/// A detected pattern
#[derive(Debug, Clone)]
pub struct DetectedPattern {
    pub team: Team,
    pub pattern_type: String,
    pub occurrence_count: usize,
    pub first_seen: u64,
    pub last_seen: u64,
}

fn group_by_team(tickets: &[TicketSummary]) -> HashMap<Team, Vec<&TicketSummary>> {
    let mut by_team: HashMap<Team, Vec<&TicketSummary>> = HashMap::new();
    for ticket in tickets {
        by_team.entry(ticket.team).or_default().push(ticket);
    }
    by_team
}

fn query_frequency(tickets: &[&TicketSummary]) -> HashMap<String, usize> {
    let mut freq = HashMap::new();
    for ticket in tickets {
        *freq.entry(ticket.query_type.clone()).or_insert(0) += 1;
    }
    freq
}

fn insight_for_pattern(
    pattern: &DetectedPattern,
) -> (InsightCategory, String, String, Vec<String>) {
    let category = match pattern.pattern_type.as_str() {
        "disk_space" | "storage" => InsightCategory::CapacityPlanning,
        "service_failed" | "systemd" => InsightCategory::MaintenanceSuggestion,
        "performance" | "slow" => InsightCategory::PerformanceTrend,
        "security" | "permission" => InsightCategory::SecurityConcern,
        _ => InsightCategory::RecurringPattern,
    };

    let title = format!(
        "{} pattern: {} ({} occurrences)",
        pattern.team, pattern.pattern_type, pattern.occurrence_count
    );

    let analysis = format!(
        "The {} team has seen {} tickets about '{}' in the last {} days. \
         This recurring pattern suggests a systematic issue that may benefit from proactive resolution.",
        pattern.team,
        pattern.occurrence_count,
        pattern.pattern_type,
        days_between(pattern.first_seen, pattern.last_seen)
    );

    let recommendations = vec![
        format!(
            "Consider creating a scheduled check for {} issues",
            pattern.pattern_type
        ),
        "Review system configuration for root causes".to_string(),
        "Add monitoring for early detection".to_string(),
    ];

    (category, title, analysis, recommendations)
}

fn priority_for_count(count: usize) -> InsightPriority {
    match count {
        0..=2 => InsightPriority::Low,
        3..=5 => InsightPriority::Medium,
        6..=10 => InsightPriority::High,
        _ => InsightPriority::Critical,
    }
}

fn days_between(start: u64, end: u64) -> u32 {
    ((end.saturating_sub(start)) / 86400) as u32
}

fn generate_session_id() -> String {
    format!(
        "SESS-{}",
        uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
    )
}

fn generate_insight_id() -> String {
    format!(
        "INS-{}",
        uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
    )
}

fn now_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Format insights for email notification
pub fn format_insights_email(session: &StrategicSession) -> String {
    let mut output = String::new();

    output.push_str("Weekly Strategic Analysis Report\n");
    output.push_str("================================\n\n");

    output.push_str(&format!(
        "Analysis period: {} days\n",
        session.days_analyzed
    ));
    output.push_str(&format!(
        "Tickets analyzed: {}\n",
        session.tickets_analyzed
    ));
    output.push_str(&format!(
        "Insights generated: {}\n\n",
        session.insights.len()
    ));

    // Group by priority
    let mut by_priority: HashMap<InsightPriority, Vec<&StrategicInsight>> = HashMap::new();
    for insight in &session.insights {
        by_priority.entry(insight.priority).or_default().push(insight);
    }

    for priority in [
        InsightPriority::Critical,
        InsightPriority::High,
        InsightPriority::Medium,
        InsightPriority::Low,
    ] {
        if let Some(insights) = by_priority.get(&priority) {
            output.push_str(&format!("\n[{}]\n", priority.display()));
            for insight in insights {
                output.push_str(&format!("\n• {}\n", insight.title));
                output.push_str(&format!("  Category: {}\n", insight.category.display()));
                output.push_str(&format!("  Team: {}\n", insight.team));
                output.push_str(&format!("  Analysis: {}\n", insight.analysis));
                output.push_str("  Recommendations:\n");
                for rec in &insight.recommendations {
                    output.push_str(&format!("    - {}\n", rec));
                }
            }
        }
    }

    output.push_str("\n--\nAnna Service Desk - Senior Strategic Analysis\n");

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = StrategicSession::new();
        assert!(session.session_id.starts_with("SESS-"));
        assert!(!session.is_complete());
    }

    #[test]
    fn test_session_completion() {
        let mut session = StrategicSession::new();
        session.complete();
        assert!(session.is_complete());
        assert!(session.completed_at.is_some());
    }

    #[test]
    fn test_checkpoint_resume() {
        let mut session = StrategicSession::new();
        session.create_checkpoint("TKT-001", Team::Storage);

        let checkpoint = session.resume().unwrap();
        assert_eq!(checkpoint.last_ticket_id, "TKT-001");
        assert_eq!(checkpoint.current_team, Team::Storage);
    }

    #[test]
    fn test_pattern_detection() {
        let tickets = vec![
            TicketSummary {
                ticket_id: "1".to_string(),
                team: Team::Storage,
                query_type: "disk_space".to_string(),
                resolution_success: true,
                created_at: 1000,
                resolved_at: Some(1100),
            },
            TicketSummary {
                ticket_id: "2".to_string(),
                team: Team::Storage,
                query_type: "disk_space".to_string(),
                resolution_success: true,
                created_at: 2000,
                resolved_at: Some(2100),
            },
            TicketSummary {
                ticket_id: "3".to_string(),
                team: Team::Storage,
                query_type: "disk_space".to_string(),
                resolution_success: true,
                created_at: 3000,
                resolved_at: Some(3100),
            },
        ];

        let patterns = PatternDetector::detect_patterns(&tickets);
        assert!(!patterns.is_empty());
        assert_eq!(patterns[0].pattern_type, "disk_space");
        assert_eq!(patterns[0].occurrence_count, 3);
    }

    #[test]
    fn test_priority_for_count() {
        assert_eq!(priority_for_count(1), InsightPriority::Low);
        assert_eq!(priority_for_count(4), InsightPriority::Medium);
        assert_eq!(priority_for_count(8), InsightPriority::High);
        assert_eq!(priority_for_count(15), InsightPriority::Critical);
    }

    #[test]
    fn test_email_formatting() {
        let mut session = StrategicSession::new();
        session.tickets_analyzed = 50;
        session.add_insight(StrategicInsight {
            id: "INS-001".to_string(),
            team: Team::Storage,
            specialist: "Eva".to_string(),
            category: InsightCategory::CapacityPlanning,
            title: "Disk space pattern".to_string(),
            analysis: "Recurring disk space queries".to_string(),
            recommendations: vec!["Add monitoring".to_string()],
            priority: InsightPriority::High,
            generated_at: 1000,
            ticket_count: 5,
            period_days: 7,
        });

        let email = format_insights_email(&session);
        assert!(email.contains("Weekly Strategic Analysis"));
        assert!(email.contains("Disk space pattern"));
        assert!(email.contains("HIGH"));
    }
}
