// v0.0.529: Escalation Tracker Implementation (Phase 105)
// Main tracker logic for managing escalations

use crate::escalation_tracker::types::{EscalationOutcome, EscalationReason, EscalationRecord};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Escalation tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EscalationTracker {
    records: Vec<EscalationRecord>,
    next_id: u32,
}

impl EscalationTracker {
    /// Create new tracker
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            next_id: 1,
        }
    }

    /// Escalate a ticket
    pub fn escalate(
        &mut self,
        ticket_id: &str,
        from: &str,
        to: &str,
        department: &str,
        reason: EscalationReason,
        timestamp: &str,
    ) -> String {
        let id = format!("ESC-{:04}", self.next_id);
        self.next_id += 1;

        let record = EscalationRecord::new(&id, ticket_id, from, to, department, reason, timestamp);
        self.records.push(record);
        id
    }

    /// Resolve escalation
    pub fn resolve(
        &mut self,
        esc_id: &str,
        outcome: EscalationOutcome,
        timestamp: &str,
        resolution_ms: u64,
    ) {
        if let Some(record) = self.records.iter_mut().find(|r| r.id == esc_id) {
            record.resolve(outcome, timestamp, resolution_ms);
        }
    }

    /// Get escalation by ID
    pub fn get(&self, id: &str) -> Option<&EscalationRecord> {
        self.records.iter().find(|r| r.id == id)
    }

    /// Get pending escalations
    pub fn pending(&self) -> Vec<&EscalationRecord> {
        self.records.iter().filter(|r| r.is_pending()).collect()
    }

    /// Get escalations by reason
    pub fn by_reason(&self, reason: &EscalationReason) -> Vec<&EscalationRecord> {
        self.records.iter().filter(|r| &r.reason == reason).collect()
    }

    /// Get escalations by department
    pub fn by_department(&self, dept: &str) -> Vec<&EscalationRecord> {
        self.records
            .iter()
            .filter(|r| r.department == dept)
            .collect()
    }

    /// Get escalation rate (total escalations / total tickets)
    pub fn escalation_rate(&self, total_tickets: u32) -> f64 {
        if total_tickets == 0 {
            0.0
        } else {
            (self.records.len() as f64 / total_tickets as f64) * 100.0
        }
    }

    /// Get senior resolution rate
    pub fn senior_resolution_rate(&self) -> f64 {
        let resolved: Vec<_> = self
            .records
            .iter()
            .filter(|r| !r.is_pending())
            .collect();

        if resolved.is_empty() {
            return 0.0;
        }

        let senior_resolved = resolved
            .iter()
            .filter(|r| r.outcome == EscalationOutcome::ResolvedBySenior)
            .count();

        (senior_resolved as f64 / resolved.len() as f64) * 100.0
    }

    /// Get average resolution time
    pub fn avg_resolution_ms(&self) -> Option<u64> {
        let resolved: Vec<_> = self
            .records
            .iter()
            .filter_map(|r| r.resolution_ms)
            .collect();

        if resolved.is_empty() {
            None
        } else {
            Some(resolved.iter().sum::<u64>() / resolved.len() as u64)
        }
    }

    /// Stats by reason
    pub fn reason_stats(&self) -> HashMap<EscalationReason, usize> {
        let mut stats = HashMap::new();
        for r in &self.records {
            *stats.entry(r.reason.clone()).or_insert(0) += 1;
        }
        stats
    }

    /// Total escalations
    pub fn total(&self) -> usize {
        self.records.len()
    }

    /// All records
    pub fn all(&self) -> &[EscalationRecord] {
        &self.records
    }
}
