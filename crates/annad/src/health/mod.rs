//! Health subsystem for system monitoring
//!
//! Phase 0.5: Operational Core - reactive monitoring with Arch Wiki compliance
//! Citation: [archwiki:System_maintenance]

mod probes;
mod runner;

pub use probes::{HealthProbe, ProbeResult, ProbeStatus, DiskSpaceProbe, SystemdUnitsProbe, PacmanDbProbe};
pub use runner::{run_all_probes, run_probe, get_health_summary, HealthSummary};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Health probe definition from YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeDefinition {
    pub name: String,
    pub description: String,
    pub citation: String,
    #[serde(default)]
    pub thresholds: Option<ProbeThresholds>,
    pub command: String,
    pub args: Vec<String>,
    pub timeout: u64,
    pub parse_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeThresholds {
    pub warn: u32,
    pub fail: u32,
}

/// Load all health probe definitions from assets directory
pub fn load_probe_definitions() -> Result<Vec<ProbeDefinition>> {
    let probe_dir = get_health_assets_dir();
    let mut definitions = Vec::new();

    if !probe_dir.exists() {
        // Fallback to embedded definitions if assets directory doesn't exist
        return Ok(get_embedded_probes());
    }

    for entry in std::fs::read_dir(&probe_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            let content = std::fs::read_to_string(&path)?;
            let definition: ProbeDefinition = serde_yaml::from_str(&content)?;
            definitions.push(definition);
        }
    }

    Ok(definitions)
}

/// Get health assets directory path
fn get_health_assets_dir() -> PathBuf {
    // Check multiple possible locations
    let possible_paths = vec![
        PathBuf::from("/usr/local/lib/anna/health"),
        PathBuf::from("./assets/health"),
        PathBuf::from("../assets/health"),
        PathBuf::from("../../assets/health"),
    ];

    for path in possible_paths {
        if path.exists() {
            return path;
        }
    }

    // Default fallback
    PathBuf::from("/usr/local/lib/anna/health")
}

/// Get embedded probe definitions as fallback
fn get_embedded_probes() -> Vec<ProbeDefinition> {
    vec![
        ProbeDefinition {
            name: "disk-space".to_string(),
            description: "Check filesystem disk space usage".to_string(),
            citation: "[archwiki:System_maintenance#Check_for_errors]".to_string(),
            thresholds: Some(ProbeThresholds { warn: 80, fail: 95 }),
            command: "df".to_string(),
            args: vec!["-h".to_string(), "/".to_string()],
            timeout: 5,
            parse_type: "df_output".to_string(),
        },
        ProbeDefinition {
            name: "systemd-units".to_string(),
            description: "Check for failed systemd units".to_string(),
            citation: "[archwiki:Systemd#Basic_systemctl_usage]".to_string(),
            thresholds: None,
            command: "systemctl".to_string(),
            args: vec![
                "--failed".to_string(),
                "--no-pager".to_string(),
                "--no-legend".to_string(),
            ],
            timeout: 10,
            parse_type: "systemd_failed".to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_embedded_probes() {
        let probes = get_embedded_probes();
        assert!(!probes.is_empty());
        assert!(probes.iter().any(|p| p.name == "disk-space"));
    }

    #[test]
    fn test_probe_definition_serialization() {
        let probe = ProbeDefinition {
            name: "test".to_string(),
            description: "Test probe".to_string(),
            citation: "[test]".to_string(),
            thresholds: Some(ProbeThresholds { warn: 80, fail: 95 }),
            command: "echo".to_string(),
            args: vec!["test".to_string()],
            timeout: 5,
            parse_type: "simple".to_string(),
        };

        let json = serde_json::to_string(&probe).unwrap();
        assert!(json.contains("test"));
    }
}
