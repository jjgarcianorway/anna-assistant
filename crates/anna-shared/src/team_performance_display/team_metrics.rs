//! Team metrics tracking

use serde::{Deserialize, Serialize};

/// Performance metrics for a single team
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamMetrics {
    /// Team identifier
    pub team_id: String,
    /// Total tickets handled
    pub tickets_handled: u64,
    /// Tickets resolved successfully
    pub tickets_resolved: u64,
    /// Tickets escalated (to senior or other team)
    pub tickets_escalated: u64,
    /// Total resolution time in milliseconds
    pub total_resolution_time_ms: u64,
    /// Fastest resolution in ms
    pub fastest_resolution_ms: Option<u64>,
    /// Slowest resolution in ms
    pub slowest_resolution_ms: Option<u64>,
    /// Junior tickets handled
    pub junior_tickets: u64,
    /// Senior tickets handled
    pub senior_tickets: u64,
    /// Current active tickets
    pub active_tickets: u64,
}

impl TeamMetrics {
    /// Create new metrics for a team
    pub fn new(team_id: impl Into<String>) -> Self {
        Self {
            team_id: team_id.into(),
            ..Default::default()
        }
    }

    /// Record a ticket being handled
    pub fn record_ticket(&mut self, resolution_time_ms: u64, resolved: bool, is_senior: bool) {
        self.tickets_handled += 1;

        if resolved {
            self.tickets_resolved += 1;
        }

        if is_senior {
            self.senior_tickets += 1;
        } else {
            self.junior_tickets += 1;
        }

        self.total_resolution_time_ms += resolution_time_ms;

        // Update fastest/slowest
        if resolution_time_ms > 0 {
            match self.fastest_resolution_ms {
                Some(f) if resolution_time_ms < f => {
                    self.fastest_resolution_ms = Some(resolution_time_ms)
                }
                None => self.fastest_resolution_ms = Some(resolution_time_ms),
                _ => {}
            }

            match self.slowest_resolution_ms {
                Some(s) if resolution_time_ms > s => {
                    self.slowest_resolution_ms = Some(resolution_time_ms)
                }
                None => self.slowest_resolution_ms = Some(resolution_time_ms),
                _ => {}
            }
        }
    }

    /// Record an escalation
    pub fn record_escalation(&mut self) {
        self.tickets_escalated += 1;
    }

    /// Success rate (resolved / handled)
    pub fn success_rate(&self) -> f64 {
        if self.tickets_handled == 0 {
            return 0.0;
        }
        (self.tickets_resolved as f64 / self.tickets_handled as f64) * 100.0
    }

    /// Escalation rate (escalated / handled)
    pub fn escalation_rate(&self) -> f64 {
        if self.tickets_handled == 0 {
            return 0.0;
        }
        (self.tickets_escalated as f64 / self.tickets_handled as f64) * 100.0
    }

    /// Average resolution time in milliseconds
    pub fn avg_resolution_ms(&self) -> u64 {
        if self.tickets_resolved == 0 {
            return 0;
        }
        self.total_resolution_time_ms / self.tickets_resolved
    }

    /// Senior/Junior ratio
    pub fn senior_ratio(&self) -> f64 {
        let total = self.junior_tickets + self.senior_tickets;
        if total == 0 {
            return 0.0;
        }
        (self.senior_tickets as f64 / total as f64) * 100.0
    }
}

/// Get performance grade for a team
pub fn team_grade(metrics: &TeamMetrics) -> &'static str {
    let success = metrics.success_rate();
    let escalation = metrics.escalation_rate();

    if metrics.tickets_handled < 3 {
        return "N/A";
    }

    if success >= 95.0 && escalation <= 10.0 {
        "A+"
    } else if success >= 90.0 && escalation <= 20.0 {
        "A"
    } else if success >= 80.0 && escalation <= 30.0 {
        "B"
    } else if success >= 70.0 {
        "C"
    } else if success >= 50.0 {
        "D"
    } else {
        "F"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_metrics_new() {
        let metrics = TeamMetrics::new("Desktop");
        assert_eq!(metrics.team_id, "Desktop");
        assert_eq!(metrics.tickets_handled, 0);
    }

    #[test]
    fn test_team_metrics_record_ticket() {
        let mut metrics = TeamMetrics::new("Desktop");
        metrics.record_ticket(5000, true, false);

        assert_eq!(metrics.tickets_handled, 1);
        assert_eq!(metrics.tickets_resolved, 1);
        assert_eq!(metrics.junior_tickets, 1);
        assert_eq!(metrics.fastest_resolution_ms, Some(5000));
    }

    #[test]
    fn test_team_metrics_success_rate() {
        let mut metrics = TeamMetrics::new("Desktop");
        metrics.record_ticket(1000, true, false);
        metrics.record_ticket(1000, true, false);
        metrics.record_ticket(1000, false, false);

        // 2 resolved out of 3 = 66.67%
        assert!((metrics.success_rate() - 66.67).abs() < 1.0);
    }

    #[test]
    fn test_team_metrics_avg_resolution() {
        let mut metrics = TeamMetrics::new("Desktop");
        metrics.record_ticket(1000, true, false);
        metrics.record_ticket(3000, true, false);

        // Average of 1000 and 3000 = 2000
        assert_eq!(metrics.avg_resolution_ms(), 2000);
    }

    #[test]
    fn test_team_grade() {
        let mut excellent = TeamMetrics::new("Test");
        for _ in 0..10 {
            excellent.record_ticket(1000, true, false);
        }
        assert_eq!(team_grade(&excellent), "A+");

        let mut poor = TeamMetrics::new("Test2");
        for _ in 0..5 {
            poor.record_ticket(1000, true, false);
        }
        for _ in 0..5 {
            poor.record_ticket(1000, false, false);
        }
        assert_eq!(team_grade(&poor), "D");
    }
}
