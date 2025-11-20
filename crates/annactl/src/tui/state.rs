//! State management - System telemetry updates and welcome message

use crate::tui_state::AnnaTuiState;

/// Beta.94: Show proactive welcome message with system info
pub fn show_welcome_message(state: &mut AnnaTuiState) {
    use std::env;

    let username = env::var("USER").unwrap_or_else(|_| "friend".to_string());

    // Gather system highlights
    let cpu_status = if state.system_panel.cpu_load_1min < 50.0 {
        "✅ running smoothly"
    } else if state.system_panel.cpu_load_1min < 80.0 {
        "⚠️ moderate load"
    } else {
        "🔥 high load"
    };

    let ram_status = if state.system_panel.ram_used_gb < state.system_panel.ram_total_gb * 0.7 {
        "✅ plenty available"
    } else if state.system_panel.ram_used_gb < state.system_panel.ram_total_gb * 0.9 {
        "⚠️ getting full"
    } else {
        "🔴 critically low"
    };

    let llm_status = if state.llm_panel.available {
        format!("✅ {} ready", state.llm_panel.model_name)
    } else {
        "⚠️ LLM not available (install Ollama)".to_string()
    };

    // Build beautiful welcome message
    let welcome = format!(
        "👋 **Hello {}!** Welcome to Anna v{}\n\n\
         Here's what I can tell you right now:\n\n\
         🖥️  **System Status:**\n\
         • CPU: {} ({:.0}% load)\n\
         • RAM: {:.1}GB / {:.1}GB used ({})\n\
         • Disk: {:.1}GB free\n\
         {}\n\n\
         🤖 **AI Assistant:**\n\
         • {}\n\n\
         💡 **Quick Actions:**\n\
         • Ask about system health: \"how is my system?\"\n\
         • Check resources: \"how much RAM do I have?\"\n\
         • Monitor services: \"show failed services\"\n\
         • Get help: Press F1\n\n\
         **What would you like to know or do?**",
        username,
        state.system_panel.anna_version,
        state.system_panel.cpu_model,
        state.system_panel.cpu_load_1min,
        state.system_panel.ram_used_gb,
        state.system_panel.ram_total_gb,
        ram_status,
        if state.system_panel.gpu_name.is_some() {
            format!(
                "• GPU: {}\n",
                state.system_panel.gpu_name.as_ref().unwrap()
            )
        } else {
            String::new()
        },
        llm_status,
        cpu_status
    );

    state.add_anna_reply(welcome);
}

/// Update telemetry data in state
pub fn update_telemetry(state: &mut AnnaTuiState) {
    use crate::system_query::query_system_telemetry;

    // Beta.91: Collect real system telemetry
    if let Ok(telemetry) = query_system_telemetry() {
        // Update system panel
        state.system_panel.cpu_model = telemetry.hardware.cpu_model.clone();
        state.system_panel.cpu_load_1min = telemetry.cpu.load_avg_1min;
        state.system_panel.cpu_load_5min = telemetry.cpu.load_avg_5min;
        state.system_panel.cpu_load_15min = 0.0; // Not collected by query_cpu yet
        state.system_panel.ram_total_gb = telemetry.memory.total_mb as f64 / 1024.0;
        state.system_panel.ram_used_gb = telemetry.memory.used_mb as f64 / 1024.0;
        state.system_panel.anna_version = env!("CARGO_PKG_VERSION").to_string();

        // Update GPU info if available
        state.system_panel.gpu_name = telemetry.hardware.gpu_info.clone();
        // GPU VRAM would need nvidia-smi or similar

        state.telemetry_ok = true;
    } else {
        state.telemetry_ok = false;
    }

    // Update LLM panel - detect actual Ollama model
    state.llm_panel.mode = "Local".to_string();

    // Run `ollama list` and parse output to detect installed models
    match std::process::Command::new("ollama").arg("list").output() {
        Ok(output) if output.status.success() => {
            state.llm_panel.available = true;

            // Parse ollama list output (format: NAME ID SIZE MODIFIED)
            let stdout = String::from_utf8_lossy(&output.stdout);

            // Get first non-header line (most recently used model)
            if let Some(first_line) = stdout.lines().nth(1) {
                let parts: Vec<&str> = first_line.split_whitespace().collect();
                if let Some(model_name) = parts.first() {
                    state.llm_panel.model_name = model_name.to_string();

                    // Extract size from model name (e.g., "llama3.1:8b" -> "8B")
                    if let Some(size_part) = model_name.split(':').nth(1) {
                        state.llm_panel.model_size = size_part.to_uppercase();
                    } else {
                        state.llm_panel.model_size = "Unknown".to_string();
                    }
                } else {
                    // Fallback if parsing fails
                    state.llm_panel.model_name = "Unknown".to_string();
                    state.llm_panel.model_size = "?".to_string();
                }
            } else {
                // No models installed
                state.llm_panel.model_name = "None".to_string();
                state.llm_panel.model_size = "-".to_string();
                state.llm_panel.available = false;
            }
        }
        _ => {
            // Ollama not available or command failed
            state.llm_panel.available = false;
            state.llm_panel.model_name = "Ollama N/A".to_string();
            state.llm_panel.model_size = "-".to_string();
        }
    }
}
