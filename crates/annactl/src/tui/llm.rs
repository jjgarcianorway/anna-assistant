//! LLM Integration - Reply generation with template matching and streaming
//! Beta.149: Now uses unified_query_handler for consistency with CLI

use crate::recipe_formatter::format_recipe_answer;
use crate::system_query::query_system_telemetry;
use crate::tui_state::AnnaTuiState;
use crate::unified_query_handler::{handle_unified_query, AnswerConfidence, UnifiedQueryResult};
use anna_common::command_recipe::Recipe;
use anna_common::llm::{LlmClient, LlmConfig, LlmPrompt};
use anna_common::template_library::TemplateLibrary;
use std::collections::HashMap;
use tokio::sync::mpsc;

use super::event_loop::TuiMessage;

/// Generate reply using LLM for questions without template match
pub async fn generate_llm_reply(input: &str, state: &AnnaTuiState) -> String {
    // Build system context from telemetry
    let system_context = format!(
        "System Information:\n\
         - CPU: {}\n\
         - CPU Load: {:.2}, {:.2}, {:.2} (1/5/15 min)\n\
         - RAM: {:.1} GB used / {:.1} GB total\n\
         - GPU: {}\n\
         - Disk: {:.1} GB free\n\
         - OS: Arch Linux\n\
         - Anna Version: {}",
        state.system_panel.cpu_model,
        state.system_panel.cpu_load_1min,
        state.system_panel.cpu_load_5min,
        state.system_panel.cpu_load_15min,
        state.system_panel.ram_used_gb,
        state.system_panel.ram_total_gb,
        state.system_panel.gpu_name.as_deref().unwrap_or("None"),
        state.system_panel.disk_free_gb,
        state.system_panel.anna_version,
    );

    // Use detected model from state, fallback to llama3.1:8b if none detected
    let model_name = if state.llm_panel.model_name == "None"
        || state.llm_panel.model_name == "Unknown"
        || state.llm_panel.model_name == "Ollama N/A"
    {
        "llama3.1:8b"
    } else {
        &state.llm_panel.model_name
    };

    let llm_config = LlmConfig::local("http://127.0.0.1:11434/v1", model_name);

    let llm_client = match LlmClient::from_config(&llm_config) {
        Ok(client) => client,
        Err(_) => {
            // LLM not available - return helpful fallback
            return format!(
                "## ⚠ LLM Unavailable\n\n\
                 I couldn't connect to the local LLM server (Ollama).\n\n\
                 **Your question:** {}\n\n\
                 **What I can help with (using templates):**\n\
                 - swap - Check swap status\n\
                 - GPU/VRAM - Check GPU memory\n\
                 - kernel - Check kernel version\n\
                 - disk/space - Check disk space\n\
                 - RAM/memory - Check system memory\n\n\
                 **To enable full LLM responses:**\n\
                 1. Install Ollama: `curl -fsSL https://ollama.com/install.sh | sh`\n\
                 2. Pull a model: `ollama pull llama3.1:8b`\n\
                 3. Ensure Ollama is running: `ollama list`",
                input
            );
        }
    };

    // Build prompt with system context
    let system_prompt = format!("{}\n\n{}", LlmClient::anna_system_prompt(), system_context);

    let prompt = LlmPrompt {
        system: system_prompt,
        user: input.to_string(),
        conversation_history: None,
    };

    // Beta.111: Word-by-word streaming for consistency with one-shot and REPL modes
    // TUI accumulates chunks into a string buffer (can't print to stdout like REPL)
    let mut full_response = String::new();
    let mut callback = |chunk: &str| {
        full_response.push_str(chunk);
    };

    match llm_client.chat_stream(&prompt, &mut callback) {
        Ok(_) => full_response,
        Err(e) => format!("## LLM Error\n\nFailed to get response: {:?}", e),
    }
}

