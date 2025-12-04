//! Answer Shaper v0.0.65 - Topic-Specific Response Generation
//!
//! Generates concise, topic-appropriate answers:
//! - Disk query → "Root: 433 GiB free (87%)" not CPU/GPU info
//! - Kernel query → "Kernel: 6.17.9-arch1-1" only
//! - Audio query → Service status + devices, not "working" claims
//!
//! Prevents over-claiming and ensures relevance.

use crate::evidence_record::EvidenceBundle;
use crate::evidence_topic::EvidenceTopic;
use serde::{Deserialize, Serialize};

// ============================================================================
// Shaped Answer
// ============================================================================

/// A shaped answer with topic-appropriate content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapedAnswer {
    /// The answer text (human-readable)
    pub text: String,
    /// Topic this answer addresses
    pub topic: EvidenceTopic,
    /// Whether the answer is complete (all required evidence present)
    pub complete: bool,
    /// Confidence notes (what we can and can't confirm)
    pub confidence_notes: Vec<String>,
    /// Human-readable evidence summary (no tool names)
    pub evidence_summary: String,
    /// Debug evidence summary (with tool names)
    pub debug_evidence: String,
}

impl ShapedAnswer {
    /// Create a shaped answer
    pub fn new(text: String, topic: EvidenceTopic, bundle: &EvidenceBundle) -> Self {
        Self {
            text,
            topic,
            complete: bundle.complete,
            confidence_notes: Vec::new(),
            evidence_summary: bundle.human_summary(),
            debug_evidence: bundle.debug_summary(),
        }
    }

    /// Add a confidence note
    pub fn with_note(mut self, note: &str) -> Self {
        self.confidence_notes.push(note.to_string());
        self
    }
}

// ============================================================================
// Main Shaping Logic
// ============================================================================

/// Shape an answer from evidence bundle
pub fn shape_answer(
    topic: EvidenceTopic,
    bundle: &EvidenceBundle,
    evidence_data: &serde_json::Value,
) -> ShapedAnswer {
    match topic {
        EvidenceTopic::DiskFree => shape_disk_answer(bundle, evidence_data),
        EvidenceTopic::KernelVersion => shape_kernel_answer(bundle, evidence_data),
        EvidenceTopic::MemoryInfo => shape_memory_answer(bundle, evidence_data),
        EvidenceTopic::CpuInfo => shape_cpu_answer(bundle, evidence_data),
        EvidenceTopic::NetworkStatus => shape_network_answer(bundle, evidence_data),
        EvidenceTopic::AudioStatus => shape_audio_answer(bundle, evidence_data),
        EvidenceTopic::ServiceState => shape_service_answer(bundle, evidence_data),
        EvidenceTopic::BootTime => shape_boot_answer(bundle, evidence_data),
        EvidenceTopic::GraphicsStatus => shape_graphics_answer(bundle, evidence_data),
        EvidenceTopic::Alerts => shape_alerts_answer(bundle, evidence_data),
        EvidenceTopic::RecentErrors => shape_errors_answer(bundle, evidence_data),
        EvidenceTopic::PackagesChanged => shape_packages_answer(bundle, evidence_data),
        EvidenceTopic::Unknown => shape_generic_answer(bundle, evidence_data),
    }
}

// ============================================================================
// Topic-Specific Shapers
// ============================================================================

fn shape_disk_answer(bundle: &EvidenceBundle, data: &serde_json::Value) -> ShapedAnswer {
    let text = if let Some(root) = data.get("root") {
        let avail = root
            .get("avail_human")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let use_pct = root
            .get("use_percent")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let free_pct = use_pct.parse::<u8>().map(|p| 100 - p).unwrap_or(0);
        format!("Root disk free: {} ({}% available).", avail, free_pct)
    } else {
        "Disk usage information unavailable.".to_string()
    };

    ShapedAnswer::new(text, EvidenceTopic::DiskFree, bundle)
}

