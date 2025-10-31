// Anna v0.10.1 - annactl capabilities command

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

const CAPABILITIES_PATH: &str = "/usr/lib/anna/CAPABILITIES.toml";
const MODULES_PATH: &str = "/etc/anna/modules.yaml";

#[derive(Debug, Deserialize, Serialize)]
pub struct CapabilityCheck {
    pub module_name: String,
    pub description: String,
    pub status: String,
    pub reason: Option<String>,
    pub action: Option<String>,
    pub impact: Option<String>,
    pub required: bool,
    pub enabled_by_user: bool,
    pub evidence: Vec<String>,
}

pub fn show_capabilities() -> Result<()> {
    // For now, we'll read the CAPABILITIES.toml and modules.yaml directly
    // In a real implementation, we'd call the daemon via RPC

    if !std::path::Path::new(CAPABILITIES_PATH).exists() {
        eprintln!("⚠ CAPABILITIES.toml not found at {}", CAPABILITIES_PATH);
        eprintln!("  Run: sudo ./scripts/install.sh");
        std::process::exit(20);
    }

    println!("\n╭─ Anna Module Capabilities ───────────────────────────────────────");
    println!("│");
    println!("│  Module          Status     Evidence                Required Deps");
    println!("│  ────────────────────────────────────────────────────────────────");

    // Parse CAPABILITIES.toml
    let content = fs::read_to_string(CAPABILITIES_PATH)?;
    let registry: toml::Value = toml::from_str(&content)?;

    if let Some(modules) = registry.get("modules").and_then(|m| m.as_table()) {
        // Sort: required first
        let mut module_list: Vec<_> = modules.iter().collect();
        module_list.sort_by(|a, b| {
            let a_req = a.1.get("required").and_then(|r| r.as_bool()).unwrap_or(false);
            let b_req = b.1.get("required").and_then(|r| r.as_bool()).unwrap_or(false);

            match (a_req, b_req) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.0.cmp(b.0),
            }
        });

        for (module_name, module_def) in module_list {
            let required = module_def.get("required").and_then(|r| r.as_bool()).unwrap_or(false);

            // Check if module's commands exist
            let commands: Vec<String> = module_def
                .get("deps")
                .and_then(|d| d.get("optional"))
                .and_then(|o| o.get("commands"))
                .and_then(|c| c.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let has_deps = commands.iter().all(|cmd| which::which(cmd).is_ok());

            let status = if has_deps {
                "✓ ACTIVE"
            } else if required {
                "⚠ DEGRADED"
            } else {
                "⚠ DEGRADED"
            };

            let evidence: Vec<String> = module_def
                .get("checks")
                .and_then(|c| c.get("evidence"))
                .and_then(|e| e.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let evidence_str = if !evidence.is_empty() {
                evidence[0].clone()
            } else {
                "system data".to_string()
            };

            let deps_str = if commands.is_empty() {
                "built-in".to_string()
            } else {
                commands.join(", ")
            };

            println!(
                "│  {:<16} {:>9}   {:<23} {}",
                module_name,
                status,
                &evidence_str[..evidence_str.len().min(23)],
                &deps_str[..deps_str.len().min(30)]
            );

            // Show degradation info
            if !has_deps {
                if let Some(degraded) = module_def.get("degraded") {
                    let reason = degraded.get("reason").and_then(|r| r.as_str()).unwrap_or("");
                    let action = degraded.get("action").and_then(|a| a.as_str()).unwrap_or("");

                    println!("│    └─ {}", reason);
                    if !action.is_empty() {
                        println!("│       Fix: {}", action);
                    }
                }
            }
        }
    }

    println!("│");
    println!("╰──────────────────────────────────────────────────────────────────");
    println!();
    println!("  ✓ ACTIVE    - Module fully functional");
    println!("  ⚠ DEGRADED  - Missing optional dependencies");
    println!("  ✗ DISABLED  - Disabled by user in /etc/anna/modules.yaml");
    println!();
    println!("  To disable a module: annactl module disable <name>");
    println!("  To enable a module:  annactl module enable <name>");
    println!();

    Ok(())
}
