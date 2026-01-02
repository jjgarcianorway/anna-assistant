//! Type definitions for REPL greeting system.

use crate::ui::colors;

/// REPL greeting data
#[derive(Debug, Clone)]
pub struct ReplGreeting {
    /// User's name (from env or "there")
    pub user_name: String,
    /// Number of tickets handled recently
    pub tickets_handled: usize,
    /// Top topics/domains
    pub top_topics: Vec<String>,
    /// System health status
    pub system_status: SystemStatus,
    /// Status summary line
    pub status_summary: String,
    /// Number of active staff (domains with activity)
    pub active_staff: usize,
    /// Departments with activity
    pub departments: Vec<String>,
    /// Is this the user's first time?
    pub first_time: bool,
    /// System errors to announce (v0.0.463)
    pub errors: Vec<SystemError>,
}

/// System error type for greeting announcements (v0.0.463)
#[derive(Debug, Clone)]
pub struct SystemError {
    /// Error category (daemon, ollama, models, etc.)
    pub category: String,
    /// Error message
    pub message: String,
    /// Suggested fix
    pub fix_hint: Option<String>,
}

/// System health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemStatus {
    Ok,
    Warn,
    Critical,
}

impl std::fmt::Display for SystemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemStatus::Ok => write!(f, "OK"),
            SystemStatus::Warn => write!(f, "WARN"),
            SystemStatus::Critical => write!(f, "CRIT"),
        }
    }
}

impl SystemStatus {
    pub fn color(&self) -> &'static str {
        match self {
            SystemStatus::Ok => colors::OK,
            SystemStatus::Warn => colors::WARN,
            SystemStatus::Critical => colors::ERR,
        }
    }
}
