//! Answer Orchestration v0.15.0
//!
//! The orchestrator manages the LLM-A -> Probe -> LLM-B loop:
//! 1. Parse question with LLM-A
//! 2. Execute requested probes
//! 3. LLM-A generates draft answer with evidence
//! 4. LLM-B audits the draft
//! 5. Loop or accept/refuse
//!
//! v0.12.2: Added fallback answer extraction when LLM fails
//! v0.15.0: Research loop engine with command whitelist

pub mod engine;
pub mod fallback;
pub mod llm_client;
pub mod probe_executor;
pub mod research_engine;

pub use engine::AnswerEngine;
pub use research_engine::{
    ProcessedCheck, ResearchConfig, ResearchEngine, ResearchState, MAX_LLM_A_ITERATIONS,
    MAX_LLM_B_PASSES,
};
