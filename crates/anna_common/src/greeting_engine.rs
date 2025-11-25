//! Greeting Engine - Situational and personality-aware greetings
//!
//! v6.47.0: Generate context-aware greetings based on system state and learned patterns

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Greeting context from system telemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreetingContext {
    /// Current system state summary
    pub system_state: SystemState,

    /// Notable changes since last interaction
    pub changes: Vec<SystemChange>,

    /// Time since last interaction
    pub time_since_last: Duration,

    /// Current time of day
    pub time_of_day: TimeOfDay,
}

/// High-level system state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SystemState {
    Healthy,
    Warning { reason: String },
    Critical { reason: String },
    Recovering { from: String },
}

/// A notable system change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemChange {
    /// Type of change
    pub change_type: ChangeType,
    /// Human-readable description
    pub description: String,
    /// Severity level
    pub severity: ChangeSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    PackageUpdate,
    ServiceRestart,
    ResourceSpike,
    ErrorCleared,
    NewIssue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeSeverity {
    Info,
    Notable,
    Important,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimeOfDay {
    Morning,   // 5-12
    Afternoon, // 12-17
    Evening,   // 17-22
    Night,     // 22-5
}

/// Personality style for greetings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PersonalityStyle {
    Professional, // Formal, concise
    Friendly,     // Warm, conversational
    Technical,    // Detailed, precise
    Casual,       // Relaxed, brief
}

/// Generated greeting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Greeting {
    /// Main greeting message
    pub message: String,

    /// Optional context notes (changes, warnings)
    pub context_notes: Vec<String>,

    /// Personality style used
    pub style: PersonalityStyle,
}

/// Generate greeting based on context and personality
pub fn generate_greeting(context: &GreetingContext, style: &PersonalityStyle) -> Greeting {
    let time_greeting = time_of_day_greeting(&context.time_of_day, style);
    let state_note = system_state_note(&context.system_state, style);

    let mut context_notes = Vec::new();

    // Add important changes
    for change in &context.changes {
        if matches!(change.severity, ChangeSeverity::Important | ChangeSeverity::Notable) {
            context_notes.push(format_change(change, style));
        }
    }

    // Build main message
    let message = if let Some(note) = state_note {
        format!("{} {}", time_greeting, note)
    } else {
        time_greeting
    };

    Greeting {
        message,
        context_notes,
        style: style.clone(),
    }
}

fn time_of_day_greeting(time: &TimeOfDay, style: &PersonalityStyle) -> String {
    match (time, style) {
        (TimeOfDay::Morning, PersonalityStyle::Professional) => "Good morning.".to_string(),
        (TimeOfDay::Morning, PersonalityStyle::Friendly) => "Good morning! ☀️".to_string(),
        (TimeOfDay::Morning, PersonalityStyle::Technical) => "Morning. System operational.".to_string(),
        (TimeOfDay::Morning, PersonalityStyle::Casual) => "Hey! Morning.".to_string(),

        (TimeOfDay::Afternoon, PersonalityStyle::Professional) => "Good afternoon.".to_string(),
        (TimeOfDay::Afternoon, PersonalityStyle::Friendly) => "Good afternoon!".to_string(),
        (TimeOfDay::Afternoon, PersonalityStyle::Technical) => "Afternoon. Systems nominal.".to_string(),
        (TimeOfDay::Afternoon, PersonalityStyle::Casual) => "Hey there.".to_string(),

        (TimeOfDay::Evening, PersonalityStyle::Professional) => "Good evening.".to_string(),
        (TimeOfDay::Evening, PersonalityStyle::Friendly) => "Good evening! 🌙".to_string(),
        (TimeOfDay::Evening, PersonalityStyle::Technical) => "Evening. Status check available.".to_string(),
        (TimeOfDay::Evening, PersonalityStyle::Casual) => "Evening!".to_string(),

        (TimeOfDay::Night, PersonalityStyle::Professional) => "Hello.".to_string(),
        (TimeOfDay::Night, PersonalityStyle::Friendly) => "Hello! Working late? 🌃".to_string(),
        (TimeOfDay::Night, PersonalityStyle::Technical) => "System active. Night mode.".to_string(),
        (TimeOfDay::Night, PersonalityStyle::Casual) => "Hey, night owl!".to_string(),
    }
}

