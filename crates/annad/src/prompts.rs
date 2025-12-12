//! Specialist prompt building for service desk (v0.0.383).
//!
//! v0.0.260: Added OS info to context.
//! v0.0.322: Improved grounding rules to prevent hallucination.
//! v0.0.324: Added self-assessment for answer quality learning.
//! v0.0.375: Added user preferences context for personalized responses.
//! v0.0.377: Added learned success hints from past high-quality answers.
//! v0.0.380: Added context memory for cross-session continuity.
//! v0.0.383: Added distro-aware package manager context.
//! COST: Enforces prompt size cap with diagnostic surfacing.

use anna_shared::context_memory::ContextMemory;
use anna_shared::distro_utils::DistroContext;
use anna_shared::probe_learning::{ProbeLearningStore, QueryCategory};
use anna_shared::resource_limits::{ResourceDiagnostic, MAX_PROMPT_CHARS};
use anna_shared::rpc::{ProbeResult, RuntimeContext, SpecialistDomain};
use anna_shared::user_profile::ResponsePreferences;

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
/// v0.0.322: Strengthened to prevent hallucination and off-topic answers
/// v0.0.324: Added self-assessment for learning
const GROUNDING_RULES: &str = r#"

=== GROUNDING RULES (MANDATORY) ===
1. Read the user's question CAREFULLY. Answer EXACTLY what they asked.
2. Use ONLY the data in the RUNTIME CONTEXT and PROBE RESULTS above.
3. NEVER invent information. If data is missing, say "I don't have that data."
4. NEVER suggest commands unless the user explicitly asked how to do something.
5. Be concise and direct. Don't over-explain.
6. If probe results are empty or missing, acknowledge this honestly.
7. Respect User Preferences - match the technical depth and verbosity they prefer.

CRITICAL: Your answer must be RELEVANT to what the user asked.
If the user asks about X, answer about X, not Y.

SELF-ASSESSMENT (end of response):
After your answer, on a new line, add: [QUALITY: X/5]
where X is your honest assessment:
5 = Complete answer from probe data
4 = Good answer, minor gaps
3 = Partial answer, missing some data
2 = Limited answer, probes didn't provide needed info
1 = Could not answer properly

=== END CONTEXT ==="#;

