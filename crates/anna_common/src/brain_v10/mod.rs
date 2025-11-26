//! Anna Brain v10.2.0 - Evidence-Based Learning Architecture
//!
//! INPUT → LLM → TOOLS → LLM → ANSWER → LEARN
//!
//! Core principles:
//! - NO hardcoded host facts - learn from THIS machine
//! - Every answer must cite evidence from tool outputs
//! - Reliability scores have explicit labels (HIGH/MEDIUM/LOW/VERY LOW)
//! - Strict JSON protocol with step_type: "decide_tool" | "final_answer" | "ask_user"
//! - Fallback pattern-matching when LLM fails to follow protocol
//! - Learn facts with freshness tiers: STATIC / SLOW / VOLATILE

pub mod contracts;
pub mod tools;
pub mod prompt;
pub mod system_prompt;
pub mod orchestrator;
pub mod fallback;

pub use contracts::{
    EvidenceItem, BrainStep, StepType, ToolRequest, ReliabilityLabel,
    BrainSession, SessionResult,
};
pub use tools::{ToolCatalog, execute_tool, ToolSchema};
pub use orchestrator::{BrainOrchestrator, BrainResult};
pub use fallback::try_fallback_answer;
