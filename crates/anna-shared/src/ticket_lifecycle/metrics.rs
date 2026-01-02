//! Reliability and specialist metrics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::errors::InternalError;
use super::record::TicketRecord;
use super::states::TicketResolution;

/// Honest reliability metrics computed from ticket records
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReliabilityMetrics {
    /// Total tickets (all created)
    pub total_tickets: usize,
    /// Resolved with full success
    pub resolved_success: usize,
    /// Resolved with partial answer
    pub resolved_partial: usize,
    /// Resolved with honest "I don't know"
    pub honest_unknown: usize,
    /// Resolved with "unsupported" routing
    pub resolved_unsupported: usize,
    /// Failed (hard internal failures)
    pub failed: usize,
    /// Cancelled by user
    pub cancelled: usize,
    /// Escalated tickets
    pub escalated: usize,
    /// Parse errors specifically
    pub parse_errors: usize,
    /// Timeout errors specifically
    pub timeout_errors: usize,
    /// Internal errors (crashes, etc)
    pub internal_errors: usize,
    /// Average response time (ms)
    pub avg_response_ms: u64,
    /// Success rate: resolved_success / total_tickets
    pub success_rate: f32,
    /// Reliability rate: resolved_success / (resolved_success + failed + parse_errors + internal_errors)
    pub reliability_rate: f32,
}

impl ReliabilityMetrics {
    /// Compute metrics from ticket records
    pub fn compute(tickets: &[TicketRecord]) -> Self {
        let mut metrics = Self::default();

        if tickets.is_empty() {
            return metrics;
        }

        let mut total_latency = 0u64;
        let mut latency_count = 0usize;

        for ticket in tickets {
            // Skip legacy tickets from strict metrics
            if ticket.is_legacy {
                continue;
            }

            metrics.total_tickets += 1;

            let resolution = ticket.resolution();
            match resolution {
                TicketResolution::ResolvedSuccess => metrics.resolved_success += 1,
                TicketResolution::ResolvedPartial => metrics.resolved_partial += 1,
                TicketResolution::ResolvedHonestUnknown => metrics.honest_unknown += 1,
                TicketResolution::ResolvedUnsupported => metrics.resolved_unsupported += 1,
                TicketResolution::Failed => metrics.failed += 1,
                TicketResolution::Cancelled => metrics.cancelled += 1,
                TicketResolution::Pending => {} // Not counted
            }

            // Count error types
            if let Some(ref error) = ticket.internal_error {
                match error {
                    InternalError::ParseError { .. } => metrics.parse_errors += 1,
                    InternalError::Timeout { .. } => metrics.timeout_errors += 1,
                    InternalError::ProbeFailure { .. } | InternalError::InternalCrash { .. } => {
                        metrics.internal_errors += 1;
                    }
                }
            }

            if ticket.escalated {
                metrics.escalated += 1;
            }

            // Latency for terminal tickets
            if ticket.is_terminal() && ticket.latency_ms > 0 {
                total_latency += ticket.latency_ms;
                latency_count += 1;
            }
        }

        // Calculate averages
        if latency_count > 0 {
            metrics.avg_response_ms = total_latency / latency_count as u64;
        }

        // Calculate rates
        if metrics.total_tickets > 0 {
            metrics.success_rate =
                (metrics.resolved_success as f32 / metrics.total_tickets as f32) * 100.0;

            // Reliability rate: success / (success + all failures)
            let failure_pool = metrics.resolved_success
                + metrics.failed
                + metrics.parse_errors
                + metrics.internal_errors;
            if failure_pool > 0 {
                metrics.reliability_rate =
                    (metrics.resolved_success as f32 / failure_pool as f32) * 100.0;
            }
        }

        metrics
    }
}

impl std::fmt::Display for ReliabilityMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[service desk]")?;
        writeln!(f, "  total_tickets         {}", self.total_tickets)?;
        writeln!(f, "  resolved_success      {}", self.resolved_success)?;
        writeln!(f, "  resolved_partial      {}", self.resolved_partial)?;
        writeln!(f, "  honest_unknown        {}", self.honest_unknown)?;
        writeln!(f, "  failed                {}", self.failed)?;
        writeln!(f, "  escalated             {}", self.escalated)?;
        writeln!(
            f,
            "  avg_response          {:.1}s",
            self.avg_response_ms as f64 / 1000.0
        )?;
        writeln!(f)?;
        writeln!(f, "[reliability]")?;
        writeln!(f, "  success_rate          {:.0}%", self.success_rate)?;
        writeln!(f, "  reliability_rate      {:.0}%", self.reliability_rate)?;
        writeln!(f, "  parse_errors          {}", self.parse_errors)?;
        writeln!(f, "  timeout_errors        {}", self.timeout_errors)?;
        writeln!(f, "  internal_errors       {}", self.internal_errors)?;
        Ok(())
    }
}

