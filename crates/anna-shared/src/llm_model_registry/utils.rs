// v0.0.531: LLM Model Registry Utilities
// Helper functions for model-related operations

/// Check if query is model-related
pub fn is_model_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("model")
        || lower.contains("llm")
        || lower.contains("ollama")
        || lower.contains("qwen")
        || lower.contains("llama")
        || lower.contains("vram")
}

/// Fun fact about models
pub fn model_fun_fact() -> &'static str {
    "Modern LLMs can contain billions of parameters - GPT-4 is estimated to have over 1.7 trillion parameters, while smaller models like Qwen 3B fit in just 2GB!"
}
