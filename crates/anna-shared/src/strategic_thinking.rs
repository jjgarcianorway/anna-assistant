//! Strategic Thinking Tracker - Phase 91
//!
//! Tracks senior strategic thinking during idle time.
//! VISION.md: "Seniors can think strategically about improvements during idle time"
//! "If interrupted, Anna can resume later"

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Thinking status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ThinkingStatus {
    #[default]
    Pending,
    InProgress,
    Paused,
    Completed,
    Abandoned,
}

impl ThinkingStatus {
    pub fn name(&self) -> &'static str {
        match self {
            ThinkingStatus::Pending => "Pending",
            ThinkingStatus::InProgress => "In Progress",
            ThinkingStatus::Paused => "Paused",
            ThinkingStatus::Completed => "Completed",
            ThinkingStatus::Abandoned => "Abandoned",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            ThinkingStatus::Pending => ".",
            ThinkingStatus::InProgress => "*",
            ThinkingStatus::Paused => "~",
            ThinkingStatus::Completed => "✓",
            ThinkingStatus::Abandoned => "x",
        }
    }
}

/// Category of strategic thinking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ThinkingCategory {
    #[default]
    Optimization,
    Security,
    Maintenance,
    Learning,
    UserExperience,
    SystemHealth,
    Performance,
}

impl ThinkingCategory {
    pub fn name(&self) -> &'static str {
        match self {
            ThinkingCategory::Optimization => "Optimization",
            ThinkingCategory::Security => "Security",
            ThinkingCategory::Maintenance => "Maintenance",
            ThinkingCategory::Learning => "Learning",
            ThinkingCategory::UserExperience => "User Experience",
            ThinkingCategory::SystemHealth => "System Health",
            ThinkingCategory::Performance => "Performance",
        }
    }
}

/// Priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ThinkingPriority {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

impl ThinkingPriority {
    pub fn name(&self) -> &'static str {
        match self {
            ThinkingPriority::Low => "Low",
            ThinkingPriority::Medium => "Medium",
            ThinkingPriority::High => "High",
            ThinkingPriority::Critical => "Critical",
        }
    }
}

/// A strategic thinking task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingTask {
    /// Unique ID
    pub id: String,
    /// Description of what to think about
    pub description: String,
    /// Category
    pub category: ThinkingCategory,
    /// Priority
    pub priority: ThinkingPriority,
    /// Status
    pub status: ThinkingStatus,
    /// Senior assigned
    pub assigned_to: Option<String>,
    /// Created timestamp
    pub created_at: u64,
    /// Started timestamp
    pub started_at: Option<u64>,
    /// Completed timestamp
    pub completed_at: Option<u64>,
    /// Time spent thinking (seconds)
    pub time_spent_secs: u64,
    /// Findings/conclusions
    pub findings: Vec<String>,
    /// Recommendations
    pub recommendations: Vec<String>,
    /// Was interrupted
    pub interrupted: bool,
    /// Resume point (for paused tasks)
    pub resume_point: Option<String>,
}

/// Strategic thinking tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StrategicThinkingTracker {
    /// All tasks
    pub tasks: Vec<ThinkingTask>,
    /// Count by category
    pub by_category: HashMap<String, u64>,
    /// Count by status
    pub by_status: HashMap<String, u64>,
    /// Total time spent thinking
    pub total_time_secs: u64,
    /// Total recommendations made
    pub total_recommendations: u64,
}

