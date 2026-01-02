//! Streaming-Safe Behavior (Part C) - v0.0.436.
//!
//! Buffer model output in memory instead of streaming raw tokens.
//! Show spinner + progress while model thinks.
//! Only render after decoding completes.

pub mod buffer;
pub mod display;
pub mod progress;
pub mod state;

#[cfg(test)]
mod tests;

pub use buffer::StreamBuffer;
pub use display::StreamDisplay;
pub use progress::{ProgressFrame, ProgressType};
pub use state::StreamState;
