//! Time-related types for activity tracking

use serde::{Deserialize, Serialize};

/// Time of day bucket
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeOfDay {
    /// 6:00 - 12:00
    Morning,
    /// 12:00 - 18:00
    Afternoon,
    /// 18:00 - 22:00
    Evening,
    /// 22:00 - 6:00
    Night,
}

impl TimeOfDay {
    /// Display name
    pub fn display(&self) -> &'static str {
        match self {
            Self::Morning => "Morning",
            Self::Afternoon => "Afternoon",
            Self::Evening => "Evening",
            Self::Night => "Night",
        }
    }

    /// Determine time of day from hour (0-23)
    pub fn from_hour(hour: u8) -> Self {
        match hour {
            6..=11 => Self::Morning,
            12..=17 => Self::Afternoon,
            18..=21 => Self::Evening,
            _ => Self::Night,
        }
    }
}

/// Day of week
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl DayOfWeek {
    /// Display name
    pub fn display(&self) -> &'static str {
        match self {
            Self::Monday => "Monday",
            Self::Tuesday => "Tuesday",
            Self::Wednesday => "Wednesday",
            Self::Thursday => "Thursday",
            Self::Friday => "Friday",
            Self::Saturday => "Saturday",
            Self::Sunday => "Sunday",
        }
    }

    /// Short display name
    pub fn short(&self) -> &'static str {
        match self {
            Self::Monday => "Mon",
            Self::Tuesday => "Tue",
            Self::Wednesday => "Wed",
            Self::Thursday => "Thu",
            Self::Friday => "Fri",
            Self::Saturday => "Sat",
            Self::Sunday => "Sun",
        }
    }

    /// From day index (0 = Monday)
    pub fn from_index(index: u8) -> Self {
        match index % 7 {
            0 => Self::Monday,
            1 => Self::Tuesday,
            2 => Self::Wednesday,
            3 => Self::Thursday,
            4 => Self::Friday,
            5 => Self::Saturday,
            _ => Self::Sunday,
        }
    }
}
