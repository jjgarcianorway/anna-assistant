//! Workflow Automation Formatting
//!
//! Display formatting functions for workflow tracker output.

use super::tracker::WorkflowAutomationTracker;

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
    use crate::workflow_automation::types::WorkflowTrigger;

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
