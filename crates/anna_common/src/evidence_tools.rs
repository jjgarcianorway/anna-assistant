//! Evidence Tools v0.0.55 - Correct tool routing for common queries
//!
//! Ensures system queries get evidence from the RIGHT tools:
//! - CPU queries → hw_snapshot or /proc/cpuinfo
//! - Memory queries → memory_info or /proc/meminfo
//! - Disk queries → mount_usage or df
//! - Kernel queries → kernel_version or uname
//! - Systemd queries → systemd probes

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::case_engine::IntentType;
use crate::intent_taxonomy::IntentClassification;
use crate::system_query_router::QueryTarget;

// ============================================================================
// Evidence Plan
// ============================================================================

/// Plan for collecting evidence for a case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidencePlan {
    /// Case ID this plan is for
    pub case_id: String,
    /// Tools to execute (in order)
    pub tools: Vec<PlannedTool>,
    /// Expected evidence IDs after execution
    pub expected_evidence: Vec<String>,
    /// Reasoning for this plan
    pub reasoning: String,
}

/// A tool planned for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedTool {
    /// Tool name
    pub tool_name: String,
    /// Tool arguments
    pub args: HashMap<String, serde_json::Value>,
    /// Why this tool is needed
    pub purpose: String,
    /// Is this tool required or optional?
    pub required: bool,
    /// Evidence ID to assign to result
    pub evidence_id: String,
}

// ============================================================================
// Evidence Collection Functions
// ============================================================================

/// Create evidence plan based on intent classification
pub fn plan_evidence(case_id: &str, classification: &IntentClassification) -> EvidencePlan {
    match classification.intent {
        IntentType::SystemQuery => plan_system_query_evidence(case_id, classification),
        IntentType::Diagnose => plan_diagnose_evidence(case_id, classification),
        IntentType::ActionRequest => plan_action_evidence(case_id, classification),
        IntentType::Howto => plan_howto_evidence(case_id, classification),
        IntentType::Meta => plan_meta_evidence(case_id, classification),
    }
}

