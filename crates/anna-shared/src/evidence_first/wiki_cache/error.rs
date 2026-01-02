//! Wiki cache error types.

/// Cache error.
#[derive(Debug, Clone)]
pub enum CacheError {
    /// IO error.
    IoError(String),
    /// Serialization error.
    SerializeError(String),
    /// Network error.
    NetworkError(String),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(msg) => write!(f, "IO error: {}", msg),
            Self::SerializeError(msg) => write!(f, "Serialize error: {}", msg),
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl std::error::Error for CacheError {}
