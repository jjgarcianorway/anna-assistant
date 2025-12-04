//! Direct Answer Generator v0.0.74
//!
//! Generates direct, human-readable answers for system queries.
//! Uses evidence from tool results to create concise responses.
//!
//! Key features:
//! - Topic-based routing (RAM → memory_info tool, kernel → kernel_version tool)
//! - Direct answers without "Proposed action plan" scaffolding
//! - Integration with evidence topic detection

use crate::evidence_topic::{detect_topic, EvidenceTopic, TopicDetection};
use crate::tools::ToolResult;

/// A direct answer ready for display
#[derive(Debug, Clone)]
pub struct DirectAnswer {
    /// The concise answer text
    pub answer: String,
    /// Topic this addresses
    pub topic: EvidenceTopic,
    /// Confidence level (0-100)
    pub confidence: u8,
    /// Human-readable evidence source
    pub source: String,
    /// Whether answer is complete
    pub complete: bool,
}

impl DirectAnswer {
    /// Format for human mode (no tool names)
    pub fn format_human(&self) -> String {
        if self.complete {
            format!("{}\n\n(Source: {})", self.answer, self.source)
        } else {
            format!(
                "{}\n\n(Source: {} - incomplete evidence)",
                self.answer, self.source
            )
        }
    }

    /// Format for debug mode (with tool names)
    pub fn format_debug(&self, tool_name: &str) -> String {
        format!(
            "{}\n\n[Evidence: {} from {} tool, confidence: {}%]",
            self.answer, self.source, tool_name, self.confidence
        )
    }
}

/// Generate a direct answer from tool results
pub fn generate_direct_answer(request: &str, tool_results: &[ToolResult]) -> Option<DirectAnswer> {
    // Detect topic from request
    let detection = detect_topic(request);

    if detection.topic == EvidenceTopic::Unknown {
        return None;
    }

    // Find the best matching tool result for this topic
    let tool_result = find_tool_for_topic(&detection, tool_results)?;

    // Generate answer based on topic
    let answer = generate_topic_answer(&detection.topic, &tool_result.data)?;

    Some(DirectAnswer {
        answer,
        topic: detection.topic,
        confidence: detection.confidence,
        source: get_human_source(&detection.topic),
        complete: tool_result.success,
    })
}

/// Find the appropriate tool result for a topic
fn find_tool_for_topic<'a>(
    detection: &TopicDetection,
    results: &'a [ToolResult],
) -> Option<&'a ToolResult> {
    let preferred_tools = get_preferred_tools(detection.topic);

    // First, try preferred tools
    for tool_name in preferred_tools {
        if let Some(result) = results
            .iter()
            .find(|r| r.tool_name == tool_name && r.success)
        {
            return Some(result);
        }
    }

    // Fallback: try any successful result that might have relevant data
    results
        .iter()
        .find(|r| r.success && has_topic_data(detection.topic, &r.data))
}

/// Get preferred tools for each topic
fn get_preferred_tools(topic: EvidenceTopic) -> Vec<&'static str> {
    match topic {
        EvidenceTopic::MemoryInfo => vec!["memory_info", "mem_summary"],
        EvidenceTopic::KernelVersion => vec!["kernel_version", "uname_summary"],
        EvidenceTopic::DiskFree => vec!["mount_usage", "disk_usage"],
        EvidenceTopic::CpuInfo => vec!["hw_snapshot_summary"],
        EvidenceTopic::NetworkStatus => vec!["network_status", "nm_summary", "net_routes_summary"],
        EvidenceTopic::AudioStatus => vec!["audio_status", "audio_services_summary"],
        EvidenceTopic::ServiceState => vec!["service_status", "systemd_service_probe_v1"],
        EvidenceTopic::BootTime => vec!["boot_time_summary", "boot_time_trend"],
        EvidenceTopic::GraphicsStatus => vec!["hw_snapshot_summary"],
        EvidenceTopic::Alerts => vec!["proactive_alerts_summary"],
        EvidenceTopic::RecentErrors => vec!["recent_errors_summary", "journal_warnings"],
        EvidenceTopic::PackagesChanged => vec!["recent_installs", "what_changed"],
        EvidenceTopic::Unknown => vec![],
    }
}

/// Check if data contains expected fields for topic
fn has_topic_data(topic: EvidenceTopic, data: &serde_json::Value) -> bool {
    match topic {
        EvidenceTopic::MemoryInfo => {
            data.get("total_gib").is_some()
                || data.get("mem_total_kb").is_some()
                || data.get("MemTotal").is_some()
        }
        EvidenceTopic::KernelVersion => {
            data.get("kernel_release").is_some() || data.get("release").is_some()
        }
        EvidenceTopic::DiskFree => data.get("root").is_some() || data.get("mounts").is_some(),
        EvidenceTopic::CpuInfo => data.get("cpu_model").is_some() || data.get("cpu").is_some(),
        EvidenceTopic::NetworkStatus => {
            data.get("has_default_route").is_some() || data.get("interfaces").is_some()
        }
        EvidenceTopic::AudioStatus => {
            data.get("pipewire_running").is_some() || data.get("audio_server").is_some()
        }
        EvidenceTopic::ServiceState => data.get("active").is_some() || data.get("state").is_some(),
        _ => true, // Allow other topics to pass through
    }
}

