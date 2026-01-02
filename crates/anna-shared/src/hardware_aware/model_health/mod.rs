//! Model health and verification (v0.0.434).
//!
//! Tracks model installation status, verifies models work, and manages lifecycle.

mod health_record;
mod health_tracker;
mod model_status;
mod verifier;

// Re-export public items
pub use health_record::ModelHealthRecord;
pub use health_tracker::ModelHealth;
pub use model_status::{InstalledBy, ModelStatus};
pub use verifier::{ModelVerifier, VerifyError, VerifyReport};