/// Generate reply using template library and recipe formatter
pub async fn generate_reply(input: &str, state: &AnnaTuiState) -> String {
    let library = TemplateLibrary::default();
    let input_lower = input.to_lowercase();

    // Beta.108: Helper function for word-boundary keyword matching
    // Prevents false positives like "programming" matching "ram"
    let contains_word = |text: &str, keyword: &str| {
        text.split(|c: char| !c.is_alphanumeric())
            .any(|word| word == keyword)
    };

    // Pattern matching for template selection (Beta.112: MASSIVELY expanded - 68 templates)
    let (template_id, params) = if contains_word(&input_lower, "swap") {
        ("check_swap_status", HashMap::new())
    } else if contains_word(&input_lower, "gpu") || contains_word(&input_lower, "vram") {
        ("check_gpu_memory", HashMap::new())
    } else if input_lower.contains("wifi")
        || input_lower.contains("wireless")
        || (input_lower.contains("network")
            && (input_lower.contains("slow")
                || input_lower.contains("issue")
                || input_lower.contains("problem")))
    {
        ("wifi_diagnostics", HashMap::new())
    } else if contains_word(&input_lower, "kernel")
        && (input_lower.contains("version")
            || input_lower.contains("what")
            || input_lower.contains("running"))
    {
        ("check_kernel_version", HashMap::new())
    } else if input_lower.contains("disk") && input_lower.contains("space") {
        ("check_disk_space", HashMap::new())
    } else if (contains_word(&input_lower, "memory") || contains_word(&input_lower, "ram"))
        && !input_lower.contains("gpu")
    {
        ("check_memory", HashMap::new())
    } else if contains_word(&input_lower, "uptime") {
        ("check_uptime", HashMap::new())
    } else if contains_word(&input_lower, "cpu")
        && (input_lower.contains("model")
            || input_lower.contains("what")
            || input_lower.contains("processor"))
    {
        ("check_cpu_model", HashMap::new())
    } else if contains_word(&input_lower, "cpu")
        && (input_lower.contains("load") || input_lower.contains("usage"))
    {
        ("check_cpu_load", HashMap::new())
    } else if contains_word(&input_lower, "distro")
        || (input_lower.contains("arch") && input_lower.contains("version"))
    {
        ("check_distro", HashMap::new())
    } else if input_lower.contains("failed") && input_lower.contains("service") {
        ("check_failed_services", HashMap::new())
    } else if input_lower.contains("journal") && input_lower.contains("error") {
        ("check_journal_errors", HashMap::new())
    } else if input_lower.contains("weak")
        || (input_lower.contains("diagnostic") && input_lower.contains("system"))
    {
        ("system_weak_points_diagnostic", HashMap::new())
    // Beta.112: PACKAGE MANAGEMENT (13 new templates)
    } else if input_lower.contains("orphan")
        || (input_lower.contains("unused") && input_lower.contains("package"))
    {
        ("list_orphaned_packages", HashMap::new())
    } else if input_lower.contains("aur") {
        ("list_aur_packages", HashMap::new())
    } else if input_lower.contains("pacman")
        && (input_lower.contains("cache") || input_lower.contains("size"))
    {
        ("check_pacman_cache_size", HashMap::new())
    } else if input_lower.contains("mirror") {
        ("check_pacman_mirrors", HashMap::new())
    } else if (input_lower.contains("update") || input_lower.contains("upgrade"))
        && !input_lower.contains("brain")
    {
        ("check_package_updates", HashMap::new())
    } else if input_lower.contains("keyring") {
        ("check_archlinux_keyring", HashMap::new())
    } else if input_lower.contains("package") && input_lower.contains("integrity") {
        ("check_package_integrity", HashMap::new())
    } else if input_lower.contains("clean") && input_lower.contains("cache") {
        ("clean_package_cache", HashMap::new())
    } else if input_lower.contains("explicit") && input_lower.contains("package") {
        ("list_explicit_packages", HashMap::new())
    } else if input_lower.contains("pacman")
        && (input_lower.contains("status") || input_lower.contains("lock"))
    {
        ("check_pacman_status", HashMap::new())
    } else if input_lower.contains("dependency") && input_lower.contains("conflict") {
        ("check_dependency_conflicts", HashMap::new())
    } else if input_lower.contains("pending") && input_lower.contains("update") {
        ("check_pending_updates", HashMap::new())
    } else if input_lower.contains("recent") && input_lower.contains("pacman") {
        ("show_recent_pacman_operations", HashMap::new())
    // Beta.112: BOOT & SYSTEMD (8 new templates)
    } else if input_lower.contains("boot")
        && (input_lower.contains("time") || input_lower.contains("slow"))
    {
        ("analyze_boot_time", HashMap::new())
    } else if input_lower.contains("boot") && input_lower.contains("error") {
        ("check_boot_errors", HashMap::new())
    } else if input_lower.contains("boot") && input_lower.contains("log") {
        ("show_boot_log", HashMap::new())
    } else if input_lower.contains("systemd") && input_lower.contains("timer") {
        ("check_systemd_timers", HashMap::new())
    } else if input_lower.contains("journal") && input_lower.contains("size") {
        ("analyze_journal_size", HashMap::new())
    } else if input_lower.contains("recent") && input_lower.contains("error") {
        ("show_recent_journal_errors", HashMap::new())
    } else if input_lower.contains("kernel") && input_lower.contains("update") {
        ("check_recent_kernel_updates", HashMap::new())
    } else if input_lower.contains("systemd") && input_lower.contains("version") {
        ("check_systemd_version", HashMap::new())
    // Beta.112: CPU & PERFORMANCE (8 new templates)
    } else if input_lower.contains("cpu")
        && (input_lower.contains("freq") || input_lower.contains("speed"))
    {
        ("check_cpu_frequency", HashMap::new())
    } else if input_lower.contains("cpu") && input_lower.contains("governor") {
        ("check_cpu_governor", HashMap::new())
    } else if input_lower.contains("cpu") && input_lower.contains("usage") {
        ("analyze_cpu_usage", HashMap::new())
    } else if (input_lower.contains("cpu") || input_lower.contains("temperature"))
        && input_lower.contains("temp")
    {
        ("check_cpu_temperature", HashMap::new())
    } else if input_lower.contains("throttl") {
        ("detect_cpu_throttling", HashMap::new())
    } else if input_lower.contains("top") && input_lower.contains("cpu") {
        ("show_top_cpu_processes", HashMap::new())
    } else if input_lower.contains("load") && input_lower.contains("average") {
        ("check_load_average", HashMap::new())
    } else if input_lower.contains("context") && input_lower.contains("switch") {
        ("analyze_context_switches", HashMap::new())
    // Beta.112: MEMORY (6 new templates)
    } else if input_lower.contains("memory") && input_lower.contains("usage") {
        ("check_memory_usage", HashMap::new())
    } else if input_lower.contains("swap") && input_lower.contains("usage") {
        ("check_swap_usage", HashMap::new())
    } else if input_lower.contains("memory") && input_lower.contains("pressure") {
        ("analyze_memory_pressure", HashMap::new())
    } else if input_lower.contains("top") && input_lower.contains("memory") {
        ("show_top_memory_processes", HashMap::new())
    } else if input_lower.contains("oom") {
        ("check_oom_killer", HashMap::new())
    } else if input_lower.contains("huge") && input_lower.contains("page") {
        ("check_huge_pages", HashMap::new())
    // Beta.112: NETWORK (7 new templates)
    } else if input_lower.contains("dns") {
        ("check_dns_resolution", HashMap::new())
    } else if input_lower.contains("network") && input_lower.contains("interface") {
        ("check_network_interfaces", HashMap::new())
    } else if input_lower.contains("routing") || input_lower.contains("route") {
        ("check_routing_table", HashMap::new())
    } else if input_lower.contains("firewall") {
        ("check_firewall_rules", HashMap::new())
    } else if input_lower.contains("port") && input_lower.contains("listen") {
        ("check_listening_ports", HashMap::new())
    } else if input_lower.contains("latency") || input_lower.contains("ping") {
        ("check_network_latency", HashMap::new())
    } else if input_lower.contains("networkmanager") {
        ("check_networkmanager_status", HashMap::new())
    // Beta.112: GPU & DISPLAY (9 new templates)
    } else if input_lower.contains("nvidia") && !input_lower.contains("install") {
        ("check_nvidia_status", HashMap::new())
    } else if input_lower.contains("amd")
        && (input_lower.contains("gpu") || input_lower.contains("radeon"))
    {
        ("check_amd_gpu", HashMap::new())
    } else if input_lower.contains("gpu") && input_lower.contains("driver") {
        ("check_gpu_drivers", HashMap::new())
    } else if input_lower.contains("gpu") && input_lower.contains("process") {
        ("check_gpu_processes", HashMap::new())
    } else if input_lower.contains("gpu") && input_lower.contains("temp") {
        ("check_gpu_temperature", HashMap::new())
    } else if input_lower.contains("display")
        || input_lower.contains("xorg")
        || input_lower.contains("wayland")
    {
        ("check_display_server", HashMap::new())
    } else if input_lower.contains("desktop") && input_lower.contains("environment") {
        ("check_desktop_environment", HashMap::new())
    } else if input_lower.contains("xorg") && input_lower.contains("error") {
        ("analyze_xorg_errors", HashMap::new())
    } else if input_lower.contains("wayland") && input_lower.contains("compositor") {
        ("check_wayland_compositor", HashMap::new())
    // Beta.112: HARDWARE (4 new templates)
    } else if input_lower.contains("temperature")
        || input_lower.contains("temp")
        || input_lower.contains("heat")
    {
        ("check_temperature", HashMap::new())
    } else if input_lower.contains("usb") {
        ("check_usb_devices", HashMap::new())
    } else if input_lower.contains("pci") {
        ("check_pci_devices", HashMap::new())
    } else if input_lower.contains("hostname") {
        ("check_hostname", HashMap::new())
    } else {
        // No matching template - use LLM to generate response
        return generate_llm_reply(input, state).await;
    };

    // Instantiate template
    match library.get(template_id) {
        Some(template) => match template.instantiate(&params) {
            Ok(recipe_step) => {
                // Wrap in full recipe structure
                let recipe = Recipe {
                    question: input.to_string(),
                    steps: vec![recipe_step.clone()],
                    overall_safety: recipe_step.safety_level,
                    all_read_only: true,
                    wiki_sources: recipe_step.doc_sources.clone(),
                    summary: recipe_step.explanation.clone(),
                    generated_by: Some("template_library".to_string()),
                    critic_approval: None,
                };

                // Format with recipe formatter
                format_recipe_answer(&recipe, input)
            }
            Err(e) => format!("## Error\n\nFailed to instantiate template: {}", e),
        },
        None => format!("## Error\n\nTemplate '{}' not found", template_id),
    }
}

