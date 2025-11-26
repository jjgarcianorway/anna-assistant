//! Anna Brain v10.0.1 - Pure LLM-Driven Evidence-Based Architecture
//!
//! INPUT → LLM → TOOLS → LLM → ANSWER
//!
//! Core principles:
//! - Every answer must cite evidence from tool outputs
//! - Reliability scores have explicit labels (HIGH/MEDIUM/LOW/VERY LOW)
//! - Strict JSON protocol with step_type: "decide_tool" | "final_answer" | "ask_user"
//! - Fallback pattern-matching when LLM fails to follow protocol

pub mod contracts;
pub mod tools;
pub mod prompt;
pub mod orchestrator;
pub mod fallback;

pub use contracts::{
    EvidenceItem, BrainStep, StepType, ToolRequest, ReliabilityLabel,
    BrainSession, SessionResult,
};
pub use tools::{ToolCatalog, execute_tool, ToolSchema};
pub use orchestrator::{BrainOrchestrator, BrainResult};
pub use fallback::try_fallback_answer;