impl StrategicThinkingTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a thinking task
    pub fn add(&mut self, task: ThinkingTask) {
        *self.by_category.entry(task.category.name().to_string()).or_insert(0) += 1;
        *self.by_status.entry(task.status.name().to_string()).or_insert(0) += 1;
        self.tasks.push(task);
    }

    /// Get task by ID
    pub fn get(&self, id: &str) -> Option<&ThinkingTask> {
        self.tasks.iter().find(|t| t.id == id)
    }

    /// Start a task
    pub fn start(&mut self, id: &str, timestamp: u64) -> bool {
        let found = self.tasks.iter().position(|t| t.id == id);
        if let Some(idx) = found {
            let old_status = self.tasks[idx].status;
            self.update_status_count(&old_status, ThinkingStatus::InProgress);
            self.tasks[idx].status = ThinkingStatus::InProgress;
            self.tasks[idx].started_at = Some(timestamp);
            true
        } else {
            false
        }
    }

    /// Pause a task (interrupted)
    pub fn pause(&mut self, id: &str, resume_point: Option<String>, time_spent: u64) -> bool {
        let found = self.tasks.iter().position(|t| t.id == id);
        if let Some(idx) = found {
            let old_status = self.tasks[idx].status;
            self.update_status_count(&old_status, ThinkingStatus::Paused);
            self.tasks[idx].status = ThinkingStatus::Paused;
            self.tasks[idx].interrupted = true;
            self.tasks[idx].resume_point = resume_point;
            self.tasks[idx].time_spent_secs += time_spent;
            self.total_time_secs += time_spent;
            true
        } else {
            false
        }
    }

    /// Resume a paused task
    pub fn resume(&mut self, id: &str, timestamp: u64) -> bool {
        let found = self.tasks.iter().position(|t| t.id == id);
        if let Some(idx) = found {
            if self.tasks[idx].status == ThinkingStatus::Paused {
                self.update_status_count(&ThinkingStatus::Paused, ThinkingStatus::InProgress);
                self.tasks[idx].status = ThinkingStatus::InProgress;
                self.tasks[idx].started_at = Some(timestamp);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Complete a task
    pub fn complete(&mut self, id: &str, findings: Vec<String>, recommendations: Vec<String>, time_spent: u64, timestamp: u64) -> bool {
        let found = self.tasks.iter().position(|t| t.id == id);
        if let Some(idx) = found {
            let old_status = self.tasks[idx].status;
            let rec_count = recommendations.len() as u64;
            self.update_status_count(&old_status, ThinkingStatus::Completed);
            self.tasks[idx].status = ThinkingStatus::Completed;
            self.tasks[idx].completed_at = Some(timestamp);
            self.tasks[idx].time_spent_secs += time_spent;
            self.tasks[idx].findings = findings;
            self.tasks[idx].recommendations = recommendations;
            self.total_time_secs += time_spent;
            self.total_recommendations += rec_count;
            true
        } else {
            false
        }
    }

    fn update_status_count(&mut self, old: &ThinkingStatus, new: ThinkingStatus) {
        if let Some(count) = self.by_status.get_mut(old.name()) {
            *count = count.saturating_sub(1);
        }
        *self.by_status.entry(new.name().to_string()).or_insert(0) += 1;
    }

    /// Get pending tasks
    pub fn pending(&self) -> Vec<&ThinkingTask> {
        self.tasks.iter().filter(|t| t.status == ThinkingStatus::Pending).collect()
    }

    /// Get paused (resumable) tasks
    pub fn paused(&self) -> Vec<&ThinkingTask> {
        self.tasks.iter().filter(|t| t.status == ThinkingStatus::Paused).collect()
    }

    /// Get completed tasks
    pub fn completed(&self) -> Vec<&ThinkingTask> {
        self.tasks.iter().filter(|t| t.status == ThinkingStatus::Completed).collect()
    }

    /// Get tasks by category
    pub fn by_thinking_category(&self, category: ThinkingCategory) -> Vec<&ThinkingTask> {
        self.tasks.iter().filter(|t| t.category == category).collect()
    }

    /// Get high priority tasks
    pub fn high_priority(&self) -> Vec<&ThinkingTask> {
        self.tasks.iter().filter(|t| matches!(t.priority, ThinkingPriority::High | ThinkingPriority::Critical)).collect()
    }

    /// Total task count
    pub fn total_count(&self) -> usize {
        self.tasks.len()
    }

    /// Completed count
    pub fn completed_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.status == ThinkingStatus::Completed).count()
    }

    /// Average time per task
    pub fn avg_time_per_task(&self) -> f64 {
        let completed = self.completed_count();
        if completed == 0 {
            return 0.0;
        }
        self.total_time_secs as f64 / completed as f64
    }
}

