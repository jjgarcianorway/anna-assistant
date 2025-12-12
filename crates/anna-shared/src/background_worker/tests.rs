//! Background worker acceptance tests (v0.0.430).

use super::*;

/// Test scenario 1: Long-running ticket workflow
/// 1. User submits ticket requiring >60s analysis
/// 2. Anna detects it and responds "I'll analyze this in the background..."
/// 3. Creates background job
/// 4. On completion, queues message for next annactl open
#[test]
fn test_long_ticket_workflow() {
    // Create scheduler
    let path = format!("/tmp/anna_test_long_ticket_{}", std::process::id());
    let mut scheduler = JobScheduler::new(&path);

    // 1. Create long ticket job
    let job = BackgroundJob::long_ticket("TKT-LONG-001").with_metadata("estimated_time", "120s");

    // 2. Enqueue the job
    let job_id = scheduler.enqueue(job);
    assert!(scheduler.get(&job_id).is_some());
    assert_eq!(scheduler.count_pending(), 1);

    // 3. Mark as running (simulating execution start)
    scheduler.mark_running(&job_id);
    assert_eq!(scheduler.count_running(), 1);
    assert_eq!(scheduler.count_pending(), 0);

    // 4. Mark as completed with summary
    let summary = "Analysis complete: Found 3 issues, suggested 2 fixes".to_string();
    scheduler.mark_completed(&job_id, Some(summary.clone()));

    // Verify completion
    let job = scheduler.get(&job_id).unwrap();
    assert!(matches!(job.status, JobStatus::Completed { .. }));

    // 5. Queue message for user
    let msg_storage = storage::PendingMessageStorage::new(&path);
    let msg = storage::PendingMessage::from_long_ticket("TKT-LONG-001", &summary);
    msg_storage.add(msg).unwrap();

    // Verify message is queued
    assert_eq!(msg_storage.count(), 1);

    // 6. Take messages (simulating annactl open)
    let messages = msg_storage.take_all().unwrap();
    assert_eq!(messages.len(), 1);
    assert!(messages[0].subject.contains("TKT-LONG-001"));

    // Queue should be empty now
    assert_eq!(msg_storage.count(), 0);

    // Cleanup
    let _ = std::fs::remove_dir_all(&path);
}

/// Test scenario 2: Monitor alert workflow
/// 1. User says "monitor disk at /data and alert if >90%"
/// 2. Anna creates monitor
/// 3. On threshold breach, alerts via configured channel
/// 4. Respects cooldown period
#[test]
fn test_monitor_alert_workflow() {
    // 1. Create monitor
    let mut monitor = Monitor::new(
        "disk-data",
        "Monitor /data disk space",
        MonitorCheck::DiskSpace {
            path: "/data".to_string(),
        },
        ThresholdCondition::GreaterThan { value: 90.0 },
    )
    .with_message("ALERT: Disk usage at {value}% on /data")
    .with_priority(notifications::AlertPriority::High);

    // Verify monitor created correctly
    assert_eq!(monitor.id, "disk-data");
    assert!(monitor.enabled);
    assert!(monitor.is_due(
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    ));

    // 2. Simulate check with high value (would trigger)
    let condition = ThresholdCondition::GreaterThan { value: 90.0 };
    assert!(condition.check(95.0)); // Should trigger
    assert!(!condition.check(85.0)); // Should not trigger

    // 3. Test cooldown logic
    let now = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // No last alert - not in cooldown
    assert!(!monitor.in_cooldown(now));

    // Set last alert to now - should be in cooldown
    monitor.last_alert = Some(now);
    assert!(monitor.in_cooldown(now));

    // Set last alert to >24h ago - not in cooldown
    monitor.last_alert = Some(now - (25 * 3600));
    assert!(!monitor.in_cooldown(now));
}

/// Test scenario 3: Idle learning workflow
/// 1. System detects idle
/// 2. Runs recipe consolidation
/// 3. Respects daily limits
#[test]
fn test_idle_learning_workflow() {
    // 1. Create manager with config
    let config = idle_learning::IdleLearningConfig {
        enabled: true,
        cpu_threshold: 0.3,
        max_jobs_per_day: 5,
        recipe_consolidation: true,
        doc_refresh: true,
        model_benchmark: false,
        min_idle_time_secs: 0, // No wait for test
    };

    let mut state = idle_learning::IdleLearningState::default();
    state.idle_since = Some(
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    );

    let manager = idle_learning::IdleLearningManager::with_state(config, state);

    // 2. Get next job (should be recipe consolidation)
    let job = manager.get_next_job();
    assert!(job.is_some());
    let job = job.unwrap();
    assert!(matches!(job.kind, JobKind::RecipeConsolidation));

    // 3. Test daily limits
    let config2 = idle_learning::IdleLearningConfig {
        enabled: true,
        max_jobs_per_day: 2,
        ..Default::default()
    };
    let mut state2 = idle_learning::IdleLearningState::default();
    state2.jobs_today = 2;
    state2.last_reset_date = idle_learning_current_date();

    let mut manager2 = idle_learning::IdleLearningManager::with_state(config2, state2);
    // Should not run because at max
    assert!(!manager2.should_run());
}

