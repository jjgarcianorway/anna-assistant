//! Workflow Automation Tracker - Phase 101
//!
//! Tracks automated workflows Anna creates and executes.
//! Enables complex multi-step automations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Workflow trigger type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WorkflowTrigger {
    #[default]
    Manual,
    Scheduled,
    Event,
    Condition,
    Webhook,
}

impl WorkflowTrigger {
    pub fn name(&self) -> &'static str {
        match self {
            WorkflowTrigger::Manual => "Manual",
            WorkflowTrigger::Scheduled => "Scheduled",
            WorkflowTrigger::Event => "Event",
            WorkflowTrigger::Condition => "Condition",
            WorkflowTrigger::Webhook => "Webhook",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            WorkflowTrigger::Manual => "▶",
            WorkflowTrigger::Scheduled => "⏰",
            WorkflowTrigger::Event => "⚡",
            WorkflowTrigger::Condition => "?",
            WorkflowTrigger::Webhook => "↯",
        }
    }
}

/// Workflow status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WorkflowStatus {
    #[default]
    Active,
    Paused,
    Running,
    Completed,
    Failed,
    Disabled,
}

impl WorkflowStatus {
    pub fn name(&self) -> &'static str {
        match self {
            WorkflowStatus::Active => "Active",
            WorkflowStatus::Paused => "Paused",
            WorkflowStatus::Running => "Running",
            WorkflowStatus::Completed => "Completed",
            WorkflowStatus::Failed => "Failed",
            WorkflowStatus::Disabled => "Disabled",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            WorkflowStatus::Active => "✓",
            WorkflowStatus::Paused => "◐",
            WorkflowStatus::Running => "◉",
            WorkflowStatus::Completed => "●",
            WorkflowStatus::Failed => "✗",
            WorkflowStatus::Disabled => "-",
        }
    }
}

/// A workflow step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Step name
    pub name: String,
    /// Step action
    pub action: String,
    /// Step order
    pub order: u32,
    /// Completed
    pub completed: bool,
}

/// A workflow record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRecord {
    /// Workflow ID
    pub id: String,
    /// Workflow name
    pub name: String,
    /// Description
    pub description: String,
    /// Trigger type
    pub trigger: WorkflowTrigger,
    /// Current status
    pub status: WorkflowStatus,
    /// Steps
    pub steps: Vec<WorkflowStep>,
    /// Run count
    pub run_count: u64,
    /// Success count
    pub success_count: u64,
    /// Created timestamp
    pub created_at: u64,
    /// Last run timestamp
    pub last_run: Option<u64>,
}

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

/// Format workflow tracker for display
pub fn format_workflow_tracker(tracker: &WorkflowAutomationTracker) -> String {
    let mut lines = vec!["=== Workflow Automation ===".to_string()];
    lines.push(String::new());

    if tracker.workflows.is_empty() {
        lines.push("No workflows created yet.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total workflows: {}", tracker.total_count()));
    lines.push(format!("Total runs: {}", tracker.total_runs));
    lines.push(format!("Success rate: {}%", tracker.success_rate()));

    // By trigger
    if !tracker.by_trigger.is_empty() {
        lines.push(String::new());
        lines.push("By trigger:".to_string());
        for (t, count) in &tracker.by_trigger {
            lines.push(format!("  {}: {}", t, count));
        }
    }

    // Active workflows
    let active = tracker.active();
    if !active.is_empty() {
        lines.push(String::new());
        lines.push("Active workflows:".to_string());
        for wf in active.iter().take(10) {
            lines.push(format!(
                "  [{}] {} - {} steps",
                wf.trigger.symbol(),
                wf.name,
                wf.steps.len()
            ));
        }
    }

    lines.join("\n")
}

/// Format workflow tracker compact
pub fn format_workflow_tracker_compact(tracker: &WorkflowAutomationTracker) -> String {
    format!(
        "Workflows: {} total | {} runs | {}% success",
        tracker.total_count(),
        tracker.total_runs,
        tracker.success_rate()
    )
}

/// Format workflow tracker one-line
pub fn format_workflow_tracker_oneline(tracker: &WorkflowAutomationTracker) -> String {
    format!("{} workflows ({}% success)", tracker.total_count(), tracker.success_rate())
}

/// Check if query is about workflows
pub fn is_workflow_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "workflow",
        "workflows",
        "automation",
        "automate",
        "scheduled task",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about workflows
pub fn workflow_fun_fact(tracker: &WorkflowAutomationTracker) -> String {
    if tracker.workflows.is_empty() {
        return "No workflows created yet!".to_string();
    }

    let facts = [
        format!("Anna manages {} automated workflows.", tracker.total_count()),
        format!("Workflows have run {} times.", tracker.total_runs),
        format!("Workflow success rate is {}%.", tracker.success_rate()),
    ];

    facts[tracker.total_count() % facts.len()].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_trigger() {
        assert_eq!(WorkflowTrigger::Scheduled.name(), "Scheduled");
        assert_eq!(WorkflowTrigger::Event.symbol(), "⚡");
    }

    #[test]
    fn test_workflow_status() {
        assert_eq!(WorkflowStatus::Running.name(), "Running");
        assert_eq!(WorkflowStatus::Failed.symbol(), "✗");
    }

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

    #[test]
    fn test_format_tracker() {
        let mut tracker = WorkflowAutomationTracker::new();
        tracker.create("wf1".to_string(), "Backup".to_string(), "Daily".to_string(), WorkflowTrigger::Scheduled, 1000);

        let output = format_workflow_tracker(&tracker);
        assert!(output.contains("Workflow Automation"));
        assert!(output.contains("Total workflows: 1"));
    }

    #[test]
    fn test_is_workflow_query() {
        assert!(is_workflow_query("show workflows"));
        assert!(is_workflow_query("create automation"));
        assert!(!is_workflow_query("what is the weather?"));
    }

    #[test]
    fn test_fun_fact() {
        let mut tracker = WorkflowAutomationTracker::new();
        tracker.create("wf1".to_string(), "Test".to_string(), "Test".to_string(), WorkflowTrigger::Manual, 1000);

        let fact = workflow_fun_fact(&tracker);
        assert!(!fact.is_empty());
    }
}