/// Plan evidence for SYSTEM_QUERY intent
fn plan_system_query_evidence(case_id: &str, classification: &IntentClassification) -> EvidencePlan {
    let mut tools = Vec::new();
    let mut evidence_id = 1;

    // Route based on detected query target
    if let Some(target) = &classification.query_target {
        match target {
            QueryTarget::Cpu => {
                tools.push(PlannedTool {
                    tool_name: "hw_snapshot_cpu".to_string(),
                    args: HashMap::new(),
                    purpose: "Get CPU model, cores, threads, frequency".to_string(),
                    required: true,
                    evidence_id: format!("E{}", evidence_id),
                });
                evidence_id += 1;
            }
            QueryTarget::Memory => {
                tools.push(PlannedTool {
                    tool_name: "memory_info".to_string(),
                    args: HashMap::new(),
                    purpose: "Get total, available, and used RAM".to_string(),
                    required: true,
                    evidence_id: format!("E{}", evidence_id),
                });
                evidence_id += 1;
            }
            QueryTarget::DiskFree => {
                tools.push(PlannedTool {
                    tool_name: "mount_usage".to_string(),
                    args: HashMap::new(),
                    purpose: "Get disk free space for all mounts".to_string(),
                    required: true,
                    evidence_id: format!("E{}", evidence_id),
                });
                evidence_id += 1;
            }
            QueryTarget::KernelVersion => {
                tools.push(PlannedTool {
                    tool_name: "kernel_version".to_string(),
                    args: HashMap::new(),
                    purpose: "Get exact kernel release string".to_string(),
                    required: true,
                    evidence_id: format!("E{}", evidence_id),
                });
                evidence_id += 1;
            }
            QueryTarget::NetworkStatus => {
                tools.push(PlannedTool {
                    tool_name: "network_status".to_string(),
                    args: HashMap::new(),
                    purpose: "Get network interfaces, IPs, routes".to_string(),
                    required: true,
                    evidence_id: format!("E{}", evidence_id),
                });
                evidence_id += 1;
            }
            QueryTarget::AudioStatus => {
                tools.push(PlannedTool {
                    tool_name: "audio_status".to_string(),
                    args: HashMap::new(),
                    purpose: "Get PipeWire/audio stack status".to_string(),
                    required: true,
                    evidence_id: format!("E{}", evidence_id),
                });
                evidence_id += 1;
            }
            QueryTarget::ServicesStatus => {
                // For service queries, we need the service name
                tools.push(PlannedTool {
                    tool_name: "systemd_service_probe_v1".to_string(),
                    args: HashMap::new(),
                    purpose: "Get service status".to_string(),
                    required: true,
                    evidence_id: format!("E{}", evidence_id),
                });
                evidence_id += 1;
            }
            QueryTarget::Hardware => {
                tools.push(PlannedTool {
                    tool_name: "hw_snapshot_summary".to_string(),
                    args: HashMap::new(),
                    purpose: "Get full hardware summary".to_string(),
                    required: true,
                    evidence_id: format!("E{}", evidence_id),
                });
                evidence_id += 1;
            }
            QueryTarget::Software => {
                tools.push(PlannedTool {
                    tool_name: "sw_snapshot_summary".to_string(),
                    args: HashMap::new(),
                    purpose: "Get installed packages summary".to_string(),
                    required: true,
                    evidence_id: format!("E{}", evidence_id),
                });
                evidence_id += 1;
            }
            QueryTarget::Alerts => {
                tools.push(PlannedTool {
                    tool_name: "proactive_alerts_summary".to_string(),
                    args: HashMap::new(),
                    purpose: "Get active alerts and warnings".to_string(),
                    required: true,
                    evidence_id: format!("E{}", evidence_id),
                });
                evidence_id += 1;
            }
            QueryTarget::Unknown => {
                // Fallback: get general system info
                tools.push(PlannedTool {
                    tool_name: "hw_snapshot_summary".to_string(),
                    args: HashMap::new(),
                    purpose: "Get general system info".to_string(),
                    required: false,
                    evidence_id: format!("E{}", evidence_id),
                });
            }
        }
    } else {
        // No specific target - use general snapshot
        tools.push(PlannedTool {
            tool_name: "hw_snapshot_summary".to_string(),
            args: HashMap::new(),
            purpose: "Get general hardware info".to_string(),
            required: true,
            evidence_id: "E1".to_string(),
        });
    }

    EvidencePlan {
        case_id: case_id.to_string(),
        tools,
        expected_evidence: (1..=evidence_id).map(|i| format!("E{}", i)).collect(),
        reasoning: format!("System query for {:?}", classification.query_target),
    }
}

