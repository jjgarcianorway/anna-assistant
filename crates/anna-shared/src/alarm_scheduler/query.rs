//! Query and information helpers for alarm scheduler

use super::types::AlarmScheduler;

/// Check if query is about alarms
pub fn is_alarm_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "alarm",
        "notify me",
        "remind me",
        "schedule notification",
        "every monday",
        "every day",
        "weekly report",
        "daily report",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about alarms
pub fn alarm_fun_fact(scheduler: &AlarmScheduler) -> String {
    if scheduler.alarms.is_empty() {
        return "No alarms scheduled yet!".to_string();
    }

    let facts = [
        format!("You have {} alarms scheduled.", scheduler.total_count()),
        format!("{} alarms are currently active.", scheduler.active_count()),
        format!("Alarms have triggered {} times.", scheduler.total_triggers),
        {
            if let Some((topic, count)) = scheduler.most_common_topic() {
                format!("Most common topic: {} ({} alarms)", topic, count)
            } else {
                "No topic stats yet.".to_string()
            }
        },
    ];

    facts[scheduler.total_count() % facts.len()].clone()
}
