//! Safety Module v0.15.0
//!
//! Protections against abuse and resource exhaustion:
//! - Rate limiting for requests
//! - Concurrent session limits
//! - Circuit breaker for failing backends
//! - Resource budgets per request

pub mod circuit_breaker;
pub mod limits;
pub mod rate_limit;

pub use circuit_breaker::*;
pub use limits::*;
pub use rate_limit::*;
