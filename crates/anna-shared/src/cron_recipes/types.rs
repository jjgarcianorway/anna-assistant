//! Cron job recipe types (v0.0.234).

use serde::{Deserialize, Serialize};

/// Cron schedule presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CronPreset {
    /// Every minute
    EveryMinute,
    /// Every hour
    Hourly,
    /// Every day at midnight
    Daily,
    /// Every week on Sunday at midnight
    Weekly,
    /// First day of month at midnight
    Monthly,
    /// Custom expression
    Custom,
}

impl CronPreset {
    pub fn display_name(&self) -> &'static str {
        match self {
            CronPreset::EveryMinute => "every minute",
            CronPreset::Hourly => "hourly",
            CronPreset::Daily => "daily",
            CronPreset::Weekly => "weekly",
            CronPreset::Monthly => "monthly",
            CronPreset::Custom => "custom",
        }
    }

    /// Get the cron expression for this preset
    pub fn expression(&self) -> &'static str {
        match self {
            CronPreset::EveryMinute => "* * * * *",
            CronPreset::Hourly => "0 * * * *",
            CronPreset::Daily => "0 0 * * *",
            CronPreset::Weekly => "0 0 * * 0",
            CronPreset::Monthly => "0 0 1 * *",
            CronPreset::Custom => "* * * * *",
        }
    }
}

impl std::fmt::Display for CronPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Cron recipe features
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CronFeature {
    /// Add a new cron job
    AddJob,
    /// List current cron jobs
    ListJobs,
    /// Edit crontab
    EditCrontab,
    /// Remove a cron job
    RemoveJob,
    /// View cron logs
    ViewLogs,
    /// Cron syntax explanation
    SyntaxHelp,
    /// Environment variables in cron
    Environment,
    /// Debug a cron job
    DebugJob,
}

impl CronFeature {
    pub fn display_name(&self) -> &'static str {
        match self {
            CronFeature::AddJob => "add cron job",
            CronFeature::ListJobs => "list cron jobs",
            CronFeature::EditCrontab => "edit crontab",
            CronFeature::RemoveJob => "remove cron job",
            CronFeature::ViewLogs => "view cron logs",
            CronFeature::SyntaxHelp => "cron syntax help",
            CronFeature::Environment => "cron environment",
            CronFeature::DebugJob => "debug cron job",
        }
    }

    /// Keywords that indicate this feature
    pub fn keywords(&self) -> &'static [&'static str] {
        match self {
            CronFeature::AddJob => &["add", "create", "new", "schedule", "set up"],
            CronFeature::ListJobs => &["list", "show", "view", "display", "current"],
            CronFeature::EditCrontab => &["edit", "modify", "change", "crontab -e"],
            CronFeature::RemoveJob => &["remove", "delete", "unschedule"],
            CronFeature::ViewLogs => &["logs", "log", "output", "history"],
            CronFeature::SyntaxHelp => &["syntax", "format", "expression", "schedule", "meaning"],
            CronFeature::Environment => &["environment", "env", "variable", "path"],
            CronFeature::DebugJob => &["debug", "not running", "failing", "troubleshoot"],
        }
    }
}

/// A cron recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronRecipe {
    pub feature: CronFeature,
    pub description: String,
    pub commands: Vec<String>,
    pub answer_template: String,
    pub notes: Vec<String>,
}

impl CronRecipe {
    pub fn new(feature: CronFeature, description: &str) -> Self {
        Self {
            feature,
            description: description.to_string(),
            commands: Vec::new(),
            answer_template: String::new(),
            notes: Vec::new(),
        }
    }

    pub fn with_command(mut self, cmd: &str) -> Self {
        self.commands.push(cmd.to_string());
        self
    }

    pub fn with_answer(mut self, answer: &str) -> Self {
        self.answer_template = answer.to_string();
        self
    }

    pub fn with_note(mut self, note: &str) -> Self {
        self.notes.push(note.to_string());
        self
    }
}