fn system_state_note(state: &SystemState, style: &PersonalityStyle) -> Option<String> {
    match (state, style) {
        (SystemState::Healthy, _) => None,

        (SystemState::Warning { reason }, PersonalityStyle::Professional) => {
            Some(format!("Note: {}", reason))
        }
        (SystemState::Warning { reason }, PersonalityStyle::Friendly) => {
            Some(format!("Heads up: {}", reason))
        }
        (SystemState::Warning { reason }, PersonalityStyle::Technical) => {
            Some(format!("⚠️ Warning: {}", reason))
        }
        (SystemState::Warning { reason }, PersonalityStyle::Casual) => {
            Some(format!("BTW: {}", reason))
        }

        (SystemState::Critical { reason }, PersonalityStyle::Professional) => {
            Some(format!("Critical issue: {}", reason))
        }
        (SystemState::Critical { reason }, PersonalityStyle::Friendly) => {
            Some(format!("Important: {} - let's check this!", reason))
        }
        (SystemState::Critical { reason }, PersonalityStyle::Technical) => {
            Some(format!("🚨 CRITICAL: {}", reason))
        }
        (SystemState::Critical { reason }, PersonalityStyle::Casual) => {
            Some(format!("Uh oh: {}", reason))
        }

        (SystemState::Recovering { from }, PersonalityStyle::Professional) => {
            Some(format!("System recovering from: {}", from))
        }
        (SystemState::Recovering { from }, PersonalityStyle::Friendly) => {
            Some(format!("Good news! Recovering from: {}", from))
        }
        (SystemState::Recovering { from }, PersonalityStyle::Technical) => {
            Some(format!("🔄 Recovery in progress: {}", from))
        }
        (SystemState::Recovering { from }, PersonalityStyle::Casual) => {
            Some(format!("Getting better: {}", from))
        }
    }
}

fn format_change(change: &SystemChange, style: &PersonalityStyle) -> String {
    match style {
        PersonalityStyle::Professional => format!("• {}", change.description),
        PersonalityStyle::Friendly => format!("→ {}", change.description),
        PersonalityStyle::Technical => {
            format!("[{}] {}", format_change_type(&change.change_type), change.description)
        }
        PersonalityStyle::Casual => format!("- {}", change.description),
    }
}

fn format_change_type(change_type: &ChangeType) -> &str {
    match change_type {
        ChangeType::PackageUpdate => "PKG",
        ChangeType::ServiceRestart => "SVC",
        ChangeType::ResourceSpike => "RES",
        ChangeType::ErrorCleared => "OK",
        ChangeType::NewIssue => "NEW",
    }
}

