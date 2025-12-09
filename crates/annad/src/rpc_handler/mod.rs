//! RPC request handlers with deterministic routing, triage, and fallback (v0.0.200).
//!
//! v0.0.166: Integrated stage modules for modularization.
//! v0.0.167: Integrated routing_stage module for further modularization.
//! v0.0.200: Modularized into domain-focused submodules.

mod dispatcher;
mod helpers;
mod llm_request;

// Re-export main handler
pub use dispatcher::handle_request;