/// Format strategic thinking tracker for display
pub fn format_strategic_tracker(tracker: &StrategicThinkingTracker) -> String {
    let mut lines = vec!["=== Strategic Thinking ===".to_string()];
    lines.push(String::new());

    if tracker.tasks.is_empty() {
        lines.push("No strategic thinking tasks.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total tasks: {}", tracker.total_count()));
    lines.push(format!("Completed: {}", tracker.completed_count()));
    lines.push(format!("Time spent: {} min", tracker.total_time_secs / 60));
    lines.push(format!("Recommendations: {}", tracker.total_recommendations));

    // Paused (resumable)
    let paused = tracker.paused();
    if !paused.is_empty() {
        lines.push(String::new());
        lines.push("Resumable tasks:".to_string());
        for t in paused.iter().take(3) {
            lines.push(format!("  [{}] {}", t.category.name(), t.description));
        }
    }

    // Pending
    let pending = tracker.pending();
    if !pending.is_empty() {
        lines.push(String::new());
        lines.push(format!("Pending: {}", pending.len()));
    }

    lines.join("\n")
}

/// Format strategic tracker compact
pub fn format_strategic_tracker_compact(tracker: &StrategicThinkingTracker) -> String {
    format!(
        "Strategic: {} tasks | {} completed | {} recommendations",
        tracker.total_count(),
        tracker.completed_count(),
        tracker.total_recommendations
    )
}

/// Format strategic tracker one-line
pub fn format_strategic_tracker_oneline(tracker: &StrategicThinkingTracker) -> String {
    format!(
        "{} thinking tasks ({} done)",
        tracker.total_count(),
        tracker.completed_count()
    )
}

/// Check if query is about strategic thinking
pub fn is_strategic_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "strategic thinking",
        "improvements",
        "recommendations",
        "idle time work",
        "background thinking",
        "system analysis",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about strategic thinking