/// Get human-readable source description
fn get_human_source(topic: &EvidenceTopic) -> String {
    match topic {
        EvidenceTopic::MemoryInfo => "memory status snapshot".to_string(),
        EvidenceTopic::KernelVersion => "kernel info".to_string(),
        EvidenceTopic::DiskFree => "disk usage snapshot".to_string(),
        EvidenceTopic::CpuInfo => "hardware inventory".to_string(),
        EvidenceTopic::NetworkStatus => "network status".to_string(),
        EvidenceTopic::AudioStatus => "audio stack status".to_string(),
        EvidenceTopic::ServiceState => "service status".to_string(),
        EvidenceTopic::BootTime => "boot time analysis".to_string(),
        EvidenceTopic::GraphicsStatus => "graphics info".to_string(),
        EvidenceTopic::Alerts => "active alerts".to_string(),
        EvidenceTopic::RecentErrors => "system journal".to_string(),
        EvidenceTopic::PackagesChanged => "package history".to_string(),
        EvidenceTopic::Unknown => "system snapshot".to_string(),
    }
}

/// Generate answer text based on topic and data
fn generate_topic_answer(topic: &EvidenceTopic, data: &serde_json::Value) -> Option<String> {
    match topic {
        EvidenceTopic::MemoryInfo => generate_memory_answer(data),
        EvidenceTopic::KernelVersion => generate_kernel_answer(data),
        EvidenceTopic::DiskFree => generate_disk_answer(data),
        EvidenceTopic::CpuInfo => generate_cpu_answer(data),
        EvidenceTopic::NetworkStatus => generate_network_answer(data),
        EvidenceTopic::AudioStatus => generate_audio_answer(data),
        EvidenceTopic::ServiceState => generate_service_answer(data),
        _ => Some(format!("System data: {}", data)),
    }
}

fn generate_memory_answer(data: &serde_json::Value) -> Option<String> {
    // Try memory_info format
    if let Some(total) = data.get("total_gib").and_then(|v| v.as_str()) {
        let available = data
            .get("available_gib")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        return Some(format!(
            "You have {} GiB of RAM total, {} GiB available.",
            total, available
        ));
    }

    // Try mem_summary format
    if let Some(total_kb) = data.get("mem_total_kb").and_then(|v| v.as_u64()) {
        let total_gib = total_kb as f64 / 1024.0 / 1024.0;
        let avail_kb = data
            .get("mem_available_kb")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let avail_gib = avail_kb as f64 / 1024.0 / 1024.0;
        return Some(format!(
            "You have {:.1} GiB of RAM total, {:.1} GiB available.",
            total_gib, avail_gib
        ));
    }

    // Try hw_snapshot_summary memory section
    if let Some(memory) = data.get("memory") {
        if let Some(total) = memory.get("total_gib").and_then(|v| v.as_str()) {
            let avail = memory
                .get("available_gib")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            return Some(format!(
                "You have {} GiB of RAM total, {} GiB available.",
                total, avail
            ));
        }
    }

    None
}

fn generate_kernel_answer(data: &serde_json::Value) -> Option<String> {
    // Try kernel_version format
    if let Some(release) = data.get("kernel_release").and_then(|v| v.as_str()) {
        return Some(format!("You are running Linux kernel {}.", release));
    }

    // Try uname_summary format
    if let Some(release) = data.get("release").and_then(|v| v.as_str()) {
        return Some(format!("You are running Linux kernel {}.", release));
    }

    None
}

fn generate_disk_answer(data: &serde_json::Value) -> Option<String> {
    // Try mount_usage format
    if let Some(root) = data.get("root") {
        let avail = root
            .get("avail_human")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let use_pct = root
            .get("use_percent")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(0);
        let free_pct = 100 - use_pct;
        return Some(format!(
            "Your root filesystem has {} free ({}% available).",
            avail, free_pct
        ));
    }

    // Try disk_usage format
    if let Some(mounts) = data.get("mounts").and_then(|v| v.as_array()) {
        if let Some(root) = mounts
            .iter()
            .find(|m| m.get("mount_point").and_then(|v| v.as_str()) == Some("/"))
        {
            let avail = root
                .get("available_human")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            return Some(format!("Your root filesystem has {} free.", avail));
        }
    }

    None
}

