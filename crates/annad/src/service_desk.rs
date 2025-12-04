//! Service desk architecture with internal roles.
//!
//! Roles (internal, not CLI commands):
//! - Translator: converts user text to structured ticket
//! - Dispatcher: Anna, routes to specialist
//! - Specialist: domain expert (system, network, storage, security, packages)
//! - Supervisor: validates output, requests more probes, assigns reliability score

use crate::probes;
use anna_shared::rpc::{
    Capabilities, HardwareSummary, RuntimeContext, ServiceDeskResult, SpecialistDomain,
};
use anna_shared::VERSION;
use std::collections::HashMap;

/// Allowlist of read-only probes that specialists can request
pub const ALLOWED_PROBES: &[&str] = &[
    "ps aux --sort=-%mem",
    "ps aux --sort=-%cpu",
    "lscpu",
    "free -h",
    "df -h",
    "lsblk",
    "ip addr show",
    "ip route",
    "ss -tulpn",
    "systemctl --failed",
    "journalctl -p warning..alert -n 200 --no-pager",
];

/// Check if a probe is in the allowlist
pub fn is_probe_allowed(probe: &str) -> bool {
    ALLOWED_PROBES.iter().any(|p| probe.starts_with(p))
}

/// Translator role: Classify query into specialist domain
pub fn translate_to_domain(query: &str) -> SpecialistDomain {
    let q = query.to_lowercase();

    if q.contains("network")
        || q.contains("ip ")
        || q.contains("interface")
        || q.contains("dns")
        || q.contains("ping")
        || q.contains("route")
        || q.contains("port")
        || q.contains("socket")
        || q.contains("connection")
    {
        return SpecialistDomain::Network;
    }

    if q.contains("disk")
        || q.contains("storage")
        || q.contains("space")
        || q.contains("mount")
        || q.contains("partition")
        || q.contains("filesystem")
        || q.contains("lsblk")
        || q.contains("df ")
    {
        return SpecialistDomain::Storage;
    }

    if q.contains("security")
        || q.contains("firewall")
        || q.contains("permission")
        || q.contains("selinux")
        || q.contains("apparmor")
        || q.contains("audit")
        || q.contains("fail2ban")
        || q.contains("ssh")
    {
        return SpecialistDomain::Security;
    }

    if q.contains("package")
        || q.contains("install")
        || q.contains("pacman")
        || q.contains("apt")
        || q.contains("dnf")
        || q.contains("yum")
        || q.contains("update")
        || q.contains("upgrade")
    {
        return SpecialistDomain::Packages;
    }

    // Default to system for CPU, memory, processes, services
    SpecialistDomain::System
}

/// Dispatcher role: Determine which probes are needed based on domain and query
pub fn dispatch_probes(domain: SpecialistDomain, query: &str) -> Vec<String> {
    let q = query.to_lowercase();
    let mut probes = Vec::new();

    match domain {
        SpecialistDomain::System => {
            if q.contains("memory") || q.contains("ram") || q.contains("process") {
                probes.push("ps aux --sort=-%mem".to_string());
                probes.push("free -h".to_string());
            }
            if q.contains("cpu") {
                if q.contains("process") || q.contains("using") || q.contains("top") {
                    probes.push("ps aux --sort=-%cpu".to_string());
                }
                probes.push("lscpu".to_string());
            }
            if q.contains("service") || q.contains("failed") || q.contains("systemd") {
                probes.push("systemctl --failed".to_string());
            }
            if q.contains("error") || q.contains("warning") || q.contains("log") {
                probes.push("journalctl -p warning..alert -n 200 --no-pager".to_string());
            }
        }
        SpecialistDomain::Network => {
            probes.push("ip addr show".to_string());
            if q.contains("route") || q.contains("gateway") {
                probes.push("ip route".to_string());
            }
            if q.contains("port") || q.contains("listen") || q.contains("socket") {
                probes.push("ss -tulpn".to_string());
            }
        }
        SpecialistDomain::Storage => {
            probes.push("df -h".to_string());
            if q.contains("disk") || q.contains("partition") || q.contains("device") {
                probes.push("lsblk".to_string());
            }
        }
        SpecialistDomain::Security => {
            if q.contains("port") || q.contains("listen") {
                probes.push("ss -tulpn".to_string());
            }
            probes.push("journalctl -p warning..alert -n 200 --no-pager".to_string());
        }
        SpecialistDomain::Packages => {
            // Package queries usually don't need probes
        }
    }

    probes
}

/// Run allowed probes and return results
pub fn run_allowed_probes(probe_commands: &[String]) -> HashMap<String, String> {
    let mut results = HashMap::new();

    for cmd in probe_commands {
        if !is_probe_allowed(cmd) {
            results.insert(
                cmd.clone(),
                format!("DENIED: probe '{}' not in allowlist", cmd),
            );
            continue;
        }

        let result = probes::run_command(cmd);
        results.insert(
            cmd.clone(),
            result.unwrap_or_else(|e| format!("ERROR: {}", e)),
        );
    }

    results
}