fn shape_kernel_answer(bundle: &EvidenceBundle, data: &serde_json::Value) -> ShapedAnswer {
    let text = if let Some(kernel) = data.get("kernel_release").and_then(|v| v.as_str()) {
        format!("Kernel: {}.", kernel)
    } else {
        "Kernel version information unavailable.".to_string()
    };

    ShapedAnswer::new(text, EvidenceTopic::KernelVersion, bundle)
}

fn shape_memory_answer(bundle: &EvidenceBundle, data: &serde_json::Value) -> ShapedAnswer {
    let text = if let (Some(total), Some(avail)) = (
        data.get("total_gib").and_then(|v| v.as_str()),
        data.get("available_gib").and_then(|v| v.as_str()),
    ) {
        format!("Memory: {} GiB total, {} GiB available.", total, avail)
    } else {
        "Memory information unavailable.".to_string()
    };

    ShapedAnswer::new(text, EvidenceTopic::MemoryInfo, bundle)
}

fn shape_cpu_answer(bundle: &EvidenceBundle, data: &serde_json::Value) -> ShapedAnswer {
    let text = if let Some(model) = data.get("cpu_model").and_then(|v| v.as_str()) {
        let cores = data.get("cores").and_then(|v| v.as_u64()).unwrap_or(0);
        let threads = data
            .get("threads")
            .and_then(|v| v.as_u64())
            .unwrap_or(cores);
        if cores > 0 {
            format!("CPU: {} ({} cores, {} threads).", model, cores, threads)
        } else {
            format!("CPU: {}.", model)
        }
    } else {
        "CPU information unavailable.".to_string()
    };

    ShapedAnswer::new(text, EvidenceTopic::CpuInfo, bundle)
}

fn shape_network_answer(bundle: &EvidenceBundle, data: &serde_json::Value) -> ShapedAnswer {
    let has_route = data
        .get("has_default_route")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let primary = data
        .get("primary_interface")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let dns_ok = data
        .get("dns_working")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let mut parts = Vec::new();

    // Link/interface status
    if has_route {
        parts.push(format!("Connected via {}", primary));
    } else {
        parts.push("No default route (disconnected)".to_string());
    }

    // DNS status
    if dns_ok {
        parts.push("DNS configured".to_string());
    } else if data.get("dns_servers").is_some() {
        parts.push("DNS servers present".to_string());
    }

    // IP info if available
    if let Some(ip) = data.get("primary_ip").and_then(|v| v.as_str()) {
        parts.push(format!("IP: {}", ip));
    }

    let text = if parts.is_empty() {
        "Network status unavailable.".to_string()
    } else {
        format!("Network: {}.", parts.join(", "))
    };

    let mut answer = ShapedAnswer::new(text, EvidenceTopic::NetworkStatus, bundle);

    // Add confidence notes for network - can't confirm actual connectivity
    if has_route {
        answer = answer.with_note("Route present but actual internet access not verified");
    }

    answer
}

fn shape_audio_answer(bundle: &EvidenceBundle, data: &serde_json::Value) -> ShapedAnswer {
    let pipewire = data
        .get("pipewire_running")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let wireplumber = data
        .get("wireplumber_running")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let pulse = data
        .get("pulseaudio_running")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let mut parts = Vec::new();

    // Audio stack detection
    if pipewire {
        parts.push("PipeWire running".to_string());
        if wireplumber {
            parts.push("WirePlumber active".to_string());
        }
    } else if pulse {
        parts.push("PulseAudio running".to_string());
    } else {
        parts.push("No audio service detected".to_string());
    }

    // Device detection
    if let Some(sinks) = data.get("sink_count").and_then(|v| v.as_u64()) {
        if sinks > 0 {
            parts.push(format!("{} sink(s) available", sinks));
        }
    }

    let text = format!("Audio: {}.", parts.join(", "));

    let mut answer = ShapedAnswer::new(text, EvidenceTopic::AudioStatus, bundle);

    // IMPORTANT: Never claim audio "works" - we can't hear it!
    answer = answer.with_note("Service status shown; actual sound output cannot be verified");

    answer
}

