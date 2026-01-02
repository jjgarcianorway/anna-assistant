// v0.0.709: Settings Digest Module (Phase 285)
// Condensed digest of settings summaries

mod types;
mod config;
mod section;
mod stats;
mod digest;
mod registry;
mod utils;

// Re-export all public types and functions to preserve the original API
pub use types::{DigestType, DigestFormat};
pub use config::DigestConfig;
pub use section::{DigestSection, DigestItem};
pub use stats::DigestStats;
pub use digest::SettingsDigest;
pub use registry::DigestRegistry;
pub use utils::{format_digest_registry, is_digest_query, digest_fun_fact};
