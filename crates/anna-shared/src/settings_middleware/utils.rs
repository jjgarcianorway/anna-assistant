// v0.0.590: Middleware Utilities (Phase 166)
// Utility functions for middleware operations

use super::pipeline::MiddlewarePipeline;

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
