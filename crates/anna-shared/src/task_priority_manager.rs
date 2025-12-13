//! Task Priority Manager - Phase 99
//!
//! Manages and prioritizes Anna's task queue.
//! Ensures critical tasks are handled first.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Task priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, PartialOrd, Ord)]
pub enum TaskPriority {
    Low,
    #[default]
    Normal,
    High,
    Urgent,
    Critical,
}

impl TaskPriority {
    pub fn name(&self) -> &'static str {
        match self {
            TaskPriority::Low => "Low",
            TaskPriority::Normal => "Normal",
            TaskPriority::High => "High",
            TaskPriority::Urgent => "Urgent",
            TaskPriority::Critical => "Critical",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            TaskPriority::Low => "▽",
            TaskPriority::Normal => "○",
            TaskPriority::High => "△",
            TaskPriority::Urgent => "◆",
            TaskPriority::Critical => "●",
        }
    }

    pub fn score(&self) -> u8 {
        match self {
            TaskPriority::Low => 1,
            TaskPriority::Normal => 2,
            TaskPriority::High => 3,
            TaskPriority::Urgent => 4,
            TaskPriority::Critical => 5,
        }
    }
}

/// Task state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TaskState {
    #[default]
    Pending,
    InProgress,
    Blocked,
    Completed,
    Cancelled,
}

impl TaskState {
    pub fn name(&self) -> &'static str {
        match self {
            TaskState::Pending => "Pending",
            TaskState::InProgress => "In Progress",
            TaskState::Blocked => "Blocked",
            TaskState::Completed => "Completed",
            TaskState::Cancelled => "Cancelled",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            TaskState::Pending => "○",
            TaskState::InProgress => "◐",
            TaskState::Blocked => "✗",
            TaskState::Completed => "✓",
            TaskState::Cancelled => "-",
        }
    }
}

/// A managed task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedTask {
    /// Task ID
    pub id: String,
    /// Task description
    pub description: String,
    /// Priority level
    pub priority: TaskPriority,
    /// Current state
    pub state: TaskState,
    /// Created timestamp
    pub created_at: u64,
    /// Started timestamp
    pub started_at: Option<u64>,
    /// Completed timestamp
    pub completed_at: Option<u64>,
    /// Blocked reason
    pub blocked_reason: Option<String>,
}

/// Task priority manager
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskPriorityManager {
    /// All tasks
    pub tasks: Vec<ManagedTask>,
    /// Count by priority
    pub by_priority: HashMap<String, u64>,
    /// Count by state
    pub by_state: HashMap<String, u64>,
    /// Total completed
    pub total_completed: u64,
    /// Total cancelled
    pub total_cancelled: u64,
}