/// Build prompt string, returning (prompt, chars_truncated)
fn build_prompt_with_budget(
    domain: SpecialistDomain,
    context: &RuntimeContext,
    probe_results: &[ProbeResult],
) -> (String, usize) {
    // v0.0.405: Expanded for all domains
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
        SpecialistDomain::Boot => {
            "You are the Boot Specialist, expert in startup, systemd boot analysis, and initialization."
        }
        SpecialistDomain::Services => {
            "You are the Services Specialist, expert in systemd units, daemons, and service management."
        }
        SpecialistDomain::Audio => {
            "You are the Audio Specialist, expert in PulseAudio, PipeWire, ALSA, and sound devices."
        }
        SpecialistDomain::Display => {
            "You are the Display Specialist, expert in monitors, resolution, GPUs, and graphics drivers."
        }
        SpecialistDomain::Desktop => {
            "You are the Desktop Specialist, expert in window managers, desktop environments, and sessions."
        }
    };

    // Build base prompt (intro + hardware + OS)
    let mut prompt = format!(
        r#"You are Anna, a local AI assistant running on this Linux machine.
{specialist_intro}

=== RUNTIME CONTEXT (AUTHORITATIVE - DO NOT CONTRADICT) ===
Version: {}
Daemon: running

System:
  - OS: {}"#,
        context.version,
        if context.hardware.distro.is_empty() {
            context.hardware.os_name.clone()
        } else {
            context.hardware.distro.clone()
        },
    );

    // Add kernel if available
    if !context.hardware.kernel.is_empty() {
        prompt.push_str(&format!("\n  - Kernel: {}", context.hardware.kernel));
    }

    prompt.push_str(&format!(
        r#"

Hardware (from system probe):
  - CPU: {} ({} cores)
  - RAM: {:.1} GB"#,
        context.hardware.cpu_model, context.hardware.cpu_cores, context.hardware.ram_gb,
    ));

    if let Some(gpu) = &context.hardware.gpu {
        if let Some(vram) = context.hardware.gpu_vram_gb {
            prompt.push_str(&format!("\n  - GPU: {} ({:.1} GB VRAM)", gpu, vram));
        } else {
            prompt.push_str(&format!("\n  - GPU: {}", gpu));
        }
    } else {
        prompt.push_str("\n  - GPU: none");
    }

    // v0.0.375: Add user preferences for personalized responses
    let prefs = ResponsePreferences::load();
    prompt.push_str(&format!(
        "\n\nUser Preferences:\n  - Technical depth: {}\n  - Verbosity: {}",
        prefs.technical_depth_desc(),
        prefs.verbosity_desc()
    ));

    // v0.0.377: Add learned success hints from past high-quality answers
    let store = ProbeLearningStore::load();
    // v0.0.405: Use Display trait for domain->string conversion
    let domain_str = domain.to_string();
    let domain_str = domain_str.as_str();
    let category = QueryCategory::from_domain(domain_str);
    let hints = store.recent_success_hints(&category);
    if !hints.is_empty() {
        prompt.push_str(&format!(
            "\n\nLearned Context (from past successes):\n  Keywords that worked well: {}",
            hints.join(", ")
        ));
    }

    // v0.0.380: Add context memory for cross-session continuity
    let memory = ContextMemory::load();
    let memory_hints = build_context_memory_hints(&memory, domain_str);
    if !memory_hints.is_empty() {
        prompt.push_str(&format!("\n\nUser Context:\n{}", memory_hints));
    }

    // v0.0.383: Add distro-aware package manager context
    let distro_name = if context.hardware.distro.is_empty() {
        &context.hardware.os_name
    } else {
        &context.hardware.distro
    };
    let distro_ctx = DistroContext::from_distro(distro_name);
    if distro_ctx.is_known() {
        let pm = distro_ctx.pm();
        prompt.push_str(&format!(
            "\n\nPackage Manager ({}):\n  - Install: {}\n  - Search: {}\n  - Update: {}",
            pm.name(),
            pm.install_command("<pkg>"),
            pm.search_command("<query>"),
            pm.upgrade_command()
        ));
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

/// v0.0.380: Build context memory hints for specialist prompt
fn build_context_memory_hints(memory: &ContextMemory, domain: &str) -> String {
    let mut hints = Vec::new();

    // Check if user frequently asks about this domain
    if let Some(pattern) = memory.patterns.get(domain) {
        if pattern.is_frequent() {
            hints.push(format!(
                "  - User asks about {} frequently ({} times)",
                domain, pattern.count
            ));
        }
        if pattern.is_recent() && pattern.count > 1 {
            hints.push(format!(
                "  - User asked about {} recently - may want more detail",
                domain
            ));
        }
    }

    // Add editor preference if relevant
    if let Some(ref editor) = memory.preferred_editor {
        if domain == "system" || domain == "packages" {
            hints.push(format!("  - User's preferred editor: {}", editor));
        }
    }

    // Add mastered commands context (so we don't over-explain)
    if !memory.mastered_commands.is_empty() {
        let relevant: Vec<_> = memory
            .mastered_commands
            .iter()
            .filter(|cmd| is_domain_relevant_command(cmd, domain))
            .take(3)
            .collect();
        if !relevant.is_empty() {
            hints.push(format!(
                "  - User knows: {} (no need to explain basics)",
                relevant
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    // Add learned preferences
    for pref in memory.preferences.iter().take(2) {
        if pref.confidence >= 0.6 {
            hints.push(format!("  - {}", pref.preference));
        }
    }

    hints.join("\n")
}

/// Check if a command is relevant to a domain
fn is_domain_relevant_command(cmd: &str, domain: &str) -> bool {
    match domain {
        "storage" => ["df", "du", "mount", "lsblk", "fdisk"]
            .iter()
            .any(|c| cmd.contains(c)),
        "network" => ["ip", "ping", "netstat", "ss", "curl"]
            .iter()
            .any(|c| cmd.contains(c)),
        "system" => ["ps", "top", "htop", "systemctl", "journalctl"]
            .iter()
            .any(|c| cmd.contains(c)),
        "security" => ["chmod", "chown", "sudo", "firewall"]
            .iter()
            .any(|c| cmd.contains(c)),
        "packages" => ["pacman", "apt", "dnf", "yum", "pip"]
            .iter()
            .any(|c| cmd.contains(c)),
        _ => false,
    }
}
