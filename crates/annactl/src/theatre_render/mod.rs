//! Theatre-style rendering for Service Desk experience (v0.0.202).
//!
//! Transforms ServiceDeskResult into cinematic narrative dialogue.
//! Shows the IT department working like a fly on the wall.
//!
//! v0.0.81: Initial implementation.
//! v0.0.202: Modularized into domain-focused submodules.

mod footer;
mod helpers;
mod narrative;
mod render;
mod tests;

// Re-export main function
pub use render::render_theatre;
