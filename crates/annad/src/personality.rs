//! Anna's Personality System - Making Anna feel alive and human.
//!
//! Anna is a living, breathing sysadmin with:
//! - Time-aware greetings (morning, afternoon, evening, night)
//! - Mood changes based on system health
//! - Dynamic responses with personality
//! - Experience-based learning
//!
//! Philosophy: Anna should feel like a trusted colleague, not a robot.

use chrono::{Local, Timelike};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Anna's current mood based on system state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mood {
    /// Everything is great, Anna is happy and energetic
    Cheerful,
    /// System is stable, Anna is calm and professional
    Calm,
    /// Minor issues detected, Anna is slightly concerned
    Concerned,
    /// Serious issues detected, Anna is focused and urgent
    Urgent,
    /// Critical system state, Anna is in crisis mode
    Critical,
}

impl Mood {
    /// Determine mood from system health metrics
    pub fn from_system_health(
        disk_usage: f32,
        memory_usage: f32,
        failed_services: usize,
        critical_errors: usize,
    ) -> Self {
        // Critical: Failed services or disk >95%
        if failed_services > 0 || disk_usage > 95.0 || critical_errors > 0 {
            return Mood::Critical;
        }

        // Urgent: Disk >85% or memory >90%
        if disk_usage > 85.0 || memory_usage > 90.0 {
            return Mood::Urgent;
        }

        // Concerned: Disk >75% or memory >80%
        if disk_usage > 75.0 || memory_usage > 80.0 {
            return Mood::Concerned;
        }

        // Calm: Everything in normal range but not perfect
        if disk_usage > 50.0 || memory_usage > 60.0 {
            return Mood::Calm;
        }

        // Cheerful: Everything is great!
        Mood::Cheerful
    }

    /// Get adjective describing this mood
    pub fn adjective(&self) -> &'static str {
        match self {
            Mood::Cheerful => "cheerful",
            Mood::Calm => "calm",
            Mood::Concerned => "concerned",
            Mood::Urgent => "focused",
            Mood::Critical => "urgent",
        }
    }

    /// Get emoji representing this mood (optional, can be disabled)
    pub fn emoji(&self) -> &'static str {
        match self {
            Mood::Cheerful => "😊",
            Mood::Calm => "😌",
            Mood::Concerned => "🤔",
            Mood::Urgent => "😰",
            Mood::Critical => "🚨",
        }
    }
}

/// Time of day for contextual greetings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeOfDay {
    Morning,   // 5:00 - 11:59
    Afternoon, // 12:00 - 17:59
    Evening,   // 18:00 - 22:59
    Night,     // 23:00 - 4:59
}

impl TimeOfDay {
    /// Get current time of day
    pub fn now() -> Self {
        let hour = Local::now().hour();
        match hour {
            5..=11 => TimeOfDay::Morning,
            12..=17 => TimeOfDay::Afternoon,
            18..=22 => TimeOfDay::Evening,
            _ => TimeOfDay::Night,
        }
    }

    /// Get appropriate greeting for this time
    pub fn greeting(&self) -> &'static str {
        match self {
            TimeOfDay::Morning => "Good morning",
            TimeOfDay::Afternoon => "Good afternoon",
            TimeOfDay::Evening => "Good evening",
            TimeOfDay::Night => "Hello",
        }
    }

    /// Get time-specific context phrase
    pub fn context_phrase(&self) -> &'static str {
        match self {
            TimeOfDay::Morning => "to start your day",
            TimeOfDay::Afternoon => "for this afternoon",
            TimeOfDay::Evening => "for tonight",
            TimeOfDay::Night => "at this late hour",
        }
    }
}

/// Anna's personality state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityState {
    /// Current mood
    pub mood: Mood,
    /// Last mood change timestamp
    pub mood_updated: String,
    /// How many times Anna has helped this session
    pub interactions_today: u32,
    /// Anna's experience level (increases over time)
    pub experience_level: u32,
    /// Things Anna has learned (pattern: lesson)
    pub learned_lessons: Vec<String>,
}

impl Default for PersonalityState {
    fn default() -> Self {
        Self {
            mood: Mood::Calm,
            mood_updated: chrono::Utc::now().to_rfc3339(),
            interactions_today: 0,
            experience_level: 1,
            learned_lessons: Vec::new(),
        }
    }
}

impl PersonalityState {
    /// Path to personality state file
    fn path() -> PathBuf {
        PathBuf::from("/var/lib/anna/personality.json")
    }

