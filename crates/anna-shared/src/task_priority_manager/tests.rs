//! Task Priority Manager Tests

#[cfg(test)]
mod tests {
    use super::super::priority::TaskPriority;
    use super::super::state::TaskState;
    use super::super::manager::TaskPriorityManager;
    use super::super::formatting::{format_task_manager, format_task_manager_compact, format_task_manager_oneline};
    use super::super::utils::{is_task_manager_query, task_manager_fun_fact};

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
    fn test_format_manager_compact() {
        let mut manager = TaskPriorityManager::new();
        manager.add("task1".to_string(), "Do something".to_string(), TaskPriority::Normal, 1000);

        let output = format_task_manager_compact(&manager);
        assert!(output.contains("1 total"));
        assert!(output.contains("1 pending"));
    }

    #[test]
    fn test_format_manager_oneline() {
        let mut manager = TaskPriorityManager::new();
        manager.add("task1".to_string(), "Do something".to_string(), TaskPriority::Normal, 1000);

        let output = format_task_manager_oneline(&manager);
        assert!(output.contains("1 tasks"));
        assert!(output.contains("1 pending"));
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
