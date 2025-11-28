//! Answer Orchestration v0.19.0
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
//! v0.18.0: Step-by-step orchestration (one action per iteration)
//! v0.19.0: Subproblem decomposition, fact-aware planning, Senior as mentor

// Legacy engines
pub mod engine;
pub mod fallback;
pub mod llm_client;
pub mod probe_executor;
pub mod research_engine;

// v0.18.0 step-by-step engine (legacy)
pub mod engine_v18;
pub mod llm_client_v18;

// v0.19.0 subproblem decomposition engine (current)
pub mod engine_v19;
pub mod llm_client_v19;

// v0.43.0 streaming debug
pub mod streaming;

// Legacy exports
pub use engine::AnswerEngine;
pub use research_engine::{
    ProcessedCheck, ResearchConfig, ResearchEngine, ResearchState, MAX_LLM_A_ITERATIONS,
    MAX_LLM_B_PASSES,
};

// v0.18.0 exports (legacy)
pub use engine_v18::AnswerEngineV18;

// v0.19.0 exports (current)
pub use engine_v19::AnswerEngineV19;

// v0.43.0 streaming exports
pub use streaming::{
    create_channel_emitter, create_noop_emitter, response::debug_stream_response,
    ChannelEmitter, NoopEmitter, SharedEmitter,
};
