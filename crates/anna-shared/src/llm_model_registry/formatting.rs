// v0.0.531: LLM Model Registry Formatting
// Display and formatting functions for models and registry

use super::types::ModelRecord;
use super::registry::LlmModelRegistry;

/// Format model for display
pub fn format_model(model: &ModelRecord) -> String {
    format!(
        "{} [{}]\n  Status: {} | Installed by: {}\n  Size: {:.1}GB | VRAM: {:.1}GB\n  Usage: {} calls | Avg: {}ms\n  Assigned to: {}",
        model.name,
        model.capability,
        model.status,
        model.installed_by,
        model.size_gb,
        model.vram_required_gb,
        model.usage_count,
        model.avg_response_ms,
        if model.assigned_specialists.is_empty() {
            "None".to_string()
        } else {
            model.assigned_specialists.join(", ")
        }
    )
}

/// Format model compact
pub fn format_model_compact(model: &ModelRecord) -> String {
    format!(
        "{} [{}] - {:.1}GB ({} calls)",
        model.name, model.capability, model.size_gb, model.usage_count
    )
}

/// Format model oneline
pub fn format_model_oneline(model: &ModelRecord) -> String {
    format!("{} [{}]", model.name, model.status)
}

/// Format registry summary
pub fn format_registry_summary(registry: &LlmModelRegistry) -> String {
    let mut output = String::new();
    output.push_str("=== LLM Model Registry ===\n\n");

    output.push_str(&format!(
        "Total: {} | Ready: {}\n",
        registry.total(),
        registry.ready_count()
    ));
    output.push_str(&format!("VRAM Used: {:.1}GB\n", registry.total_vram_gb()));
    output.push_str(&format!("Disk Used: {:.1}GB\n\n", registry.total_disk_gb()));

    output.push_str("--- Ready Models ---\n");
    for model in registry.ready() {
        output.push_str(&format!("  {}\n", format_model_compact(model)));
    }

    let anna_models = registry.installed_by_anna();
    if !anna_models.is_empty() {
        output.push_str("\n--- Installed by Anna ---\n");
        for model in anna_models {
            output.push_str(&format!("  {}\n", model.name));
        }
    }

    output
}
