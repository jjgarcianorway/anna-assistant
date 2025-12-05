//! Specialist prompt building for service desk.
//!
//! COST: Enforces prompt size cap with diagnostic surfacing.

use anna_shared::resource_limits::{ResourceDiagnostic, MAX_PROMPT_CHARS};
use anna_shared::rpc::{ProbeResult, RuntimeContext, SpecialistDomain};

/// Result of building a prompt (includes truncation diagnostic if capped)
#[derive(Debug)]
pub struct PromptResult {
    /// The built prompt (possibly truncated)
    pub prompt: String,
    /// Diagnostic if prompt was truncated
    pub diagnostic: Option<ResourceDiagnostic>,
    /// Whether prompt was truncated
    pub was_truncated: bool,
}

/// Build grounded system prompt with runtime context for specialist.
/// COST: Enforces MAX_PROMPT_CHARS cap - truncates probe results to fit.
pub fn build_specialist_prompt(
    domain: SpecialistDomain,
    context: &RuntimeContext,
    probe_results: &[ProbeResult],
) -> PromptResult {
    let (prompt, truncated_chars) = build_prompt_with_budget(domain, context, probe_results);

    let was_truncated = truncated_chars > 0;
    let diagnostic = if was_truncated {
        Some(ResourceDiagnostic::prompt_truncated(truncated_chars))
    } else {
        None
    };

    PromptResult {
        prompt,
        diagnostic,
        was_truncated,
    }
}

/// Grounding rules suffix (constant size, always included)
const GROUNDING_RULES: &str = r#"

=== GROUNDING RULES (MANDATORY) ===
1. NEVER invent or guess information not in the runtime context above.
2. For hardware questions: Answer DIRECTLY from the hardware section above.
3. For process/memory/disk questions: Use the probe results above if available.
4. If data is missing: Say exactly what data is missing.
5. NEVER suggest manual commands when you have the data above.
6. Use the exact version shown above when discussing Anna's version.
7. Reference the specific data you're using in your answer.

=== END CONTEXT ===

Answer using ONLY the data provided above. Be concise and direct."#;

/// Build prompt string, returning (prompt, chars_truncated)
fn build_prompt_with_budget(
    domain: SpecialistDomain,
    context: &RuntimeContext,
    probe_results: &[ProbeResult],
) -> (String, usize) {
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

    // Build base prompt (intro + hardware)
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

    // Calculate budget for probe results
    // Budget = MAX_PROMPT_CHARS - base_prompt - grounding_rules - margin
    let base_len = prompt.len();
    let rules_len = GROUNDING_RULES.len();
    let margin = 200; // Buffer for headers and formatting
    let probe_budget = MAX_PROMPT_CHARS.saturating_sub(base_len + rules_len + margin);

    // Add probe results within budget
    let mut truncated_chars: usize = 0;
    if !probe_results.is_empty() {
        prompt.push_str("\n\n=== PROBE RESULTS ===");
        let mut probe_chars_used = 0;

        for probe in probe_results {
            let probe_text = if probe.exit_code == 0 {
                format!("\n[{}]\n{}", probe.command, probe.stdout)
            } else {
                format!(
                    "\n[{}] FAILED (exit {}): {}",
                    probe.command, probe.exit_code, probe.stderr
                )
            };

            let probe_len = probe_text.len();
            if probe_chars_used + probe_len <= probe_budget {
                prompt.push_str(&probe_text);
                probe_chars_used += probe_len;
            } else {
                // Truncate this probe to fit remaining budget
                let remaining = probe_budget.saturating_sub(probe_chars_used);
                if remaining > 50 {
                    // Only include if meaningful space remains
                    let truncated = &probe_text[..remaining.min(probe_text.len())];
                    prompt.push_str(truncated);
                    prompt.push_str("\n... (truncated)");
                    truncated_chars += probe_len - remaining;
                } else {
                    truncated_chars += probe_len;
                }
                // Skip remaining probes
                break;
            }
        }

        // Count chars from skipped probes
        let probes_added = probe_results
            .iter()
            .take_while(|p| {
                let text = if p.exit_code == 0 {
                    format!("\n[{}]\n{}", p.command, p.stdout)
                } else {
                    format!("\n[{}] FAILED: {}", p.command, p.stderr)
                };
                probe_chars_used >= text.len()
            })
            .count();

        for probe in probe_results.iter().skip(probes_added + 1) {
            let probe_text = if probe.exit_code == 0 {
                format!("\n[{}]\n{}", probe.command, probe.stdout)
            } else {
                format!("\n[{}] FAILED: {}", probe.command, probe.stderr)
            };
            truncated_chars += probe_text.len();
        }
    }

    prompt.push_str(GROUNDING_RULES);

    (prompt, truncated_chars)
}