/// Test job priority ordering
#[test]
fn test_job_priority_ordering() {
    let path = format!("/tmp/anna_test_priority_{}", std::process::id());
    let mut scheduler = JobScheduler::new(&path);

    // Add jobs with different priorities
    scheduler.enqueue(BackgroundJob::doc_refresh()); // Low priority
    scheduler.enqueue(BackgroundJob::long_ticket("TKT-1")); // Normal priority
    scheduler.enqueue(
        BackgroundJob::monitor_check("disk", JobPriority::High), // High priority
    );

    let now = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Get due jobs with low CPU (all should be eligible)
    let due = scheduler.get_due_jobs(now, 0.1);

    // Should be ordered by priority: High, Normal, Low
    assert!(due.len() >= 2);
    assert_eq!(due[0].priority, JobPriority::High);

    let _ = std::fs::remove_dir_all(&path);
}

/// Test job retry mechanism
#[test]
fn test_job_retry_mechanism() {
    let mut job = BackgroundJob::doc_refresh().with_max_retries(3);

    // First failure - should retry (1/3 retries used)
    job.mark_failed("Network error");
    assert_eq!(job.retry_count, 1);
    assert!(job.status.is_runnable()); // Back to pending

    // Second failure - should retry (2/3 retries used)
    job.mark_failed("Network error again");
    assert_eq!(job.retry_count, 2);
    assert!(job.status.is_runnable()); // Still pending

    // Third failure - should retry (3/3 retries used)
    job.mark_failed("Network error third");
    assert_eq!(job.retry_count, 3);
    assert!(job.status.is_terminal()); // Failed permanently - no more retries
}

/// Test notification rate limiting
#[test]
fn test_notification_rate_limiting() {
    let config = notifications::NotificationConfig {
        desktop_enabled: true,
        rate_limits: [("desktop".to_string(), 300)].into_iter().collect(),
        ..Default::default()
    };

    let dispatcher = notifications::NotificationDispatcher::new(config);

    // First send should be allowed (no previous sends)
    assert!(dispatcher.can_send_to(notifications::NotificationChannel::Desktop));
}

/// Test scheduler status summary
#[test]
fn test_scheduler_status_summary() {
    let path = format!("/tmp/anna_test_status_{}", std::process::id());
    let mut scheduler = JobScheduler::new(&path);

    scheduler.enqueue(BackgroundJob::doc_refresh());
    scheduler.enqueue(BackgroundJob::long_ticket("TKT-1"));

    let summary = scheduler.status_summary();
    assert_eq!(summary.pending, 2);
    assert_eq!(summary.running, 0);
    assert!(summary.enabled);

    let _ = std::fs::remove_dir_all(&path);
}

/// Test pending message queue
#[test]
fn test_pending_message_queue() {
    let path = format!("/tmp/anna_test_messages_{}", std::process::id());
    let storage = storage::PendingMessageStorage::new(&path);

    // Add messages
    storage
        .add(storage::PendingMessage::new(
            "Test Subject",
            "Test body",
            "test",
        ))
        .unwrap();

    storage
        .add(storage::PendingMessage::from_monitor(
            "disk-check",
            "Disk space critical!",
        ))
        .unwrap();

    assert_eq!(storage.count(), 2);

    // Take all clears queue
    let messages = storage.take_all().unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(storage.count(), 0);

    let _ = std::fs::remove_dir_all(&path);
}

/// Test reminder scheduling
#[test]
fn test_reminder_scheduling() {
    let reminder = monitors::Reminder::new(
        "weekly-backup",
        "Run weekly backup",
        monitors::ReminderSchedule::Weekly {
            day: 1, // Monday
            hour: 2,
            minute: 0,
        },
    );

    assert!(reminder.enabled);
    assert!(reminder.next_trigger.is_some());

    // One-time reminder in the past should have no next trigger
    let past_reminder = monitors::Reminder::new(
        "past",
        "Past reminder",
        monitors::ReminderSchedule::Once { at: 1000 },
    );
    assert!(past_reminder.next_trigger.is_none());
}

/// Helper to get current date for idle learning tests
fn idle_learning_current_date() -> u32 {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let days = secs / 86400;
    let year = 1970 + (days / 365) as u32;
    let day_of_year = (days % 365) as u32;
    let month = (day_of_year / 30) + 1;
    let day = (day_of_year % 30) + 1;
    year * 10000 + month * 100 + day
}
