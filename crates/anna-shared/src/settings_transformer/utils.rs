// v0.0.598: Settings Transformer Utilities (Phase 174)
// Helper functions for transformers

use super::pipeline::TransformPipeline;

/// Format transform pipeline
pub fn format_transform_pipeline(pipeline: &TransformPipeline) -> String {
    let mut output = String::new();
    output.push_str("Transform Pipeline:\n");
    output.push_str(&format!("  Transforms: {}\n", pipeline.count()));
    output.push_str(&format!("  Enabled: {}\n", pipeline.enabled_count()));

    for t in &pipeline.transforms {
        let status = if t.enabled { "✓" } else { "✗" };
        output.push_str(&format!(
            "  {} [{}] {} ({}, {})\n",
            status, t.transform_type, t.name, t.direction, t.priority
        ));
    }

    output
}

/// Check if query is about transformer
pub fn is_transformer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("transform")
        || lower.contains("convert settings")
        || lower.contains("normalize")
}

/// Fun fact about transformers
pub fn transformer_fun_fact() -> &'static str {
    "Anna uses transform pipelines to normalize and convert settings values automatically!"
}
