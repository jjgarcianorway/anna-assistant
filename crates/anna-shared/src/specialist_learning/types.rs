//! Core types for the Specialist Learning System

use crate::probe_learning::QueryCategory;
use crate::revision::RevisionIssue;
use crate::rpc::SpecialistDomain;
use serde::{Deserialize, Serialize};

/// A lesson learned from specialist interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistLesson {
    pub id: String,                              // Unique lesson ID
    pub query_pattern: String,                   // Normalized query pattern
    pub domain: SpecialistDomain,                // Domain this applies to
    pub category: QueryCategory,                 // For probe learning
    pub issues_fixed: Vec<RevisionIssue>,        // What was wrong
    pub solution_type: SolutionType,             // How solution was obtained
    pub effective_probes: Vec<String>,           // Probes that worked
    pub answer_template: String,                 // Successful answer
    pub confidence: u8,                          // Confidence 0-100
    pub success_count: u32,                      // Times this succeeded
    pub learned_at: u64,                         // First learning timestamp
    pub last_success_at: u64,                    // Last success timestamp
    pub generic_pattern: Option<GenericPattern>, // Generic pattern if any
}

/// How the solution was obtained
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SolutionType {
    /// Senior staff provided guidance
    SeniorGuidance {
        /// The revision instruction that worked
        instruction_summary: String,
    },
    /// LLM self-healing corrected the answer
    LlmSelfHealing {
        /// What constraint/correction was applied
        correction_type: String,
    },
    /// User confirmed the answer was helpful
    UserFeedback {
        /// Whether user said helpful
        helpful: bool,
    },
}

/// A generic pattern extracted from a lesson
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericPattern {
    pub category: PatternCategory,
    pub variables: Vec<PatternVariable>,
    pub probe_templates: Vec<String>,
    pub answer_template: String,
}

/// Categories of generic patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatternCategory {
    ConfigCheck,   // "check X config"
    ConfigEdit,    // "enable Y in X"
    ServiceAction, // "start/stop/restart X"
    PackageQuery,  // "is X installed"
    DiskAnalysis,  // "what's using space"
    ProcessQuery,  // "what's using CPU/memory"
    Other,
}

/// A variable in a generic pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternVariable {
    pub name: String,
    pub detection_hint: String,
    pub example_values: Vec<String>,
}

/// A pattern waiting for more successes before becoming a lesson
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingPattern {
    pub query_pattern: String,
    pub domain: SpecialistDomain,
    pub success_count: u32,
    pub last_answer: String,
    pub last_probes: Vec<String>,
    pub confidence: u8,
}