    /// Load personality state from disk
    pub fn load() -> Self {
        let path = Self::path();
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save personality state to disk
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, json)?;
        Ok(())
    }

    /// Update mood based on system state
    pub fn update_mood(&mut self, disk: f32, memory: f32, failed: usize, errors: usize) {
        let new_mood = Mood::from_system_health(disk, memory, failed, errors);
        if new_mood != self.mood {
            self.mood = new_mood;
            self.mood_updated = chrono::Utc::now().to_rfc3339();
        }
    }

    /// Record an interaction
    pub fn record_interaction(&mut self) {
        self.interactions_today += 1;
    }

    /// Learn a lesson (add to memory)
    pub fn learn_lesson(&mut self, lesson: String) {
        if !self.learned_lessons.contains(&lesson) {
            self.learned_lessons.push(lesson);
            self.experience_level += 1;
        }
    }

    /// Get a personalized greeting
    pub fn personalized_greeting(&self, username: Option<&str>) -> String {
        let time = TimeOfDay::now();
        let greeting = time.greeting();

        let name_part = if let Some(name) = username {
            format!(", {}", name)
        } else {
            String::new()
        };

        match self.mood {
            Mood::Cheerful => {
                format!("{}{}! Your system looks fantastic today.", greeting, name_part)
            }
            Mood::Calm => {
                format!("{}{}. Everything is running smoothly.", greeting, name_part)
            }
            Mood::Concerned => {
                format!("{}{}. I've noticed a few things that could use attention.", greeting, name_part)
            }
            Mood::Urgent => {
                format!("{}{}. We have some important issues to address.", greeting, name_part)
            }
            Mood::Critical => {
                format!("{}{}! Immediate attention needed on critical issues.", greeting, name_part)
            }
        }
    }

    /// Get closing message based on mood and time
    pub fn closing_message(&self) -> &'static str {
        let time = TimeOfDay::now();

        match (self.mood, time) {
            (Mood::Cheerful, TimeOfDay::Morning) => "Have a wonderful day ahead!",
            (Mood::Cheerful, TimeOfDay::Afternoon) => "Keep up the great work!",
            (Mood::Cheerful, TimeOfDay::Evening) => "Enjoy your evening!",
            (Mood::Cheerful, TimeOfDay::Night) => "Rest well!",

            (Mood::Calm, TimeOfDay::Morning) => "Have a productive day.",
            (Mood::Calm, TimeOfDay::Afternoon) => "Things are looking good.",
            (Mood::Calm, TimeOfDay::Evening) => "Everything's under control.",
            (Mood::Calm, TimeOfDay::Night) => "Sleep soundly.",

            (Mood::Concerned, _) => "I'll keep monitoring things.",
            (Mood::Urgent, _) => "Let me know if you need help.",
            (Mood::Critical, _) => "I'm here if you need assistance.",
        }
    }

    /// Get Anna's tone modifier for LLM prompts
    pub fn tone_instruction(&self) -> &'static str {
        match self.mood {
            Mood::Cheerful => "Be upbeat, friendly, and encouraging. Use positive language.",
            Mood::Calm => "Be professional, clear, and reassuring. Stay composed.",
            Mood::Concerned => "Be attentive and slightly cautious. Show you care.",
            Mood::Urgent => "Be direct and focused. Prioritize important information.",
            Mood::Critical => "Be urgent but not panicked. Focus on solutions immediately.",
        }
    }
}

/// Generate a dynamic, personality-driven response intro
pub fn generate_response_intro(mood: Mood, context: &str) -> String {
    let time = TimeOfDay::now();

    match (mood, time) {
        (Mood::Cheerful, TimeOfDay::Morning) => {
            format!("Great question {}! Let me help you with that.", time.context_phrase())
        }
        (Mood::Cheerful, _) => {
            "I'd be happy to help with that!".to_string()
        }
        (Mood::Calm, _) => {
            "Let me check that for you.".to_string()
        }
        (Mood::Concerned, _) => {
            format!("I understand your concern about {}. Let me investigate.", context)
        }
        (Mood::Urgent, _) => {
            format!("Looking into {} right away.", context)
        }
        (Mood::Critical, _) => {
            format!("Checking {} immediately.", context)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mood_from_health_cheerful() {
        let mood = Mood::from_system_health(30.0, 40.0, 0, 0);
        assert_eq!(mood, Mood::Cheerful);
    }

    #[test]
    fn test_mood_from_health_critical() {
        let mood = Mood::from_system_health(96.0, 50.0, 0, 0);
        assert_eq!(mood, Mood::Critical);
    }

    #[test]
    fn test_mood_from_health_failed_service() {
        let mood = Mood::from_system_health(50.0, 50.0, 1, 0);
        assert_eq!(mood, Mood::Critical);
    }

    #[test]
    fn test_time_of_day() {
        // Can't test actual time, but test the enum works
        let time = TimeOfDay::Morning;
        assert_eq!(time.greeting(), "Good morning");
    }

    #[test]
    fn test_personality_state_default() {
        let state = PersonalityState::default();
        assert_eq!(state.mood, Mood::Calm);
        assert_eq!(state.experience_level, 1);
    }

    #[test]
    fn test_learn_lesson() {
        let mut state = PersonalityState::default();
        let initial_level = state.experience_level;

        state.learn_lesson("Never run rm -rf / without checking".to_string());
        assert_eq!(state.experience_level, initial_level + 1);
        assert_eq!(state.learned_lessons.len(), 1);

        // Learning same lesson doesn't increase experience
        state.learn_lesson("Never run rm -rf / without checking".to_string());
        assert_eq!(state.experience_level, initial_level + 1);
    }
}
