//! Knowledge Engine - Main orchestrator (v0.0.424).
//!
//! The KnowledgeEngine coordinates all adapters to answer knowledge queries.
//! It tries sources in priority order and returns structured, citable snippets.

use std::time::Instant;

use super::adapters::{ManAdapter, HelpAdapter, DocAdapter, WikiAdapter};
use super::config::KnowledgeConfig;
use super::query::{KnowledgeQuery, KnowledgeSource};
use super::snippet::{KnowledgeSnippet, KnowledgeResult};

/// The main Knowledge Engine
pub struct KnowledgeEngine {
    /// Configuration
    config: KnowledgeConfig,
    /// Man page adapter
    man_adapter: ManAdapter,
    /// Help output adapter
    help_adapter: HelpAdapter,
    /// Local docs adapter
    doc_adapter: DocAdapter,
    /// Wiki adapter (lazy initialized)
    wiki_adapter: Option<WikiAdapter>,
}

impl Default for KnowledgeEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeEngine {
    /// Create engine with default config
    pub fn new() -> Self {
        Self::with_config(KnowledgeConfig::default())
    }

    /// Create engine with custom config
    pub fn with_config(config: KnowledgeConfig) -> Self {
        let man_adapter = ManAdapter::new().with_max_chars(config.max_snippet_chars);
        let help_adapter = HelpAdapter::new().with_max_chars(config.max_snippet_chars);
        let doc_adapter = DocAdapter::new()
            .with_paths(config.doc_paths.clone())
            .with_max_chars(config.max_snippet_chars);

        let wiki_adapter = config.wiki_path.as_ref().map(|p| {
            WikiAdapter::new(Some(p.clone())).with_max_chars(config.max_snippet_chars)
        });

        Self {
            config,
            man_adapter,
            help_adapter,
            doc_adapter,
            wiki_adapter,
        }
    }

    /// Query knowledge sources
    pub fn query(&mut self, q: &KnowledgeQuery) -> KnowledgeResult {
        let start = Instant::now();
        let mut result = KnowledgeResult::empty();

        // Log query if enabled
        if self.config.log_queries {
            tracing::debug!(
                "KnowledgeEngine query: ticket={}, topic={}, entities={:?}",
                q.ticket_id, q.topic, q.entities
            );
        }

        // Query each source in priority order
        for source in &q.preferred_sources {
            result.sources_queried.push(*source);

            if result.snippets.len() >= self.config.max_results_per_query {
                break;
            }

            match source {
                KnowledgeSource::ManPage => {
                    self.query_man(&q, &mut result);
                }
                KnowledgeSource::CommandHelp => {
                    self.query_help(&q, &mut result);
                }
                KnowledgeSource::LocalDocs => {
                    self.query_docs(&q, &mut result);
                }
                KnowledgeSource::ArchWiki => {
                    self.query_wiki(&q, &mut result);
                }
            }
        }

        // Sort by relevance and truncate
        result.sort_by_relevance();
        result.truncate(self.config.max_results_per_query);

        result.duration_ms = start.elapsed().as_millis() as u64;

        if self.config.log_queries {
            tracing::debug!(
                "KnowledgeEngine result: {} snippets in {}ms",
                result.snippets.len(),
                result.duration_ms
            );
        }

        result
    }

    /// Query man pages
    fn query_man(&self, q: &KnowledgeQuery, result: &mut KnowledgeResult) {
        // Try command-like entities
        let commands = q.command_entities();
        for cmd in commands {
            if let Some(mut snippet) = self.man_adapter.query(cmd, Some(&q.topic), None) {
                // Boost relevance if topic is in excerpt
                if snippet.excerpt.to_lowercase().contains(&q.topic.to_lowercase()) {
                    snippet.relevance = 0.9;
                }
                result.add_snippet(snippet);
                return; // One man page per query
            }
        }

        // Try primary entity
        if let Some(entity) = q.primary_entity() {
            if is_command_like(entity) {
                if let Some(snippet) = self.man_adapter.query(entity, Some(&q.topic), None) {
                    result.add_snippet(snippet);
                }
            }
        }
    }

