//! Workflow Automation Tracker
//!
//! Main tracker implementation for managing workflow lifecycles.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{WorkflowRecord, WorkflowStatus, WorkflowStep, WorkflowTrigger};

/// Workflow automation tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkflowAutomationTracker {
    /// All workflows
    pub workflows: Vec<WorkflowRecord>,
    /// Count by trigger
    pub by_trigger: HashMap<String, u64>,
    /// Count by status
    pub by_status: HashMap<String, u64>,
    /// Total runs
    pub total_runs: u64,
    /// Total successes
    pub total_successes: u64,
}

impl WorkflowAutomationTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a workflow
    pub fn create(&mut self, id: String, name: String, description: String, trigger: WorkflowTrigger, timestamp: u64) {
        let workflow = WorkflowRecord {
            id,
            name,
            description,
            trigger,
            status: WorkflowStatus::Active,
            steps: vec![],
            run_count: 0,
            success_count: 0,
            created_at: timestamp,
            last_run: None,
        };
        *self.by_trigger.entry(trigger.name().to_string()).or_insert(0) += 1;
        *self.by_status.entry(WorkflowStatus::Active.name().to_string()).or_insert(0) += 1;
        self.workflows.push(workflow);
    }

    /// Add step to workflow
    pub fn add_step(&mut self, workflow_id: &str, name: &str, action: &str) -> bool {
        let found = self.workflows.iter().position(|w| w.id == workflow_id);
        if let Some(idx) = found {
            let order = self.workflows[idx].steps.len() as u32 + 1;
            self.workflows[idx].steps.push(WorkflowStep {
                name: name.to_string(),
                action: action.to_string(),
                order,
                completed: false,
            });
            true
        } else {
            false
        }
    }

    /// Run workflow
    pub fn run(&mut self, id: &str, timestamp: u64) -> bool {
        let found = self.workflows.iter().position(|w| w.id == id);
        if let Some(idx) = found {
            let old_status = self.workflows[idx].status;
            if let Some(count) = self.by_status.get_mut(old_status.name()) {
                *count = count.saturating_sub(1);
            }
            *self.by_status.entry(WorkflowStatus::Running.name().to_string()).or_insert(0) += 1;

            self.workflows[idx].status = WorkflowStatus::Running;
            self.workflows[idx].run_count += 1;
            self.workflows[idx].last_run = Some(timestamp);
            self.total_runs += 1;
            true
        } else {
            false
        }
    }

    /// Complete workflow
    pub fn complete(&mut self, id: &str, success: bool) -> bool {
        let found = self.workflows.iter().position(|w| w.id == id);
        if let Some(idx) = found {
            let old_status = self.workflows[idx].status;
            if let Some(count) = self.by_status.get_mut(old_status.name()) {
                *count = count.saturating_sub(1);
            }

            let new_status = if success {
                self.workflows[idx].success_count += 1;
                self.total_successes += 1;
                WorkflowStatus::Completed
            } else {
                WorkflowStatus::Failed
            };

            *self.by_status.entry(new_status.name().to_string()).or_insert(0) += 1;
            self.workflows[idx].status = new_status;
            true
        } else {
            false
        }
    }

    /// Get workflow by ID
    pub fn get(&self, id: &str) -> Option<&WorkflowRecord> {
        self.workflows.iter().find(|w| w.id == id)
    }

    /// Get active workflows
    pub fn active(&self) -> Vec<&WorkflowRecord> {
        self.workflows.iter().filter(|w| w.status == WorkflowStatus::Active).collect()
    }

    /// Get by trigger
    pub fn by_wf_trigger(&self, trigger: WorkflowTrigger) -> Vec<&WorkflowRecord> {
        self.workflows.iter().filter(|w| w.trigger == trigger).collect()
    }

    /// Overall success rate
    pub fn success_rate(&self) -> u8 {
        if self.total_runs == 0 {
            0
        } else {
            ((self.total_successes * 100) / self.total_runs) as u8
        }
    }

    /// Total workflow count
    pub fn total_count(&self) -> usize {
        self.workflows.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_workflow() {
        let mut tracker = WorkflowAutomationTracker::new();
        tracker.create("wf1".to_string(), "Backup".to_string(), "Daily backup".to_string(), WorkflowTrigger::Scheduled, 1000);

        assert_eq!(tracker.total_count(), 1);
        assert!(tracker.get("wf1").is_some());
    }

    #[test]
    fn test_add_step() {
        let mut tracker = WorkflowAutomationTracker::new();
        tracker.create("wf1".to_string(), "Backup".to_string(), "Daily backup".to_string(), WorkflowTrigger::Scheduled, 1000);
        tracker.add_step("wf1", "Stop service", "systemctl stop app");
        tracker.add_step("wf1", "Create backup", "tar -czf backup.tar.gz /data");

        let wf = tracker.get("wf1").unwrap();
        assert_eq!(wf.steps.len(), 2);
        assert_eq!(wf.steps[0].order, 1);
        assert_eq!(wf.steps[1].order, 2);
    }

    #[test]
    fn test_run_workflow() {
        let mut tracker = WorkflowAutomationTracker::new();
        tracker.create("wf1".to_string(), "Backup".to_string(), "Daily".to_string(), WorkflowTrigger::Scheduled, 1000);
        tracker.run("wf1", 2000);

        let wf = tracker.get("wf1").unwrap();
        assert_eq!(wf.status, WorkflowStatus::Running);
        assert_eq!(wf.run_count, 1);
    }

    #[test]
    fn test_complete_workflow() {
        let mut tracker = WorkflowAutomationTracker::new();
        tracker.create("wf1".to_string(), "Backup".to_string(), "Daily".to_string(), WorkflowTrigger::Scheduled, 1000);
        tracker.run("wf1", 2000);
        tracker.complete("wf1", true);

        let wf = tracker.get("wf1").unwrap();
        assert_eq!(wf.status, WorkflowStatus::Completed);
        assert_eq!(wf.success_count, 1);
    }

    #[test]
    fn test_success_rate() {
        let mut tracker = WorkflowAutomationTracker::new();
        tracker.create("wf1".to_string(), "Test".to_string(), "Test".to_string(), WorkflowTrigger::Manual, 1000);
        tracker.run("wf1", 2000);
        tracker.complete("wf1", true);
        tracker.run("wf1", 3000);
        tracker.complete("wf1", false);

        assert_eq!(tracker.success_rate(), 50);
    }

    #[test]
    fn test_by_trigger() {
        let mut tracker = WorkflowAutomationTracker::new();
        tracker.create("wf1".to_string(), "Test1".to_string(), "Test".to_string(), WorkflowTrigger::Scheduled, 1000);
        tracker.create("wf2".to_string(), "Test2".to_string(), "Test".to_string(), WorkflowTrigger::Event, 1000);

        assert_eq!(tracker.by_wf_trigger(WorkflowTrigger::Scheduled).len(), 1);
        assert_eq!(tracker.by_wf_trigger(WorkflowTrigger::Event).len(), 1);
    }
}
