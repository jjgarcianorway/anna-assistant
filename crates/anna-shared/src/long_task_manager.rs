// v0.0.534: Long Task Manager (Phase 110)
// Manages long-running tasks with email notification per VISION.md

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum LongTaskStatus {
    #[default]
    Queued,
    WaitingIdle,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for LongTaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Queued => write!(f, "Queued"),
            Self::WaitingIdle => write!(f, "Waiting for Idle"),
            Self::Running => write!(f, "Running"),
            Self::Paused => write!(f, "Paused"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed => write!(f, "Failed"),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// Task type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LongTaskType {
    Research,
    Installation,
    Backup,
    Download,
    Analysis,
    Compilation,
    Custom(String),
}

impl Default for LongTaskType {
    fn default() -> Self {
        Self::Research
    }
}

impl std::fmt::Display for LongTaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Research => write!(f, "Research"),
            Self::Installation => write!(f, "Installation"),
            Self::Backup => write!(f, "Backup"),
            Self::Download => write!(f, "Download"),
            Self::Analysis => write!(f, "Analysis"),
            Self::Compilation => write!(f, "Compilation"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Individual long task record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongTaskRecord {
    pub id: String,
    pub task_type: LongTaskType,
    pub description: String,
    pub status: LongTaskStatus,
    pub ticket_id: Option<String>,
    pub progress_pct: u8,
    pub estimated_minutes: Option<u32>,
    pub email_on_complete: bool,
    pub user_email: Option<String>,
    pub chain_of_thought: Vec<String>,
    pub result: Option<String>,
    pub error: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

impl LongTaskRecord {
    /// Create new long task
    pub fn new(id: &str, task_type: LongTaskType, description: &str, timestamp: &str) -> Self {
        Self {
            id: id.to_string(),
            task_type,
            description: description.to_string(),
            status: LongTaskStatus::Queued,
            ticket_id: None,
            progress_pct: 0,
            estimated_minutes: None,
            email_on_complete: false,
            user_email: None,
            chain_of_thought: Vec::new(),
            result: None,
            error: None,
            created_at: timestamp.to_string(),
            started_at: None,
            completed_at: None,
        }
    }

    /// Enable email notification
    pub fn enable_email(&mut self, email: &str) {
        self.email_on_complete = true;
        self.user_email = Some(email.to_string());
    }

    /// Link to ticket
    pub fn link_ticket(&mut self, ticket_id: &str) {
        self.ticket_id = Some(ticket_id.to_string());
    }

    /// Set estimated time
    pub fn set_estimate(&mut self, minutes: u32) {
        self.estimated_minutes = Some(minutes);
    }

    /// Start task (wait for idle)
    pub fn wait_for_idle(&mut self) {
        self.status = LongTaskStatus::WaitingIdle;
    }

    /// Start running
    pub fn start(&mut self, timestamp: &str) {
        self.status = LongTaskStatus::Running;
        self.started_at = Some(timestamp.to_string());
    }

    /// Update progress
    pub fn update_progress(&mut self, pct: u8) {
        self.progress_pct = pct.min(100);
    }

    /// Add thought to chain
    pub fn add_thought(&mut self, thought: &str) {
        self.chain_of_thought.push(thought.to_string());
    }

    /// Pause task
    pub fn pause(&mut self) {
        self.status = LongTaskStatus::Paused;
    }

    /// Resume task
    pub fn resume(&mut self) {
        self.status = LongTaskStatus::Running;
    }

    /// Complete task
    pub fn complete(&mut self, result: &str, timestamp: &str) {
        self.status = LongTaskStatus::Completed;
        self.result = Some(result.to_string());
        self.completed_at = Some(timestamp.to_string());
        self.progress_pct = 100;
    }

    /// Fail task
    pub fn fail(&mut self, error: &str, timestamp: &str) {
        self.status = LongTaskStatus::Failed;
        self.error = Some(error.to_string());
        self.completed_at = Some(timestamp.to_string());
    }

    /// Cancel task
    pub fn cancel(&mut self) {
        self.status = LongTaskStatus::Cancelled;
    }

    /// Is task active?
    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            LongTaskStatus::Queued | LongTaskStatus::WaitingIdle | LongTaskStatus::Running | LongTaskStatus::Paused
        )
    }

    /// Needs email notification?
    pub fn needs_email(&self) -> bool {
        self.email_on_complete
            && self.user_email.is_some()
            && matches!(self.status, LongTaskStatus::Completed | LongTaskStatus::Failed)
    }
}

/// Long task manager
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LongTaskManager {
    tasks: HashMap<String, LongTaskRecord>,
    next_id: u32,
    idle_threshold_minutes: u32,
}

impl LongTaskManager {
    /// Create new manager
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            next_id: 1,
            idle_threshold_minutes: 5,
        }
    }

    /// Set idle threshold
    pub fn set_idle_threshold(&mut self, minutes: u32) {
        self.idle_threshold_minutes = minutes;
    }

    /// Create a long task
    pub fn create(
        &mut self,
        task_type: LongTaskType,
        description: &str,
        timestamp: &str,
    ) -> String {
        let id = format!("LTASK-{:05}", self.next_id);
        self.next_id += 1;

        let task = LongTaskRecord::new(&id, task_type, description, timestamp);
        self.tasks.insert(id.clone(), task);
        id
    }

    /// Get task by ID
    pub fn get(&self, id: &str) -> Option<&LongTaskRecord> {
        self.tasks.get(id)
    }

    /// Get mutable task
    pub fn get_mut(&mut self, id: &str) -> Option<&mut LongTaskRecord> {
        self.tasks.get_mut(id)
    }

    /// Get active tasks
    pub fn active(&self) -> Vec<&LongTaskRecord> {
        self.tasks.values().filter(|t| t.is_active()).collect()
    }

    /// Get tasks waiting for idle
    pub fn waiting_for_idle(&self) -> Vec<&LongTaskRecord> {
        self.tasks
            .values()
            .filter(|t| t.status == LongTaskStatus::WaitingIdle)
            .collect()
    }

    /// Get tasks needing email
    pub fn pending_emails(&self) -> Vec<&LongTaskRecord> {
        self.tasks.values().filter(|t| t.needs_email()).collect()
    }

    /// Get tasks by status
    pub fn by_status(&self, status: LongTaskStatus) -> Vec<&LongTaskRecord> {
        self.tasks.values().filter(|t| t.status == status).collect()
    }

    /// Get tasks by type
    pub fn by_type(&self, task_type: &LongTaskType) -> Vec<&LongTaskRecord> {
        self.tasks
            .values()
            .filter(|t| &t.task_type == task_type)
            .collect()
    }

    /// Status statistics
    pub fn status_stats(&self) -> HashMap<LongTaskStatus, usize> {
        let mut stats = HashMap::new();
        for t in self.tasks.values() {
            *stats.entry(t.status).or_insert(0) += 1;
        }
        stats
    }

    /// Total tasks
    pub fn total(&self) -> usize {
        self.tasks.len()
    }

    /// All tasks
    pub fn all(&self) -> Vec<&LongTaskRecord> {
        self.tasks.values().collect()
    }
}