impl TaskPriorityManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a task
    pub fn add(&mut self, id: String, description: String, priority: TaskPriority, timestamp: u64) {
        let task = ManagedTask {
            id,
            description,
            priority,
            state: TaskState::Pending,
            created_at: timestamp,
            started_at: None,
            completed_at: None,
            blocked_reason: None,
        };
        *self.by_priority.entry(priority.name().to_string()).or_insert(0) += 1;
        *self.by_state.entry(TaskState::Pending.name().to_string()).or_insert(0) += 1;
        self.tasks.push(task);
    }

    /// Start a task
    pub fn start(&mut self, id: &str, timestamp: u64) -> bool {
        let found = self.tasks.iter().position(|t| t.id == id);
        if let Some(idx) = found {
            let old_state = self.tasks[idx].state;
            if let Some(count) = self.by_state.get_mut(old_state.name()) {
                *count = count.saturating_sub(1);
            }
            *self.by_state.entry(TaskState::InProgress.name().to_string()).or_insert(0) += 1;

            self.tasks[idx].state = TaskState::InProgress;
            self.tasks[idx].started_at = Some(timestamp);
            true
        } else {
            false
        }
    }

    /// Complete a task
    pub fn complete(&mut self, id: &str, timestamp: u64) -> bool {
        let found = self.tasks.iter().position(|t| t.id == id);
        if let Some(idx) = found {
            let old_state = self.tasks[idx].state;
            if let Some(count) = self.by_state.get_mut(old_state.name()) {
                *count = count.saturating_sub(1);
            }
            *self.by_state.entry(TaskState::Completed.name().to_string()).or_insert(0) += 1;

            self.tasks[idx].state = TaskState::Completed;
            self.tasks[idx].completed_at = Some(timestamp);
            self.total_completed += 1;
            true
        } else {
            false
        }
    }

    /// Block a task
    pub fn block(&mut self, id: &str, reason: &str) -> bool {
        let found = self.tasks.iter().position(|t| t.id == id);
        if let Some(idx) = found {
            let old_state = self.tasks[idx].state;
            if let Some(count) = self.by_state.get_mut(old_state.name()) {
                *count = count.saturating_sub(1);
            }
            *self.by_state.entry(TaskState::Blocked.name().to_string()).or_insert(0) += 1;

            self.tasks[idx].state = TaskState::Blocked;
            self.tasks[idx].blocked_reason = Some(reason.to_string());
            true
        } else {
            false
        }
    }

    /// Cancel a task
    pub fn cancel(&mut self, id: &str) -> bool {
        let found = self.tasks.iter().position(|t| t.id == id);
        if let Some(idx) = found {
            let old_state = self.tasks[idx].state;
            if let Some(count) = self.by_state.get_mut(old_state.name()) {
                *count = count.saturating_sub(1);
            }
            *self.by_state.entry(TaskState::Cancelled.name().to_string()).or_insert(0) += 1;

            self.tasks[idx].state = TaskState::Cancelled;
            self.total_cancelled += 1;
            true
        } else {
            false
        }
    }

    /// Get task by ID
    pub fn get(&self, id: &str) -> Option<&ManagedTask> {
        self.tasks.iter().find(|t| t.id == id)
    }

    /// Get next task (highest priority pending)
    pub fn next(&self) -> Option<&ManagedTask> {
        self.tasks
            .iter()
            .filter(|t| t.state == TaskState::Pending)
            .max_by_key(|t| t.priority.score())
    }

    /// Get pending tasks sorted by priority
    pub fn pending(&self) -> Vec<&ManagedTask> {
        let mut pending: Vec<&ManagedTask> =
            self.tasks.iter().filter(|t| t.state == TaskState::Pending).collect();
        pending.sort_by(|a, b| b.priority.cmp(&a.priority));
        pending
    }

    /// Get in-progress tasks
    pub fn in_progress(&self) -> Vec<&ManagedTask> {
        self.tasks.iter().filter(|t| t.state == TaskState::InProgress).collect()
    }

    /// Get blocked tasks
    pub fn blocked(&self) -> Vec<&ManagedTask> {
        self.tasks.iter().filter(|t| t.state == TaskState::Blocked).collect()
    }

    /// Total task count
    pub fn total_count(&self) -> usize {
        self.tasks.len()
    }

    /// Pending count
    pub fn pending_count(&self) -> usize {
        self.pending().len()
    }
}

/// Format task manager for display
pub fn format_task_manager(manager: &TaskPriorityManager) -> String {
    let mut lines = vec!["=== Task Priority Manager ===".to_string()];
    lines.push(String::new());

    if manager.tasks.is_empty() {
        lines.push("No tasks in queue.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total tasks: {}", manager.total_count()));
    lines.push(format!("Pending: {}", manager.pending_count()));
    lines.push(format!("Completed: {}", manager.total_completed));

    // By priority
    if !manager.by_priority.is_empty() {
        lines.push(String::new());
        lines.push("By priority:".to_string());
        for (p, count) in &manager.by_priority {
            lines.push(format!("  {}: {}", p, count));
        }
    }

    // Pending tasks
    let pending = manager.pending();
    if !pending.is_empty() {
        lines.push(String::new());
        lines.push("Pending tasks:".to_string());
        for task in pending.iter().take(10) {
            lines.push(format!(
                "  [{}] {} - {}",
                task.priority.symbol(),
                task.id,
                task.description
            ));
        }
    }

    lines.join("\n")
}

/// Format task manager compact
pub fn format_task_manager_compact(manager: &TaskPriorityManager) -> String {
    format!(
        "Tasks: {} total | {} pending | {} completed",
        manager.total_count(),
        manager.pending_count(),
        manager.total_completed
    )
}

/// Format task manager one-line
pub fn format_task_manager_oneline(manager: &TaskPriorityManager) -> String {
    format!("{} tasks ({} pending)", manager.total_count(), manager.pending_count())
}

/// Check if query is about tasks
pub fn is_task_manager_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "task queue",
        "pending tasks",
        "task priority",
        "what tasks",
        "next task",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about tasks
