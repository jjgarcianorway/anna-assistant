//! Specialist prompt building for service desk.

use anna_shared::rpc::{ProbeResult, RuntimeContext, SpecialistDomain};

/// Build grounded system prompt with runtime context for specialist
pub fn build_specialist_prompt(
    domain: SpecialistDomain,
    context: &RuntimeContext,
    probe_results: &[ProbeResult],
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
        for probe in probe_results {
            if probe.exit_code == 0 {
                prompt.push_str(&format!("\n[{}]\n{}", probe.command, probe.stdout));
            } else {
                prompt.push_str(&format!(
                    "\n[{}] FAILED (exit {}): {}",
                    probe.command, probe.exit_code, probe.stderr
                ));
            }
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
7. Reference the specific data you're using in your answer.

=== END CONTEXT ===

Answer using ONLY the data provided above. Be concise and direct."#,
    );

    prompt
}
