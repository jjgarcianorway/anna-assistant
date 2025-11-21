//! Unified Query Handler - Version 149
//!
//! Single source of truth for ALL query processing (CLI and TUI).
//! Fixes inconsistent responses between modes.
//!
//! Architecture (in order):
//! 1. Beta.151 Deterministic Recipes (instant, zero hallucination)
//! 2. Template Matching (instant shell commands)
//! 3. V3 JSON Dialogue (structured action plans)
//! 4. Streaming LLM (conversational fallback)

use anna_common::action_plan_v3::ActionPlan;
use anna_common::llm::LlmConfig;
use anna_common::telemetry::SystemTelemetry;
use anyhow::Result;

use crate::dialogue_v3_json;
use crate::query_handler;

/// Unified query result - all query types in one enum
/// Version 150: ALL responses are now structured JSON - no freeform streaming
#[derive(Debug)]
pub enum UnifiedQueryResult {
    /// Beta.151 deterministic recipe matched
    DeterministicRecipe {
        recipe_name: String,
        action_plan: ActionPlan,
    },
    /// Template matched - instant result
    Template {
        command: String,
        output: String,
    },
    /// V3 JSON dialogue - structured action plan
    ActionPlan {
        action_plan: ActionPlan,
        raw_json: String,
    },
    /// Conversational answer - structured response for info queries
    /// Version 150: Replaces freeform streaming with structured JSON
    ConversationalAnswer {
        answer: String,
        confidence: AnswerConfidence,
        sources: Vec<String>,
    },
}

/// Answer confidence level
#[derive(Debug, Clone, Copy)]
pub enum AnswerConfidence {
    High,    // From telemetry/system data
    Medium,  // From LLM with validation
    Low,     // From LLM without validation
}

/// Unified query handler - single entry point for all queries
///
/// This ensures CLI and TUI get IDENTICAL responses for the same question.
pub async fn handle_unified_query(
    user_text: &str,
    telemetry: &SystemTelemetry,
    llm_config: &LlmConfig,
) -> Result<UnifiedQueryResult> {
    // TIER 0: System Report (Version 150 - highest priority, no LLM)
    // Intercept "full report" queries to prevent hallucination
    if crate::system_report::is_system_report_query(user_text) {
        let report = crate::system_report::generate_full_report()
            .unwrap_or_else(|e| format!("Error generating system report: {}", e));

        return Ok(UnifiedQueryResult::ConversationalAnswer {
            answer: report,
            confidence: AnswerConfidence::High,
            sources: vec!["verified system telemetry".to_string()],
        });
    }

    // TIER 1: Beta.151 Deterministic Recipes (NEW - highest priority)
    // These are hard-coded, tested, zero-hallucination plans
    let telemetry_map = telemetry_to_hashmap(telemetry);
    if let Some(recipe_result) = crate::recipes::try_recipe_match(user_text, &telemetry_map) {
        let action_plan = recipe_result.map_err(|e| anyhow::anyhow!("Recipe build_plan failed: {}", e))?;

        // Validate recipe output
        action_plan
            .validate()
            .map_err(|e| anyhow::anyhow!("Recipe generated invalid ActionPlan: {}", e))?;

        let recipe_name = action_plan
            .meta
            .template_used
            .clone()
            .unwrap_or_else(|| "unknown_recipe".to_string());

        return Ok(UnifiedQueryResult::DeterministicRecipe {
            recipe_name,
            action_plan,
        });
    }

    // TIER 2: Template Matching (fast, accurate for simple queries)
    if let Some((template_id, params)) = query_handler::try_template_match(user_text) {
        match query_handler::execute_template(template_id, &params) {
            Ok(query_handler::QueryResult::Template {
                template_id: _,
                command,
                output,
            }) => {
                return Ok(UnifiedQueryResult::Template {
                    command,
                    output,
                });
            }
            _ => {
                // Template execution failed, continue to next tier
            }
        }
    }

    // TIER 3: V3 JSON Dialogue (structured action plans with LLM)
    // Version 150: RE-ENABLED - V3 dialogue now works with SystemTelemetry
    // Check if this looks like an actionable request
    if should_use_action_plan(user_text) {
        match dialogue_v3_json::run_dialogue_v3_json(user_text, telemetry, llm_config).await {
            Ok(result) => {
                return Ok(UnifiedQueryResult::ActionPlan {
                    action_plan: result.action_plan,
                    raw_json: result.raw_json,
                });
            }
            Err(e) => {
                // V3 dialogue failed, fall through to conversational answer
                eprintln!("V3 dialogue error (falling back to conversational): {}", e);
            }
        }
    }

    // TIER 4: Structured Conversational Answer (Version 150: NO STREAMING!)
    // Generate structured answer from telemetry or LLM
    generate_conversational_answer(user_text, telemetry, llm_config).await
}

