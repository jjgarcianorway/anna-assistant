// v0.0.568: Settings Scheduler - Trigger Types
// Schedule triggers and system events

use serde::{Deserialize, Serialize};

/// Schedule trigger type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleTrigger {
    /// At a specific time
    AtTime(chrono::DateTime<chrono::Utc>),
    /// After a duration from now
    AfterDuration(chrono::Duration),
    /// Daily at a specific hour (0-23)
    DailyAt { hour: u8, minute: u8 },
    /// On specific weekdays
    Weekly { days: Vec<chrono::Weekday>, hour: u8, minute: u8 },
    /// On system event
    OnEvent(ScheduleEvent),
}

impl std::fmt::Display for ScheduleTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AtTime(t) => write!(f, "At {}", t.format("%Y-%m-%d %H:%M")),
            Self::AfterDuration(d) => write!(f, "After {} seconds", d.num_seconds()),
            Self::DailyAt { hour, minute } => write!(f, "Daily at {:02}:{:02}", hour, minute),
            Self::Weekly { days, hour, minute } => {
                let day_names: Vec<_> = days.iter().map(|d| format!("{:?}", d)).collect();
                write!(f, "{} at {:02}:{:02}", day_names.join(", "), hour, minute)
            }
            Self::OnEvent(e) => write!(f, "On {}", e),
        }
    }
}

/// System events that can trigger schedule
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleEvent {
    /// System startup
    Startup,
    /// System shutdown
    Shutdown,
    /// Network connected
    NetworkConnected,
    /// Network disconnected
    NetworkDisconnected,
    /// Battery low
    BatteryLow,
    /// Battery charging
    BatteryCharging,
}

impl std::fmt::Display for ScheduleEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Startup => write!(f, "Startup"),
            Self::Shutdown => write!(f, "Shutdown"),
            Self::NetworkConnected => write!(f, "Network Connected"),
            Self::NetworkDisconnected => write!(f, "Network Disconnected"),
            Self::BatteryLow => write!(f, "Battery Low"),
            Self::BatteryCharging => write!(f, "Battery Charging"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_display() {
        let t = ScheduleTrigger::DailyAt { hour: 9, minute: 30 };
        assert_eq!(format!("{}", t), "Daily at 09:30");
    }

    #[test]
    fn test_event_display() {
        assert_eq!(format!("{}", ScheduleEvent::Startup), "Startup");
    }
}
