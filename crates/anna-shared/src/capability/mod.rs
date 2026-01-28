//! Capability Layer - Response formatting and policy violation handling.
//!
//! Retained for ExposureGate fallback support.

mod registry;
mod router;
mod response;
mod noise;

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
