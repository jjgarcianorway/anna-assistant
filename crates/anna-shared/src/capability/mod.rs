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

pub use registry::{
    Capability, CapabilityId, CapabilityMode, CapabilityRegistry, CAPABILITY_REGISTRY,
};
pub use router::{route_request, CapabilityRoutingResult, UnsupportedReason};
pub use response::{format_response, ResponseOutcome, ResponseArtifact};
pub use noise::{filter_warnings, WarningRelevance};
pub use display_scale::execute_display_scale_gdm;
