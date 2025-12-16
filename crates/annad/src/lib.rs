//! Anna daemon library - exposes modules for testing.
//! v0.0.75: UX realism, stats integration, benchmark scheduler.
//! v0.0.99: Package install and service management.
//! v0.0.101: Recipe fast path - skip LLM for learned queries.
//! v0.0.102: Recipe direct answers - skip probes too.
//! v0.0.115: File-based inbox for async queries.
//! v0.0.146: Internal comms for fly-on-wall experience.
//! v0.0.149: ConfigureEditor handler extracted to separate module.
//! v0.0.150: Timeout handler extracted to separate module.
//! v0.0.155: Response builders extracted to separate module.
//! v0.0.156: Clarification builders extracted to separate module.
//! v0.0.157: Best-effort summary extracted to separate module.
//! v0.0.158: Ollama streaming extracted to separate module.
//! v0.0.159: Update check loop extracted to separate module.
//! v0.0.160: System verifiers extracted to separate module.
//! v0.0.161: Update operations extracted to separate module.
//! v0.0.162: Model registry extracted to separate module.
//! v0.0.163: Built-in recipe matchers extracted to separate module.
//! v0.0.164: Probe registry and translator fallback extracted to separate modules.
//! v0.0.165: RPC handler stages extracted to separate modules.
//! v0.0.167: Routing stage extracted to separate module.
//! v0.0.404: JSON-only specialists + personality renderer.

pub mod action_handlers;
pub mod core_loop; // v0.0.811: Simple core request loop
pub mod answer_validator;
pub mod answers;
pub mod auto_select;
pub mod benchmark;
pub mod benchmark_scheduler;
pub mod best_effort_summary;
pub mod clarification_builders;
pub mod collectors;
pub mod comms;
pub mod config;
pub mod config_registry;
pub mod configure_editor;
pub mod configure_shell;
pub mod cross_reference; // v0.0.448: Cross-reference claims with external sources
pub mod desktop_wallpaper;
pub mod det;
pub mod det_extended;
pub mod deterministic;
pub mod editor_config;
pub mod evidence_integration; // v0.0.410: Evidence pipeline integration
pub mod fast_path_handler;
pub mod feedback_handler; // v0.0.401: User feedback handler
pub mod file_recipe_path; // v0.0.406: TOML-based authored recipes
pub mod greeting_generator;
pub mod handlers;
pub mod hardware;
pub mod health;
pub mod health_brief_builder;
pub mod inbox;
pub mod internal_comms; // v0.0.413: Event-driven IT department chatter
pub mod learning_capture; // v0.0.401: Specialist learning capture
pub mod learning_loop;
pub mod ollama;
pub mod ollama_streaming;
pub mod parsers;
pub mod permissions;
pub mod probe_answers;
pub mod probe_direct; // v0.0.403: Direct probe answers (bypass dumb LLM)
pub mod probe_domain; // v0.0.405: Domain→probes mapping
pub mod probe_registry;
pub mod probe_runner;
pub mod probe_stage;
pub mod probes;
pub mod progress_tracker;
pub mod prompts;
pub mod query_classify;
pub mod rag_answerer;
pub mod recipe_builtins;
pub mod recipe_engine_v2; // v0.0.412: Self-learning recipe system
pub mod recipe_fast_path;
pub mod recipe_similarity;
pub mod redact;
pub mod response_builders;
pub mod response_formatter;
pub mod response_renderer; // v0.0.404: Personality rendering layer
pub mod result_stage;
pub mod router;
#[cfg(test)]
pub mod router_tests;
pub mod routing_stage;
pub mod rpc_handler;
pub mod scoring;
pub mod server;
pub mod service_desk;
pub mod snapshot_loop;
pub mod specialist_handler;
pub mod specialist_json; // v0.0.404: JSON-only specialist handler
pub mod specialist_prompt; // v0.0.404: JSON-only specialist prompts
pub mod specialist_stage;
pub mod state;
pub mod state_types;
pub mod summarizer;
pub mod system_update;
pub mod system_verifiers;
pub mod telemetry_collector; // v0.0.280: System telemetry collection
pub mod theatre;
pub mod ticket_loop;
pub mod ticket_persistence; // v0.0.411: Ticket persistence for stats
pub mod ticket_service;
pub mod timeout_handler;
pub mod translator;
pub mod translator_fallback;
pub mod triage;
pub mod triage_answer;
pub mod update;
pub mod update_loop;
pub mod update_ops;
pub mod verify_probes;