/// Per-specialist metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpecialistMetrics {
    /// Specialist identifier (e.g., "desktop.junior")
    pub specialist_id: String,
    /// All tickets where specialist appears in chain
    pub tickets_handled: usize,
    /// Tickets where specialist is lead (final in chain)
    pub tickets_lead: usize,
    /// Success count (as lead)
    pub success_count: usize,
    /// Partial count (as lead)
    pub partial_count: usize,
    /// Honest unknown count (as lead)
    pub honest_unknown_count: usize,
    /// Failed count (as lead)
    pub failed_count: usize,
    /// Total XP earned
    pub xp: i64,
    /// Success rate as lead
    pub success_rate: f32,
}

impl SpecialistMetrics {
    /// Get title based on XP and success rate
    pub fn title(&self) -> &'static str {
        // Require minimum success rate for promotion
        let rate = self.success_rate / 100.0;

        if self.xp >= 2000 && rate >= 0.85 {
            "Senior"
        } else if self.xp >= 500 && rate >= 0.70 {
            "Proficient"
        } else {
            "Apprentice"
        }
    }
}

/// Compute per-specialist metrics from ticket records
pub fn compute_specialist_metrics(tickets: &[TicketRecord]) -> HashMap<String, SpecialistMetrics> {
    let mut metrics: HashMap<String, SpecialistMetrics> = HashMap::new();

    for ticket in tickets {
        if ticket.is_legacy {
            continue;
        }

        let resolution = ticket.resolution();

        // Track all specialists in chain
        for specialist in &ticket.escalation_chain {
            let m = metrics
                .entry(specialist.clone())
                .or_insert_with(|| SpecialistMetrics {
                    specialist_id: specialist.clone(),
                    ..Default::default()
                });
            m.tickets_handled += 1;
        }

        // Track lead specialist
        if let Some(lead) = ticket.lead_specialist() {
            let m = metrics
                .entry(lead.to_string())
                .or_insert_with(|| SpecialistMetrics {
                    specialist_id: lead.to_string(),
                    ..Default::default()
                });
            m.tickets_lead += 1;

            // Count by resolution
            match resolution {
                TicketResolution::ResolvedSuccess => {
                    m.success_count += 1;
                    m.xp += resolution.xp_value() as i64;
                }
                TicketResolution::ResolvedPartial => {
                    m.partial_count += 1;
                    m.xp += resolution.xp_value() as i64;
                }
                TicketResolution::ResolvedHonestUnknown | TicketResolution::ResolvedUnsupported => {
                    m.honest_unknown_count += 1;
                    m.xp += resolution.xp_value() as i64;
                }
                TicketResolution::Failed => {
                    m.failed_count += 1;
                    // No XP for failures
                }
                _ => {}
            }
        }
    }

    // Calculate success rates
    for m in metrics.values_mut() {
        if m.tickets_lead > 0 {
            m.success_rate = (m.success_count as f32 / m.tickets_lead as f32) * 100.0;
        }
    }

    metrics
}

/// Format specialist roster for display
pub fn format_specialist_roster(metrics: &HashMap<String, SpecialistMetrics>) -> String {
    let mut output = String::new();
    output.push_str("[staff roster]\n");

    // Group by department
    let mut by_dept: HashMap<String, Vec<&SpecialistMetrics>> = HashMap::new();
    for m in metrics.values() {
        let dept = m
            .specialist_id
            .split('.')
            .next()
            .unwrap_or("unknown")
            .to_uppercase();
        by_dept.entry(dept).or_default().push(m);
    }

    // Sort departments
    let mut depts: Vec<_> = by_dept.keys().collect();
    depts.sort();

    for dept in depts {
        output.push_str(&format!("  {}\n", dept));
        if let Some(staff) = by_dept.get(dept) {
            let mut sorted_staff: Vec<_> = staff.iter().collect();
            sorted_staff.sort_by(|a, b| b.xp.cmp(&a.xp));

            for m in sorted_staff {
                let name = m
                    .specialist_id
                    .split('.')
                    .last()
                    .unwrap_or(&m.specialist_id);
                let title = m.title();
                output.push_str(&format!(
                    "    {} ({})    tickets: {:3}   lead: {:3}   success: {:3}   failed: {:2}   honest_unknown: {:2}   rate: {:3.0}%   {}\n",
                    name, if m.specialist_id.contains("senior") { "Sr" } else { "Jr" },
                    m.tickets_handled, m.tickets_lead, m.success_count, m.failed_count,
                    m.honest_unknown_count, m.success_rate, title
                ));
            }
        }
    }

    output
}
