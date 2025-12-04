//! Human Labels v0.0.63 - Tool/Evidence to Human Description Registry
//!
//! Maps internal tool names and evidence IDs to human-readable descriptions.
//! Used by the human transcript mode to present a professional IT department
//! dialogue without exposing internal implementation details.
//!
//! Rules:
//! - No tool names like "hw_snapshot_summary" shown to users
//! - No evidence IDs like "[E1]" shown to users
//! - Descriptions should be truthful and accurate
//! - Describe WHAT is being checked, not HOW

use lazy_static::lazy_static;
use std::collections::HashMap;

// ============================================================================
// Tool Name -> Human Description
// ============================================================================

lazy_static! {
    /// Human-readable descriptions for tools
    static ref TOOL_LABELS: HashMap<&'static str, ToolLabel> = {
        let mut m = HashMap::new();

    // System snapshots
    m.insert("status_snapshot", ToolLabel {
        action: "checking system and daemon status",
        evidence: "daemon status check",
        working: "Checking system status...",
    });
    m.insert("sw_snapshot_summary", ToolLabel {
        action: "reviewing installed software and services",
        evidence: "software and services inventory",
        working: "Reviewing installed software...",
    });
    m.insert("hw_snapshot_summary", ToolLabel {
        action: "checking hardware inventory",
        evidence: "hardware inventory snapshot",
        working: "Checking hardware...",
    });
    m.insert("hw_snapshot_cpu", ToolLabel {
        action: "reading CPU information",
        evidence: "CPU details (model, cores, frequency)",
        working: "Reading CPU info...",
    });

    // Package and service tools
    m.insert("recent_installs", ToolLabel {
        action: "checking recently installed packages",
        evidence: "recent package installation history",
        working: "Checking recent installs...",
    });
    m.insert("package_info", ToolLabel {
        action: "looking up package details",
        evidence: "package information",
        working: "Looking up package...",
    });
    m.insert("service_status", ToolLabel {
        action: "checking service status",
        evidence: "service status check",
        working: "Checking service...",
    });

    // System resources
    m.insert("disk_usage", ToolLabel {
        action: "checking disk space",
        evidence: "disk usage report",
        working: "Checking disk space...",
    });
    m.insert("memory_info", ToolLabel {
        action: "checking memory usage",
        evidence: "memory usage report",
        working: "Checking memory...",
    });
    m.insert("mount_usage", ToolLabel {
        action: "checking mounted filesystems",
        evidence: "filesystem mount status",
        working: "Checking mounts...",
    });

    // Journal and logs
    m.insert("journal_warnings", ToolLabel {
        action: "checking system logs for warnings and errors",
        evidence: "recent system log entries",
        working: "Checking system logs...",
    });
    m.insert("journal_errors", ToolLabel {
        action: "checking system logs for errors",
        evidence: "recent error log entries",
        working: "Checking error logs...",
    });

    // Boot analysis
    m.insert("boot_time_trend", ToolLabel {
        action: "analyzing boot time history",
        evidence: "boot performance trends",
        working: "Analyzing boot times...",
    });
    m.insert("systemd_boot_analysis", ToolLabel {
        action: "analyzing boot sequence",
        evidence: "boot sequence analysis",
        working: "Analyzing boot sequence...",
    });

    // Performance
    m.insert("top_resource_processes", ToolLabel {
        action: "identifying high resource usage",
        evidence: "top resource consumers",
        working: "Checking resource usage...",
    });
    m.insert("slowness_hypotheses", ToolLabel {
        action: "analyzing potential causes of slowness",
        evidence: "slowness analysis",
        working: "Analyzing system performance...",
    });

    // Knowledge and search
    m.insert("knowledge_search", ToolLabel {
        action: "searching local documentation",
        evidence: "documentation search results",
        working: "Searching documentation...",
    });
    m.insert("knowledge_stats", ToolLabel {
        action: "checking knowledge base status",
        evidence: "knowledge base statistics",
        working: "Checking knowledge base...",
    });

    // What changed
    m.insert("what_changed", ToolLabel {
        action: "analyzing recent system changes",
        evidence: "recent system changes",
        working: "Checking what changed...",
    });

    // Alerts and anomalies
    m.insert("active_alerts", ToolLabel {
        action: "checking for active alerts",
        evidence: "active alert status",
        working: "Checking alerts...",
    });

    // Network tools
    m.insert("network_status", ToolLabel {
        action: "checking network connectivity",
        evidence: "network status check",
        working: "Checking network...",
    });
    m.insert("network_interfaces", ToolLabel {
        action: "listing network interfaces",
        evidence: "network interface status",
        working: "Checking interfaces...",
    });
    m.insert("dns_check", ToolLabel {
        action: "testing DNS resolution",
        evidence: "DNS resolution test",
        working: "Testing DNS...",
    });
    m.insert("route_check", ToolLabel {
        action: "checking network routes",
        evidence: "routing table check",
        working: "Checking routes...",
    });

    // Audio tools
    m.insert("audio_status", ToolLabel {
        action: "checking audio configuration",
        evidence: "audio device and mixer status",
        working: "Checking audio...",
    });

    // Doctor tools
    m.insert("networking_doctor_probe", ToolLabel {
        action: "running network diagnostics",
        evidence: "network diagnostics results",
        working: "Running network diagnostics...",
    });
    m.insert("storage_doctor_probe", ToolLabel {
        action: "running storage diagnostics",
        evidence: "storage diagnostics results",
        working: "Running storage diagnostics...",
    });
    m.insert("audio_doctor_probe", ToolLabel {
        action: "running audio diagnostics",
        evidence: "audio diagnostics results",
        working: "Running audio diagnostics...",
    });
    m.insert("boot_doctor_probe", ToolLabel {
        action: "running boot diagnostics",
        evidence: "boot diagnostics results",
        working: "Running boot diagnostics...",
    });

    // Systemd probes
    m.insert("systemd_service_probe_v1", ToolLabel {
        action: "checking systemd service status",
        evidence: "systemd service details",
        working: "Checking systemd service...",
    });

    // File tools
    m.insert("file_stat", ToolLabel {
        action: "checking file properties",
        evidence: "file metadata",
        working: "Checking file...",
    });
    m.insert("file_preview", ToolLabel {
        action: "previewing file contents",
        evidence: "file content preview",
        working: "Reading file...",
    });
    m.insert("file_hash", ToolLabel {
        action: "computing file checksum",
        evidence: "file integrity check",
        working: "Computing checksum...",
    });
    m.insert("path_policy_check", ToolLabel {
        action: "checking path safety policy",
        evidence: "path policy verification",
        working: "Checking policy...",
    });

    // Self-diagnostics
    m.insert("self_diagnostics", ToolLabel {
        action: "generating self-diagnostics report",
        evidence: "self-diagnostics report",
        working: "Generating diagnostics...",
    });
    m.insert("metrics_summary", ToolLabel {
        action: "checking reliability metrics",
        evidence: "reliability metrics summary",
        working: "Checking metrics...",
    });
    m.insert("error_budgets", ToolLabel {
        action: "checking error budgets",
        evidence: "error budget status",
        working: "Checking error budgets...",
    });

    // Context and planning
    m.insert("answer_context", ToolLabel {
        action: "gathering context for answer",
        evidence: "answer context",
        working: "Gathering context...",
    });
    m.insert("source_plan", ToolLabel {
        action: "planning information sources",
        evidence: "source plan",
        working: "Planning sources...",
    });
    m.insert("qa_stats", ToolLabel {
        action: "checking Q&A statistics",
        evidence: "Q&A statistics",
        working: "Checking Q&A stats...",
    });

    // Kernel and system info
    m.insert("kernel_version", ToolLabel {
        action: "checking kernel version",
        evidence: "kernel information",
        working: "Checking kernel...",
    });
    m.insert("uptime", ToolLabel {
        action: "checking system uptime",
        evidence: "system uptime",
        working: "Checking uptime...",
    });

        m
    };
}

