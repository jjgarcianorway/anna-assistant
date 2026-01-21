//! Capability Layer - Operator-facing capability routing and response.
//!
//! v0.3.74: Deterministic capability routing and total response formatting.
//!
//! This layer sits above the existing architecture and provides:
//! - Static, canonical capability registry
//! - Deterministic request routing (no fallbacks, no guessing)
//! - Total response formatting (always produces valid output)
//! - Noise containment (warnings only where relevant)
//!
//! # Architecture
//!
//! ```text
//! [User Request]
//!       |
//!       v
//! [route_request] --> CapabilityRoutingResult
//!       |
//!       +-- Unsupported { reason_code, short_message }
//!       |         |
//!       |         v
//!       |   [format_response] --> Abstained
//!       |
//!       +-- Supported { capability_id }
//!             |
//!             v
//!       [execute_capability]
//!             |
//!             +-- ReadOnly --> gather facts, propose plan
//!             |         |
//!             |         v
//!             |   [format_response] --> Resolved | Abstained
//!             |
//!             +-- Mutating --> blocked at ExecutionGate
//!                       |
//!                       v
//!                 [format_response] --> Abstained
//! ```
//!
//! # Constraints
//!
//! - No dynamic registration
//! - No inference
//! - No fallback behavior
//! - No partial matches
//! - No "best guess"
//! - Unknown capabilities are explicitly rejected
//!
//! # Response Guarantee
//!
//! Every request produces exactly one of:
//! - Resolved (with explanation)
//! - Abstained (with explicit reason and capability reference)
//! - Failed (with explicit structural error)
//!
//! It is impossible to emit "could not format a valid response".

mod registry;
mod router;
mod response;
mod noise;
mod display_scale;
mod config_review_group;
mod config_review_passwd;
mod power_inhibit;
mod thermal_status;
mod audio_stack;

pub use registry::{
    Capability, CapabilityId, CapabilityMode, CapabilityRegistry, WarningCategory,
    CAPABILITY_REGISTRY,
};
pub use router::{route_request, CapabilityRoutingResult, UnsupportedReason};
pub use response::{
    build_policy_violation_response, format_outcome_to_string, format_response, AbstainReason,
    CapabilityExecutionResult, FailedReason, ResponseArtifact, ResponseOutcome,
};
pub use noise::{filter_warnings, SystemWarning, WarningRelevance};
pub use display_scale::{execute_display_scale_gdm, gather_probes, GdmScalingProbes};
pub use config_review_group::{
    execute_config_review_group_change, gather_probes as gather_group_change_probes,
    GroupChangeProbes,
};
pub use config_review_passwd::{
    execute_passwd_change_review, gather_probes as gather_passwd_change_probes,
    PasswdChangeProbes,
};
pub use power_inhibit::{
    execute_power_inhibit_sleep, gather_probes as gather_power_probes,
    InhibitAction, InhibitTarget, PowerInhibitProbes,
};
pub use thermal_status::{
    execute_thermal_status, gather_probes as gather_thermal_probes,
    ThermalProbes,
};
pub use audio_stack::{
    execute_audio_stack_detect, gather_probes as gather_audio_probes,
    AudioProbes, AudioStack,
};