fn shape_service_answer(bundle: &EvidenceBundle, data: &serde_json::Value) -> ShapedAnswer {
    let service_name = data
        .get("service_name")
        .and_then(|v| v.as_str())
        .unwrap_or("service");
    let active = data
        .get("active")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let enabled = data.get("enabled").and_then(|v| v.as_bool());

    let status = if active { "running" } else { "stopped" };

    let text = if let Some(enabled) = enabled {
        let enable_str = if enabled { "enabled" } else { "disabled" };
        format!("Service {}: {} ({}).", service_name, status, enable_str)
    } else {
        format!("Service {}: {}.", service_name, status)
    };

    ShapedAnswer::new(text, EvidenceTopic::ServiceState, bundle)
}

fn shape_boot_answer(bundle: &EvidenceBundle, data: &serde_json::Value) -> ShapedAnswer {
    let text = if let Some(boot_time) = data.get("boot_time_secs").and_then(|v| v.as_f64()) {
        format!("Boot time: {:.1} seconds.", boot_time)
    } else if let Some(uptime) = data.get("uptime_secs").and_then(|v| v.as_u64()) {
        let hours = uptime / 3600;
        let mins = (uptime % 3600) / 60;
        format!("System uptime: {}h {}m.", hours, mins)
    } else {
        "Boot time information unavailable.".to_string()
    };

    ShapedAnswer::new(text, EvidenceTopic::BootTime, bundle)
}

fn shape_graphics_answer(bundle: &EvidenceBundle, data: &serde_json::Value) -> ShapedAnswer {
    let text = if let Some(gpu) = data.get("gpu").and_then(|v| v.as_str()) {
        format!("GPU: {}.", gpu)
    } else if let Some(gpus) = data.get("gpus").as_ref().and_then(|v| v.as_array()) {
        if gpus.is_empty() {
            "No GPU detected.".to_string()
        } else {
            let gpu_names: Vec<&str> = gpus
                .iter()
                .filter_map(|g| g.get("name").and_then(|n| n.as_str()))
                .collect();
            format!("GPU(s): {}.", gpu_names.join(", "))
        }
    } else {
        "Graphics information unavailable.".to_string()
    };

    ShapedAnswer::new(text, EvidenceTopic::GraphicsStatus, bundle)
}

fn shape_alerts_answer(bundle: &EvidenceBundle, data: &serde_json::Value) -> ShapedAnswer {
    let count = data
        .get("alert_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let text = if count == 0 {
        "No active alerts.".to_string()
    } else {
        format!(
            "{} active alert(s). Check 'annactl status' for details.",
            count
        )
    };

    ShapedAnswer::new(text, EvidenceTopic::Alerts, bundle)
}

fn shape_errors_answer(bundle: &EvidenceBundle, data: &serde_json::Value) -> ShapedAnswer {
    let count = data
        .get("error_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let text = if count == 0 {
        "No recent errors in system journal.".to_string()
    } else {
        format!("{} recent error(s) in journal.", count)
    };

    ShapedAnswer::new(text, EvidenceTopic::RecentErrors, bundle)
}

fn shape_packages_answer(bundle: &EvidenceBundle, data: &serde_json::Value) -> ShapedAnswer {
    let count = data
        .get("changes_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let text = if count == 0 {
        "No recent package changes.".to_string()
    } else {
        format!("{} package(s) changed recently.", count)
    };

    ShapedAnswer::new(text, EvidenceTopic::PackagesChanged, bundle)
}

fn shape_generic_answer(bundle: &EvidenceBundle, _data: &serde_json::Value) -> ShapedAnswer {
    let text = if bundle.records.is_empty() {
        "Unable to gather relevant information.".to_string()
    } else {
        bundle.human_summary()
    };

    ShapedAnswer::new(text, EvidenceTopic::Unknown, bundle)
}

// ============================================================================
// Human Mode Formatting
// ============================================================================

