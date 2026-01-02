// v0.0.590: Middleware Pipeline (Phase 166)
// Pipeline implementation for middleware management

use super::types::{Middleware, MiddlewareContext};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
