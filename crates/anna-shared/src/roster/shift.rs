//! Shift preference enum (v0.0.182).

use serde::{Deserialize, Serialize};

/// Shift preference for staff
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Shift {
    /// Morning shift (6am - 2pm)
    Morning,
    /// Day shift (9am - 5pm)
    Day,
    /// Evening shift (2pm - 10pm)
    Evening,
    /// Night shift (10pm - 6am)
    Night,
    /// Flexible (available any time)
    Flexible,
}

impl Shift {
    /// Check if this shift is currently active
    pub fn is_active(&self) -> bool {
        use chrono::Timelike;
        let hour = chrono::Local::now().hour();
        match self {
            Shift::Morning => (6..14).contains(&hour),
            Shift::Day => (9..17).contains(&hour),
            Shift::Evening => (14..22).contains(&hour),
            Shift::Night => !(6..22).contains(&hour),
            Shift::Flexible => true,
        }
    }

    /// Get shift description
    pub fn description(&self) -> &'static str {
        match self {
            Shift::Morning => "morning (6am-2pm)",
            Shift::Day => "day (9am-5pm)",
            Shift::Evening => "evening (2pm-10pm)",
            Shift::Night => "night (10pm-6am)",
            Shift::Flexible => "flexible hours",
        }
    }
}

impl std::fmt::Display for Shift {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Shift::Morning => write!(f, "morning"),
            Shift::Day => write!(f, "day"),
            Shift::Evening => write!(f, "evening"),
            Shift::Night => write!(f, "night"),
            Shift::Flexible => write!(f, "flexible"),
        }
    }
}
