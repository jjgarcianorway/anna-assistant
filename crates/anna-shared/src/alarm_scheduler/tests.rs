//! Tests for alarm scheduler

#[cfg(test)]
mod tests {
    use crate::alarm_scheduler::*;

    fn make_alarm(topic: &str, freq: AlarmFrequency) -> AlarmRecord {
        AlarmRecord {
            id: format!("ALM-{}", topic),
            description: format!("Report on {}", topic),
            topic: topic.to_string(),
            frequency: freq,
            day_of_week: None,
            hour: 9,
            minute: 0,
            status: AlarmStatus::Active,
            created_at: 1234567890,
            last_triggered: None,
            next_trigger: None,
            trigger_count: 0,
        }
    }

    #[test]
    fn test_alarm_frequency() {
        assert_eq!(AlarmFrequency::Daily.name(), "Daily");
        assert_eq!(AlarmFrequency::Weekly.name(), "Weekly");
    }

    #[test]
    fn test_day_of_week() {
        assert_eq!(DayOfWeek::Monday.name(), "Monday");
        assert_eq!(DayOfWeek::Friday.short(), "Fri");
    }

    #[test]
    fn test_alarm_status() {
        assert_eq!(AlarmStatus::Active.name(), "Active");
        assert_eq!(AlarmStatus::Paused.name(), "Paused");
    }

    #[test]
    fn test_add_alarm() {
        let mut scheduler = AlarmScheduler::new();
        scheduler.add(make_alarm("storage", AlarmFrequency::Weekly));

        assert_eq!(scheduler.total_count(), 1);
        assert!(scheduler.get("ALM-storage").is_some());
    }

    #[test]
    fn test_trigger_alarm() {
        let mut scheduler = AlarmScheduler::new();
        scheduler.add(make_alarm("storage", AlarmFrequency::Weekly));

        assert!(scheduler.trigger("ALM-storage", 1234567890));
        assert_eq!(scheduler.total_triggers, 1);
        assert_eq!(scheduler.get("ALM-storage").unwrap().trigger_count, 1);
    }

    #[test]
    fn test_once_expires() {
        let mut scheduler = AlarmScheduler::new();
        scheduler.add(make_alarm("backup", AlarmFrequency::Once));

        scheduler.trigger("ALM-backup", 1234567890);
        assert_eq!(scheduler.get("ALM-backup").unwrap().status, AlarmStatus::Expired);
    }

    #[test]
    fn test_pause_resume() {
        let mut scheduler = AlarmScheduler::new();
        scheduler.add(make_alarm("storage", AlarmFrequency::Daily));

        assert!(scheduler.pause("ALM-storage"));
        assert_eq!(scheduler.get("ALM-storage").unwrap().status, AlarmStatus::Paused);

        assert!(scheduler.resume("ALM-storage"));
        assert_eq!(scheduler.get("ALM-storage").unwrap().status, AlarmStatus::Active);
    }

    #[test]
    fn test_cancel() {
        let mut scheduler = AlarmScheduler::new();
        scheduler.add(make_alarm("storage", AlarmFrequency::Daily));

        assert!(scheduler.cancel("ALM-storage"));
        assert_eq!(scheduler.get("ALM-storage").unwrap().status, AlarmStatus::Cancelled);
    }

    #[test]
    fn test_due_at() {
        let mut scheduler = AlarmScheduler::new();
        scheduler.add(make_alarm("storage", AlarmFrequency::Daily));

        let due = scheduler.due_at(9, 0, None);
        assert_eq!(due.len(), 1);

        let not_due = scheduler.due_at(10, 0, None);
        assert_eq!(not_due.len(), 0);
    }

    #[test]
    fn test_parse_day_of_week() {
        assert_eq!(parse_day_of_week("monday"), Some(DayOfWeek::Monday));
        assert_eq!(parse_day_of_week("Mon"), Some(DayOfWeek::Monday));
        assert_eq!(parse_day_of_week("invalid"), None);
    }

    #[test]
    fn test_format_alarm_scheduler() {
        let mut scheduler = AlarmScheduler::new();
        scheduler.add(make_alarm("storage", AlarmFrequency::Weekly));

        let output = format_alarm_scheduler(&scheduler);
        assert!(output.contains("Alarm Scheduler"));
        assert!(output.contains("storage"));
    }

    #[test]
    fn test_is_alarm_query() {
        assert!(is_alarm_query("notify me every monday"));
        assert!(is_alarm_query("set an alarm"));
        assert!(is_alarm_query("daily report on storage"));
        assert!(!is_alarm_query("what is the weather?"));
    }

    #[test]
    fn test_alarm_fun_fact() {
        let mut scheduler = AlarmScheduler::new();
        scheduler.add(make_alarm("storage", AlarmFrequency::Weekly));

        let fact = alarm_fun_fact(&scheduler);
        assert!(!fact.is_empty());
    }
}
