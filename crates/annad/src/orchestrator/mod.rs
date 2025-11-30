//! Answer Orchestration v3.0.0
//!
//! Unified Brain → Router → Recipe → Junior → Senior pipeline.
//!
//! Flow:
//! 1. Brain fast path (<150ms, no LLM)
//! 2. Recipe match (if learned pattern exists)
//! 3. Junior planning + probe execution
//! 4. Senior audit (if confidence < 80%)
//!
//! Invariants:
//! - Max 3 LLM calls per question (2 Junior + 1 Senior)
//! - Hard timeout: 10 seconds
//! - All answers have origin, reliability, timing metadata

// Core orchestration
pub mod engine_v90;
pub mod fallback;
pub mod llm_client;
pub mod probe_executor;

// v1.0.0 LLM trait abstraction for testing
pub mod llm_trait;

// v1.0.0 Probe trait abstraction for testing
pub mod probe_trait;

// v0.43.0 streaming debug
pub mod streaming;

// v0.90.0 exports (current) - Unified Architecture
pub use engine_v90::UnifiedEngine;

// LLM client exports - used by UnifiedEngine
pub use llm_client::{DraftAnswerV80, JuniorResponseV80, OllamaClient, SeniorResponseV80};

// v1.0.0 LLM trait exports - for testing
pub use llm_trait::{
    FakeJuniorResponse, FakeLlmClient, FakeLlmClientBuilder, FakeSeniorResponse, LlmClient,
};

// v1.0.0 Probe trait exports - for testing
pub use probe_trait::{
    FakeProbeExecutor, FakeProbeExecutorBuilder, FakeProbeResponse, ProbeExecutor,
    RealProbeExecutor,
};

// v0.43.0 streaming exports
pub use streaming::{
    create_channel_emitter, create_noop_emitter, response::debug_stream_response,
    ChannelEmitter, NoopEmitter, SharedEmitter,
};
