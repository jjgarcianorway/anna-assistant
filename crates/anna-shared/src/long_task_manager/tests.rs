// v0.0.534: Long Task Manager - Tests (Phase 110)
// Comprehensive test suite for long task management

#[cfg(test)]
mod tests {
    use crate::long_task_manager::types::{LongTaskStatus, LongTaskType};
    use crate::long_task_manager::record::LongTaskRecord;
    use crate::long_task_manager::manager::LongTaskManager;
    use crate::long_task_manager::utils::{is_long_task_query, long_task_fun_fact};

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
