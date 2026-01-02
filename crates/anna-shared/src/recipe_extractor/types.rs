//! Data types for recipe extraction.

use crate::recipe_eligibility::TicketForEligibility;
use std::collections::HashMap;

/// Data needed to extract a recipe from a ticket.
#[derive(Debug, Clone)]
pub struct TicketData {
    /// Ticket ID for generating recipe ID
    pub ticket_id: String,
    /// Eligibility data
    pub eligibility: TicketForEligibility,
    /// Probe results used in resolution
    pub probes_used: HashMap<String, String>,
    /// Commands that were executed
    pub commands: Vec<CommandRecord>,
    /// File edits that were made
    pub file_edits: Vec<FileEdit>,
    /// Citations from knowledge engine
    pub citations: Vec<String>,
    /// Translator-extracted slots
    pub slots: HashMap<String, String>,
}

/// Record of a command that was executed.
#[derive(Debug, Clone)]
pub struct CommandRecord {
    pub command: String,
    pub description: Option<String>,
    pub success: bool,
    pub is_verification: bool,
}

/// Record of a file edit.
#[derive(Debug, Clone)]
pub struct FileEdit {
    pub path: String,
    pub edit_type: FileEditType,
    pub content: Option<String>,
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileEditType {
    AppendLine,
    PrependLine,
    ReplaceLine,
    EnsureLine,
    RemoveLines,
    WriteFile,
}

/// Result of recipe extraction.
#[derive(Debug)]
pub enum ExtractionResult {
    /// Successfully extracted a new recipe
    NewRecipe(crate::recipe_schema::Recipe),
    /// Should update existing recipe
    UpdateExisting { recipe_id: String, new_version: u32 },
    /// Not eligible for extraction
    NotEligible(String),
    /// Failed to extract
    ExtractionFailed(String),
}
