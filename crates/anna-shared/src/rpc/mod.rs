//! JSON-RPC 2.0 types for annad communication (v0.0.220).
//!
//! v0.0.73: Added GetDaemonInfo for version truth.
//! v0.0.220: Modularized into domain-focused submodules.

mod context;
mod method;
mod params;
mod request_response;
mod result;
mod routing;
#[cfg(test)]
mod tests;

// Re-export for backwards compatibility
pub use context::{Capabilities, HardwareSummary, RuntimeContext};
pub use method::{DaemonInfo, RpcMethod};
pub use params::{
    ChangeParams, CommandExecutionResult, ExecuteCommandParams, FeedbackParams, FeedbackResult,
    PlanChangeParams, ProbeParams, ProbeType, RequestParams,
};
pub use request_response::{RpcError, RpcRequest, RpcResponse};
pub use result::{EvidenceBlock, ProbeResult, ReliabilitySignals, ServiceDeskResult};
pub use routing::{QueryIntent, SpecialistDomain, TranslatorTicket};
