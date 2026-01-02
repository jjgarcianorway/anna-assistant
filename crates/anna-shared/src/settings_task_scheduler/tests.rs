// v0.0.610: Settings Task Scheduler - Tests (Phase 186)
// Test suite for task scheduler

#[cfg(test)]
mod tests {
    use super::super::types::{TaskFrequency, TaskType, TaskState};
    use super::super::task_definition::TaskDefinition;
    use super::super::task_instance::TaskInstance;
    use super::super::scheduler::SettingsTaskScheduler;
    use super::super::utils::{is_task_scheduler_query, task_scheduler_fun_fact};

    #[test]
    fn test_frequency_display() {
        assert_eq!(format!("{}", TaskFrequency::Daily), "daily");
        assert_eq!(format!("{}", TaskFrequency::Hourly), "hourly");
    }

    #[test]
    fn test_type_display() {
        assert_eq!(format!("{}", TaskType::Backup), "backup");
        assert_eq!(format!("{}", TaskType::Sync), "sync");
    }

    #[test]
    fn test_state_display() {
        assert_eq!(format!("{}", TaskState::Running), "running");
        assert_eq!(format!("{}", TaskState::Completed), "completed");
    }

    #[test]
    fn test_definition_new() {
        let d = TaskDefinition::new("d1", TaskType::Backup);
        assert!(d.enabled);
    }

    #[test]
    fn test_definition_builder() {
        let d = TaskDefinition::new("d1", TaskType::Sync)
            .name("Daily Sync")
            .frequency(TaskFrequency::Daily)
            .priority(10);
        assert_eq!(d.priority, 10);
    }

    #[test]
    fn test_instance_new() {
        let i = TaskInstance::new("i1", "d1");
        assert_eq!(i.state, TaskState::Pending);
    }

    #[test]
    fn test_instance_lifecycle() {
        let mut i = TaskInstance::new("i1", "d1");
        i.start(100);
        assert_eq!(i.state, TaskState::Running);
        i.complete(200, "Done");
        assert_eq!(i.state, TaskState::Completed);
    }

    #[test]
    fn test_scheduler_new() {
        let s = SettingsTaskScheduler::new();
        assert_eq!(s.definition_count(), 0);
    }

    #[test]
    fn test_scheduler_add_definition() {
        let mut s = SettingsTaskScheduler::new();
        s.add_definition(TaskDefinition::new("d1", TaskType::Backup));
        assert_eq!(s.definition_count(), 1);
    }

    #[test]
    fn test_scheduler_schedule() {
        let mut s = SettingsTaskScheduler::new();
        s.schedule(TaskInstance::new("i1", "d1"));
        assert_eq!(s.instance_count(), 1);
    }

    #[test]
    fn test_is_task_scheduler_query() {
        assert!(is_task_scheduler_query("scheduled task"));
        assert!(!is_task_scheduler_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = task_scheduler_fun_fact();
        assert!(fact.contains("schedule"));
    }
}
