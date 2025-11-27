//! Answer Orchestration v0.12.2
//!
//! The orchestrator manages the LLM-A -> Probe -> LLM-B loop:
//! 1. Parse question with LLM-A
//! 2. Execute requested probes
//! 3. LLM-A generates draft answer with evidence
//! 4. LLM-B audits the draft
//! 5. Loop or accept/refuse
//!
//! v0.12.2: Added fallback answer extraction when LLM fails

pub mod engine;
pub mod fallback;
pub mod llm_client;
pub mod probe_executor;

pub use engine::AnswerEngine;