/// Format task for display
pub fn format_long_task(task: &LongTaskRecord) -> String {
    let mut output = format!(
        "{} [{}]\n  Type: {} | Status: {} | Progress: {}%\n  Description: {}",
        task.id, task.created_at, task.task_type, task.status, task.progress_pct, task.description
    );

    if let Some(est) = task.estimated_minutes {
        output.push_str(&format!("\n  Estimated: {} minutes", est));
    }

    if !task.chain_of_thought.is_empty() {
        output.push_str("\n  Thoughts:");
        for thought in &task.chain_of_thought {
            output.push_str(&format!("\n    - {}", thought));
        }
    }

    if let Some(result) = &task.result {
        output.push_str(&format!("\n  Result: {}", result));
    }

    if let Some(error) = &task.error {
        output.push_str(&format!("\n  Error: {}", error));
    }

    output
}

/// Format task compact
pub fn format_long_task_compact(task: &LongTaskRecord) -> String {
    format!(
        "{}: {} [{}] {}%",
        task.id, task.task_type, task.status, task.progress_pct
    )
}

/// Format task oneline
pub fn format_long_task_oneline(task: &LongTaskRecord) -> String {
    format!("{} [{}]", task.id, task.status)
}

/// Format manager summary
pub fn format_manager_summary(manager: &LongTaskManager) -> String {
    let mut output = String::new();
    output.push_str("=== Long Task Manager ===\n\n");

    output.push_str(&format!("Total Tasks: {}\n", manager.total()));
    output.push_str(&format!("Active: {}\n", manager.active().len()));
    output.push_str(&format!(
        "Waiting for Idle: {}\n\n",
        manager.waiting_for_idle().len()
    ));

    output.push_str("--- By Status ---\n");
    for (status, count) in manager.status_stats() {
        output.push_str(&format!("  {}: {}\n", status, count));
    }

    let active = manager.active();
    if !active.is_empty() {
        output.push_str("\n--- Active Tasks ---\n");
        for task in active {
            output.push_str(&format!("  {}\n", format_long_task_compact(task)));
        }
    }

    output
}