/// Labels for a tool
#[derive(Debug, Clone, Copy)]
pub struct ToolLabel {
    /// Human description of what the tool does (e.g., "checking hardware inventory")
    pub action: &'static str,
    /// Human description of evidence produced (e.g., "hardware inventory snapshot")
    pub evidence: &'static str,
    /// Working/progress message (e.g., "Checking hardware...")
    pub working: &'static str,
}

/// Get human label for a tool name
pub fn get_tool_label(tool_name: &str) -> Option<&'static ToolLabel> {
    TOOL_LABELS.get(tool_name)
}

/// Get human action description for a tool
pub fn tool_action(tool_name: &str) -> String {
    TOOL_LABELS
        .get(tool_name)
        .map(|l| l.action.to_string())
        .unwrap_or_else(|| format!("running {} check", tool_name.replace('_', " ")))
}

/// Get human evidence description for a tool
pub fn tool_evidence_desc(tool_name: &str) -> String {
    TOOL_LABELS
        .get(tool_name)
        .map(|l| l.evidence.to_string())
        .unwrap_or_else(|| format!("{} results", tool_name.replace('_', " ")))
}

/// Get working message for a tool
pub fn tool_working_msg(tool_name: &str) -> String {
    TOOL_LABELS
        .get(tool_name)
        .map(|l| l.working.to_string())
        .unwrap_or_else(|| format!("Running {}...", tool_name.replace('_', " ")))
}

