//! Anna daemon library - exposes modules for testing.
//! v0.0.75: UX realism, stats integration, benchmark scheduler.
//! v0.0.99: Package install and service management.
//! v0.0.101: Recipe fast path - skip LLM for learned queries.
//! v0.0.102: Recipe direct answers - skip probes too.
//! v0.0.115: File-based inbox for async queries.
//! v0.0.146: Internal comms for fly-on-wall experience.

pub mod action_handlers;
pub mod comms;
pub mod editor_config;
pub mod answers;
pub mod benchmark;
pub mod benchmark_scheduler;
pub mod collectors;
pub mod config;
pub mod det_extended;
pub mod deterministic;
pub mod fast_path_handler;
pub mod handlers;
pub mod hardware;
pub mod health;
pub mod health_brief_builder;
pub mod inbox;
pub mod ollama;
pub mod parsers;
pub mod permissions;
pub mod probe_answers;
pub mod probe_runner;
pub mod probes;
pub mod progress_tracker;
pub mod prompts;
pub mod query_classify;
pub mod rag_answerer;
pub mod recipe_fast_path;
pub mod redact;
pub mod router;
#[cfg(test)]
pub mod router_tests;
pub mod rpc_handler;
pub mod scoring;
pub mod server;
pub mod service_desk;
pub mod specialist_handler;
pub mod state;
pub mod state_types;
pub mod summarizer;
pub mod theatre;
pub mod ticket_loop;
pub mod ticket_service;
pub mod translator;
pub mod triage;
pub mod triage_answer;
pub mod update;
pub mod verify_probes;