/// Format answer for Human Mode (no tool names, no evidence IDs)
pub fn format_human_answer(answer: &ShapedAnswer) -> String {
    let mut output = answer.text.clone();

    // Add confidence notes if any
    if !answer.confidence_notes.is_empty() {
        output.push_str("\n\nNote: ");
        output.push_str(&answer.confidence_notes.join("; "));
    }

    output
}

/// Format answer for Debug Mode (with tool names and evidence IDs)
pub fn format_debug_answer(answer: &ShapedAnswer) -> String {
    let mut output = answer.text.clone();

    output.push_str("\n\n--- Evidence ---\n");
    output.push_str(&answer.debug_evidence);

    if !answer.confidence_notes.is_empty() {
        output.push_str("\n\n--- Notes ---\n");
        for note in &answer.confidence_notes {
            output.push_str(&format!("- {}\n", note));
        }
    }

    output
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence_record::EvidenceRecord;

    fn make_bundle(topic: EvidenceTopic) -> EvidenceBundle {
        let mut bundle = EvidenceBundle::new(topic);
        bundle.add(EvidenceRecord::new(
            topic,
            "Test evidence".to_string(),
            "test_tool",
            "E1",
        ));
        bundle.complete = true;
        bundle
    }

    #[test]
    fn test_disk_answer() {
        let bundle = make_bundle(EvidenceTopic::DiskFree);
        let data = serde_json::json!({
            "root": {
                "avail_human": "433 GiB",
                "use_percent": "13"
            }
        });

        let answer = shape_answer(EvidenceTopic::DiskFree, &bundle, &data);
        assert!(answer.text.contains("433 GiB"));
        assert!(answer.text.contains("87%")); // 100 - 13
        assert!(!answer.text.contains("CPU")); // No CPU info
    }

    #[test]
    fn test_kernel_answer() {
        let bundle = make_bundle(EvidenceTopic::KernelVersion);
        let data = serde_json::json!({
            "kernel_release": "6.17.9-arch1-1"
        });

        let answer = shape_answer(EvidenceTopic::KernelVersion, &bundle, &data);
        assert!(answer.text.contains("6.17.9-arch1-1"));
        assert!(answer.text.contains("Kernel:"));
    }

    #[test]
    fn test_audio_answer_no_overclaim() {
        let bundle = make_bundle(EvidenceTopic::AudioStatus);
        let data = serde_json::json!({
            "pipewire_running": true,
            "wireplumber_running": true,
            "sink_count": 2
        });

        let answer = shape_answer(EvidenceTopic::AudioStatus, &bundle, &data);
        // Should mention services running, NOT claim audio "works"
        assert!(answer.text.contains("PipeWire running"));
        assert!(!answer.text.to_lowercase().contains("working"));
        // Should have confidence note about verification
        assert!(!answer.confidence_notes.is_empty());
    }

    #[test]
    fn test_network_answer_no_overclaim() {
        let bundle = make_bundle(EvidenceTopic::NetworkStatus);
        let data = serde_json::json!({
            "has_default_route": true,
            "primary_interface": "wlan0",
            "dns_servers": ["8.8.8.8"]
        });

        let answer = shape_answer(EvidenceTopic::NetworkStatus, &bundle, &data);
        assert!(answer.text.contains("Connected"));
        assert!(answer.text.contains("wlan0"));
        // Should have note about actual connectivity
        assert!(answer
            .confidence_notes
            .iter()
            .any(|n| n.contains("not verified")));
    }

    #[test]
    fn test_human_format_no_tool_names() {
        let bundle = make_bundle(EvidenceTopic::MemoryInfo);
        let data = serde_json::json!({
            "total_gib": "32",
            "available_gib": "24"
        });

        let answer = shape_answer(EvidenceTopic::MemoryInfo, &bundle, &data);
        let human = format_human_answer(&answer);

        // Human format should NOT contain tool names or evidence IDs
        assert!(!human.contains("memory_info"));
        assert!(!human.contains("[E1]"));
        assert!(human.contains("32 GiB"));
    }
}
