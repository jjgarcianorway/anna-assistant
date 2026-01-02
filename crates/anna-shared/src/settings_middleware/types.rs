// v0.0.590: Settings Middleware Types (Phase 166)
// Core types for settings middleware

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

/// Middleware priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MiddlewarePriority {
    /// Highest priority (runs first)
    Critical,
    /// High priority
    High,
    /// Normal priority
    Normal,
    /// Low priority
    Low,
}

impl Default for MiddlewarePriority {
    fn default() -> Self {
        Self::Normal
    }
}

impl std::fmt::Display for MiddlewarePriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "critical"),
            Self::High => write!(f, "high"),
            Self::Normal => write!(f, "normal"),
            Self::Low => write!(f, "low"),
        }
    }
}

/// Middleware action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MiddlewareAction {
    /// Continue to next middleware
    Continue,
    /// Skip remaining middleware
    Skip,
    /// Abort the operation
    Abort,
    /// Modify and continue
    Modify,
}

impl std::fmt::Display for MiddlewareAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Continue => write!(f, "continue"),
            Self::Skip => write!(f, "skip"),
            Self::Abort => write!(f, "abort"),
            Self::Modify => write!(f, "modify"),
        }
    }
}

/// Middleware context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiddlewareContext {
    /// Operation type
    pub operation: String,
    /// Target category
    pub category: Option<SettingsCategory>,
    /// Key path
    pub key: Option<String>,
    /// Value (serialized)
    pub value: Option<String>,
    /// Metadata
    pub metadata: std::collections::HashMap<String, String>,
    /// Aborted flag
    pub aborted: bool,
    /// Abort reason
    pub abort_reason: Option<String>,
}

impl MiddlewareContext {
    /// Create new context
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            category: None,
            key: None,
            value: None,
            metadata: std::collections::HashMap::new(),
            aborted: false,
            abort_reason: None,
        }
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set key
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Set value
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Add metadata
    pub fn meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Abort context
    pub fn abort(&mut self, reason: impl Into<String>) {
        self.aborted = true;
        self.abort_reason = Some(reason.into());
    }

    /// Check if aborted
    pub fn is_aborted(&self) -> bool {
        self.aborted
    }
}

/// Middleware result
#[derive(Debug, Clone)]
pub struct MiddlewareResult {
    /// Action to take
    pub action: MiddlewareAction,
    /// Modified context
    pub context: MiddlewareContext,
    /// Message
    pub message: Option<String>,
}

impl MiddlewareResult {
    /// Continue with context
    pub fn cont(context: MiddlewareContext) -> Self {
        Self {
            action: MiddlewareAction::Continue,
            context,
            message: None,
        }
    }

    /// Abort with reason
    pub fn abort(mut context: MiddlewareContext, reason: impl Into<String>) -> Self {
        let msg = reason.into();
        context.abort(&msg);
        Self {
            action: MiddlewareAction::Abort,
            context,
            message: Some(msg),
        }
    }

    /// Skip remaining
    pub fn skip(context: MiddlewareContext) -> Self {
        Self {
            action: MiddlewareAction::Skip,
            context,
            message: None,
        }
    }
}

/// Middleware definition
#[derive(Debug, Clone)]
pub struct Middleware {
    /// Name
    pub name: String,
    /// Priority
    pub priority: MiddlewarePriority,
    /// Enabled
    pub enabled: bool,
    /// Operations to handle (empty = all)
    pub operations: Vec<String>,
    /// Categories to handle (empty = all)
    pub categories: Vec<SettingsCategory>,
}

impl Middleware {
    /// Create new middleware
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            priority: MiddlewarePriority::Normal,
            enabled: true,
            operations: Vec::new(),
            categories: Vec::new(),
        }
    }

    /// Set priority
    pub fn priority(mut self, priority: MiddlewarePriority) -> Self {
        self.priority = priority;
        self
    }

    /// Add operation filter
    pub fn operation(mut self, op: impl Into<String>) -> Self {
        self.operations.push(op.into());
        self
    }

    /// Add category filter
    pub fn category(mut self, cat: SettingsCategory) -> Self {
        self.categories.push(cat);
        self
    }

    /// Disable middleware
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Check if applies to context
    pub fn applies(&self, ctx: &MiddlewareContext) -> bool {
        if !self.enabled {
            return false;
        }

        if !self.operations.is_empty() && !self.operations.contains(&ctx.operation) {
            return false;
        }

        if !self.categories.is_empty() {
            if let Some(cat) = ctx.category {
                if !self.categories.contains(&cat) {
                    return false;
                }
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_display() {
        assert_eq!(format!("{}", MiddlewarePriority::High), "high");
        assert_eq!(format!("{}", MiddlewarePriority::Normal), "normal");
    }

    #[test]
    fn test_action_display() {
        assert_eq!(format!("{}", MiddlewareAction::Continue), "continue");
        assert_eq!(format!("{}", MiddlewareAction::Abort), "abort");
    }

    #[test]
    fn test_context_new() {
        let ctx = MiddlewareContext::new("write");
        assert_eq!(ctx.operation, "write");
        assert!(!ctx.is_aborted());
    }

    #[test]
    fn test_context_builder() {
        let ctx = MiddlewareContext::new("read")
            .category(SettingsCategory::Personality)
            .key("formality")
            .meta("source", "cli");
        assert!(ctx.category.is_some());
        assert!(ctx.key.is_some());
    }

    #[test]
    fn test_context_abort() {
        let mut ctx = MiddlewareContext::new("write");
        ctx.abort("validation failed");
        assert!(ctx.is_aborted());
        assert!(ctx.abort_reason.is_some());
    }

    #[test]
    fn test_middleware_new() {
        let mw = Middleware::new("validator");
        assert_eq!(mw.name, "validator");
        assert!(mw.enabled);
    }

    #[test]
    fn test_middleware_applies() {
        let mw = Middleware::new("test").operation("write");
        let ctx_write = MiddlewareContext::new("write");
        let ctx_read = MiddlewareContext::new("read");
        assert!(mw.applies(&ctx_write));
        assert!(!mw.applies(&ctx_read));
    }
}
