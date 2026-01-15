//! Streaming request handling for real-time responses.
//! v0.0.993: Added automatic fix detection and offer
//! v0.0.998: Added configuration recipes
//! v0.0.998: Added Hollywood IT teams experience
//! v0.3.49: Phase 16 - Action plan execution

mod confirm_handlers;
mod handlers;
mod helpers;
mod main_handler;

pub use handlers::handle_streaming_request;