    /// Query help output
    fn query_help(&self, q: &KnowledgeQuery, result: &mut KnowledgeResult) {
        let commands = q.command_entities();
        for cmd in commands {
            if let Some(snippet) = self.help_adapter.query(cmd, Some(&q.topic)) {
                result.add_snippet(snippet);
                return; // One help per query
            }
        }

        // Try primary entity
        if let Some(entity) = q.primary_entity() {
            if is_command_like(entity) {
                if let Some(snippet) = self.help_adapter.query(entity, Some(&q.topic)) {
                    result.add_snippet(snippet);
                }
            }
        }
    }

    /// Query local docs
    fn query_docs(&self, q: &KnowledgeQuery, result: &mut KnowledgeResult) {
        let entities: Vec<&str> = q.entities.iter().map(|s| s.as_str()).collect();
        if let Some(snippet) = self.doc_adapter.query(&q.topic, &entities) {
            result.add_snippet(snippet);
        }
    }

    /// Query Arch Wiki
    fn query_wiki(&mut self, q: &KnowledgeQuery, result: &mut KnowledgeResult) {
        let wiki = match self.wiki_adapter.as_mut() {
            Some(w) => w,
            None => return,
        };

        // Try topic directly
        if let Some(snippet) = wiki.query(&q.topic, None) {
            result.add_snippet(snippet);
            return;
        }

        // Try entities
        let entities: Vec<&str> = q.entities.iter().map(|s| s.as_str()).collect();
        if let Some(snippet) = wiki.query_entities(&entities, Some(&q.topic)) {
            result.add_snippet(snippet);
        }
    }

    /// Quick query for a single command
    pub fn query_command(&mut self, command: &str) -> KnowledgeResult {
        let q = KnowledgeQuery::new("quick", command)
            .with_entity(command)
            .with_sources(vec![KnowledgeSource::ManPage, KnowledgeSource::CommandHelp]);

        self.query(&q)
    }

    /// Quick query for a topic
    pub fn query_topic(&mut self, topic: &str, entities: &[&str]) -> KnowledgeResult {
        let mut q = KnowledgeQuery::new("quick", topic);
        for entity in entities {
            q = q.with_entity(entity);
        }

        self.query(&q)
    }

    /// Check if wiki is available
    pub fn wiki_available(&self) -> bool {
        self.wiki_adapter.as_ref().map(|w| w.is_available()).unwrap_or(false)
    }

    /// Get config reference
    pub fn config(&self) -> &KnowledgeConfig {
        &self.config
    }
}

/// Check if string looks like a command
fn is_command_like(s: &str) -> bool {
    !s.is_empty()
        && !s.contains(' ')
        && !s.starts_with('/')
        && !s.starts_with('~')
        && s.len() < 50
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
}

/// Builder for ticket integration
pub struct EngineQueryBuilder {
    engine: KnowledgeEngine,
}

impl EngineQueryBuilder {
    /// Create builder with new engine
    pub fn new() -> Self {
        Self {
            engine: KnowledgeEngine::new(),
        }
    }

    /// Use existing engine
    pub fn with_engine(engine: KnowledgeEngine) -> Self {
        Self { engine }
    }

    /// Query from ticket context
    pub fn from_ticket(
        &mut self,
        ticket_id: &str,
        domain: &str,
        topic: &str,
        entities: &[String],
    ) -> KnowledgeResult {
        let mut q = KnowledgeQuery::new(ticket_id, topic).with_domain(domain);

        for entity in entities {
            q = q.with_entity(entity);
        }

        self.engine.query(&q)
    }
}

impl Default for EngineQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = KnowledgeEngine::new();
        assert!(engine.config.max_snippet_chars > 0);
    }

    #[test]
    fn test_is_command_like() {
        assert!(is_command_like("vim"));
        assert!(is_command_like("systemctl"));
        assert!(!is_command_like("/etc/passwd"));
        assert!(!is_command_like("some command"));
    }

    #[test]
    fn test_query_command() {
        let mut engine = KnowledgeEngine::new();
        // This test depends on system having 'ls' man page
        let result = engine.query_command("ls");
        // Just verify it doesn't panic
        assert!(result.duration_ms >= 0);
    }

    #[test]
    fn test_query_topic() {
        let mut engine = KnowledgeEngine::new();
        let result = engine.query_topic("vim syntax", &["vim"]);
        // Just verify it doesn't panic
        assert!(result.duration_ms >= 0);
    }

    #[test]
    fn test_query_builder() {
        let mut builder = EngineQueryBuilder::new();
        let result = builder.from_ticket("t1", "editor", "vim config", &["vim".to_string()]);
        assert!(result.duration_ms >= 0);
    }
}
