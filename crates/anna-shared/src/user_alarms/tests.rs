//! Tests for user alarms functionality.

#[cfg(test)]
mod tests {
    use crate::user_alarms::parsing::{parse_alarm_request, parse_time_from_input};
    use crate::user_alarms::types::{AlarmSchedule, UserAlarm, Weekday};

    #[test]
    fn test_weekday_from_str() {
        assert_eq!(Weekday::from_str("monday"), Some(Weekday::Monday));
        assert_eq!(Weekday::from_str("Mon"), Some(Weekday::Monday));
        assert_eq!(Weekday::from_str("fri"), Some(Weekday::Friday));
        assert_eq!(Weekday::from_str("invalid"), None);
    }

    #[test]
    fn test_alarm_creation() {
        let alarm = UserAlarm::new(
            "Storage check",
            "disk usage",
            AlarmSchedule::Daily { hour: 9, minute: 0 },
        );
        assert!(alarm.enabled);
        assert!(alarm.id.starts_with("ALM-"));
    }

    #[test]
    fn test_parse_weekly_alarm() {
        let alarm = parse_alarm_request("remind me every monday at 9 about storage");
        assert!(alarm.is_some());
        let a = alarm.unwrap();
        assert!(matches!(a.schedule, AlarmSchedule::Weekly { day: Weekday::Monday, .. }));
    }

    #[test]
    fn test_parse_daily_alarm() {
        let alarm = parse_alarm_request("notify me daily at 10:30 about failed services");
        assert!(alarm.is_some());
        let a = alarm.unwrap();
        assert!(matches!(a.schedule, AlarmSchedule::Daily { hour: 10, minute: 30 }));
    }

    #[test]
    fn test_parse_disk_condition() {
        use crate::user_alarms::types::AlarmCondition;
        let alarm = parse_alarm_request("alert me when disk is above 90%");
        assert!(alarm.is_some());
        let a = alarm.unwrap();
        assert!(matches!(
            a.schedule,
            AlarmSchedule::Conditional { condition: AlarmCondition::DiskAbove { threshold_percent: 90, .. } }
        ));
    }

    #[test]
    fn test_time_parsing() {
        assert_eq!(parse_time_from_input("at 9"), (9, 0));
        assert_eq!(parse_time_from_input("at 9:30"), (9, 30));
        assert_eq!(parse_time_from_input("at 14:00"), (14, 0));
    }
}
