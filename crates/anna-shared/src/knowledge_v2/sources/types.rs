//! Source fetch result types (v0.0.422).

/// Result of a fetch operation
#[derive(Debug, Clone)]
pub struct SourceFetchResult {
    /// Fetched content
    pub content: String,
    /// Whether content was from cache
    pub from_cache: bool,
    /// Source path or URL
    pub source_path: String,
}

impl SourceFetchResult {
    /// Create new result
    pub fn new(content: String, source_path: &str) -> Self {
        Self {
            content,
            from_cache: false,
            source_path: source_path.to_string(),
        }
    }

    /// Mark as from cache
    pub fn cached(mut self) -> Self {
        self.from_cache = true;
        self
    }

    /// Check if fetch was successful
    pub fn is_ok(&self) -> bool {
        !self.content.is_empty()
    }
}