/// Beta.149: Generate reply using unified query handler
/// This ensures TUI and CLI get IDENTICAL responses
pub async fn generate_reply_streaming(
    input: &str,
    state: &AnnaTuiState,
    tx: mpsc::Sender<TuiMessage>,
) -> String {
    // Get telemetry
    let telemetry = match query_system_telemetry() {
        Ok(t) => t,
        Err(e) => {
            return format!("⚠ Error reading system telemetry: {}", e);
        }
    };

    // Get LLM config from state
    let model_name = if state.llm_panel.model_name == "None"
        || state.llm_panel.model_name == "Unknown"
        || state.llm_panel.model_name == "Ollama N/A"
    {
        "llama3.1:8b"
    } else {
        &state.llm_panel.model_name
    };
    let llm_config = LlmConfig::local("http://127.0.0.1:11434/v1", model_name);

    // Use unified query handler
    match handle_unified_query(input, &telemetry, &llm_config).await {
        Ok(UnifiedQueryResult::DeterministicRecipe {
            recipe_name,
            action_plan,
        }) => {
            // Format recipe output for TUI
            format!(
                "**🎯 Using deterministic recipe: {}**\n\n\
                 ## Analysis\n{}\n\n\
                 ## Goals\n{}\n\n\
                 ## Commands\n{}\n\n\
                 ## Notes\n{}",
                recipe_name,
                action_plan.analysis,
                action_plan
                    .goals
                    .iter()
                    .enumerate()
                    .map(|(i, g)| format!("{}. {}", i + 1, g))
                    .collect::<Vec<_>>()
                    .join("\n"),
                action_plan
                    .command_plan
                    .iter()
                    .enumerate()
                    .map(|(i, cmd)| format!(
                        "{}. {} [Risk: {:?}]\n   $ {}",
                        i + 1,
                        cmd.description,
                        cmd.risk_level,
                        cmd.command
                    ))
                    .collect::<Vec<_>>()
                    .join("\n"),
                action_plan.notes_for_user
            )
        }
        Ok(UnifiedQueryResult::Template {
            command, output, ..
        }) => {
            // Format template output for TUI
            format!("**Running:** `{}`\n\n```\n{}\n```", command, output)
        }
        Ok(UnifiedQueryResult::ActionPlan {
            action_plan,
            raw_json: _,
        }) => {
            // Format action plan for TUI
            format!(
                "## Analysis\n{}\n\n\
                 ## Goals\n{}\n\n\
                 ## Commands\n{}\n\n\
                 ## Notes\n{}",
                action_plan.analysis,
                action_plan
                    .goals
                    .iter()
                    .enumerate()
                    .map(|(i, g)| format!("{}. {}", i + 1, g))
                    .collect::<Vec<_>>()
                    .join("\n"),
                action_plan
                    .command_plan
                    .iter()
                    .enumerate()
                    .map(|(i, cmd)| format!(
                        "{}. {} [Risk: {:?}]\n   $ {}",
                        i + 1,
                        cmd.description,
                        cmd.risk_level,
                        cmd.command
                    ))
                    .collect::<Vec<_>>()
                    .join("\n"),
                action_plan.notes_for_user
            )
        }
        Ok(UnifiedQueryResult::ConversationalAnswer {
            answer,
            confidence,
            sources,
        }) => {
            // Format conversational answer for TUI
            let confidence_str = match confidence {
                AnswerConfidence::High => "✅ High",
                AnswerConfidence::Medium => "🟡 Medium",
                AnswerConfidence::Low => "⚠️  Low",
            };

            format!(
                "{}\n\n---\n*Confidence: {} | Sources: {}*",
                answer,
                confidence_str,
                sources.join(", ")
            )
        }
        Err(e) => {
            format!("⚠ Query failed: {}", e)
        }
    }
}

