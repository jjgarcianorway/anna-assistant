//! Task scheduler for reminders and scheduled tasks.
//!
//! Supports:
//! - One-time reminders ("remind me in 30 minutes")
//! - Recurring tasks ("every day at 8am")

use chrono::{DateTime, Duration, Local, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A scheduled task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: String,
    pub description: String,
    pub trigger: TaskTrigger,
    pub action: TaskAction,
    pub created_at: DateTime<Utc>,
    pub last_run: Option<DateTime<Utc>>,
    pub enabled: bool,
}

/// When the task should run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskTrigger {
    /// Run once at a specific time.
    Once(DateTime<Utc>),
    /// Run daily at a specific local time.
    Daily { time: NaiveTime },
    /// Run every N minutes.
    Interval { minutes: u32 },
}

/// What the task should do.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskAction {
    /// Send a reminder message.
    Reminder { message: String },
    /// Run a system health check and report.
    /// v0.3.156: Added username for personalized briefings.
    HealthCheck {
        #[serde(default)]
        username: Option<String>
    },
    /// Execute a question through Anna.
    Question { question: String },
}

impl ScheduledTask {
    /// Create a new reminder.
    pub fn reminder(message: &str, when: DateTime<Utc>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            description: format!("Reminder: {}", message),
            trigger: TaskTrigger::Once(when),
            action: TaskAction::Reminder { message: message.to_string() },
            created_at: Utc::now(),
            last_run: None,
            enabled: true,
        }
    }

    /// Create a daily task.
    pub fn daily(description: &str, time: NaiveTime, action: TaskAction) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            description: description.to_string(),
            trigger: TaskTrigger::Daily { time },
            action,
            created_at: Utc::now(),
            last_run: None,
            enabled: true,
        }
    }

    /// Create a morning briefing (daily health check).
    /// v0.3.156: Added username parameter for personalized greetings.
    pub fn morning_briefing(time: NaiveTime, username: Option<String>) -> Self {
        Self::daily("Morning Briefing", time, TaskAction::HealthCheck { username })
    }

    /// Check if this task should run now.
    pub fn should_run(&self) -> bool {
        if !self.enabled {
            return false;
        }

        let now = Utc::now();

        match &self.trigger {
            TaskTrigger::Once(when) => {
                if self.last_run.is_some() {
                    return false; // Already ran
                }
                now >= *when
            }
            TaskTrigger::Daily { time } => {
                let local_now = Local::now();
                let today_target = local_now.date_naive().and_time(*time);
                let today_target_utc = DateTime::<Local>::from_naive_utc_and_offset(
                    today_target - Duration::seconds(Local::now().offset().local_minus_utc() as i64),
                    *Local::now().offset(),
                ).with_timezone(&Utc);

                // Check if we should run today
                if now >= today_target_utc {
                    // Haven't run today yet?
                    if let Some(last) = self.last_run {
                        let last_local = last.with_timezone(&Local);
                        last_local.date_naive() < local_now.date_naive()
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
            TaskTrigger::Interval { minutes } => {
                if let Some(last) = self.last_run {
                    now - last >= Duration::minutes(*minutes as i64)
                } else {
                    true
                }
            }
        }
    }

    /// Mark this task as run.
    pub fn mark_run(&mut self) {
        self.last_run = Some(Utc::now());
        // Disable one-time tasks after running
        if matches!(self.trigger, TaskTrigger::Once(_)) {
            self.enabled = false;
        }
    }
}

/// Persistent store for scheduled tasks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskStore {
    pub tasks: Vec<ScheduledTask>,
}

impl TaskStore {
    fn path() -> PathBuf {
        PathBuf::from("/var/lib/anna/scheduled_tasks.json")
    }

    /// Load from disk.
    pub fn load() -> Self {
        std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save to disk.
    pub fn save(&self) -> std::io::Result<()> {
        if let Some(parent) = Self::path().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(Self::path(), json)
    }

    /// Add a task.
    pub fn add(&mut self, task: ScheduledTask) {
        self.tasks.push(task);
    }

    /// Remove a task by ID.
    pub fn remove(&mut self, id: &str) -> bool {
        let len_before = self.tasks.len();
        self.tasks.retain(|t| t.id != id);
        self.tasks.len() < len_before
    }

    /// Get tasks that should run now.
    pub fn get_due(&self) -> Vec<&ScheduledTask> {
        self.tasks.iter().filter(|t| t.should_run()).collect()
    }

    /// Mark a task as run.
    pub fn mark_run(&mut self, id: &str) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.mark_run();
        }
    }

    /// Clean up disabled one-time tasks.
    pub fn cleanup(&mut self) {
        self.tasks.retain(|t| t.enabled || !matches!(t.trigger, TaskTrigger::Once(_)));
    }

    /// Check if morning briefing is already set up.
    pub fn has_morning_briefing(&self) -> bool {
        self.tasks.iter().any(|t| {
            t.enabled && t.description == "Morning Briefing" && matches!(t.action, TaskAction::HealthCheck { .. })
        })
    }

    /// Remove existing morning briefing.
    pub fn remove_morning_briefing(&mut self) {
        self.tasks.retain(|t| !(t.description == "Morning Briefing" && matches!(t.action, TaskAction::HealthCheck { .. })));
    }
}