/// Plan evidence for DIAGNOSE intent
fn plan_diagnose_evidence(case_id: &str, classification: &IntentClassification) -> EvidencePlan {
    let mut tools = Vec::new();

    // Base evidence depends on problem domain
    if let Some(domain) = &classification.problem_domain {
        match domain.as_str() {
            "network" => {
                tools.push(PlannedTool {
                    tool_name: "network_status".to_string(),
                    args: HashMap::new(),
                    purpose: "Get network interfaces and connectivity".to_string(),
                    required: true,
                    evidence_id: "E1".to_string(),
                });
                tools.push(PlannedTool {
                    tool_name: "nm_status".to_string(),
                    args: HashMap::new(),
                    purpose: "Get NetworkManager state".to_string(),
                    required: false,
                    evidence_id: "E2".to_string(),
                });
                tools.push(PlannedTool {
                    tool_name: "journalctl_network".to_string(),
                    args: HashMap::new(),
                    purpose: "Get network-related journal errors".to_string(),
                    required: false,
                    evidence_id: "E3".to_string(),
                });
            }
            "audio" => {
                tools.push(PlannedTool {
                    tool_name: "audio_status".to_string(),
                    args: HashMap::new(),
                    purpose: "Get PipeWire/audio stack status".to_string(),
                    required: true,
                    evidence_id: "E1".to_string(),
                });
                tools.push(PlannedTool {
                    tool_name: "pactl_info".to_string(),
                    args: HashMap::new(),
                    purpose: "Get PulseAudio/PipeWire sinks".to_string(),
                    required: false,
                    evidence_id: "E2".to_string(),
                });
            }
            "boot" => {
                tools.push(PlannedTool {
                    tool_name: "boot_timing".to_string(),
                    args: HashMap::new(),
                    purpose: "Get systemd-analyze output".to_string(),
                    required: true,
                    evidence_id: "E1".to_string(),
                });
                tools.push(PlannedTool {
                    tool_name: "boot_offenders".to_string(),
                    args: HashMap::new(),
                    purpose: "Get slow boot units".to_string(),
                    required: true,
                    evidence_id: "E2".to_string(),
                });
            }
            "storage" => {
                tools.push(PlannedTool {
                    tool_name: "mount_usage".to_string(),
                    args: HashMap::new(),
                    purpose: "Get disk usage".to_string(),
                    required: true,
                    evidence_id: "E1".to_string(),
                });
                tools.push(PlannedTool {
                    tool_name: "btrfs_status".to_string(),
                    args: HashMap::new(),
                    purpose: "Get BTRFS health if applicable".to_string(),
                    required: false,
                    evidence_id: "E2".to_string(),
                });
            }
            "graphics" => {
                tools.push(PlannedTool {
                    tool_name: "graphics_info".to_string(),
                    args: HashMap::new(),
                    purpose: "Get GPU/display info".to_string(),
                    required: true,
                    evidence_id: "E1".to_string(),
                });
            }
            _ => {
                // General diagnosis
                tools.push(PlannedTool {
                    tool_name: "hw_snapshot_summary".to_string(),
                    args: HashMap::new(),
                    purpose: "Get system overview".to_string(),
                    required: true,
                    evidence_id: "E1".to_string(),
                });
            }
        }
    }

    let expected_evidence: Vec<String> = tools.iter().map(|t| t.evidence_id.clone()).collect();
    EvidencePlan {
        case_id: case_id.to_string(),
        tools,
        expected_evidence,
        reasoning: format!("Diagnosis for {:?}", classification.problem_domain),
    }
}

/// Plan evidence for ACTION_REQUEST intent
fn plan_action_evidence(case_id: &str, classification: &IntentClassification) -> EvidencePlan {
    let mut tools = Vec::new();

    if let Some(action_type) = &classification.action_type {
        match action_type.as_str() {
            "package_install" | "package_remove" => {
                tools.push(PlannedTool {
                    tool_name: "package_info".to_string(),
                    args: HashMap::new(),
                    purpose: "Check if package exists/is installed".to_string(),
                    required: true,
                    evidence_id: "E1".to_string(),
                });
            }
            "service_action" => {
                tools.push(PlannedTool {
                    tool_name: "systemd_service_probe_v1".to_string(),
                    args: HashMap::new(),
                    purpose: "Get current service state".to_string(),
                    required: true,
                    evidence_id: "E1".to_string(),
                });
            }
            "file_edit" => {
                tools.push(PlannedTool {
                    tool_name: "file_stat".to_string(),
                    args: HashMap::new(),
                    purpose: "Get file info and permissions".to_string(),
                    required: true,
                    evidence_id: "E1".to_string(),
                });
            }
            _ => {}
        }
    }

    let expected_evidence: Vec<String> = tools.iter().map(|t| t.evidence_id.clone()).collect();
    EvidencePlan {
        case_id: case_id.to_string(),
        tools,
        expected_evidence,
        reasoning: format!("Action evidence for {:?}", classification.action_type),
    }
}

/// Plan evidence for HOWTO intent (minimal evidence needed)
fn plan_howto_evidence(case_id: &str, _classification: &IntentClassification) -> EvidencePlan {
    // HOWTO questions typically need knowledge search, not system evidence
    EvidencePlan {
        case_id: case_id.to_string(),
        tools: vec![PlannedTool {
            tool_name: "knowledge_search".to_string(),
            args: HashMap::new(),
            purpose: "Search local documentation".to_string(),
            required: false,
            evidence_id: "K1".to_string(),
        }],
        expected_evidence: vec!["K1".to_string()],
        reasoning: "HOWTO query - knowledge search".to_string(),
    }
}