fn generate_cpu_answer(data: &serde_json::Value) -> Option<String> {
    // Try hw_snapshot_summary format
    if let Some(cpu_model) = data.get("cpu_model").and_then(|v| v.as_str()) {
        let cores = data.get("cores").and_then(|v| v.as_u64()).unwrap_or(0);
        let threads = data
            .get("threads")
            .and_then(|v| v.as_u64())
            .unwrap_or(cores);
        if cores > 0 {
            return Some(format!(
                "Your CPU is {} ({} cores, {} threads).",
                cpu_model, cores, threads
            ));
        }
        return Some(format!("Your CPU is {}.", cpu_model));
    }

    // Try cpu section
    if let Some(cpu) = data.get("cpu") {
        if let Some(model) = cpu.get("model").and_then(|v| v.as_str()) {
            let cores = cpu.get("cores").and_then(|v| v.as_u64()).unwrap_or(0);
            if cores > 0 {
                return Some(format!("Your CPU is {} ({} cores).", model, cores));
            }
            return Some(format!("Your CPU is {}.", model));
        }
    }

    None
}

fn generate_network_answer(data: &serde_json::Value) -> Option<String> {
    // Try network_status format
    if let Some(has_route) = data.get("has_default_route").and_then(|v| v.as_bool()) {
        let status = if has_route {
            "connected"
        } else {
            "disconnected"
        };
        let iface = data
            .get("primary_interface")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        return Some(format!(
            "Network is {}, primary interface: {}.",
            status, iface
        ));
    }

    // Try nm_summary format
    if let Some(state) = data.get("nm_state").and_then(|v| v.as_str()) {
        return Some(format!("NetworkManager state: {}.", state));
    }

    None
}

fn generate_audio_answer(data: &serde_json::Value) -> Option<String> {
    // Try audio_status format
    if let Some(running) = data.get("pipewire_running").and_then(|v| v.as_bool()) {
        let status = if running { "running" } else { "not running" };
        return Some(format!("PipeWire is {}.", status));
    }

    // Try audio_services_summary format
    if let Some(server) = data.get("audio_server").and_then(|v| v.as_str()) {
        let running = data
            .get("server_running")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let status = if running { "running" } else { "not running" };
        return Some(format!("{} is {}.", server, status));
    }

    None
}

fn generate_service_answer(data: &serde_json::Value) -> Option<String> {
    // Try service_status format
    if let Some(active) = data.get("active").and_then(|v| v.as_bool()) {
        let service = data
            .get("service_name")
            .and_then(|v| v.as_str())
            .unwrap_or("Service");
        let status = if active { "running" } else { "stopped" };
        return Some(format!("{} is {}.", service, status));
    }

    // Try systemd_service_probe_v1 format
    if let Some(state) = data.get("state").and_then(|v| v.as_str()) {
        let unit = data
            .get("unit")
            .and_then(|v| v.as_str())
            .unwrap_or("Service");
        return Some(format!("{}: {}.", unit, state));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_memory_answer() {
        let data = serde_json::json!({
            "total_gib": "32",
            "available_gib": "16"
        });
        let answer = generate_memory_answer(&data);
        assert!(answer.is_some());
        let text = answer.unwrap();
        assert!(text.contains("32 GiB"));
        assert!(text.contains("16 GiB"));
    }

    #[test]
    fn test_generate_kernel_answer() {
        let data = serde_json::json!({
            "kernel_release": "6.17.9-arch1-1"
        });
        let answer = generate_kernel_answer(&data);
        assert!(answer.is_some());
        assert!(answer.unwrap().contains("6.17.9-arch1-1"));
    }

    #[test]
    fn test_generate_disk_answer() {
        let data = serde_json::json!({
            "root": {
                "avail_human": "100 GiB",
                "use_percent": "60"
            }
        });
        let answer = generate_disk_answer(&data);
        assert!(answer.is_some());
        let text = answer.unwrap();
        assert!(text.contains("100 GiB"));
        assert!(text.contains("40%")); // 100 - 60
    }

    #[test]
    fn test_generate_cpu_answer() {
        let data = serde_json::json!({
            "cpu_model": "AMD Ryzen 9 5900X",
            "cores": 12,
            "threads": 24
        });
        let answer = generate_cpu_answer(&data);
        assert!(answer.is_some());
        let text = answer.unwrap();
        assert!(text.contains("AMD Ryzen 9 5900X"));
        assert!(text.contains("12 cores"));
    }

    #[test]
    fn test_generate_network_answer() {
        let data = serde_json::json!({
            "has_default_route": true,
            "primary_interface": "enp5s0"
        });
        let answer = generate_network_answer(&data);
        assert!(answer.is_some());
        let text = answer.unwrap();
        assert!(text.contains("connected"));
        assert!(text.contains("enp5s0"));
    }

    #[test]
    fn test_generate_service_answer() {
        let data = serde_json::json!({
            "active": true,
            "service_name": "docker"
        });
        let answer = generate_service_answer(&data);
        assert!(answer.is_some());
        let text = answer.unwrap();
        assert!(text.contains("docker"));
        assert!(text.contains("running"));
    }
}
