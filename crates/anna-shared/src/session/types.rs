//! Session data types and structures.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// A single turn in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turn {
    /// User's question
    pub question: String,

    /// Anna's answer
    pub answer: String,

    /// Commands that were run
    pub commands: Vec<String>,

    /// When this turn occurred
    pub timestamp: String,

    /// Entities extracted from this turn
    pub entities_mentioned: Vec<String>,
}

/// Current session context
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionContext {
    /// Main topic being discussed
    pub current_topic: Option<String>,

    /// Sub-topics explored
    pub explored_topics: Vec<String>,

    /// What the user seems to be trying to accomplish
    pub apparent_goal: Option<String>,

    /// Level of detail the user prefers
    pub detail_preference: DetailLevel,
}

/// How much detail the user wants
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum DetailLevel {
    /// Just the facts
    Minimal,
    /// Normal explanations
    #[default]
    Normal,
    /// Detailed with context
    Verbose,
}

/// Entities mentioned in the session
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionEntities {
    /// Packages mentioned
    pub packages: Vec<String>,

    /// Services mentioned
    pub services: Vec<String>,

    /// Files/paths mentioned
    pub files: Vec<String>,

    /// Users mentioned
    pub users: Vec<String>,

    /// Commands that were run
    pub commands_run: Vec<String>,

    /// Errors encountered
    pub errors: Vec<String>,
}

/// A session maintains conversational context
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Session {
    /// Session ID
    pub id: String,

    /// Conversation history
    pub history: VecDeque<Turn>,

    /// Current topic/context
    pub context: SessionContext,

    /// Entities mentioned in conversation
    pub entities: SessionEntities,

    /// When this session started
    pub started_at: String,

    /// Last activity timestamp
    pub last_activity: String,
}
