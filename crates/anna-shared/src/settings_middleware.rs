// v0.0.590: Settings Middleware (Phase 166)
// Middleware pipeline for settings operations

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

/// Middleware pipeline
#[derive(Debug, Clone, Default)]
pub struct MiddlewarePipeline {
    /// Registered middleware
    middleware: Vec<Middleware>,
}

impl MiddlewarePipeline {
    /// Create new pipeline
    pub fn new() -> Self {
        Self::default()
    }

    /// Add middleware
    pub fn add(&mut self, mw: Middleware) {
        self.middleware.push(mw);
        self.middleware.sort_by(|a, b| a.priority.cmp(&b.priority));
    }

    /// Remove middleware by name
    pub fn remove(&mut self, name: &str) -> bool {
        let len = self.middleware.len();
        self.middleware.retain(|m| m.name != name);
        self.middleware.len() < len
    }

    /// Get middleware by name
    pub fn get(&self, name: &str) -> Option<&Middleware> {
        self.middleware.iter().find(|m| m.name == name)
    }

    /// Enable middleware
    pub fn enable(&mut self, name: &str) -> bool {
        if let Some(mw) = self.middleware.iter_mut().find(|m| m.name == name) {
            mw.enabled = true;
            return true;
        }
        false
    }

    /// Disable middleware
    pub fn disable(&mut self, name: &str) -> bool {
        if let Some(mw) = self.middleware.iter_mut().find(|m| m.name == name) {
            mw.enabled = false;
            return true;
        }
        false
    }

    /// Get applicable middleware for context
    pub fn applicable(&self, ctx: &MiddlewareContext) -> Vec<&Middleware> {
        self.middleware.iter().filter(|m| m.applies(ctx)).collect()
    }

    /// Count middleware
    pub fn count(&self) -> usize {
        self.middleware.len()
    }

    /// Count enabled
    pub fn enabled_count(&self) -> usize {
        self.middleware.iter().filter(|m| m.enabled).count()
    }

    /// List all middleware
    pub fn all(&self) -> &[Middleware] {
        &self.middleware
    }

    /// Clear all middleware
    pub fn clear(&mut self) {
        self.middleware.clear();
    }
}

/// Format pipeline
pub fn format_pipeline(pipeline: &MiddlewarePipeline) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Middleware ===\n\n");
    output.push_str(&format!(
        "Total: {} ({} enabled)\n\n",
        pipeline.count(),
        pipeline.enabled_count()
    ));

    for mw in pipeline.all() {
        let status = if mw.enabled { "enabled" } else { "disabled" };
        output.push_str(&format!("{} [{}] - {}\n", mw.name, status, mw.priority));
    }

    output
}

/// Check if query is about middleware
pub fn is_middleware_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("middleware")
        || lower.contains("pipeline")
        || lower.contains("intercept")
}

/// Fun fact about middleware
pub fn settings_middleware_fun_fact() -> &'static str {
    "Anna uses middleware to validate, log, and transform settings operations!"
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

    #[test]
    fn test_pipeline_new() {
        let pipeline = MiddlewarePipeline::new();
        assert_eq!(pipeline.count(), 0);
    }

    #[test]
    fn test_pipeline_add_remove() {
        let mut pipeline = MiddlewarePipeline::new();
        pipeline.add(Middleware::new("test"));
        assert_eq!(pipeline.count(), 1);
        assert!(pipeline.remove("test"));
        assert_eq!(pipeline.count(), 0);
    }

    #[test]
    fn test_pipeline_enable_disable() {
        let mut pipeline = MiddlewarePipeline::new();
        pipeline.add(Middleware::new("test"));
        assert!(pipeline.disable("test"));
        assert!(!pipeline.get("test").unwrap().enabled);
        assert!(pipeline.enable("test"));
        assert!(pipeline.get("test").unwrap().enabled);
    }

    #[test]
    fn test_format_pipeline() {
        let pipeline = MiddlewarePipeline::new();
        let output = format_pipeline(&pipeline);
        assert!(output.contains("Middleware"));
    }

    #[test]
    fn test_is_middleware_query() {
        assert!(is_middleware_query("add middleware"));
        assert!(is_middleware_query("settings pipeline"));
        assert!(!is_middleware_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_middleware_fun_fact();
        assert!(fact.contains("middleware"));
    }
}