/// Parse a morning briefing request like "set up morning briefing at 8am".
/// Returns the time if valid.
pub fn parse_morning_briefing(input: &str) -> Option<NaiveTime> {
    let lower = input.to_lowercase();

    // Check for morning briefing keywords
    if !lower.contains("morning briefing") && !lower.contains("morning brief") && !lower.contains("daily briefing") {
        return None;
    }

    // Look for time patterns
    // "at 8am", "at 8:00", "at 8:30am", "at 08:00"
    if let Some(at_idx) = lower.find(" at ") {
        let time_part = lower[at_idx + 4..].trim();
        return parse_time_str(time_part);
    }

    // Default to 8:00 AM if no time specified
    Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap())
}

/// Parse a time string like "8am", "8:30am", "08:00", "14:30".
fn parse_time_str(s: &str) -> Option<NaiveTime> {
    let s = s.trim().to_lowercase();

    // Handle "8am", "8pm", "8:30am", "8:30pm"
    let is_pm = s.contains("pm");
    let s = s.replace("am", "").replace("pm", "").trim().to_string();

    if let Some(colon_idx) = s.find(':') {
        // "8:30" format
        let hours: u32 = s[..colon_idx].trim().parse().ok()?;
        let minutes: u32 = s[colon_idx + 1..].trim().parse().ok()?;
        let hours = if is_pm && hours < 12 { hours + 12 } else if !is_pm && hours == 12 { 0 } else { hours };
        NaiveTime::from_hms_opt(hours, minutes, 0)
    } else {
        // "8" format (just hour)
        let hours: u32 = s.trim().parse().ok()?;
        let hours = if is_pm && hours < 12 { hours + 12 } else if !is_pm && hours == 12 { 0 } else { hours };
        NaiveTime::from_hms_opt(hours, 0, 0)
    }
}

/// Parse a reminder request like "remind me in 30 minutes to check email".
pub fn parse_reminder(input: &str) -> Option<(String, DateTime<Utc>)> {
    let lower = input.to_lowercase();

    // "remind me in X minutes/hours to Y"
    if lower.starts_with("remind me in ") {
        let rest = &input[13..];

        // Parse duration
        let duration = if let Some(idx) = rest.find(" minute") {
            let num_str = rest[..idx].trim();
            num_str.parse::<i64>().ok().map(Duration::minutes)
        } else if let Some(idx) = rest.find(" hour") {
            let num_str = rest[..idx].trim();
            num_str.parse::<i64>().ok().map(Duration::hours)
        } else if let Some(idx) = rest.find(" second") {
            let num_str = rest[..idx].trim();
            num_str.parse::<i64>().ok().map(Duration::seconds)
        } else {
            None
        }?;

        // Parse message (after "to" or entire thing)
        let message = if let Some(idx) = rest.find(" to ") {
            rest[idx + 4..].trim().to_string()
        } else {
            "Reminder".to_string()
        };

        let when = Utc::now() + duration;
        return Some((message, when));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_reminder_minutes() {
        let (msg, when) = parse_reminder("remind me in 30 minutes to check email").unwrap();
        assert_eq!(msg, "check email");
        assert!(when > Utc::now());
        assert!(when < Utc::now() + Duration::minutes(31));
    }

    #[test]
    fn test_parse_reminder_hours() {
        let (msg, when) = parse_reminder("remind me in 2 hours to call mom").unwrap();
        assert_eq!(msg, "call mom");
        assert!(when > Utc::now() + Duration::hours(1));
    }

    #[test]
    fn test_task_should_run_once() {
        let task = ScheduledTask::reminder("test", Utc::now() - Duration::minutes(1));
        assert!(task.should_run());
    }

    #[test]
    fn test_task_should_not_run_future() {
        let task = ScheduledTask::reminder("test", Utc::now() + Duration::hours(1));
        assert!(!task.should_run());
    }

    #[test]
    fn test_parse_morning_briefing_default() {
        let time = parse_morning_briefing("set up morning briefing").unwrap();
        assert_eq!(time, NaiveTime::from_hms_opt(8, 0, 0).unwrap());
    }

    #[test]
    fn test_parse_morning_briefing_with_time() {
        let time = parse_morning_briefing("set up morning briefing at 7am").unwrap();
        assert_eq!(time, NaiveTime::from_hms_opt(7, 0, 0).unwrap());
    }

    #[test]
    fn test_parse_morning_briefing_pm() {
        let time = parse_morning_briefing("enable daily briefing at 9pm").unwrap();
        assert_eq!(time, NaiveTime::from_hms_opt(21, 0, 0).unwrap());
    }

    #[test]
    fn test_parse_time_str() {
        assert_eq!(parse_time_str("8am"), NaiveTime::from_hms_opt(8, 0, 0));
        assert_eq!(parse_time_str("8:30am"), NaiveTime::from_hms_opt(8, 30, 0));
        assert_eq!(parse_time_str("14:30"), NaiveTime::from_hms_opt(14, 30, 0));
        assert_eq!(parse_time_str("2pm"), NaiveTime::from_hms_opt(14, 0, 0));
    }
}
