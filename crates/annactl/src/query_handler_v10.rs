//! Anna Brain v10.0.0 - Query Handler
//!
//! INPUT → LLM → TOOLS → LLM → ANSWER
//! Evidence-based answers with explicit reliability labels.

use anna_common::brain_v10::{BrainOrchestrator, BrainResult, ReliabilityLabel};
use anna_common::llm_client::LlmConfig;
use anna_common::telemetry::SystemTelemetry;
use anyhow::Result;

/// Handle a query using the v10.0.0 evidence-based architecture
pub async fn handle_query_v10(
    query: &str,
    telemetry: &SystemTelemetry,
    llm_config: Option<&LlmConfig>,
) -> Result<String> {
    // Check if this is a meta query about Anna
    if is_meta_query(query) {
        return Ok(handle_meta_query(query));
    }

    // Get LLM config
    let config = match llm_config {
        Some(c) if c.enabled => c.clone(),
        _ => {
            return Ok(format_no_llm_message());
        }
    };

    // Create orchestrator and process
    let orchestrator = BrainOrchestrator::new(config)?;

    match orchestrator.process(query, telemetry, None)? {
        BrainResult::Answer { text, reliability, label } => {
            Ok(format_answer(&text, reliability, &label))
        }
        BrainResult::NeedsUserInput { question } => {
            Ok(format!("❓  I need more information:\n\n{}", question))
        }
    }
}

/// Handle a query with user-provided context (for follow-up questions)
pub async fn handle_query_v10_with_context(
    query: &str,
    telemetry: &SystemTelemetry,
    llm_config: Option<&LlmConfig>,
    user_context: &str,
) -> Result<String> {
    let config = match llm_config {
        Some(c) if c.enabled => c.clone(),
        _ => return Ok(format_no_llm_message()),
    };

    let orchestrator = BrainOrchestrator::new(config)?;

    match orchestrator.process(query, telemetry, Some(user_context))? {
        BrainResult::Answer { text, reliability, label } => {
            Ok(format_answer(&text, reliability, &label))
        }
        BrainResult::NeedsUserInput { question } => {
            Ok(format!("❓  I need more information:\n\n{}", question))
        }
    }
}

/// Format the answer with proper styling
fn format_answer(text: &str, reliability: f32, label: &ReliabilityLabel) -> String {
    // The orchestrator already formats the answer with reliability
    // Just ensure proper styling
    text.to_string()
}

/// Format the "no LLM configured" message
fn format_no_llm_message() -> String {
    r#"🔴  LLM is not configured. Anna requires a local LLM to answer questions.

📋  To enable:

  1. Install Ollama:
     curl -fsSL https://ollama.com/install.sh | sh

  2. Pull a model (recommended):
     ollama pull qwen2.5:14b

  3. Restart Anna:
     systemctl --user restart annad

💡  Alternatively, configure in ~/.config/anna/config.toml:
    [llm]
    enabled = true
    model = "qwen2.5:14b""#
        .to_string()
}

/// Check if this is a meta query about Anna itself
fn is_meta_query(query: &str) -> bool {
    let q = query.to_lowercase();
    q.contains("anna version")
        || q.contains("who are you")
        || q.contains("about anna")
        || q.contains("upgrade your brain")
        || q.contains("upgrade brain")
        || q.contains("change model")
        || q.contains("what version")
        || q.contains("what are you")
}

/// Handle meta queries about Anna
fn handle_meta_query(query: &str) -> String {
    let q = query.to_lowercase();

    if q.contains("upgrade") || q.contains("brain") || q.contains("model") {
        return r#"🧠  How to upgrade Anna's LLM:

  1. List available models:
     ollama list

  2. Pull a new model:
     ollama pull qwen2.5:14b   # Recommended: good reasoning
     ollama pull llama3.2:3b   # Fast: simple queries
     ollama pull mistral:7b    # Alternative: good general

  3. Edit ~/.config/anna/config.toml:
     [llm]
     model = "qwen2.5:14b"

  4. Restart Anna:
     systemctl --user restart annad"#
            .to_string();
    }

    if q.contains("version") {
        return "🤖  Anna Assistant v10.0.0 - Evidence-Based Architecture".to_string();
    }

    if q.contains("who are you") || q.contains("about anna") || q.contains("what are you") {
        return r#"👋  I am Anna, your local Arch Linux system assistant.

🧠  Architecture (v10.0.0 - Evidence-Based):
    • Every answer is grounded in tool output evidence
    • Reliability scores: HIGH/MEDIUM/LOW/VERY LOW
    • Citations reference evidence IDs like [E1], [E2]
    • Max 8 iterations to gather sufficient evidence

📊  Philosophy:
    • I never guess or hallucinate
    • I only report what I observe from your system
    • If I can't find evidence, I say so honestly

🔧  Components:
    • annad: Background daemon (telemetry, tools)
    • annactl: CLI interface (you're using this)
    • brain: LLM orchestration (qwen2.5:14b)"#
            .to_string();
    }

    "🤖  I'm Anna, your local Arch Linux assistant. Ask me about your system!".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_query_detection() {
        assert!(is_meta_query("what is Anna version"));
        assert!(is_meta_query("upgrade your brain"));
        assert!(is_meta_query("who are you"));
        assert!(is_meta_query("what are you"));
        assert!(!is_meta_query("how much RAM"));
    }

    #[test]
    fn test_meta_query_responses() {
        let upgrade = handle_meta_query("upgrade brain");
        assert!(upgrade.contains("ollama"));

        let version = handle_meta_query("anna version");
        assert!(version.contains("v10.0.0"));

        let about = handle_meta_query("who are you");
        assert!(about.contains("Evidence-Based"));
    }

    #[test]
    fn test_no_llm_message() {
        let msg = format_no_llm_message();
        assert!(msg.contains("Ollama"));
        assert!(msg.contains("config.toml"));
    }
}
