//! Specialist types and data structures

use serde::{Deserialize, Serialize};

/// Specialist level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SpecialistLevel {
    #[default]
    Junior,
    Senior,
    Lead,
}

impl SpecialistLevel {
    pub fn name(&self) -> &'static str {
        match self {
            SpecialistLevel::Junior => "Junior",
            SpecialistLevel::Senior => "Senior",
            SpecialistLevel::Lead => "Lead",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            SpecialistLevel::Junior => "J",
            SpecialistLevel::Senior => "S",
            SpecialistLevel::Lead => "L",
        }
    }
}

/// Department type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Department {
    #[default]
    Desktop,
    Network,
    Security,
    Database,
    DevOps,
    Sound,
    Video,
    Storage,
    Performance,
    General,
}

impl Department {
    pub fn name(&self) -> &'static str {
        match self {
            Department::Desktop => "Desktop",
            Department::Network => "Network",
            Department::Security => "Security",
            Department::Database => "Database",
            Department::DevOps => "DevOps",
            Department::Sound => "Sound",
            Department::Video => "Video",
            Department::Storage => "Storage",
            Department::Performance => "Performance",
            Department::General => "General",
        }
    }
}

/// A specialist profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistProfile {
    /// Unique ID
    pub id: String,
    /// Human name
    pub name: String,
    /// Department
    pub department: Department,
    /// Level
    pub level: SpecialistLevel,
    /// Tickets resolved
    pub tickets_resolved: u64,
    /// Currently available
    pub available: bool,
    /// Model used (for LLM specialists)
    pub model: Option<String>,
    /// Specialties/skills
    pub skills: Vec<String>,
    /// Join date (timestamp)
    pub joined_at: u64,
}