/// Determine time of day from hour (0-23)
pub fn time_of_day_from_hour(hour: u32) -> TimeOfDay {
    match hour {
        5..=11 => TimeOfDay::Morning,
        12..=16 => TimeOfDay::Afternoon,
        17..=21 => TimeOfDay::Evening,
        _ => TimeOfDay::Night,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_context_healthy() -> GreetingContext {
        GreetingContext {
            system_state: SystemState::Healthy,
            changes: vec![],
            time_since_last: Duration::hours(2),
            time_of_day: TimeOfDay::Morning,
        }
    }

    fn test_context_warning() -> GreetingContext {
        GreetingContext {
            system_state: SystemState::Warning {
                reason: "High memory usage".to_string(),
            },
            changes: vec![SystemChange {
                change_type: ChangeType::ResourceSpike,
                description: "Memory usage increased to 85%".to_string(),
                severity: ChangeSeverity::Notable,
            }],
            time_since_last: Duration::hours(1),
            time_of_day: TimeOfDay::Afternoon,
        }
    }

    #[test]
    fn test_time_of_day_from_hour() {
        assert_eq!(time_of_day_from_hour(6), TimeOfDay::Morning);
        assert_eq!(time_of_day_from_hour(13), TimeOfDay::Afternoon);
        assert_eq!(time_of_day_from_hour(19), TimeOfDay::Evening);
        assert_eq!(time_of_day_from_hour(23), TimeOfDay::Night);
        assert_eq!(time_of_day_from_hour(3), TimeOfDay::Night);
    }

    #[test]
    fn test_greeting_healthy_professional() {
        let context = test_context_healthy();
        let greeting = generate_greeting(&context, &PersonalityStyle::Professional);

        assert_eq!(greeting.style, PersonalityStyle::Professional);
        assert!(greeting.message.contains("Good morning"));
        assert!(greeting.context_notes.is_empty());
    }

    #[test]
    fn test_greeting_healthy_friendly() {
        let context = test_context_healthy();
        let greeting = generate_greeting(&context, &PersonalityStyle::Friendly);

        assert_eq!(greeting.style, PersonalityStyle::Friendly);
        assert!(greeting.message.contains("Good morning"));
        assert!(greeting.context_notes.is_empty());
    }

    #[test]
    fn test_greeting_warning_professional() {
        let context = test_context_warning();
        let greeting = generate_greeting(&context, &PersonalityStyle::Professional);

        assert!(greeting.message.contains("Good afternoon"));
        assert!(greeting.message.contains("Note"));
        assert!(greeting.message.contains("High memory usage"));
        assert_eq!(greeting.context_notes.len(), 1);
    }

    #[test]
    fn test_greeting_warning_friendly() {
        let context = test_context_warning();
        let greeting = generate_greeting(&context, &PersonalityStyle::Friendly);

        assert!(greeting.message.contains("Heads up"));
        assert_eq!(greeting.context_notes.len(), 1);
    }

    #[test]
    fn test_greeting_critical() {
        let context = GreetingContext {
            system_state: SystemState::Critical {
                reason: "Service failure".to_string(),
            },
            changes: vec![],
            time_since_last: Duration::hours(1),
            time_of_day: TimeOfDay::Evening,
        };

        let greeting = generate_greeting(&context, &PersonalityStyle::Technical);
        assert!(greeting.message.contains("CRITICAL"));
        assert!(greeting.message.contains("Service failure"));
    }

    #[test]
    fn test_greeting_recovering() {
        let context = GreetingContext {
            system_state: SystemState::Recovering {
                from: "network outage".to_string(),
            },
            changes: vec![],
            time_since_last: Duration::minutes(30),
            time_of_day: TimeOfDay::Night,
        };

        let greeting = generate_greeting(&context, &PersonalityStyle::Friendly);
        assert!(greeting.message.contains("Good news") || greeting.message.contains("Recovering"));
    }

    #[test]
    fn test_change_formatting_technical() {
        let change = SystemChange {
            change_type: ChangeType::PackageUpdate,
            description: "Updated 5 packages".to_string(),
            severity: ChangeSeverity::Info,
        };

        let formatted = format_change(&change, &PersonalityStyle::Technical);
        assert!(formatted.contains("[PKG]"));
        assert!(formatted.contains("Updated 5 packages"));
    }

    #[test]
    fn test_multiple_changes() {
        let context = GreetingContext {
            system_state: SystemState::Healthy,
            changes: vec![
                SystemChange {
                    change_type: ChangeType::PackageUpdate,
                    description: "Updated 3 packages".to_string(),
                    severity: ChangeSeverity::Notable,
                },
                SystemChange {
                    change_type: ChangeType::ServiceRestart,
                    description: "Nginx restarted".to_string(),
                    severity: ChangeSeverity::Important,
                },
                SystemChange {
                    change_type: ChangeType::ErrorCleared,
                    description: "Disk space warning cleared".to_string(),
                    severity: ChangeSeverity::Info,
                },
            ],
            time_since_last: Duration::hours(6),
            time_of_day: TimeOfDay::Morning,
        };

        let greeting = generate_greeting(&context, &PersonalityStyle::Professional);
        // Should include Notable and Important, but not Info
        assert_eq!(greeting.context_notes.len(), 2);
    }

    #[test]
    fn test_all_personality_styles() {
        let context = test_context_healthy();

        for style in &[
            PersonalityStyle::Professional,
            PersonalityStyle::Friendly,
            PersonalityStyle::Technical,
            PersonalityStyle::Casual,
        ] {
            let greeting = generate_greeting(&context, style);
            assert_eq!(greeting.style, *style);
            assert!(!greeting.message.is_empty());
        }
    }
}