pub fn strategic_fun_fact(tracker: &StrategicThinkingTracker) -> String {
    if tracker.tasks.is_empty() {
        return "No strategic thinking done yet!".to_string();
    }

    let facts = [
        format!(
            "Seniors have completed {} strategic tasks.",
            tracker.completed_count()
        ),
        format!(
            "{} minutes spent on strategic thinking.",
            tracker.total_time_secs / 60
        ),
        format!(
            "{} improvement recommendations made.",
            tracker.total_recommendations
        ),
        format!(
            "{} tasks are paused and can be resumed.",
            tracker.paused().len()
        ),
    ];

    facts[tracker.total_count() % facts.len()].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_task(desc: &str, category: ThinkingCategory) -> ThinkingTask {
        ThinkingTask {
            id: format!("THK-{}", desc.len()),
            description: desc.to_string(),
            category,
            priority: ThinkingPriority::Medium,
            status: ThinkingStatus::Pending,
            assigned_to: Some("Senior Maya".to_string()),
            created_at: 1234567890,
            started_at: None,
            completed_at: None,
            time_spent_secs: 0,
            findings: vec![],
            recommendations: vec![],
            interrupted: false,
            resume_point: None,
        }
    }

    #[test]
    fn test_thinking_status() {
        assert_eq!(ThinkingStatus::InProgress.name(), "In Progress");
        assert_eq!(ThinkingStatus::Completed.symbol(), "✓");
    }

    #[test]
    fn test_thinking_category() {
        assert_eq!(ThinkingCategory::Security.name(), "Security");
        assert_eq!(ThinkingCategory::Optimization.name(), "Optimization");
    }

    #[test]
    fn test_thinking_priority() {
        assert_eq!(ThinkingPriority::High.name(), "High");
        assert_eq!(ThinkingPriority::Critical.name(), "Critical");
    }

    #[test]
    fn test_add_task() {
        let mut tracker = StrategicThinkingTracker::new();
        tracker.add(make_task("Optimize boot", ThinkingCategory::Optimization));

        assert_eq!(tracker.total_count(), 1);
    }

    #[test]
    fn test_start_task() {
        let mut tracker = StrategicThinkingTracker::new();
        let mut task = make_task("Optimize boot", ThinkingCategory::Optimization);
        task.id = "THK-001".to_string();
        tracker.add(task);

        assert!(tracker.start("THK-001", 1234567890));
        assert_eq!(tracker.get("THK-001").unwrap().status, ThinkingStatus::InProgress);
    }

    #[test]
    fn test_pause_and_resume() {
        let mut tracker = StrategicThinkingTracker::new();
        let mut task = make_task("Optimize boot", ThinkingCategory::Optimization);
        task.id = "THK-001".to_string();
        tracker.add(task);

        tracker.start("THK-001", 1000);
        assert!(tracker.pause("THK-001", Some("Step 3".to_string()), 60));
        assert_eq!(tracker.get("THK-001").unwrap().status, ThinkingStatus::Paused);
        assert!(tracker.get("THK-001").unwrap().interrupted);

        assert!(tracker.resume("THK-001", 2000));
        assert_eq!(tracker.get("THK-001").unwrap().status, ThinkingStatus::InProgress);
    }

    #[test]
    fn test_complete_task() {
        let mut tracker = StrategicThinkingTracker::new();
        let mut task = make_task("Optimize boot", ThinkingCategory::Optimization);
        task.id = "THK-001".to_string();
        tracker.add(task);

        tracker.start("THK-001", 1000);
        let findings = vec!["Found slow service".to_string()];
        let recommendations = vec!["Disable service X".to_string(), "Enable parallel boot".to_string()];

        assert!(tracker.complete("THK-001", findings, recommendations, 120, 2000));
        assert_eq!(tracker.completed_count(), 1);
        assert_eq!(tracker.total_recommendations, 2);
    }

    #[test]
    fn test_paused_tasks() {
        let mut tracker = StrategicThinkingTracker::new();
        let mut task = make_task("Task 1", ThinkingCategory::Security);
        task.id = "THK-001".to_string();
        tracker.add(task);

        tracker.start("THK-001", 1000);
        tracker.pause("THK-001", None, 30);

        assert_eq!(tracker.paused().len(), 1);
    }

    #[test]
    fn test_high_priority() {
        let mut tracker = StrategicThinkingTracker::new();
        let mut task = make_task("Critical task", ThinkingCategory::Security);
        task.priority = ThinkingPriority::Critical;
        tracker.add(task);

        assert_eq!(tracker.high_priority().len(), 1);
    }

    #[test]
    fn test_format_strategic_tracker() {
        let mut tracker = StrategicThinkingTracker::new();
        tracker.add(make_task("Test task", ThinkingCategory::Optimization));

        let output = format_strategic_tracker(&tracker);
        assert!(output.contains("Strategic Thinking"));
    }

    #[test]
    fn test_is_strategic_query() {
        assert!(is_strategic_query("show strategic thinking"));
        assert!(is_strategic_query("what recommendations?"));
        assert!(is_strategic_query("system analysis"));
        assert!(!is_strategic_query("what is the weather?"));
    }

    #[test]
    fn test_strategic_fun_fact() {
        let mut tracker = StrategicThinkingTracker::new();
        tracker.add(make_task("Test", ThinkingCategory::Optimization));

        let fact = strategic_fun_fact(&tracker);
        assert!(!fact.is_empty());
    }
}