/// Check if query is long-task related
pub fn is_long_task_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("long task")
        || lower.contains("background")
        || lower.contains("research")
        || lower.contains("idle")
        || lower.contains("email when done")
        || lower.contains("takes a while")
}

/// Fun fact about long tasks
pub fn long_task_fun_fact() -> &'static str {
    "Anna can research complex questions when your machine is idle and email you with a complete chain of thought - like having a researcher on call 24/7!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = LongTaskRecord::new("LTASK-001", LongTaskType::Research, "Test", "2024-01-01");
        assert_eq!(task.status, LongTaskStatus::Queued);
        assert_eq!(task.progress_pct, 0);
    }

    #[test]
    fn test_task_lifecycle() {
        let mut task = LongTaskRecord::new("T-1", LongTaskType::Analysis, "Test", "ts");
        task.wait_for_idle();
        assert_eq!(task.status, LongTaskStatus::WaitingIdle);
        task.start("ts2");
        assert_eq!(task.status, LongTaskStatus::Running);
        task.update_progress(50);
        assert_eq!(task.progress_pct, 50);
        task.complete("Done", "ts3");
        assert_eq!(task.status, LongTaskStatus::Completed);
    }

    #[test]
    fn test_task_failure() {
        let mut task = LongTaskRecord::new("T-1", LongTaskType::Download, "Test", "ts");
        task.start("ts");
        task.fail("Network error", "ts2");
        assert_eq!(task.status, LongTaskStatus::Failed);
        assert!(task.error.is_some());
    }

    #[test]
    fn test_chain_of_thought() {
        let mut task = LongTaskRecord::new("T-1", LongTaskType::Research, "Test", "ts");
        task.add_thought("First I'll check the Arch Wiki");
        task.add_thought("Then I'll look at man pages");
        assert_eq!(task.chain_of_thought.len(), 2);
    }

    #[test]
    fn test_email_notification() {
        let mut task = LongTaskRecord::new("T-1", LongTaskType::Research, "Test", "ts");
        task.enable_email("user@example.com");
        assert!(!task.needs_email()); // Not completed yet
        task.complete("Done", "ts2");
        assert!(task.needs_email());
    }

    #[test]
    fn test_manager_create() {
        let mut manager = LongTaskManager::new();
        let id = manager.create(LongTaskType::Backup, "Full backup", "ts");
        assert_eq!(manager.total(), 1);
        assert!(manager.get(&id).is_some());
    }

    #[test]
    fn test_active_tasks() {
        let mut manager = LongTaskManager::new();
        let id1 = manager.create(LongTaskType::Research, "Task 1", "ts");
        let id2 = manager.create(LongTaskType::Research, "Task 2", "ts");
        manager.get_mut(&id2).unwrap().complete("Done", "ts2");
        assert_eq!(manager.active().len(), 1);
    }

    #[test]
    fn test_waiting_for_idle() {
        let mut manager = LongTaskManager::new();
        let id = manager.create(LongTaskType::Analysis, "Analyze logs", "ts");
        manager.get_mut(&id).unwrap().wait_for_idle();
        assert_eq!(manager.waiting_for_idle().len(), 1);
    }

    #[test]
    fn test_is_long_task_query() {
        assert!(is_long_task_query("This research takes a while"));
        assert!(is_long_task_query("Run in background"));
        assert!(is_long_task_query("Email when done"));
        assert!(!is_long_task_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = long_task_fun_fact();
        assert!(fact.contains("idle") || fact.contains("email"));
    }
}