pub fn task_manager_fun_fact(manager: &TaskPriorityManager) -> String {
    if manager.tasks.is_empty() {
        return "No tasks in the queue!".to_string();
    }

    let facts = [
        format!("Anna has {} tasks in the queue.", manager.total_count()),
        format!("{} tasks are pending.", manager.pending_count()),
        format!("{} tasks have been completed.", manager.total_completed),
    ];

    facts[manager.total_count() % facts.len()].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_priority() {
        assert_eq!(TaskPriority::Critical.name(), "Critical");
        assert_eq!(TaskPriority::Critical.score(), 5);
        assert!(TaskPriority::Critical > TaskPriority::Normal);
    }

    #[test]
    fn test_task_state() {
        assert_eq!(TaskState::InProgress.name(), "In Progress");
        assert_eq!(TaskState::Completed.symbol(), "✓");
    }

    #[test]
    fn test_add_task() {
        let mut manager = TaskPriorityManager::new();
        manager.add("task1".to_string(), "Do something".to_string(), TaskPriority::Normal, 1000);

        assert_eq!(manager.total_count(), 1);
        assert!(manager.get("task1").is_some());
    }

    #[test]
    fn test_task_lifecycle() {
        let mut manager = TaskPriorityManager::new();
        manager.add("task1".to_string(), "Do something".to_string(), TaskPriority::Normal, 1000);
        manager.start("task1", 2000);
        manager.complete("task1", 3000);

        let task = manager.get("task1").unwrap();
        assert_eq!(task.state, TaskState::Completed);
        assert_eq!(task.started_at, Some(2000));
        assert_eq!(task.completed_at, Some(3000));
    }

    #[test]
    fn test_next_task() {
        let mut manager = TaskPriorityManager::new();
        manager.add("task1".to_string(), "Low priority".to_string(), TaskPriority::Low, 1000);
        manager.add("task2".to_string(), "High priority".to_string(), TaskPriority::High, 1000);

        let next = manager.next().unwrap();
        assert_eq!(next.id, "task2");
    }

    #[test]
    fn test_pending_sorted() {
        let mut manager = TaskPriorityManager::new();
        manager.add("task1".to_string(), "Low".to_string(), TaskPriority::Low, 1000);
        manager.add("task2".to_string(), "Critical".to_string(), TaskPriority::Critical, 1000);
        manager.add("task3".to_string(), "Normal".to_string(), TaskPriority::Normal, 1000);

        let pending = manager.pending();
        assert_eq!(pending[0].id, "task2"); // Critical first
        assert_eq!(pending[2].id, "task1"); // Low last
    }

    #[test]
    fn test_block_task() {
        let mut manager = TaskPriorityManager::new();
        manager.add("task1".to_string(), "Do something".to_string(), TaskPriority::Normal, 1000);
        manager.block("task1", "Waiting for user input");

        let task = manager.get("task1").unwrap();
        assert_eq!(task.state, TaskState::Blocked);
        assert_eq!(task.blocked_reason, Some("Waiting for user input".to_string()));
    }

    #[test]
    fn test_cancel_task() {
        let mut manager = TaskPriorityManager::new();
        manager.add("task1".to_string(), "Do something".to_string(), TaskPriority::Normal, 1000);
        manager.cancel("task1");

        let task = manager.get("task1").unwrap();
        assert_eq!(task.state, TaskState::Cancelled);
        assert_eq!(manager.total_cancelled, 1);
    }

    #[test]
    fn test_format_manager() {
        let mut manager = TaskPriorityManager::new();
        manager.add("task1".to_string(), "Do something".to_string(), TaskPriority::Normal, 1000);

        let output = format_task_manager(&manager);
        assert!(output.contains("Task Priority Manager"));
        assert!(output.contains("Total tasks: 1"));
    }

    #[test]
    fn test_is_task_query() {
        assert!(is_task_manager_query("show task queue"));
        assert!(is_task_manager_query("what is the next task?"));
        assert!(!is_task_manager_query("what is the weather?"));
    }

    #[test]
    fn test_fun_fact() {
        let mut manager = TaskPriorityManager::new();
        manager.add("task1".to_string(), "Do something".to_string(), TaskPriority::Normal, 1000);

        let fact = task_manager_fun_fact(&manager);
        assert!(!fact.is_empty());
    }
}