/// Beta.115: Generate LLM reply with streaming chunks sent via channel
pub async fn generate_llm_reply_streaming(
    input: &str,
    state: &AnnaTuiState,
    tx: mpsc::Sender<TuiMessage>,
) {
    // Build system context (same as generate_llm_reply)
    let system_context = format!(
        "System Information:\n\
         - CPU: {}\n\
         - CPU Load: {:.2}, {:.2}, {:.2} (1/5/15 min)\n\
         - RAM: {:.1} GB used / {:.1} GB total\n\
         - GPU: {}\n\
         - Disk: {:.1} GB free\n\
         - OS: Arch Linux\n\
         - Anna Version: {}",
        state.system_panel.cpu_model,
        state.system_panel.cpu_load_1min,
        state.system_panel.cpu_load_5min,
        state.system_panel.cpu_load_15min,
        state.system_panel.ram_used_gb,
        state.system_panel.ram_total_gb,
        state
            .system_panel
            .gpu_name
            .as_ref()
            .unwrap_or(&"None".to_string()),
        state.system_panel.disk_free_gb,
        state.system_panel.anna_version
    );

    // Detect model
    let model_name = if state.llm_panel.model_name == "None"
        || state.llm_panel.model_name == "Unknown"
        || state.llm_panel.model_name == "Ollama N/A"
    {
        "llama3.1:8b"
    } else {
        &state.llm_panel.model_name
    };

    let llm_config = LlmConfig::local("http://127.0.0.1:11434/v1", model_name);

    let llm_client = match LlmClient::from_config(&llm_config) {
        Ok(client) => client,
        Err(_) => {
            // LLM not available
            let _ = tx
                .send(TuiMessage::AnnaReply(
                    "## ⚠ LLM Unavailable\n\nI couldn't connect to the local LLM server (Ollama)."
                        .to_string(),
                ))
                .await;
            return;
        }
    };

    // Build prompt
    let system_prompt = format!("{}\n\n{}", LlmClient::anna_system_prompt(), system_context);

    let prompt = LlmPrompt {
        system: system_prompt,
        user: input.to_string(),
        conversation_history: None,
    };

    // Beta.115: Streaming callback - send each chunk via channel
    let tx_clone = tx.clone();
    let mut callback = move |chunk: &str| {
        let chunk_string = chunk.to_string();
        let tx_inner = tx_clone.clone();
        // Send chunk asynchronously
        tokio::spawn(async move {
            let _ = tx_inner
                .send(TuiMessage::AnnaReplyChunk(chunk_string))
                .await;
        });
    };

    match llm_client.chat_stream(&prompt, &mut callback) {
        Ok(_) => {
            // Streaming complete
            let _ = tx.send(TuiMessage::AnnaReplyComplete).await;
        }
        Err(_) => {
            let _ = tx
                .send(TuiMessage::AnnaReply(
                    "## LLM Error\n\nFailed to get response.".to_string(),
                ))
                .await;
        }
    }
}

