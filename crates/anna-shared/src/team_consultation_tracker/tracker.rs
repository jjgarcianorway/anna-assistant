// v0.0.539: Team Consultation Tracker - Main Tracker
// Tracks team consultations and provides analytics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::record::ConsultationRecord;
use super::types::{ConsultationOutcome, SeniorityConsulted, TeamDepartment};

/// Team consultation tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamConsultationTracker {
    consultations: HashMap<String, ConsultationRecord>,
    next_id: u64,
}

impl Default for TeamConsultationTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl TeamConsultationTracker {
    /// Create new tracker
    pub fn new() -> Self {
        Self {
            consultations: HashMap::new(),
            next_id: 1,
        }
    }

    /// Record consultation
    pub fn consult(&mut self, department: TeamDepartment) -> String {
        let id = format!("TC{:05}", self.next_id);
        self.next_id += 1;

        let record = ConsultationRecord::new(&id, department);
        self.consultations.insert(id.clone(), record);
        id
    }

    /// Record with seniority
    pub fn consult_with_seniority(
        &mut self,
        department: TeamDepartment,
        seniority: SeniorityConsulted,
    ) -> String {
        let id = format!("TC{:05}", self.next_id);
        self.next_id += 1;

        let record = ConsultationRecord::new(&id, department).with_seniority(seniority);
        self.consultations.insert(id.clone(), record);
        id
    }

    /// Get record by ID
    pub fn get(&self, id: &str) -> Option<&ConsultationRecord> {
        self.consultations.get(id)
    }

    /// Get mutable record
    pub fn get_mut(&mut self, id: &str) -> Option<&mut ConsultationRecord> {
        self.consultations.get_mut(id)
    }

    /// Mark resolved
    pub fn resolve(&mut self, id: &str, duration_ms: u64) {
        if let Some(c) = self.consultations.get_mut(id) {
            c.outcome = ConsultationOutcome::Resolved;
            c.duration_ms = Some(duration_ms);
        }
    }

    /// Mark escalated
    pub fn escalate(&mut self, id: &str) {
        if let Some(c) = self.consultations.get_mut(id) {
            c.outcome = ConsultationOutcome::Escalated;
            c.seniority = SeniorityConsulted::Both;
        }
    }

    /// Add interaction to consultation
    pub fn add_interaction(&mut self, id: &str) {
        if let Some(c) = self.consultations.get_mut(id) {
            c.add_interaction();
        }
    }

    /// Get most consulted team
    pub fn most_consulted(&self) -> Option<TeamDepartment> {
        self.department_stats()
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(dept, _)| dept)
    }

    /// Department consultation stats
    pub fn department_stats(&self) -> Vec<(TeamDepartment, u32)> {
        let mut counts: HashMap<TeamDepartment, u32> = HashMap::new();
        for c in self.consultations.values() {
            *counts.entry(c.department.clone()).or_default() += 1;
        }

        let mut stats: Vec<_> = counts.into_iter().collect();
        stats.sort_by(|a, b| b.1.cmp(&a.1));
        stats
    }

    /// Seniority stats
    pub fn seniority_stats(&self) -> HashMap<SeniorityConsulted, u32> {
        let mut counts = HashMap::new();
        for c in self.consultations.values() {
            *counts.entry(c.seniority).or_default() += 1;
        }
        counts
    }

    /// Outcome stats
    pub fn outcome_stats(&self) -> HashMap<ConsultationOutcome, u32> {
        let mut counts = HashMap::new();
        for c in self.consultations.values() {
            *counts.entry(c.outcome).or_default() += 1;
        }
        counts
    }

    /// Average interaction count
    pub fn average_interactions(&self) -> Option<f64> {
        if self.consultations.is_empty() {
            return None;
        }
        let sum: u32 = self.consultations.values().map(|c| c.interaction_count).sum();
        Some(sum as f64 / self.consultations.len() as f64)
    }

    /// Get by department
    pub fn by_department(&self, dept: &TeamDepartment) -> Vec<&ConsultationRecord> {
        self.consultations.values().filter(|c| &c.department == dept).collect()
    }

    /// Get by seniority
    pub fn by_seniority(&self, seniority: SeniorityConsulted) -> Vec<&ConsultationRecord> {
        self.consultations.values().filter(|c| c.seniority == seniority).collect()
    }

    /// Recent consultations
    pub fn recent(&self, limit: usize) -> Vec<&ConsultationRecord> {
        let mut records: Vec<_> = self.consultations.values().collect();
        records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        records.into_iter().take(limit).collect()
    }

    /// Total count
    pub fn total(&self) -> usize {
        self.consultations.len()
    }

    /// Resolution rate
    pub fn resolution_rate(&self) -> f64 {
        if self.consultations.is_empty() {
            return 0.0;
        }
        let resolved = self.consultations.values()
            .filter(|c| c.outcome == ConsultationOutcome::Resolved)
            .count();
        resolved as f64 / self.consultations.len() as f64 * 100.0
    }

    /// Escalation rate
    pub fn escalation_rate(&self) -> f64 {
        if self.consultations.is_empty() {
            return 0.0;
        }
        let escalated = self.consultations.values()
            .filter(|c| c.outcome == ConsultationOutcome::Escalated)
            .count();
        escalated as f64 / self.consultations.len() as f64 * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_creation() {
        let tracker = TeamConsultationTracker::new();
        assert_eq!(tracker.total(), 0);
    }

    #[test]
    fn test_consult() {
        let mut tracker = TeamConsultationTracker::new();
        let id = tracker.consult(TeamDepartment::Network);
        assert!(tracker.get(&id).is_some());
        assert_eq!(tracker.total(), 1);
    }

    #[test]
    fn test_resolve() {
        let mut tracker = TeamConsultationTracker::new();
        let id = tracker.consult(TeamDepartment::Storage);
        tracker.resolve(&id, 500);

        let c = tracker.get(&id).unwrap();
        assert_eq!(c.outcome, ConsultationOutcome::Resolved);
        assert_eq!(c.duration_ms, Some(500));
    }

    #[test]
    fn test_most_consulted() {
        let mut tracker = TeamConsultationTracker::new();
        tracker.consult(TeamDepartment::Network);
        tracker.consult(TeamDepartment::Network);
        tracker.consult(TeamDepartment::Desktop);

        let most = tracker.most_consulted().unwrap();
        assert_eq!(most, TeamDepartment::Network);
    }

    #[test]
    fn test_department_stats() {
        let mut tracker = TeamConsultationTracker::new();
        tracker.consult(TeamDepartment::Audio);
        tracker.consult(TeamDepartment::Audio);
        tracker.consult(TeamDepartment::Video);

        let stats = tracker.department_stats();
        assert!(!stats.is_empty());
    }

    #[test]
    fn test_escalation_rate() {
        let mut tracker = TeamConsultationTracker::new();
        let id1 = tracker.consult(TeamDepartment::Network);
        let id2 = tracker.consult(TeamDepartment::Network);
        tracker.resolve(&id1, 100);
        tracker.escalate(&id2);

        assert!((tracker.escalation_rate() - 50.0).abs() < 0.1);
    }
}
