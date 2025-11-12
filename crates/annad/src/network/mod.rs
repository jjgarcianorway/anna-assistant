//! Network module for distributed consensus (Phase 1.9 + 1.11 + 1.14 + 1.15 + 1.16 + Phase 2)
//!
//! Provides RPC endpoints for peer-to-peer consensus communication with TLS/mTLS,
//! body size limits, rate limiting, hot reload, and certificate pinning.
//!
//! Phase 2 additions:
//! - pinning_config: Configuration loading for certificate pinning
//! - pinning_verifier: rustls ServerCertVerifier for enforcing pinning in TLS handshake

pub mod idempotency;
pub mod metrics;
pub mod middleware;
pub mod peers;
pub mod pinning;
pub mod pinning_config;
pub mod pinning_verifier;
pub mod reload;
pub mod rpc;

pub use idempotency::IdempotencyStore;
pub use metrics::ConsensusMetrics;
pub use middleware::{
    RateLimiter, MAX_BODY_SIZE,
    RATE_LIMIT_BURST_REQUESTS, RATE_LIMIT_BURST_WINDOW,
    RATE_LIMIT_SUSTAINED_REQUESTS, RATE_LIMIT_SUSTAINED_WINDOW,
};
pub use peers::{PeerClient, PeerConfig, PeerList, TlsConfig};
pub use pinning::PinningConfig;
pub use reload::{ReloadableConfig, sighup_handler};
pub use rpc::ConsensusRpcServer;
