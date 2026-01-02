// v0.0.568: Settings Scheduler - Utility Functions
// Helper functions for formatting and queries

use super::scheduler::SettingsScheduler;

/// Format schedules for display
pub fn format_schedules(scheduler: &SettingsScheduler) -> String {
    let mut output = String::new();

    output.push_str("=== Scheduled Settings ===\n\n");

    if scheduler.count() == 0 {
        output.push_str("No scheduled changes.\n");
        return output;
    }

    for s in scheduler.list() {
        let status = if s.enabled { "enabled" } else { "disabled" };
        output.push_str(&format!(
            "• {} [{}]\n  Trigger: {}\n  Action: {}\n  Runs: {}\n\n",
            s.name, status, s.trigger, s.action, s.run_count
        ));
    }

    output
}

/// Check if query is about scheduling
pub fn is_schedule_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("schedule")
        || lower.contains("at time")
        || lower.contains("every day")
        || lower.contains("automate settings")
}

/// Fun fact about scheduling
pub fn scheduler_fun_fact() -> &'static str {
    "You can schedule Anna to automatically switch settings at specific times!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_schedules() {
        let scheduler = SettingsScheduler::new();
        let output = format_schedules(&scheduler);
        assert!(output.contains("Scheduled"));
    }

    #[test]
    fn test_is_schedule_query() {
        assert!(is_schedule_query("schedule settings change"));
        assert!(is_schedule_query("every day at 9am"));
        assert!(!is_schedule_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = scheduler_fun_fact();
        assert!(fact.contains("schedule"));
    }
}