/// Generate structured conversational answer (Version 150: enforces JSON)
async fn generate_conversational_answer(
    user_text: &str,
    telemetry: &SystemTelemetry,
    llm_config: &LlmConfig,
) -> Result<UnifiedQueryResult> {
    use anna_common::llm::{LlmClient, LlmPrompt};

    // Try to answer directly from telemetry first (highest confidence)
    if let Some(answer) = try_answer_from_telemetry(user_text, telemetry) {
        return Ok(UnifiedQueryResult::ConversationalAnswer {
            answer,
            confidence: AnswerConfidence::High,
            sources: vec!["system telemetry".to_string()],
        });
    }

    // Use LLM for complex conversational queries
    let client = LlmClient::from_config(llm_config)
        .map_err(|e| anyhow::anyhow!("LLM not available: {}", e))?;

    let prompt = build_conversational_prompt(user_text, telemetry);
    let llm_prompt = LlmPrompt {
        system: LlmClient::anna_system_prompt().to_string(),
        user: prompt,
        conversation_history: None,
    };

    // Get LLM response (blocking, not streaming!)
    let response = client
        .chat(&llm_prompt)
        .map_err(|e| anyhow::anyhow!("LLM query failed: {}", e))?;

    Ok(UnifiedQueryResult::ConversationalAnswer {
        answer: response.text,
        confidence: AnswerConfidence::Medium,
        sources: vec!["LLM".to_string()],
    })
}

/// Try to answer question directly from telemetry (zero-latency, high confidence)
fn try_answer_from_telemetry(user_text: &str, telemetry: &SystemTelemetry) -> Option<String> {
    let query_lower = user_text.to_lowercase();

    // Anna's personality/design philosophy queries
    if query_lower.contains("your") && query_lower.contains("personality") {
        return Some(
            "I'm Anna, your Arch Linux assistant. My design philosophy:\n\n\
            • **Telemetry-first**: I gather real system data before answering\n\
            • **Deterministic recipes**: 77 zero-hallucination command sequences\n\
            • **Safety-conscious**: User confirmation for write operations\n\
            • **Transparent**: I explain what I'm doing and why\n\
            • **Local-first**: I run locally with optional LLM support\n\
            • **Minimalist**: Three-command simplicity (TUI, status, query)\n\n\
            My goal is reliable, explainable system management with zero surprises.".to_string()
        );
    }

    // User profile/usage pattern queries - use Context Engine data
    if (query_lower.contains("describe me") || query_lower.contains("what kind of user")
        || query_lower.contains("my personality"))
        && (query_lower.contains("trait") || query_lower.contains("am i")
            || query_lower.contains("as a user") || query_lower.contains("my usage")) {
        // Load context engine for usage patterns
        if let Ok(ctx) = crate::context_engine::ContextEngine::load() {
            let total_commands = ctx.usage_patterns.command_frequency.values().sum::<u32>();
            let top_commands: Vec<_> = {
                let mut commands: Vec<_> = ctx.usage_patterns.command_frequency.iter().collect();
                commands.sort_by(|a, b| b.1.cmp(a.1));
                commands.into_iter().take(3).collect()
            };

            let profile = if total_commands < 10 {
                "New user - still exploring Anna's capabilities"
            } else if total_commands < 50 {
                "Regular user - building familiarity with system administration"
            } else {
                "Power user - confident with system management"
            };

            let mut response = format!("Based on your usage patterns: {}\n\n", profile);
            if !top_commands.is_empty() {
                response.push_str("Most frequent commands:\n");
                for (cmd, count) in top_commands {
                    response.push_str(&format!("  • {} ({} times)\n", cmd, count));
                }
            }
            response.push_str(&format!("\nTotal interactions: {}", total_commands));

            return Some(response);
        }
    }

    // CPU queries
    if query_lower.contains("cpu") && (query_lower.contains("what") || query_lower.contains("model")) {
        return Some(format!(
            "Your CPU is a {} with {} cores. Current load: {:.2} (1-min avg).",
            telemetry.hardware.cpu_model,
            telemetry.cpu.cores,
            telemetry.cpu.load_avg_1min
        ));
    }

    // RAM queries
    if (query_lower.contains("ram") || query_lower.contains("memory"))
        && (query_lower.contains("how much") || query_lower.contains("total")) {
        let used_gb = telemetry.memory.used_mb as f64 / 1024.0;
        let total_gb = telemetry.memory.total_mb as f64 / 1024.0;
        let percent = (used_gb / total_gb) * 100.0;
        return Some(format!(
            "You have {:.1} GB of RAM total. Currently using {:.1} GB ({:.1}%).",
            total_gb, used_gb, percent
        ));
    }

    // Disk space queries
    if (query_lower.contains("disk") || query_lower.contains("storage") || query_lower.contains("space"))
        && (query_lower.contains("free") || query_lower.contains("available") || query_lower.contains("how much")) {
        if let Some(root_disk) = telemetry.disks.iter().find(|d| d.mount_point == "/") {
            let free_gb = (root_disk.total_mb - root_disk.used_mb) as f64 / 1024.0;
            let total_gb = root_disk.total_mb as f64 / 1024.0;
            return Some(format!(
                "Your root partition (/) has {:.1} GB free out of {:.1} GB total ({:.1}% used).",
                free_gb, total_gb, root_disk.usage_percent
            ));
        }
    }

    // GPU queries
    if query_lower.contains("gpu") {
        if let Some(ref gpu) = telemetry.hardware.gpu_info {
            return Some(format!("Your GPU is: {}", gpu));
        } else {
            return Some("No GPU detected on this system.".to_string());
        }
    }

    // Failed services
    if query_lower.contains("failed") && query_lower.contains("service") {
        if telemetry.services.failed_units.is_empty() {
            return Some("✅ No failed services detected.".to_string());
        } else {
            let failed_list: Vec<String> = telemetry.services.failed_units
                .iter()
                .map(|u| format!("- {}", u.name))
                .collect();
            return Some(format!(
                "❌ {} failed service(s):\n{}",
                telemetry.services.failed_units.len(),
                failed_list.join("\n")
            ));
        }
    }

    None
}

