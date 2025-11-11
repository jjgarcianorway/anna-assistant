//! State machine for Anna 1.0
//!
//! Six explicit states determine available commands:
//! - iso_live: Running from Arch ISO
//! - recovery_candidate: Broken system detected
//! - post_install_minimal: Fresh Arch, no Anna state
//! - configured: Managed system, health OK
//! - degraded: Managed system, health failing
//! - unknown: Unable to classify
//!
//! Citation: [archwiki:installation_guide], [archwiki:system_maintenance]

pub mod capabilities;
pub mod detector;
pub mod types;

pub use capabilities::{get_capabilities, CommandCapability};
pub use detector::detect_state;
pub use types::{NetworkStatus, StateDetails, StateDetection, SystemState};