/// Build runtime context for LLM
pub fn build_context(
    hardware: &anna_shared::status::HardwareInfo,
    probe_results: &HashMap<String, String>,
) -> RuntimeContext {
    RuntimeContext {
        version: VERSION.to_string(),
        daemon_running: true,
        capabilities: Capabilities::default(),
        hardware: HardwareSummary {
            cpu_model: hardware.cpu_model.clone(),
            cpu_cores: hardware.cpu_cores,
            ram_gb: hardware.ram_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
            gpu: hardware.gpu.as_ref().map(|g| g.model.clone()),
            gpu_vram_gb: hardware
                .gpu
                .as_ref()
                .map(|g| g.vram_bytes as f64 / (1024.0 * 1024.0 * 1024.0)),
        },
        probes: probe_results.clone(),
    }
}

/// Check if query is ambiguous and needs clarification
pub fn check_ambiguity(query: &str) -> Option<String> {
    let q = query.to_lowercase();

    // Very short queries are often ambiguous
    if q.split_whitespace().count() <= 2 && !q.contains("cpu") && !q.contains("memory") {
        return Some("Could you provide more details about what you'd like to know?".to_string());
    }

    // "Help" without context
    if q == "help" || q == "help me" {
        return Some("What specifically do you need help with?".to_string());
    }

    None
}

/// Build grounded system prompt with runtime context for specialist
pub fn build_specialist_prompt(
    domain: SpecialistDomain,
    context: &RuntimeContext,
    probe_results: &HashMap<String, String>,
) -> String {
    let specialist_intro = match domain {
        SpecialistDomain::System => {
            "You are the System Specialist, expert in CPU, memory, processes, and services."
        }
        SpecialistDomain::Network => {
            "You are the Network Specialist, expert in interfaces, routing, DNS, and connectivity."
        }
        SpecialistDomain::Storage => {
            "You are the Storage Specialist, expert in disks, partitions, mounts, and filesystems."
        }
        SpecialistDomain::Security => {
            "You are the Security Specialist, expert in permissions, firewalls, and audit logs."
        }
        SpecialistDomain::Packages => {
            "You are the Package Specialist, expert in package managers and software installation."
        }
    };

    let mut prompt = format!(
        r#"You are Anna, a local AI assistant running on this Linux machine.
{specialist_intro}

=== RUNTIME CONTEXT (AUTHORITATIVE - DO NOT CONTRADICT) ===
Version: {}
Daemon: running

Hardware (from system probe):
  - CPU: {} ({} cores)
  - RAM: {:.1} GB"#,
        context.version,
        context.hardware.cpu_model,
        context.hardware.cpu_cores,
        context.hardware.ram_gb,
    );

    if let Some(gpu) = &context.hardware.gpu {
        if let Some(vram) = context.hardware.gpu_vram_gb {
            prompt.push_str(&format!("\n  - GPU: {} ({:.1} GB VRAM)", gpu, vram));
        } else {
            prompt.push_str(&format!("\n  - GPU: {}", gpu));
        }
    } else {
        prompt.push_str("\n  - GPU: none");
    }

    // Add probe results if any
    if !probe_results.is_empty() {
        prompt.push_str("\n\n=== PROBE RESULTS ===");
        for (name, result) in probe_results {
            prompt.push_str(&format!("\n[{}]\n{}", name, result));
        }
    }

    prompt.push_str(
        r#"

=== GROUNDING RULES (MANDATORY) ===
1. NEVER invent or guess information not in the runtime context above.
2. For hardware questions: Answer DIRECTLY from the hardware section above.
3. For process/memory/disk questions: Use the probe results above if available.
4. If data is missing: Say exactly what data is missing.
5. NEVER suggest manual commands when you have the data above.
6. Use the exact version shown above when discussing Anna's version.

=== END CONTEXT ===

Answer using ONLY the data provided above. Be concise and direct."#,
    );

    prompt
}

/// Supervisor role: Estimate reliability score based on context
pub fn estimate_reliability(
    _query: &str,
    probe_results: &HashMap<String, String>,
    domain: SpecialistDomain,
) -> u8 {
    let mut score: u8 = 50; // Base score

    // More probes = higher reliability
    let successful_probes = probe_results
        .values()
        .filter(|v| !v.starts_with("ERROR:") && !v.starts_with("DENIED:"))
        .count();

    score = score.saturating_add((successful_probes * 10).min(30) as u8);

    // Domain-specific adjustments
    match domain {
        SpecialistDomain::System | SpecialistDomain::Storage => {
            // These domains have good probe support
            if successful_probes > 0 {
                score = score.saturating_add(10);
            }
        }
        SpecialistDomain::Network => {
            if probe_results.contains_key("ip addr show") {
                score = score.saturating_add(10);
            }
        }
        SpecialistDomain::Packages => {
            // Package queries without probes are less reliable
            score = score.saturating_sub(10);
        }
        SpecialistDomain::Security => {
            // Security requires careful verification
            score = score.saturating_sub(5);
        }
    }

    score.min(95) // Cap at 95, never claim 100% reliability
}

/// Create a clarification response
pub fn create_clarification_response(question: &str) -> ServiceDeskResult {
    ServiceDeskResult {
        answer: String::new(),
        reliability_score: 0,
        domain: SpecialistDomain::System,
        probes_used: vec![],
        needs_clarification: true,
        clarification_question: Some(question.to_string()),
    }
}