// ============================================================================
// Department Labels
// ============================================================================

/// Human-readable department descriptions
pub fn department_working_msg(department: &str) -> &'static str {
    match department.to_lowercase().as_str() {
        "networking" | "network" => "Networking team is checking connectivity, routes, and DNS.",
        "storage" => "Storage team is reviewing disk health and filesystem status.",
        "boot" => "Boot team is analyzing startup sequence and service timing.",
        "audio" => "Audio team is checking sound stack and device configuration.",
        "graphics" => "Graphics team is reviewing display and GPU configuration.",
        "security" => "Security team is checking system integrity and permissions.",
        "performance" => "Performance team is analyzing resource usage patterns.",
        "service_desk" | "servicedesk" => "I'm reviewing your request.",
        _ => "Team is investigating.",
    }
}

/// Human-readable description of what a department does
pub fn department_description(department: &str) -> &'static str {
    match department.to_lowercase().as_str() {
        "networking" | "network" => "network connectivity and configuration",
        "storage" => "disk and filesystem health",
        "boot" => "system startup and service timing",
        "audio" => "audio devices and sound configuration",
        "graphics" => "display and GPU configuration",
        "security" => "system integrity and permissions",
        "performance" => "resource usage and optimization",
        _ => "system configuration",
    }
}

// ============================================================================
// v0.0.63: Topic-Aware Evidence Narration
// ============================================================================

/// Human-readable narration for evidence collection based on topic
/// Used in Human Mode to describe what evidence was gathered without tool names
pub fn topic_evidence_narration(topic_label: &str, tool_names: &[String]) -> String {
    // Generate human descriptions for each tool
    let evidence_types: Vec<String> = tool_names.iter().map(|t| tool_evidence_desc(t)).collect();

    if evidence_types.is_empty() {
        return format!("I checked your {} information.", topic_label.to_lowercase());
    }

    // Build a nice narrative
    match evidence_types.len() {
        1 => format!(
            "To answer your {} question, I gathered: {}.",
            topic_label.to_lowercase(),
            evidence_types[0]
        ),
        2 => format!(
            "To answer your {} question, I gathered: {} and {}.",
            topic_label.to_lowercase(),
            evidence_types[0],
            evidence_types[1]
        ),
        _ => {
            let last = evidence_types.last().unwrap();
            let rest = &evidence_types[..evidence_types.len() - 1];
            format!(
                "To answer your {} question, I gathered: {}, and {}.",
                topic_label.to_lowercase(),
                rest.join(", "),
                last
            )
        }
    }
}

/// Short progress message for evidence collection by topic
pub fn topic_working_msg(topic_label: &str) -> String {
    format!("Gathering {} information...", topic_label.to_lowercase())
}

// ============================================================================
// Evidence Grouping for Human Display
// ============================================================================

/// Group evidence items by category for human display
#[derive(Debug, Clone)]
pub struct EvidenceGroup {
    pub category: &'static str,
    pub items: Vec<String>,
}

