//! Source retrieval errors (v0.0.435).

/// Source retrieval error.
#[derive(Debug, Clone)]
pub enum SourceError {
    /// Command failed to run.
    CommandFailed(String),
    /// Source not found.
    NotFound(String),
    /// Failed to read content.
    ReadFailed(String),
    /// Timeout.
    Timeout,
}

impl std::fmt::Display for SourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CommandFailed(msg) => write!(f, "Command failed: {}", msg),
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
            Self::ReadFailed(msg) => write!(f, "Read failed: {}", msg),
            Self::Timeout => write!(f, "Timeout"),
        }
    }
}

impl std::error::Error for SourceError {}
