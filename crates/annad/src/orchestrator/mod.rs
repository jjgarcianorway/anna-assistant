//! Answer Orchestration v0.10.0
//!
//! The orchestrator manages the LLM-A -> Probe -> LLM-B loop:
//! 1. Parse question with LLM-A
//! 2. Execute requested probes
//! 3. LLM-A generates draft answer with evidence
//! 4. LLM-B audits the draft
//! 5. Loop or accept/refuse

pub mod engine;
pub mod llm_client;
pub mod probe_executor;

pub use engine::AnswerEngine;
pub use llm_client::OllamaClient;
