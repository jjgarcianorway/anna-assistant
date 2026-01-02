//! Response formatting utilities (v0.0.291).
//!
//! Extracted from llm_request.rs to keep files under 400 lines.
//! v0.0.794: Skip formatting for data-listing answers (ports, services, etc.)

use crate::response_formatter;
use tracing::info;

/// v0.0.794: Check if a route class should skip LLM formatting
/// Data-listing answers (ports, services, env vars, etc.) are already well-formatted
/// and don't benefit from LLM rephrasing - they just need the raw data displayed
/// v0.0.795: Fixed to use snake_case (QueryClass::to_string() format)
/// v0.0.796: Added more query types (installed_tool_check, swap_files, etc.)
pub fn should_skip_formatting(route_class: &str) -> bool {
    matches!(
        route_class,
        // Data listings that display raw system info
        "listening_ports"
            | "running_services"
            | "environment_vars"
            | "mounted_filesystems"
            | "usb_devices"
            | "logged_in_users"
            | "network_interfaces"
            | "top_cpu_processes"
            | "top_memory_processes"
            // Direct probe answers are already formatted
            | "probe_direct"
            // Knowledge index answers don't need reformatting
            | "knowledge_index"
            // Simple status queries
            | "system_architecture"
            | "hostname"
            | "os_info"
            | "kernel_version"
            | "current_user"
            | "last_boot"
            | "system_uptime"
            | "battery_status"
            | "system_load"
            | "timezone_info"
            // System info queries
            | "cpu_info"
            | "cpu_cores"
            | "cpu_temp"
            | "ram_info"
            | "gpu_info"
            | "disk_space"
            | "disk_usage"
            | "memory_usage"
            | "memory_free"
            | "swap_info"
            | "swap_files"
            | "process_tree"
            | "dns_servers"
            | "default_gateway"
            // v0.0.796: Package and tool queries
            | "installed_tool_check"
            | "package_count"
            | "package_updates"
            // v0.0.796: Storage queries
            | "largest_folders"
            | "fstab_entries"
            // v0.0.796: Network queries
            | "network_connectivity"
            | "ip_routes"
            | "open_files"
            // v0.0.796: Hardware queries
            | "hardware_audio"
            | "pci_devices"
            | "block_devices"
            // v0.0.799: Boot queries
            | "boot_blame"
            | "boot_time_status"
    )
}

/// Format deterministic answer via LLM (if needed).
/// v0.0.794: Skip formatting for data-listing answers.
pub async fn format_deterministic_answer(
    answer: String,
    used_deterministic: bool,
    det_result: Option<&crate::deterministic::DeterministicResult>,
    translator_model: &str,
    query: &str,
) -> String {
    if !used_deterministic || answer.is_empty() {
        return answer;
    }

    let skip_formatting = det_result
        .as_ref()
        .map(|det| should_skip_formatting(&det.route_class))
        .unwrap_or(false);

    if skip_formatting {
        info!(
            "v0.0.794: Skipping LLM formatting for data listing (route_class={})",
            det_result
                .as_ref()
                .map(|d| d.route_class.as_str())
                .unwrap_or("unknown")
        );
        return answer;
    }

    response_formatter::format_response(
        translator_model,
        &answer,
        query,
        8, // 8 second timeout for formatting
    )
    .await
}
