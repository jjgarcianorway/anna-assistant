//! Knowledge and documentation-related module declarations
//! All modules for knowledge engine, documentation, and learning

// Core knowledge modules
#[path = "knowledge/mod.rs"]
pub mod knowledge;
#[path = "knowledge_config.rs"]
pub mod knowledge_config; // v0.0.414: Knowledge source configuration
#[path = "knowledge_engine/mod.rs"]
pub mod knowledge_engine; // v0.0.416: Knowledge engine (man, help, wiki)
#[path = "knowledge_executor.rs"]
pub mod knowledge_executor; // v0.0.414: Knowledge query executor
#[path = "knowledge_index/mod.rs"]
pub mod knowledge_index; // v0.0.410: Compiled knowledge store
#[path = "knowledge_item.rs"]
pub mod knowledge_item; // v0.0.408: Knowledge item abstraction
#[path = "knowledge_learning/mod.rs"]
pub mod knowledge_learning; // v0.0.414: Self-learning from docs and tickets
#[path = "knowledge_pipeline/mod.rs"]
pub mod knowledge_pipeline; // v0.0.432: Priority-ordered knowledge fetching and learning
#[path = "knowledge_query.rs"]
pub mod knowledge_query; // v0.0.414: Doc-first knowledge query interface
#[path = "knowledge_v2/mod.rs"]
pub mod knowledge_v2; // v0.0.422: Research-first knowledge layer
#[path = "knowledge_v4/mod.rs"]
pub mod knowledge_v4; // v0.0.424: Complete local knowledge engine with citations
#[path = "knowledge_citation/mod.rs"]
pub mod knowledge_citation; // v0.0.530: Knowledge citation tracker (modularized)
#[path = "knowledge_base_stats/mod.rs"]
pub mod knowledge_base_stats; // v0.0.499: Knowledge base stats

// Documentation modules
#[path = "doc_brain.rs"]
pub mod doc_brain; // v0.0.406: Unified doc search (man pages, wiki, help)
#[path = "doc_engine/mod.rs"]
pub mod doc_engine; // v0.0.429: Documentation brain - local knowledge graph
#[path = "doc_fetcher/mod.rs"]
pub mod doc_fetcher; // v0.0.410: Enhanced doc fetchers
#[path = "doc_first_workflow.rs"]
pub mod doc_first_workflow; // v0.0.414: Doc-first specialist reasoning
#[path = "doc_search/mod.rs"]
pub mod doc_search; // v0.0.408: Local documentation search (modular)
#[path = "doc_snippet/mod.rs"]
pub mod doc_snippet; // v0.0.412: Documentation source integration

// Evidence modules
#[path = "evidence_engine/mod.rs"]
pub mod evidence_engine; // v0.0.410: Evidence engine core types
#[path = "evidence_first/mod.rs"]
pub mod evidence_first; // v0.0.435: Evidence-first knowledge engine
#[path = "evidence_gatherer.rs"]
pub mod evidence_gatherer; // v0.0.410: Evidence orchestration
#[path = "evidence_pipeline.rs"]
pub mod evidence_pipeline; // v0.0.410: Full evidence integration

// Learning modules
#[path = "learning_engine/mod.rs"]
pub mod learning_engine; // v0.0.427: Self-learning recipe engine
#[path = "learning_explanations/mod.rs"]
pub mod learning_explanations; // v0.0.457: Learning mode command explanations
#[path = "learning_progress.rs"]
pub mod learning_progress; // v0.0.288: Learning progress tracking
#[path = "learning_stats.rs"]
pub mod learning_stats; // v0.0.401: Learning progress statistics
#[path = "learning_suggestions.rs"]
pub mod learning_suggestions; // v0.0.282: Idle-time learning suggestions

// Specialist learning modules
#[path = "specialist_learning/mod.rs"]
pub mod specialist_learning;
#[path = "specialist_patterns.rs"]
pub mod specialist_patterns; // v0.0.401: Generic pattern matching
#[path = "specialist_recipes.rs"]
pub mod specialist_recipes; // v0.0.401: Recipes from specialist lessons
#[path = "clarification_learning.rs"]
pub mod clarification_learning; // v0.0.401: Learning from user clarifications

// Probe modules
#[path = "probe_registry/mod.rs"]
pub mod probe_registry; // v0.0.410: Composable probe definitions
#[path = "probe_spine/mod.rs"]
pub mod probe_spine;
#[path = "probe_learning/mod.rs"]
pub mod probe_learning; // v0.0.322: Probe effectiveness learning
#[path = "deterministic_probes/mod.rs"]
pub mod deterministic_probes; // v0.0.448: Intent → probes deterministic mapping

// Wiki and caching
#[path = "wiki_cache/mod.rs"]
pub mod wiki_cache; // v0.0.472: Arch Wiki local caching

// User preference learning
#[path = "user_preference_learner.rs"]
pub mod user_preference_learner; // v0.0.522: User preference learner