/// Beta.149: Generate LLM reply with pre-built prompt (for unified handler)
async fn generate_llm_reply_streaming_with_prompt(
    prompt: &str,
    llm_config: &LlmConfig,
    tx: mpsc::Sender<TuiMessage>,
) {
    let llm_client = match LlmClient::from_config(llm_config) {
        Ok(client) => client,
        Err(_) => {
            // LLM not available
            let _ = tx
                .send(TuiMessage::AnnaReply(
                    "## ⚠ LLM Unavailable\n\nI couldn't connect to the local LLM server (Ollama)."
                        .to_string(),
                ))
                .await;
            return;
        }
    };

    // Build LLM prompt with unified handler's pre-built prompt
    let llm_prompt = LlmPrompt {
        system: LlmClient::anna_system_prompt().to_string(),
        user: prompt.to_string(),
        conversation_history: None,
    };

    // Streaming callback - send each chunk via channel
    let tx_clone = tx.clone();
    let mut callback = move |chunk: &str| {
        let chunk_string = chunk.to_string();
        let tx_inner = tx_clone.clone();
        // Send chunk asynchronously
        tokio::spawn(async move {
            let _ = tx_inner
                .send(TuiMessage::AnnaReplyChunk(chunk_string))
                .await;
        });
    };

    match llm_client.chat_stream(&llm_prompt, &mut callback) {
        Ok(_) => {
            // Streaming complete
            let _ = tx.send(TuiMessage::AnnaReplyComplete).await;
        }
        Err(_) => {
            let _ = tx
                .send(TuiMessage::AnnaReply(
                    "## LLM Error\n\nFailed to get response.".to_string(),
                ))
                .await;
        }
    }
}
