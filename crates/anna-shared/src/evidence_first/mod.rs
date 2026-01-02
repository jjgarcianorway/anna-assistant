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

pub mod citations;
pub mod enforcement;
pub mod primitives;
pub mod probe_plan;
pub mod research;
pub mod research_helpers;
pub mod research_types;
pub mod sources;
pub mod wiki_cache;

#[cfg(test)]
mod tests;

// Recipe modules (split for better organization)
mod recipes_candidate;
mod recipes_helpers;
mod recipes_promoter;
mod recipes_types;

pub use citations::{Citation, CitationStore, EvidenceId};
pub use enforcement::{Claim, ClaimValidator, SupportedClaim};
pub use primitives::{Domain, ParserId, PrimitiveLibrary, ProbePrimitive};
pub use probe_plan::{ProbeOutput, ProbePlan, ProbeSelection};
pub use research::ResearchLoop;
pub use research_helpers::QuickResearch;
pub use research_types::{Confidence, DocResult, Finding, ResearchPlan, ResearchResult};
pub use sources::{
    HelpTextSource, HelpVariant, KnowledgeSource, LocalDocsSource, ManPageSource, ManSection,
    SourceError,
};
pub use wiki_cache::{WikiCache, WikiPage, WikiSearchResult};

// Re-export recipe types
pub use recipes_candidate::{Confirmation, Failure, RecipeCandidate};
pub use recipes_promoter::{PromoterStatus, RecipePromoter};
pub use recipes_types::{RecipeInstance, RecipeOutcome, RecipeStep, RecipeTemplate};

/// Maximum probes per ticket.
pub const MAX_PROBES_PER_TICKET: usize = 5;

/// Maximum research iterations.
pub const MAX_RESEARCH_ITERATIONS: usize = 2;

/// Default probe timeout in milliseconds.
pub const DEFAULT_PROBE_TIMEOUT_MS: u64 = 5000;

/// Minimum confirmations to promote a recipe.
pub const MIN_CONFIRMATIONS_FOR_RECIPE: usize = 3;

/// Maximum citation excerpt length.
pub const MAX_CITATION_EXCERPT_LEN: usize = 200;
