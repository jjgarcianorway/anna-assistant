//! ExecutionLog - tracks command execution history and statistics

use super::types::{ExecStatus, ExecutionRecord};
use super::utils::extract_command_pattern;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Command execution tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionLog {
    /// All execution records
    pub records: Vec<ExecutionRecord>,
    /// Count by command pattern
    pub command_counts: HashMap<String, u64>,
    /// Success rate by command pattern
    pub success_counts: HashMap<String, u64>,
    /// Total elevated executions
    pub elevated_count: u64,
    /// Commands that failed most
    pub failure_counts: HashMap<String, u64>,
}

impl ExecutionLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a command execution
    pub fn record(&mut self, record: ExecutionRecord) {
        let pattern = extract_command_pattern(&record.command);

        *self.command_counts.entry(pattern.clone()).or_insert(0) += 1;

        if record.status == ExecStatus::Success {
            *self.success_counts.entry(pattern.clone()).or_insert(0) += 1;
        } else if record.status == ExecStatus::Failed {
            *self.failure_counts.entry(pattern.clone()).or_insert(0) += 1;
        }

        if record.elevated {
            self.elevated_count += 1;
        }

        self.records.push(record);
    }

    /// Get recent executions
    pub fn recent(&self, limit: usize) -> Vec<&ExecutionRecord> {
        self.records.iter().rev().take(limit).collect()
    }

    /// Get executions by status
    pub fn by_status(&self, status: ExecStatus) -> Vec<&ExecutionRecord> {
        self.records.iter().filter(|r| r.status == status).collect()
    }

    /// Get failed executions
    pub fn failed(&self) -> Vec<&ExecutionRecord> {
        self.by_status(ExecStatus::Failed)
    }

    /// Get elevated executions
    pub fn elevated(&self) -> Vec<&ExecutionRecord> {
        self.records.iter().filter(|r| r.elevated).collect()
    }

    /// Get high risk executions
    pub fn high_risk(&self) -> Vec<&ExecutionRecord> {
        self.records
            .iter()
            .filter(|r| r.risk.level() >= super::types::CommandRisk::HighRisk.level())
            .collect()
    }

    /// Total execution count
    pub fn total_count(&self) -> usize {
        self.records.len()
    }

    /// Success rate percentage
    pub fn success_rate(&self) -> f64 {
        let completed: usize = self
            .records
            .iter()
            .filter(|r| r.status == ExecStatus::Success || r.status == ExecStatus::Failed)
            .count();

        if completed == 0 {
            return 100.0;
        }

        let successful = self
            .records
            .iter()
            .filter(|r| r.status == ExecStatus::Success)
            .count();

        (successful as f64 / completed as f64) * 100.0
    }

    /// Average execution time in ms
    pub fn average_duration_ms(&self) -> u64 {
        if self.records.is_empty() {
            return 0;
        }
        let total: u64 = self.records.iter().map(|r| r.duration_ms).sum();
        total / self.records.len() as u64
    }

    /// Most used commands
    pub fn most_used(&self, limit: usize) -> Vec<(&str, u64)> {
        let mut commands: Vec<_> = self.command_counts.iter().collect();
        commands.sort_by(|a, b| b.1.cmp(a.1));
        commands.into_iter().take(limit).map(|(k, v)| (k.as_str(), *v)).collect()
    }

    /// Most failed commands
    pub fn most_failed(&self, limit: usize) -> Vec<(&str, u64)> {
        let mut commands: Vec<_> = self.failure_counts.iter().collect();
        commands.sort_by(|a, b| b.1.cmp(a.1));
        commands.into_iter().take(limit).map(|(k, v)| (k.as_str(), *v)).collect()
    }

    /// Commands for a specific ticket
    pub fn by_ticket(&self, ticket_id: &str) -> Vec<&ExecutionRecord> {
        self.records
            .iter()
            .filter(|r| r.ticket_id.as_deref() == Some(ticket_id))
            .collect()
    }
}