/// Determine if query should use ActionPlan mode
fn should_use_action_plan(user_text: &str) -> bool {
    let input_lower = user_text.to_lowercase();

    // Action keywords
    let action_keywords = [
        "install",
        "setup",
        "configure",
        "fix",
        "repair",
        "update",
        "upgrade",
        "enable",
        "disable",
        "start",
        "stop",
        "restart",
        "create",
        "delete",
        "remove",
        "change",
        "modify",
    ];

    action_keywords
        .iter()
        .any(|keyword| input_lower.contains(keyword))
}

/// Build conversational prompt for streaming LLM
fn build_conversational_prompt(user_text: &str, telemetry: &SystemTelemetry) -> String {
    let hostname = std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "localhost".to_string());

    format!(
        "You are Anna, an Arch Linux system administrator assistant.\n\n\
         System Context:\n\
         - CPU: {}\n\
         - RAM: {:.1} GB used / {:.1} GB total\n\
         - Hostname: {}\n\n\
         User Question: {}\n\n\
         Provide a helpful, concise answer. If you don't know, say so.",
        telemetry.hardware.cpu_model,
        telemetry.memory.used_mb as f64 / 1024.0,
        telemetry.memory.total_mb as f64 / 1024.0,
        hostname,
        user_text
    )
}

/// Convert SystemTelemetry to HashMap for recipes
fn telemetry_to_hashmap(telemetry: &SystemTelemetry) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();

    // Hardware
    map.insert("cpu_model".to_string(), telemetry.hardware.cpu_model.clone());
    map.insert("cpu_cores".to_string(), telemetry.cpu.cores.to_string());
    map.insert("total_ram_gb".to_string(), (telemetry.hardware.total_ram_mb as f64 / 1024.0).to_string());
    if let Some(ref gpu) = telemetry.hardware.gpu_info {
        map.insert("gpu_model".to_string(), gpu.clone());
    }

    // System (hostname and kernel from system calls)
    map.insert(
        "hostname".to_string(),
        std::process::Command::new("hostname")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "localhost".to_string()),
    );
    map.insert(
        "kernel".to_string(),
        std::process::Command::new("uname")
            .arg("-r")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string()),
    );
    map.insert("user".to_string(), std::env::var("USER").unwrap_or_else(|_| "user".to_string()));
    map.insert("home".to_string(), std::env::var("HOME").unwrap_or_else(|_| "~".to_string()));

    // Disk space (iterate directly on Vec, calculate available_gb)
    if let Some(root_disk) = telemetry.disks.iter().find(|d| d.mount_point == "/") {
        let available_gb = (root_disk.total_mb - root_disk.used_mb) as f64 / 1024.0;
        map.insert("disk_free_gb".to_string(), available_gb.to_string());
        map.insert("disk_used_percent".to_string(), root_disk.usage_percent.to_string());
    }

    // Internet connectivity
    map.insert("internet_connected".to_string(), telemetry.network.is_connected.to_string());

    // Desktop environment (use correct field names)
    if let Some(ref desktop) = telemetry.desktop {
        if let Some(ref de) = desktop.de_name {
            map.insert("desktop_environment".to_string(), de.clone());
        }
        if let Some(ref wm) = desktop.wm_name {
            map.insert("window_manager".to_string(), wm.clone());
        }
        if let Some(ref display) = desktop.display_server {
            map.insert("display_protocol".to_string(), display.clone());
        }
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_use_action_plan() {
        assert!(should_use_action_plan("install docker"));
        assert!(should_use_action_plan("fix broken packages"));
        assert!(should_use_action_plan("setup neovim"));

        assert!(!should_use_action_plan("what is my CPU?"));
        assert!(!should_use_action_plan("how much RAM do I have?"));
    }
}
