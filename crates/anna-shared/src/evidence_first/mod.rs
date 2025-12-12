//! Evidence-First Knowledge Engine (v0.0.435).
//!
//! Anna answers by retrieving citations from local sources first:
//! 1. System probes (live evidence)
//! 2. Local documentation (man pages, --help)
//! 3. Cached Arch Wiki (offline)
//! 4. LLMs only for interpretation and synthesis
//!
//! Key principles:
//! - No citations, no claims
//! - Probes are primitives, not one-off scripts
//! - Recipes require proof before promotion

pub mod sources;
pub mod citations;
pub mod research;
pub mod primitives;
pub mod probe_plan;
pub mod recipes;
pub mod wiki_cache;
pub mod enforcement;
pub mod tests;

pub use sources::{KnowledgeSource, ManPageSource, HelpTextSource, LocalDocsSource};
pub use citations::{Citation, CitationStore, EvidenceId};
pub use research::{ResearchLoop, ResearchPlan, ResearchResult};
pub use primitives::{ProbePrimitive, PrimitiveLibrary, Domain, ParserId};
pub use probe_plan::{ProbePlan, ProbeSelection, ProbeOutput};
pub use recipes::{RecipeTemplate, RecipeCandidate, RecipePromoter};
pub use wiki_cache::{WikiCache, WikiPage, WikiSearchResult};
pub use enforcement::{ClaimValidator, Claim, SupportedClaim};

/// Maximum probes per ticket.
pub const MAX_PROBES_PER_TICKET: usize = 5;

/// Maximum research iterations.
pub const MAX_RESEARCH_ITERATIONS: usize = 2;

/// Default probe timeout in milliseconds.
pub const DEFAULT_PROBE_TIMEOUT_MS: u64 = 5000;

/// Minimum confirmations to promote a recipe.
pub const MIN_CONFIRMATIONS_FOR_RECIPE: usize = 3;

/// Wiki cache directory.
pub const WIKI_CACHE_DIR: &str = "/var/lib/anna/wiki";

/// Maximum citation excerpt length.
pub const MAX_CITATION_EXCERPT_LEN: usize = 200;
