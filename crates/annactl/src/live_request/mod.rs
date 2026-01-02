//! Live request handling with real-time progress display (v0.0.312).
//!
//! Polls for progress events during request processing to show
//! internal IT department chatter (fly-on-wall experience).
//!
//! v0.0.237: Enhanced display format with conversational headers.
//! v0.0.238: Added streaming token support for word-by-word output.
//! v0.0.253: Enhanced specialist dialogue with role titles and visual polish.
//! v0.0.278: Enhanced Hollywood-style stage indicators and spinners.
//! v0.0.284: Added idle tips during wait times.
//! v0.0.285: Integrated telemetry-based health tips.
//! v0.0.312: Added timestamps to internal comms for dialogue rhythm visibility.

mod display;
mod helpers;
mod request;
mod state;

#[cfg(test)]
mod tests;

// Re-export the main public API
pub use request::send_request_with_progress;