/// Categorize evidence descriptions for cleaner human display
pub fn categorize_evidence(descriptions: &[String]) -> Vec<EvidenceGroup> {
    let mut hardware = Vec::new();
    let mut software = Vec::new();
    let mut logs = Vec::new();
    let mut network = Vec::new();
    let mut other = Vec::new();

    for desc in descriptions {
        let lower = desc.to_lowercase();
        if lower.contains("hardware")
            || lower.contains("cpu")
            || lower.contains("memory")
            || lower.contains("disk")
            || lower.contains("gpu")
        {
            hardware.push(desc.clone());
        } else if lower.contains("software")
            || lower.contains("package")
            || lower.contains("service")
        {
            software.push(desc.clone());
        } else if lower.contains("log")
            || lower.contains("journal")
            || lower.contains("warning")
            || lower.contains("error")
        {
            logs.push(desc.clone());
        } else if lower.contains("network")
            || lower.contains("dns")
            || lower.contains("route")
            || lower.contains("interface")
        {
            network.push(desc.clone());
        } else {
            other.push(desc.clone());
        }
    }

    let mut groups = Vec::new();
    if !hardware.is_empty() {
        groups.push(EvidenceGroup {
            category: "Hardware",
            items: hardware,
        });
    }
    if !software.is_empty() {
        groups.push(EvidenceGroup {
            category: "Software & Services",
            items: software,
        });
    }
    if !logs.is_empty() {
        groups.push(EvidenceGroup {
            category: "System Logs",
            items: logs,
        });
    }
    if !network.is_empty() {
        groups.push(EvidenceGroup {
            category: "Network",
            items: network,
        });
    }
    if !other.is_empty() {
        groups.push(EvidenceGroup {
            category: "Other",
            items: other,
        });
    }

    groups
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_tool_labels() {
        assert!(get_tool_label("hw_snapshot_summary").is_some());
        assert!(get_tool_label("sw_snapshot_summary").is_some());
        assert!(get_tool_label("status_snapshot").is_some());
    }

    #[test]
    fn test_tool_action_fallback() {
        // Known tool
        assert_eq!(
            tool_action("hw_snapshot_summary"),
            "checking hardware inventory"
        );

        // Unknown tool - should still produce reasonable output
        let action = tool_action("some_unknown_tool");
        assert!(action.contains("some unknown tool"));
    }

    #[test]
    fn test_department_descriptions() {
        assert!(department_working_msg("networking").contains("Networking"));
        assert!(department_working_msg("storage").contains("Storage"));
    }

    #[test]
    fn test_evidence_categorization() {
        let descs = vec![
            "hardware inventory snapshot".to_string(),
            "CPU details".to_string(),
            "software and services inventory".to_string(),
            "recent system log entries".to_string(),
            "network status check".to_string(),
        ];

        let groups = categorize_evidence(&descs);
        assert!(!groups.is_empty());

        // Should have at least hardware and software groups
        let categories: Vec<_> = groups.iter().map(|g| g.category).collect();
        assert!(categories.contains(&"Hardware"));
    }

    // v0.0.63: Topic narration tests
    #[test]
    fn test_topic_evidence_narration_single() {
        let tools = vec!["memory_info".to_string()];
        let narration = topic_evidence_narration("Memory", &tools);
        assert!(narration.contains("memory"));
        assert!(narration.contains("memory usage report"));
        assert!(!narration.contains("memory_info")); // No raw tool names
    }

    #[test]
    fn test_topic_evidence_narration_multiple() {
        let tools = vec!["network_status".to_string(), "dns_check".to_string()];
        let narration = topic_evidence_narration("Network", &tools);
        assert!(narration.contains("network"));
        assert!(narration.contains("network status check"));
        assert!(narration.contains("DNS resolution test"));
        assert!(narration.contains(" and ")); // Uses proper grammar
    }

    #[test]
    fn test_topic_working_msg() {
        let msg = topic_working_msg("Disk Space");
        assert!(msg.contains("disk space"));
        assert!(msg.contains("Gathering"));
    }

    #[test]
    fn test_audio_status_label() {
        // Verify the new audio_status label is defined
        let label = get_tool_label("audio_status");
        assert!(label.is_some());
        assert!(label.unwrap().evidence.contains("audio"));
    }
}
