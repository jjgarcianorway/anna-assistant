//! Command error types for intelligent recovery.

/// Error categories for intelligent recovery
#[derive(Debug, Clone, PartialEq)]
pub enum CommandErrorType {
    CommandNotFound,
    PermissionDenied,
    PathNotFound,
    Timeout,
    SyntaxError,
    MissingDependency,
    EmptyOutput,
    Unknown,
}
