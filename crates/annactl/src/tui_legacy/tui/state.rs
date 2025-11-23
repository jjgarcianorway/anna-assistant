//! State management - System telemetry updates and welcome message
//! Beta.213: Integrated deterministic welcome engine with RPC telemetry

use crate::startup::welcome::{load_last_session, save_session_metadata};
use crate::llm_integration::fetch_telemetry_snapshot;
use crate::output::normalize_for_tui;
use crate::tui_state::AnnaTuiState;

/// Beta.221: Enhanced welcome with system state from brain analysis
/// Beta.230: Fixed to always show welcome, with fallback to local telemetry if daemon unavailable
pub async fn show_welcome_message(state: &mut AnnaTuiState) {
    // Try to fetch telemetry snapshot from daemon via RPC (Beta.213)
    // This is non-blocking and doesn't use LLM
    let current_snapshot = fetch_telemetry_snapshot().await;

    // Fallback to local telemetry if daemon unavailable
    let snapshot = current_snapshot.or_else(|| {
        // Beta.230: Generate snapshot from local telemetry collection
        use crate::system_query::query_system_telemetry;
        query_system_telemetry()
            .ok()
            .map(|telemetry| crate::startup::welcome::create_telemetry_snapshot(&telemetry))
    });

    if let Some(snapshot) = snapshot {
        // Load last session metadata
        let last_session = load_last_session().ok().flatten();

        // Beta.221: Determine system state from brain insights
        let system_state = determine_system_state(state);

        // Generate welcome report with system state (deterministic, zero LLM calls)
        let welcome_report = crate::startup::welcome::generate_welcome_with_state(
            last_session,
            snapshot.clone(),
            system_state
        );

        // Normalize for TUI display
        let welcome = normalize_for_tui(&welcome_report);

        // Display welcome report
        state.add_anna_reply(welcome);

        // Save current session for next run (best effort)
        let _ = save_session_metadata(snapshot);
    } else {
        // Ultimate fallback: even local telemetry failed
        let fallback = "Welcome to Anna! Unable to collect system telemetry.".to_string();
        state.add_anna_reply(fallback);
    }
}

/// Beta.221: Determine system state from brain insights
fn determine_system_state(state: &AnnaTuiState) -> &'static str {
    if !state.brain_available {
        return "Unknown";
    }

    let has_critical = state.brain_insights.iter().any(|i| i.severity.to_lowercase() == "critical");
    let has_warning = state.brain_insights.iter().any(|i| i.severity.to_lowercase() == "warning");

    if has_critical {
        "Critical"
    } else if has_warning {
        "Warning"
    } else {
        "Healthy"
    }
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

        // Version 150: Get real hostname from telemetry_truth (not env vars!)
        use crate::telemetry_truth::VerifiedSystemReport;
        let verified = VerifiedSystemReport::from_telemetry(&telemetry);
        if let crate::telemetry_truth::SystemFact::Known { value, .. } = verified.hostname {
            state.system_panel.hostname = value;
        } else {
            state.system_panel.hostname = "unknown".to_string();
        }

        // Update GPU info if available
        state.system_panel.gpu_name = telemetry.hardware.gpu_info.clone();
        // GPU VRAM would need nvidia-smi or similar

        state.telemetry_ok = true;
    } else {
        state.telemetry_ok = false;
    }

    // Beta.234: LLM panel - get model from config instead of executing `ollama list`
    // REMOVED: Blocking std::process::Command that could freeze TUI if ollama hangs
    // Instead, read model from LLM config (already cached, no blocking)
    state.llm_panel.mode = "Local".to_string();

    // Get model from cached config (non-blocking, instant)
    let config = crate::query_handler::get_llm_config();
    if let Some(model) = &config.model {
        state.llm_panel.model_name = model.clone();
        state.llm_panel.available = true;

        // Extract size from model name (e.g., "llama3.1:8b" -> "8B")
        if let Some(size_part) = model.split(':').nth(1) {
            state.llm_panel.model_size = size_part.to_uppercase();
        } else {
            state.llm_panel.model_size = "Unknown".to_string();
        }
    } else {
        state.llm_panel.available = false;
        state.llm_panel.model_name = "Not configured".to_string();
        state.llm_panel.model_size = "-".to_string();
    }
}
