//! Evidence Engine - Anna's real knowledge layer (v0.0.410).
//!
//! The evidence engine is Anna's "brain" for gathering facts:
//! 1. Runs targeted probes based on domain/intent/tags
//! 2. Fetches relevant docs (man pages, Arch wiki, help output)
//! 3. Produces a compact EvidenceBundle for specialists
//!
//! Key principle: LLMs interpret evidence, they don't invent it.

mod bundle;
mod domain;
mod evidence;
mod intent;
mod request;
mod utils;

// Re-export all public types
pub use bundle::{BundleMetadata, EvidenceBundle};
pub use domain::EvidenceDomain;
pub use evidence::{DocSnippet, DocSource, ProbeEvidence, RecipeMatch};
pub use intent::EvidenceIntent;
pub use request::EvidenceRequest;
