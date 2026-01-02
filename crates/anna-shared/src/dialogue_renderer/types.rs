//! Dialogue types - Phase 89
//!
//! Types for representing dialogues and conversations.

use serde::{Deserialize, Serialize};

/// Speaker in a dialogue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Speaker {
    #[default]
    Anna,
    User,
    Junior,
    Senior,
    Lead,
    System,
}

impl Speaker {
    pub fn name(&self) -> &'static str {
        match self {
            Speaker::Anna => "Anna",
            Speaker::User => "User",
            Speaker::Junior => "Junior",
            Speaker::Senior => "Senior",
            Speaker::Lead => "Lead",
            Speaker::System => "System",
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            Speaker::Anna => "\x1b[36m",      // Cyan
            Speaker::User => "\x1b[32m",      // Green
            Speaker::Junior => "\x1b[33m",    // Yellow
            Speaker::Senior => "\x1b[35m",    // Magenta
            Speaker::Lead => "\x1b[34m",      // Blue
            Speaker::System => "\x1b[90m",    // Gray
        }
    }
}

/// Dialogue mood/tone
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DialogueMood {
    #[default]
    Neutral,
    Confident,
    Uncertain,
    Apologetic,
    Helpful,
    Thinking,
}

impl DialogueMood {
    pub fn prefix(&self) -> &'static str {
        match self {
            DialogueMood::Neutral => "",
            DialogueMood::Confident => "I know this! ",
            DialogueMood::Uncertain => "I'm not entirely sure, but ",
            DialogueMood::Apologetic => "I apologize, ",
            DialogueMood::Helpful => "Let me help you. ",
            DialogueMood::Thinking => "Let me think... ",
        }
    }
}

/// A single dialogue turn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueTurn {
    /// Who is speaking
    pub speaker: Speaker,
    /// Speaker's name (human name)
    pub speaker_name: Option<String>,
    /// Department (if specialist)
    pub department: Option<String>,
    /// The message content
    pub content: String,
    /// Mood/tone
    pub mood: DialogueMood,
    /// Timestamp
    pub timestamp: u64,
    /// Is internal communication
    pub internal: bool,
}

/// A complete dialogue
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Dialogue {
    /// Ticket ID this dialogue belongs to
    pub ticket_id: Option<String>,
    /// All turns in order
    pub turns: Vec<DialogueTurn>,
    /// Subject/topic
    pub subject: Option<String>,
}

impl Dialogue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a turn
    pub fn add_turn(&mut self, turn: DialogueTurn) {
        self.turns.push(turn);
    }

    /// Add Anna speaking
    pub fn anna_says(&mut self, content: &str, mood: DialogueMood, timestamp: u64) {
        self.turns.push(DialogueTurn {
            speaker: Speaker::Anna,
            speaker_name: Some("Anna".to_string()),
            department: None,
            content: content.to_string(),
            mood,
            timestamp,
            internal: false,
        });
    }

    /// Add user speaking
    pub fn user_says(&mut self, content: &str, timestamp: u64) {
        self.turns.push(DialogueTurn {
            speaker: Speaker::User,
            speaker_name: None,
            department: None,
            content: content.to_string(),
            mood: DialogueMood::Neutral,
            timestamp,
            internal: false,
        });
    }

    /// Add specialist speaking (internal)
    pub fn specialist_says(
        &mut self,
        speaker: Speaker,
        name: &str,
        department: &str,
        content: &str,
        timestamp: u64,
    ) {
        self.turns.push(DialogueTurn {
            speaker,
            speaker_name: Some(name.to_string()),
            department: Some(department.to_string()),
            content: content.to_string(),
            mood: DialogueMood::Neutral,
            timestamp,
            internal: true,
        });
    }

    /// Get turn count
    pub fn turn_count(&self) -> usize {
        self.turns.len()
    }

    /// Get internal turns only
    pub fn internal_turns(&self) -> Vec<&DialogueTurn> {
        self.turns.iter().filter(|t| t.internal).collect()
    }

    /// Get external (user-facing) turns only
    pub fn external_turns(&self) -> Vec<&DialogueTurn> {
        self.turns.iter().filter(|t| !t.internal).collect()
    }
}