/// Plan evidence for META intent
fn plan_meta_evidence(case_id: &str, _classification: &IntentClassification) -> EvidencePlan {
    // META queries need Anna's internal state
    EvidencePlan {
        case_id: case_id.to_string(),
        tools: vec![PlannedTool {
            tool_name: "anna_status".to_string(),
            args: HashMap::new(),
            purpose: "Get Anna's current status".to_string(),
            required: true,
            evidence_id: "E1".to_string(),
        }],
        expected_evidence: vec!["E1".to_string()],
        reasoning: "META query - internal status".to_string(),
    }
}

// ============================================================================
// Evidence Validation
// ============================================================================

/// Validate that evidence matches the expected target
pub fn validate_evidence_for_query(target: &QueryTarget, evidence: &str) -> (bool, String) {
    let evidence_lower = evidence.to_lowercase();

    match target {
        QueryTarget::Cpu => {
            let has_cpu = evidence_lower.contains("cpu")
                || evidence_lower.contains("processor")
                || evidence_lower.contains("amd")
                || evidence_lower.contains("intel")
                || evidence_lower.contains("ryzen")
                || evidence_lower.contains("core");
            if !has_cpu {
                return (false, "Evidence missing CPU information".to_string());
            }
            (true, String::new())
        }
        QueryTarget::Memory => {
            let has_mem = evidence_lower.contains("memory")
                || evidence_lower.contains("ram")
                || evidence_lower.contains("gib")
                || evidence_lower.contains("total");
            if !has_mem {
                return (false, "Evidence missing memory information".to_string());
            }
            (true, String::new())
        }
        QueryTarget::DiskFree => {
            let has_disk = evidence_lower.contains("free")
                || evidence_lower.contains("used")
                || evidence_lower.contains("mount")
                || evidence_lower.contains("/");
            if !has_disk {
                return (false, "Evidence missing disk space information".to_string());
            }
            (true, String::new())
        }
        QueryTarget::KernelVersion => {
            let has_kernel = evidence_lower.contains("kernel")
                || evidence_lower.contains("linux")
                || evidence.contains("6.")
                || evidence.contains("5.");
            if !has_kernel {
                return (false, "Evidence missing kernel version".to_string());
            }
            (true, String::new())
        }
        _ => (true, String::new()),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_cpu_query() {
        let classification = IntentClassification {
            intent: IntentType::SystemQuery,
            confidence: 95,
            matched_patterns: vec!["what cpu".to_string()],
            query_target: Some(QueryTarget::Cpu),
            problem_domain: None,
            action_type: None,
            reasoning: "test".to_string(),
        };
        let plan = plan_evidence("test-1", &classification);
        assert!(!plan.tools.is_empty());
        assert!(plan.tools[0].tool_name.contains("hw_snapshot") || plan.tools[0].tool_name.contains("cpu"));
    }

    #[test]
    fn test_plan_disk_query() {
        let classification = IntentClassification {
            intent: IntentType::SystemQuery,
            confidence: 95,
            matched_patterns: vec!["disk space".to_string()],
            query_target: Some(QueryTarget::DiskFree),
            problem_domain: None,
            action_type: None,
            reasoning: "test".to_string(),
        };
        let plan = plan_evidence("test-2", &classification);
        assert!(!plan.tools.is_empty());
        assert!(plan.tools[0].tool_name.contains("mount"));
    }

    #[test]
    fn test_validate_cpu_evidence() {
        let (valid, _) = validate_evidence_for_query(
            &QueryTarget::Cpu,
            "CPU: AMD Ryzen 7 5800X, 8 cores, 16 threads"
        );
        assert!(valid);

        let (valid, critique) = validate_evidence_for_query(
            &QueryTarget::Cpu,
            "Disk free: 100 GiB on /"
        );
        assert!(!valid);
        assert!(critique.contains("CPU"));
    }

    #[test]
    fn test_validate_disk_evidence() {
        let (valid, _) = validate_evidence_for_query(
            &QueryTarget::DiskFree,
            "Mount /: 100 GiB free, 50% used"
        );
        assert!(valid);

        let (valid, critique) = validate_evidence_for_query(
            &QueryTarget::DiskFree,
            "CPU: Intel Core i9"
        );
        assert!(!valid);
        assert!(critique.contains("disk"));
    }
}
