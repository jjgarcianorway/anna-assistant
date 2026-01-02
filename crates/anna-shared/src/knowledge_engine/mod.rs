//! Knowledge Engine (v0.0.416).
//!
//! Fetches structured knowledge from local sources:
//! - Man pages (man <command>)
//! - CLI help output (<command> --help)
//! - Local documentation (/usr/share/doc, /usr/share/help)
//! - Arch Wiki offline cache (optional)
//!
//! NO LLM calls. Just text retrieval, slicing, and caching.

mod engine;
pub(crate) mod fetchers;
mod types;
pub(crate) mod utils;

// Re-export public types and engine
pub use engine::KnowledgeEngine;
pub use types::{
    KnowledgeContext, KnowledgeEngineHit, KnowledgeKind, KnowledgeRequest, KnowledgeResponse,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_request() {
        let req = KnowledgeRequest {
            topic: "failed services".to_string(),
            context: KnowledgeContext {
                intent: "check_status".to_string(),
                domain: "services".to_string(),
                commands: vec!["systemctl".to_string()],
            },
            sources: vec![KnowledgeKind::ManPage, KnowledgeKind::CliHelp],
            limit: 3,
        };

        let engine = KnowledgeEngine::new();
        let response = engine.query(&req);

        // Should have searched the sources
        assert!(response.sources_searched.contains(&KnowledgeKind::ManPage));
    }
}
