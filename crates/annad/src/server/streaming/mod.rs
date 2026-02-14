//! Streaming request handling for real-time responses.
//! v0.0.993: Added automatic fix detection and offer
//! v0.0.998: Added configuration recipes
//! v0.0.998: Added Hollywood IT teams experience
//! v0.3.49: Phase 16 - Action plan execution
//! v0.3.76: Phase 34 - Unified capability response formatter

mod confirm_handlers;
mod handlers;
mod helpers;
mod instant_answers;
mod main_handler;
mod recovery_handler;

pub use handlers::handle_streaming_request;
