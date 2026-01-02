//! Strategic Thinking Tests - Phase 91
//!
//! Unit tests for strategic thinking functionality.

#[cfg(test)]
mod tests {
    use super::super::*;

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
