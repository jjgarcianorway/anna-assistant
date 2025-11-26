//! Anna Brain v10.3.0 - Query Handler
//!
//! INPUT → LLM → TOOLS → LLM → ANSWER → LEARN
//! Evidence-based answers with explicit reliability labels.
//!
//! v10.3.0: Strict evidence discipline - NO KNOWLEDGE WITHOUT EVIDENCE
//! v10.2.1: Better output formatting, timing diagnostics, honest answers

use anna_common::anna_config::AnnaConfig;
use anna_common::brain_v10::{BrainOrchestrator, BrainResult, ReliabilityLabel};
use anna_common::llm_client::LlmConfig;
use anna_common::telemetry::SystemTelemetry;
use anyhow::Result;
use std::time::Instant;

/// Handle a query using the v10.2.1 evidence-based architecture
pub async fn handle_query_v10(
    query: &str,
    telemetry: &SystemTelemetry,
    llm_config: Option<&LlmConfig>,
) -> Result<String> {
    let start = Instant::now();
    let config_file = AnnaConfig::load().unwrap_or_default();
    let show_timing = config_file.dev.is_debug_enabled();

    // Check if this is a meta query about Anna
    if is_meta_query(query) {
        let answer = handle_meta_query(query, telemetry);
        return Ok(maybe_add_timing(answer, start, show_timing, 0));
    }

    // Get LLM config
    let config = match llm_config {
        Some(c) if c.enabled => c.clone(),
        _ => {
            return Ok(format_no_llm_message());
        }
    };

    let telemetry_time = start.elapsed();

    // Create orchestrator and process
    let llm_start = Instant::now();
    let mut orchestrator = BrainOrchestrator::new(config)?;

    match orchestrator.process(query, telemetry, None)? {
        BrainResult::Answer { text, reliability, label } => {
            let llm_time = llm_start.elapsed();
            let answer = format_answer(&text, reliability, &label);
            Ok(maybe_add_timing(answer, start, show_timing, llm_time.as_millis() as u64))
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

    let mut orchestrator = BrainOrchestrator::new(config)?;

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
fn format_answer(text: &str, _reliability: f32, _label: &ReliabilityLabel) -> String {
    // The orchestrator already formats the answer with reliability
    // Just ensure proper styling
    text.to_string()
}

/// Add timing diagnostics if enabled
fn maybe_add_timing(answer: String, start: Instant, show: bool, llm_ms: u64) -> String {
    if !show {
        return answer;
    }

    let total = start.elapsed();
    format!(
        "{}\n\n⏱️  debug: total {:.2}s, LLM {:.2}s",
        answer,
        total.as_secs_f64(),
        llm_ms as f64 / 1000.0
    )
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
        || q.contains("can you upgrade")
        || q.contains("change model")
        || q.contains("what version")
        || q.contains("what are you")
        || q.contains("dependencies outdated")
        || q.contains("dependencies missing")
        || q.contains("your own dependencies")
}

/// Handle meta queries about Anna
/// v10.2.1: Use telemetry for evidence-based answers
fn handle_meta_query(query: &str, _telemetry: &SystemTelemetry) -> String {
    let q = query.to_lowercase();

    // v10.2.1: LLM upgrade instructions - concrete steps for THIS host
    if q.contains("upgrade") || q.contains("brain") || q.contains("model") {
        // Get config path using HOME env var
        let config_path = std::env::var("HOME")
            .map(|h| std::path::PathBuf::from(h).join(".config/anna/config.toml"))
            .unwrap_or_else(|_| std::path::PathBuf::from("~/.config/anna/config.toml"));

        return format!(r#"🧠  How to upgrade Anna's LLM on THIS machine:

📋  Step 1 - See what models you have:
    ollama list

📋  Step 2 - Pull a new/better model:
    ollama pull qwen2.5:14b   # Recommended: good reasoning
    ollama pull llama3.2:3b   # Fast: simple queries
    ollama pull mistral:7b    # Alternative: good general

📋  Step 3 - Update Anna's config:
    Edit: {}
    [llm]
    model = "qwen2.5:14b"

📋  Step 4 - Restart Anna:
    systemctl --user restart annad

⚠️  Note: Anna uses a LOCAL Ollama backend. She is not connected to any cloud service.
    The LLM runs entirely on your machine."#, config_path.display());
    }

    // v10.2.1: Dependency health - run actual health check
    if q.contains("dependencies") || q.contains("outdated") || q.contains("missing") {
        let health = anna_common::anna_self_health::check_anna_self_health();
        let mut answer = String::from("🔧  Anna's Toolchain Status:\n\n");

        answer.push_str("📊  Required tools:\n");
        let required_tools = ["systemctl", "journalctl", "ps", "df", "ip"];
        for tool in &required_tools {
            let is_missing = health.missing_deps.iter().any(|d| d.contains(tool));
            let status = if is_missing {
                "❌  MISSING"
            } else {
                "✅  present"
            };
            answer.push_str(&format!("    {} - {}\n", tool, status));
        }

        if health.deps_ok {
            answer.push_str("\n✅  All required tools are present.\n");
        } else {
            answer.push_str(&format!("\n⚠️  Missing tools: {}\n", health.missing_deps.join(", ")));
        }

        // LLM status
        answer.push_str(&format!("\n🧠  LLM Backend: {}\n", health.llm_details));

        answer.push_str("\n⚠️  Note: I do not have version comparison logic yet.\n");
        answer.push_str("    I can confirm tools are present and executable, but cannot check if they are outdated.");

        return answer;
    }

    if q.contains("version") {
        return "🤖  Anna Assistant v10.3.0 - Strict Evidence Discipline".to_string();
    }

    if q.contains("who are you") || q.contains("about anna") || q.contains("what are you") {
        return r#"👋  I am Anna, your local Arch Linux system assistant.

🧠  Architecture (v10.3.0 - Strict Evidence Discipline):
    • NO KNOWLEDGE WITHOUT EVIDENCE - every claim requires proof
    • I iterate until HIGH confidence (>=90%) or evidence is impossible
    • Citations reference evidence IDs like [E1], [E2]
    • Confidence labels: HIGH/MEDIUM/LOW/VERY_LOW

📊  Philosophy:
    • I NEVER guess or fabricate
    • No placeholders like "Model name [E2]" - those are failures
    • If evidence is missing, I request probes
    • SSH/TMUX sessions: I say "No graphical session detected"
    • I learn from THIS machine only

🔧  Components:
    • annad: Background daemon (telemetry, tools)
    • annactl: CLI interface (you're using this)
    • brain: LLM orchestration via local Ollama"#
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
        // Note: _telemetry is unused in handle_meta_query, but we need to pass something
        // Use crate::system_query to get real telemetry for testing
        let telemetry = crate::system_query::query_system_telemetry()
            .expect("Failed to query telemetry for test");

        let upgrade = handle_meta_query("upgrade brain", &telemetry);
        assert!(upgrade.contains("ollama"));

        let version = handle_meta_query("anna version", &telemetry);
        assert!(version.contains("v10.3.0"));

        let about = handle_meta_query("who are you", &telemetry);
        assert!(about.contains("Strict Evidence Discipline"));
    }

    #[test]
    fn test_no_llm_message() {
        let msg = format_no_llm_message();
        assert!(msg.contains("Ollama"));
        assert!(msg.contains("config.toml"));
    }
}
