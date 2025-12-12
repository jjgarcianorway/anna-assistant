//! Knowledge query types (v0.0.424).

use serde::{Deserialize, Serialize};

/// Source of knowledge, in priority order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeSource {
    /// Man pages (highest priority for commands)
    ManPage,
    /// Command --help / -h output
    CommandHelp,
    /// Local documentation (/usr/share/doc, etc.)
    LocalDocs,
    /// Arch Wiki (offline snapshot)
    ArchWiki,
}

impl KnowledgeSource {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ManPage => "Man Page",
            Self::CommandHelp => "Help Output",
            Self::LocalDocs => "Local Docs",
            Self::ArchWiki => "Arch Wiki",
        }
    }

    /// Get citation prefix
    pub fn citation_prefix(&self) -> &'static str {
        match self {
            Self::ManPage => "man",
            Self::CommandHelp => "help",
            Self::LocalDocs => "doc",
            Self::ArchWiki => "wiki",
        }
    }

    /// Default priority order for system topics
    pub fn system_priority() -> Vec<Self> {
        vec![
            Self::ManPage,
            Self::ArchWiki,
            Self::LocalDocs,
            Self::CommandHelp,
        ]
    }

    /// Default priority order for tool/command topics
    pub fn tool_priority() -> Vec<Self> {
        vec![
            Self::ManPage,
            Self::CommandHelp,
            Self::LocalDocs,
            Self::ArchWiki,
        ]
    }

    /// Default priority order for configuration topics
    pub fn config_priority() -> Vec<Self> {
        vec![
            Self::ArchWiki,
            Self::LocalDocs,
            Self::ManPage,
            Self::CommandHelp,
        ]
    }
}

impl Default for KnowledgeSource {
    fn default() -> Self {
        Self::ManPage
    }
}

/// A query to the KnowledgeEngine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeQuery {
    /// Ticket ID for context tracking
    pub ticket_id: String,

    /// Domain of the question (desktop, network, storage, etc.)
    pub domain: String,

    /// Topic being queried (e.g., "vim syntax highlighting")
    pub topic: String,

    /// Extracted entities (commands, paths, packages)
    pub entities: Vec<String>,

    /// Preferred sources in priority order
    pub preferred_sources: Vec<KnowledgeSource>,

    /// Optional section hints (e.g., "SYNOPSIS", "OPTIONS")
    pub section_hints: Vec<String>,

    /// Whether to search broadly or precisely
    pub broad_search: bool,
}

impl KnowledgeQuery {
    /// Create a new query
    pub fn new(ticket_id: &str, topic: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            domain: String::new(),
            topic: topic.to_string(),
            entities: vec![],
            preferred_sources: KnowledgeSource::tool_priority(),
            section_hints: vec![],
            broad_search: false,
        }
    }

    /// Set domain
    pub fn with_domain(mut self, domain: &str) -> Self {
        self.domain = domain.to_string();

        // Auto-adjust source priority based on domain
        self.preferred_sources = match domain.to_lowercase().as_str() {
            "desktop" | "config" | "configuration" => KnowledgeSource::config_priority(),
            "network" | "systemd" | "storage" | "disk" => KnowledgeSource::system_priority(),
            _ => KnowledgeSource::tool_priority(),
        };

        self
    }

    /// Add an entity
    pub fn with_entity(mut self, entity: &str) -> Self {
        self.entities.push(entity.to_string());
        self
    }

    /// Add multiple entities
    pub fn with_entities(mut self, entities: &[&str]) -> Self {
        self.entities.extend(entities.iter().map(|s| s.to_string()));
        self
    }

    /// Set preferred sources
    pub fn with_sources(mut self, sources: Vec<KnowledgeSource>) -> Self {
        self.preferred_sources = sources;
        self
    }

    /// Add section hints
    pub fn with_section_hints(mut self, hints: &[&str]) -> Self {
        self.section_hints = hints.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Enable broad search
    pub fn broad(mut self) -> Self {
        self.broad_search = true;
        self
    }

    /// Get primary entity (first one, typically the main command)
    pub fn primary_entity(&self) -> Option<&str> {
        self.entities.first().map(|s| s.as_str())
    }

    /// Extract command-like entities (lowercase, no spaces)
    pub fn command_entities(&self) -> Vec<&str> {
        self.entities
            .iter()
            .filter(|e| is_command_like(e))
            .map(|s| s.as_str())
            .collect()
    }

    /// Extract path-like entities (start with / or ~)
    pub fn path_entities(&self) -> Vec<&str> {
        self.entities
            .iter()
            .filter(|e| e.starts_with('/') || e.starts_with('~'))
            .map(|s| s.as_str())
            .collect()
    }
}

/// Check if string looks like a command name
fn is_command_like(s: &str) -> bool {
    !s.is_empty()
        && !s.contains(' ')
        && !s.starts_with('/')
        && !s.starts_with('~')
        && s.len() < 50
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
}

/// Builder for creating queries from ticket context
pub struct QueryBuilder {
    query: KnowledgeQuery,
}

impl QueryBuilder {
    /// Start building a query
    pub fn new(ticket_id: &str) -> Self {
        Self {
            query: KnowledgeQuery::new(ticket_id, ""),
        }
    }

    /// Set topic from question
    pub fn topic(mut self, topic: &str) -> Self {
        self.query.topic = topic.to_string();
        self
    }

    /// Set domain
    pub fn domain(mut self, domain: &str) -> Self {
        self.query = self.query.with_domain(domain);
        self
    }

    /// Add entity
    pub fn entity(mut self, entity: &str) -> Self {
        self.query.entities.push(entity.to_string());
        self
    }

    /// Add entities from a list
    pub fn entities(mut self, entities: &[String]) -> Self {
        self.query.entities.extend(entities.iter().cloned());
        self
    }

    /// Build the query
    pub fn build(self) -> KnowledgeQuery {
        self.query
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_source_priority() {
        let sys = KnowledgeSource::system_priority();
        assert_eq!(sys[0], KnowledgeSource::ManPage);

        let tool = KnowledgeSource::tool_priority();
        assert_eq!(tool[0], KnowledgeSource::ManPage);

        let config = KnowledgeSource::config_priority();
        assert_eq!(config[0], KnowledgeSource::ArchWiki);
    }

    #[test]
    fn test_query_builder() {
        let query = QueryBuilder::new("ticket-123")
            .topic("vim syntax highlighting")
            .domain("editor")
            .entity("vim")
            .entity("~/.vimrc")
            .build();

        assert_eq!(query.ticket_id, "ticket-123");
        assert_eq!(query.topic, "vim syntax highlighting");
        assert!(query.entities.contains(&"vim".to_string()));
    }

    #[test]
    fn test_command_entities() {
        let query = KnowledgeQuery::new("t1", "test")
            .with_entity("systemctl")
            .with_entity("/etc/systemd")
            .with_entity("nginx.service");

        let cmds = query.command_entities();
        assert!(cmds.contains(&"systemctl"));
        assert!(cmds.contains(&"nginx.service"));
        assert!(!cmds.iter().any(|c| c.starts_with('/')));
    }

    #[test]
    fn test_is_command_like() {
        assert!(is_command_like("vim"));
        assert!(is_command_like("systemctl"));
        assert!(is_command_like("nginx.service"));
        assert!(!is_command_like("/etc/passwd"));
        assert!(!is_command_like("some command with spaces"));
    }
}
