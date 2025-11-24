//! Unified Query Handler - Version 150 (Beta.204: Determinism Lock)
//!
//! Single source of truth for ALL query processing (CLI and TUI).
//! Fixes inconsistent responses between modes.
//!
//! ## Query Processing Architecture (Priority Order)
//!
//! **TIER 0: System Report** (lines 64-75)
//! - Intercepts "full report" queries
//! - Returns verified system telemetry
//! - 100% deterministic ✅
//!
//! **TIER 1: Deterministic Recipes** (lines 77-98)
//! - 77 hard-coded, tested ActionPlans
//! - Zero hallucination, consistent output
//! - 100% deterministic ✅
//!
//! **TIER 2: Template Matching** (lines 100-117)
//! - Simple command templates (query_handler.rs)
//! - Fast, accurate for simple queries
//! - 100% deterministic ✅
//!
//! **TIER 3: V3 JSON Dialogue** (lines 119-135)
//! - LLM-based ActionPlan generation
//! - For complex actionable requests
//! - Non-deterministic (by design) ❌
//!
//! **TIER 4: Conversational Answer** (lines 137-180)
//! - **Deterministic path**: try_answer_from_telemetry() (lines 183-297) ✅
//!   - Fixed templates using SystemTelemetry data
//!   - CPU, RAM, disk, services queries
//! - **LLM fallback**: For complex questions (lines 159-179) ❌
//!   - Used when telemetry doesn't cover the query
//!
//! ## Determinism Guarantees (Beta.204)
//!
//! **Deterministic Query Types** (same input + same system state = same output):
//! - System telemetry (CPU, RAM, disk, GPU, network status)
//! - Failed services list
//! - Package management (install, update, clean cache via recipes)
//! - Service management (enable, start, stop, logs via recipes)
//! - System updates (via system_update recipe)
//!
//! **Non-Deterministic Query Types** (may vary between runs):
//! - Complex procedures (boot repair, Xorg configuration, networking setup)
//! - Questions requiring decision-making based on multiple factors
//! - Troubleshooting queries with unknown failure modes
//!
//! **Design Philosophy**:
//! Maximize determinism for common system management tasks while gracefully
//! degrading to LLM reasoning for legitimately complex questions.

use anna_common::action_plan_v3::ActionPlan;
use anna_common::llm::LlmConfig;
use anna_common::telemetry::SystemTelemetry;
use anyhow::Result;

use crate::dialogue_v3_json;
use crate::output;
use crate::query_handler;

/// Unified query result - all query types in one enum
/// Version 150: ALL responses are now structured JSON - no freeform streaming
#[derive(Debug)]
pub enum UnifiedQueryResult {
    /// Beta.151 deterministic recipe matched
    DeterministicRecipe {
        recipe_name: String,
        action_plan: ActionPlan,
    },
    /// Template matched - instant result
    Template {
        command: String,
        output: String,
    },
    /// V3 JSON dialogue - structured action plan
    ActionPlan {
        action_plan: ActionPlan,
        raw_json: String,
    },
    /// Conversational answer - structured response for info queries
    /// Version 150: Replaces freeform streaming with structured JSON
    ConversationalAnswer {
        answer: String,
        confidence: AnswerConfidence,
        sources: Vec<String>,
    },
}

/// Answer confidence level
#[derive(Debug, Clone, Copy)]
pub enum AnswerConfidence {
    High,    // From telemetry/system data
    Medium,  // From LLM with validation
    Low,     // From LLM without validation
}

/// Unified query handler - single entry point for all queries
///
/// This ensures CLI and TUI get IDENTICAL responses for the same question.
/// Beta.228: Added comprehensive logging
pub async fn handle_unified_query(
    user_text: &str,
    telemetry: &SystemTelemetry,
    llm_config: &LlmConfig,
) -> Result<UnifiedQueryResult> {

    // ========================================================================
    // Beta.277: NL Routing Stability Rules
    // ========================================================================
    //
    // Rule A - Mutual Exclusion:
    //   If a query matches status, it NEVER falls into diagnostic.
    //   Enforced by: checking status (TIER 0) before diagnostic (TIER 0.5)
    //
    // Rule B - Conversational Catch-All:
    //   If no route matches, fallback is ALWAYS conversational.
    //   Enforced by: returning conversational at end of function (TIER 4)
    //
    // Rule C - Diagnostic Requires Clear System Intent:
    //   Diagnostic only matches when query contains system keywords:
    //   system | machine | computer | health | diagnostic | check | errors | problems | issues
    //   Enforced by: is_ambiguous_query() check inside is_full_diagnostic_query()
    //
    // Priority Order: status → diagnostic → proactive → recipe → template → conversational
    // ========================================================================

    // TIER 0: System Report (Version 150 - highest priority, no LLM)
    // Intercept "full report" queries to prevent hallucination
    // Beta.277: Rule A enforced - status checked FIRST
    if crate::system_report::is_system_report_query(user_text) {
        let report = crate::system_report::generate_full_report()
            .unwrap_or_else(|e| format!("Error generating system report: {}", e));

        return Ok(UnifiedQueryResult::ConversationalAnswer {
            answer: report,
            confidence: AnswerConfidence::High,
            sources: vec!["verified system telemetry (direct system query)".to_string()],
        });
    }

    // TIER 0.5: Beta.238 Full Diagnostic Routing (natural language → brain diagnostics)
    // Detects "run a full diagnostic", "check my system health", "show any problems"
    // Routes directly to internal diagnostic engine (same as hidden brain command)
    // Beta.257: Pass query for temporal wording support
    if is_full_diagnostic_query(user_text) {
        return handle_diagnostic_query(user_text).await;
    }

    // TIER 0.6: Beta.273 Proactive Status Summary Routing
    // Detects "show proactive status", "summarize top issues", "summarize findings"
    // Routes to proactive engine for status summary
    if is_proactive_status_query(user_text) {
        return handle_proactive_status_query(user_text).await;
    }

    // TIER 0.6: Beta.272 Proactive Remediation Routing
    // Detects "what should I fix first", "what are my top issues", "any critical problems"
    // Routes to proactive engine for correlated issue remediation
    if is_proactive_remediation_query(user_text) {
        return handle_proactive_remediation_query(user_text).await;
    }

    // TIER 0.7: Beta.278 Sysadmin Report Routing
    // Detects "full sysadmin report", "give me a full system report", "overall situation"
    // Generates comprehensive sysadmin briefing combining health, proactive, and session data
    if is_sysadmin_report_query(user_text) {
        return handle_sysadmin_report_query(user_text).await;
    }

    // 6.3.1: Recipe system removed - planner is the only planning path
    // Old TIER 1 recipe matching disabled:
    // let telemetry_map = telemetry_to_hashmap(telemetry);
    // if let Some(recipe_result) = crate::recipes::try_recipe_match(user_text, &telemetry_map) {
    //     return Ok(UnifiedQueryResult::DeterministicRecipe { recipe_name, action_plan });
    // }

    // TIER 2: Template Matching (fast, accurate for simple queries)
    if let Some((template_id, params)) = query_handler::try_template_match(user_text) {
        match query_handler::execute_template(template_id, &params) {
            Ok(query_handler::QueryResult::Template {
                template_id: _,
                command,
                output,
            }) => {
                return Ok(UnifiedQueryResult::Template {
                    command,
                    output,
                });
            }
            _ => {
                // Template execution failed, continue to next tier
            }
        }
    } else {
    }

    // TIER 3: V3 JSON Dialogue (structured action plans with LLM)
    // Version 150: RE-ENABLED - V3 dialogue now works with SystemTelemetry
    // Check if this looks like an actionable request
    if should_use_action_plan(user_text) {
        let v3_start = std::time::Instant::now();
        match dialogue_v3_json::run_dialogue_v3_json(user_text, telemetry, llm_config).await {
            Ok(result) => {
                return Ok(UnifiedQueryResult::ActionPlan {
                    action_plan: result.action_plan,
                    raw_json: result.raw_json,
                });
            }
            Err(e) => {
                // V3 dialogue failed, fall through to conversational answer
            }
        }
    } else {
    }

    // TIER 4: Structured Conversational Answer (Version 150: NO STREAMING!)
    // Generate structured answer from telemetry or LLM
    generate_conversational_answer(user_text, telemetry, llm_config).await
}

/// Generate structured conversational answer (Version 150: enforces JSON)
/// Beta.228: Added comprehensive logging
async fn generate_conversational_answer(
    user_text: &str,
    telemetry: &SystemTelemetry,
    llm_config: &LlmConfig,
) -> Result<UnifiedQueryResult> {
    use anna_common::llm::{LlmClient, LlmPrompt};
    use std::sync::Arc;

    // Try to answer directly from telemetry first (highest confidence)
    // Beta.208: Apply canonical normalization to all answers
    if let Some(answer) = try_answer_from_telemetry(user_text, telemetry) {
        return Ok(UnifiedQueryResult::ConversationalAnswer {
            answer: output::normalize_for_cli(&answer),
            confidence: AnswerConfidence::High,
            sources: vec!["system telemetry (direct system query)".to_string()],
        });
    }

    // Use LLM for complex conversational queries
    let client = Arc::new(LlmClient::from_config(llm_config)
        .map_err(|e| anyhow::anyhow!("LLM not available: {}", e))?);

    let prompt = build_conversational_prompt_for_tui(user_text, telemetry);
    let llm_prompt = Arc::new(LlmPrompt {
        system: LlmClient::anna_system_prompt().to_string(),
        user: prompt,
        conversation_history: None,
    });

    // Beta.229: Use streaming for word-by-word output
    let llm_start = std::time::Instant::now();
    let client_clone = Arc::clone(&client);
    let prompt_clone = Arc::clone(&llm_prompt);

    let response_text = tokio::task::spawn_blocking(move || {
        let call_start = std::time::Instant::now();
        let mut accumulated = String::new();

        // Stream the response, printing each word as it arrives
        let result = client_clone.chat_stream(&prompt_clone, &mut |chunk: &str| {
            print!("{}", chunk);
            use std::io::Write;
            let _ = std::io::stdout().flush();
            accumulated.push_str(chunk);
        });


        match result {
            Ok(()) => Ok(accumulated),
            Err(e) => Err(e),
        }
    })
    .await
    .map_err(|e| {
        anyhow::anyhow!("LLM task panicked: {}", e)
    })?
    .map_err(|e| {
        anyhow::anyhow!("LLM query failed: {}", e)
    })?;


    Ok(UnifiedQueryResult::ConversationalAnswer {
        answer: output::normalize_for_cli(&response_text),
        confidence: AnswerConfidence::Medium,
        sources: vec!["LLM".to_string()],
    })
}

/// Try to answer question directly from telemetry (zero-latency, high confidence)
fn try_answer_from_telemetry(user_text: &str, telemetry: &SystemTelemetry) -> Option<String> {
    let query_lower = user_text.to_lowercase();

    // Anna's personality/design philosophy queries
    if query_lower.contains("your") && query_lower.contains("personality") {
        return Some(
            "I'm Anna, your Arch Linux assistant. My design philosophy:\n\n\
            • **Telemetry-first**: I gather real system data before answering\n\
            • **Deterministic recipes**: 77 zero-hallucination command sequences\n\
            • **Safety-conscious**: User confirmation for write operations\n\
            • **Transparent**: I explain what I'm doing and why\n\
            • **Local-first**: I run locally with optional LLM support\n\
            • **Minimalist**: Three-command simplicity (TUI, status, query)\n\n\
            My goal is reliable, explainable system management with zero surprises.".to_string()
        );
    }

    // User profile/usage pattern queries - use Context Engine data
    if (query_lower.contains("describe me") || query_lower.contains("what kind of user")
        || query_lower.contains("my personality"))
        && (query_lower.contains("trait") || query_lower.contains("am i")
            || query_lower.contains("as a user") || query_lower.contains("my usage")) {
        // Load context engine for usage patterns
        if let Ok(ctx) = crate::context_engine::ContextEngine::load() {
            let total_commands = ctx.usage_patterns.command_frequency.values().sum::<u32>();
            let top_commands: Vec<_> = {
                let mut commands: Vec<_> = ctx.usage_patterns.command_frequency.iter().collect();
                commands.sort_by(|a, b| b.1.cmp(a.1));
                commands.into_iter().take(3).collect()
            };

            let profile = if total_commands < 10 {
                "New user - still exploring Anna's capabilities"
            } else if total_commands < 50 {
                "Regular user - building familiarity with system administration"
            } else {
                "Power user - confident with system management"
            };

            let mut response = format!("Based on your usage patterns: {}\n\n", profile);
            if !top_commands.is_empty() {
                response.push_str("Most frequent commands:\n");
                for (cmd, count) in top_commands {
                    response.push_str(&format!("  • {} ({} times)\n", cmd, count));
                }
            }
            response.push_str(&format!("\nTotal interactions: {}", total_commands));

            return Some(response);
        }
    }

    // Beta.207: CPU queries (deterministic telemetry - SUMMARY only)
    if query_lower.contains("cpu") && (query_lower.contains("what") || query_lower.contains("model")) {
        return Some(format!(
            "[SUMMARY]\n\
            Your CPU is a {} with {} cores. Current load: {:.2} (1-min avg).",
            telemetry.hardware.cpu_model,
            telemetry.cpu.cores,
            telemetry.cpu.load_avg_1min
        ));
    }

    // Beta.207: RAM queries (deterministic telemetry - SUMMARY only)
    if (query_lower.contains("ram") || query_lower.contains("memory"))
        && (query_lower.contains("how much") || query_lower.contains("total")) {
        let used_gb = telemetry.memory.used_mb as f64 / 1024.0;
        let total_gb = telemetry.memory.total_mb as f64 / 1024.0;
        let percent = (used_gb / total_gb) * 100.0;
        return Some(format!(
            "[SUMMARY]\n\
            You have {:.1} GB of RAM total. Currently using {:.1} GB ({:.1}%).",
            total_gb, used_gb, percent
        ));
    }

    // Beta.207: Disk space queries (deterministic telemetry - SUMMARY only)
    if (query_lower.contains("disk") || query_lower.contains("storage") || query_lower.contains("space"))
        && (query_lower.contains("free") || query_lower.contains("available") || query_lower.contains("how much")) {
        if let Some(root_disk) = telemetry.disks.iter().find(|d| d.mount_point == "/") {
            let free_gb = (root_disk.total_mb - root_disk.used_mb) as f64 / 1024.0;
            let total_gb = root_disk.total_mb as f64 / 1024.0;
            return Some(format!(
                "[SUMMARY]\n\
                Your root partition (/) has {:.1} GB free out of {:.1} GB total ({:.1}% used).",
                free_gb, total_gb, root_disk.usage_percent
            ));
        }
    }

    // Beta.207: Disk space troubleshooting (SUMMARY + DETAILS + COMMANDS)
    if (query_lower.contains("disk") || query_lower.contains("space"))
        && (query_lower.contains("error") || query_lower.contains("full")
            || query_lower.contains("using") || query_lower.contains("find what")) {
        let disk_summary: Vec<String> = telemetry.disks.iter()
            .map(|d| format!("  {} - {:.1}% used ({:.1} GB / {:.1} GB)",
                d.mount_point,
                d.usage_percent,
                d.used_mb as f64 / 1024.0,
                d.total_mb as f64 / 1024.0))
            .collect();

        return Some(format!(
            "[SUMMARY]\n\
            Finding what's using disk space.\n\n\
            [DETAILS]\n\
            Current disk usage:\n{}\n\n\
            [COMMANDS]\n\
            $ df -h\n\
            $ sudo du -sh /* | sort -h\n\
            $ sudo find / -type f -size +100M -exec ls -lh {{}} \\;\n\
            $ du -sh /var/cache/pacman/pkg/\n\
            $ sudo pacman -Sc",
            disk_summary.join("\n")
        ));
    }

    // Beta.207: GPU queries (deterministic telemetry - SUMMARY only)
    if query_lower.contains("gpu") {
        if let Some(ref gpu) = telemetry.hardware.gpu_info {
            return Some(format!("[SUMMARY]\nYour GPU is: {}", gpu));
        } else {
            return Some("[SUMMARY]\nNo GPU detected on this system.".to_string());
        }
    }

    // Beta.207: Failed services (deterministic telemetry - SUMMARY only)
    if query_lower.contains("failed") && query_lower.contains("service") {
        if telemetry.services.failed_units.is_empty() {
            return Some("[SUMMARY]\nNo failed services detected.".to_string());
        } else {
            let failed_list: Vec<String> = telemetry.services.failed_units
                .iter()
                .map(|u| format!("- {}", u.name))
                .collect();
            return Some(format!(
                "[SUMMARY]\n\
                {} failed service(s):\n{}",
                telemetry.services.failed_units.len(),
                failed_list.join("\n")
            ));
        }
    }

    // Beta.207: RAM and swap usage queries with structured fallback
    if (query_lower.contains("swap") || (query_lower.contains("ram") && query_lower.contains("swap")))
        && !query_lower.contains("how much") && !query_lower.contains("total") {
        let ram_used_gb = telemetry.memory.used_mb as f64 / 1024.0;
        let ram_total_gb = telemetry.memory.total_mb as f64 / 1024.0;
        let ram_percent = (ram_used_gb / ram_total_gb) * 100.0;

        // Check if swap exists
        match std::process::Command::new("swapon").arg("--show").output() {
            Ok(output) if output.status.success() => {
                if let Ok(swap_output) = String::from_utf8(output.stdout) {
                    if swap_output.is_empty() || swap_output.lines().count() <= 1 {
                        return Some(format!(
                            "[SUMMARY]\n\
                            RAM: {:.1} GB / {:.1} GB ({:.1}% used). Swap not configured.\n\n\
                            [DETAILS]\n\
                            No active swap space detected. Adding swap can prevent out-of-memory errors.\n\n\
                            [COMMANDS]\n\
                            $ sudo fallocate -l 4G /swapfile\n\
                            $ sudo chmod 600 /swapfile\n\
                            $ sudo mkswap /swapfile\n\
                            $ sudo swapon /swapfile\n\
                            $ echo '/swapfile none swap defaults 0 0' | sudo tee -a /etc/fstab",
                            ram_used_gb, ram_total_gb, ram_percent
                        ));
                    } else {
                        let swap_lines: Vec<&str> = swap_output.lines().skip(1).collect();
                        let swap_info = if !swap_lines.is_empty() {
                            swap_lines.join("\n")
                        } else {
                            "No active swap".to_string()
                        };
                        return Some(format!(
                            "[SUMMARY]\n\
                            RAM: {:.1} GB / {:.1} GB ({:.1}% used).\n\n\
                            [DETAILS]\n\
                            Swap status:\n{}",
                            ram_used_gb, ram_total_gb, ram_percent, swap_info
                        ));
                    }
                }
            }
            _ => {
                return Some(format!(
                    "[SUMMARY]\n\
                    Unable to check swap status.\n\n\
                    [DETAILS]\n\
                    Reason: swapon command failed or not found\n\
                    Missing: util-linux package\n\
                    RAM: {:.1} GB / {:.1} GB ({:.1}% used)\n\n\
                    [COMMANDS]\n\
                    $ pacman -Qi util-linux\n\
                    $ sudo pacman -S util-linux",
                    ram_used_gb, ram_total_gb, ram_percent
                ));
            }
        }
    }

    // Beta.207: GPU VRAM usage queries with structured fallback
    if query_lower.contains("vram") || (query_lower.contains("gpu") && query_lower.contains("memory")) {
        // Try nvidia-smi for NVIDIA GPUs
        if let Ok(output) = std::process::Command::new("nvidia-smi")
            .args(&["--query-gpu=memory.used,memory.total", "--format=csv,noheader,nounits"])
            .output() {
            if output.status.success() {
                if let Ok(result) = String::from_utf8(output.stdout) {
                    let parts: Vec<&str> = result.trim().split(',').collect();
                    if parts.len() == 2 {
                        let used_mb: f64 = parts[0].trim().parse().unwrap_or(0.0);
                        let total_mb: f64 = parts[1].trim().parse().unwrap_or(0.0);
                        let percent = if total_mb > 0.0 { (used_mb / total_mb) * 100.0 } else { 0.0 };
                        return Some(format!(
                            "[SUMMARY]\n\
                            GPU VRAM: {:.0} MB / {:.0} MB ({:.1}% used)",
                            used_mb, total_mb, percent
                        ));
                    }
                }
            }
        }

        // Try radeontop for AMD GPUs
        if let Some(ref gpu) = telemetry.hardware.gpu_info {
            if gpu.to_lowercase().contains("amd") || gpu.to_lowercase().contains("radeon") {
                return Some(format!(
                    "[SUMMARY]\n\
                    AMD GPU detected: {}\n\n\
                    [DETAILS]\n\
                    AMD GPU VRAM usage requires radeontop package.\n\n\
                    [COMMANDS]\n\
                    $ sudo pacman -S radeontop\n\
                    $ sudo radeontop -d - -l 1",
                    gpu
                ));
            }
        }

        return Some(
            "[SUMMARY]\n\
            Unable to determine GPU VRAM usage.\n\n\
            [DETAILS]\n\
            Reason: No GPU detected or VRAM query tools not installed\n\
            Missing: nvidia-smi (NVIDIA) or radeontop (AMD)\n\n\
            [COMMANDS]\n\
            $ lspci | grep -i vga\n\
            $ sudo pacman -S nvidia-utils\n\
            $ nvidia-smi --query-gpu=memory.used,memory.total --format=csv".to_string()
        );
    }

    // Beta.207: CPU governor status queries with structured fallback
    if query_lower.contains("cpu") && (query_lower.contains("governor") || query_lower.contains("frequency")) {
        match std::process::Command::new("cat")
            .arg("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor")
            .output() {
            Ok(output) if output.status.success() => {
                if let Ok(governor) = String::from_utf8(output.stdout) {
                    let governor = governor.trim();
                    let available_governors = std::process::Command::new("cat")
                        .arg("/sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors")
                        .output()
                        .ok()
                        .and_then(|o| String::from_utf8(o.stdout).ok())
                        .map(|s| s.trim().to_string())
                        .unwrap_or_else(|| "unknown".to_string());

                    return Some(format!(
                        "[SUMMARY]\n\
                        CPU Governor: {}\n\n\
                        [DETAILS]\n\
                        Available governors: {}\n\n\
                        [COMMANDS]\n\
                        $ sudo cpupower frequency-set -g <governor>",
                        governor, available_governors
                    ));
                }
            }
            _ => {
                return Some(
                    "[SUMMARY]\n\
                    Unable to determine CPU governor status.\n\n\
                    [DETAILS]\n\
                    Reason: CPU frequency scaling sysfs not accessible\n\
                    Missing: cpufreq kernel module or cpupower tools\n\n\
                    [COMMANDS]\n\
                    $ ls /sys/devices/system/cpu/cpu0/cpufreq/\n\
                    $ sudo pacman -S cpupower\n\
                    $ sudo modprobe acpi-cpufreq".to_string()
                );
            }
        }
    }

    // Beta.207: Systemd units list queries with structured fallback
    if (query_lower.contains("systemd") || query_lower.contains("service"))
        && (query_lower.contains("list") || query_lower.contains("all") || query_lower.contains("running")) {
        let unit_type = if query_lower.contains("timer") {
            "timer"
        } else if query_lower.contains("socket") {
            "socket"
        } else {
            "service"
        };

        match std::process::Command::new("systemctl")
            .args(&["list-units", &format!("--type={}", unit_type), "--state=running", "--no-pager", "--no-legend"])
            .output() {
            Ok(output) if output.status.success() => {
                if let Ok(result) = String::from_utf8(output.stdout) {
                    let running_units: Vec<&str> = result.lines().take(20).collect();
                    let total_count = result.lines().count();

                    return Some(format!(
                        "[SUMMARY]\n\
                        Running {} units: {} total, showing first 20.\n\n\
                        [DETAILS]\n\
                        {}\n\n\
                        [COMMANDS]\n\
                        $ systemctl list-units --type={}",
                        unit_type, total_count, running_units.join("\n"), unit_type
                    ));
                }
            }
            _ => {
                return Some(format!(
                    "[SUMMARY]\n\
                    Unable to list systemd {} units.\n\n\
                    [DETAILS]\n\
                    Reason: systemctl command failed or not found\n\
                    Missing: systemd package\n\n\
                    [COMMANDS]\n\
                    $ pacman -Qi systemd\n\
                    $ sudo pacman -S systemd",
                    unit_type
                ));
            }
        }
    }

    // Beta.207: NVMe/SSD health queries with structured fallback
    if (query_lower.contains("nvme") || query_lower.contains("ssd"))
        && (query_lower.contains("health") || query_lower.contains("smart") || query_lower.contains("status")) {
        // List NVMe devices
        let nvme_list = std::process::Command::new("sh")
            .arg("-c")
            .arg("ls /dev/nvme* 2>/dev/null | grep -E 'nvme[0-9]+$' || true")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        if nvme_list.is_empty() {
            return Some(
                "[SUMMARY]\n\
                No NVMe devices detected.\n\n\
                [DETAILS]\n\
                Reason: No /dev/nvme* devices found\n\
                System may have SATA SSDs instead\n\n\
                [COMMANDS]\n\
                $ lsblk -d -o NAME,ROTA\n\
                $ sudo pacman -S smartmontools\n\
                $ sudo smartctl -a /dev/sda".to_string()
            );
        }

        let devices: Vec<&str> = nvme_list.lines().collect();
        let mut health_reports = Vec::new();

        for device in devices.iter().take(3) {
            if let Ok(output) = std::process::Command::new("sudo")
                .args(&["nvme", "smart-log", device])
                .output() {
                if output.status.success() {
                    if let Ok(result) = String::from_utf8(output.stdout) {
                        health_reports.push(format!("{}:\n{}", device, result));
                    }
                }
            }
        }

        if health_reports.is_empty() {
            return Some(format!(
                "[SUMMARY]\n\
                NVMe devices detected but health data unavailable.\n\n\
                [DETAILS]\n\
                Reason: nvme command not found or requires sudo\n\
                Missing: nvme-cli package\n\
                Devices found: {}\n\n\
                [COMMANDS]\n\
                $ sudo pacman -S nvme-cli\n\
                $ sudo nvme smart-log /dev/nvme0",
                devices.join(", ")
            ));
        }

        return Some(format!("[SUMMARY]\nNVMe health report for {} device(s).\n\n[DETAILS]\n{}", devices.len(), health_reports.join("\n\n")));
    }

    // Beta.207: fstrim status queries with structured fallback
    if query_lower.contains("trim") || query_lower.contains("fstrim") {
        match std::process::Command::new("systemctl")
            .args(&["is-enabled", "fstrim.timer"])
            .output() {
            Ok(enabled_output) => {
                let timer_status = String::from_utf8(enabled_output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                let timer_active = std::process::Command::new("systemctl")
                    .args(&["is-active", "fstrim.timer"])
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                let last_run = std::process::Command::new("sh")
                    .arg("-c")
                    .arg("journalctl -u fstrim.service -n 1 --no-pager 2>/dev/null | grep -E '(Started|Finished)' || echo 'No recent runs'")
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .unwrap_or_else(|| "Unknown".to_string());

                return Some(format!(
                    "[SUMMARY]\n\
                    fstrim.timer status: {} / {}\n\n\
                    [DETAILS]\n\
                    - Enabled: {}\n\
                    - Active: {}\n\
                    - Last run: {}\n\n\
                    [COMMANDS]\n\
                    $ sudo systemctl enable --now fstrim.timer",
                    timer_status, timer_active, timer_status, timer_active, last_run.trim()
                ));
            }
            _ => {
                return Some(
                    "[SUMMARY]\n\
                    Unable to check fstrim.timer status.\n\n\
                    [DETAILS]\n\
                    Reason: systemctl command failed or fstrim.timer not found\n\
                    Missing: util-linux package or systemd\n\n\
                    [COMMANDS]\n\
                    $ pacman -Qi util-linux\n\
                    $ sudo pacman -S util-linux\n\
                    $ systemctl list-timers".to_string()
                );
            }
        }
    }

    // Beta.207: Network interface queries with structured fallback
    if (query_lower.contains("network") || query_lower.contains("interface") || query_lower.contains("ip"))
        && (query_lower.contains("list") || query_lower.contains("show") || query_lower.contains("what")) {
        match std::process::Command::new("ip")
            .args(&["-brief", "address"])
            .output() {
            Ok(output) if output.status.success() => {
                if let Ok(interfaces) = String::from_utf8(output.stdout) {
                    return Some(format!(
                        "[SUMMARY]\n\
                        Network interfaces on system.\n\n\
                        [DETAILS]\n\
                        {}\n\n\
                        [COMMANDS]\n\
                        $ ip addr show",
                        interfaces
                    ));
                }
            }
            _ => {
                return Some(
                    "[SUMMARY]\n\
                    Unable to list network interfaces.\n\n\
                    [DETAILS]\n\
                    Reason: ip command failed or not found\n\
                    Missing: iproute2 package\n\n\
                    [COMMANDS]\n\
                    $ pacman -Qi iproute2\n\
                    $ sudo pacman -S iproute2".to_string()
                );
            }
        }
    }

    // Beta.207: arch-019 - Package file search queries with structured fallback
    if (query_lower.contains("which") || query_lower.contains("what") || query_lower.contains("find"))
        && query_lower.contains("package")
        && (query_lower.contains("provides")
            || query_lower.contains("contains")
            || query_lower.contains("owns")
            || query_lower.contains("owning")
            || query_lower.contains("file"))
    {
        // Try to extract a file path or binary name from the query
        // Common patterns: "which package provides <file>", "what package contains <binary>"
        let potential_file = query_lower
            .split_whitespace()
            .last()
            .unwrap_or("")
            .trim_matches(|c: char| !c.is_alphanumeric() && c != '/' && c != '.' && c != '-' && c != '_');

        if !potential_file.is_empty() && potential_file.len() > 1 {
            // First try: pacman -Qo for installed files (full path)
            if potential_file.starts_with('/') {
                match std::process::Command::new("pacman")
                    .arg("-Qo")
                    .arg(potential_file)
                    .output()
                {
                    Ok(output) if output.status.success() => {
                        if let Ok(pacman_output) = String::from_utf8(output.stdout) {
                            let lines: Vec<&str> = pacman_output.trim().lines().collect();
                            if !lines.is_empty() {
                                return Some(format!(
                                    "[SUMMARY]\n\
                                    File {} is owned by an installed package.\n\n\
                                    [DETAILS]\n\
                                    {}\n\n\
                                    [COMMANDS]\n\
                                    $ pacman -Qo {}",
                                    potential_file, lines.join("\n"), potential_file
                                ));
                            }
                        }
                    }
                    Ok(output) if !output.status.success() => {
                        // File not found or not owned by a package, try pacman -F
                        match std::process::Command::new("pacman")
                            .arg("-F")
                            .arg(potential_file.trim_start_matches('/'))
                            .output()
                        {
                            Ok(search_output) if search_output.status.success() => {
                                if let Ok(search_result) = String::from_utf8(search_output.stdout) {
                                    let matches: Vec<&str> = search_result.trim().lines().take(10).collect();
                                    if !matches.is_empty() {
                                        return Some(format!(
                                            "[SUMMARY]\n\
                                            File {} not currently installed, but available in {} package(s).\n\n\
                                            [DETAILS]\n\
                                            {}\n\n\
                                            [COMMANDS]\n\
                                            $ pacman -F {}",
                                            potential_file,
                                            matches.len(),
                                            matches.join("\n"),
                                            potential_file.trim_start_matches('/')
                                        ));
                                    }
                                }
                            }
                            _ => {
                                return Some(
                                    format!(
                                        "[SUMMARY]\n\
                                        Unable to find package for file {}.\n\n\
                                        [DETAILS]\n\
                                        Reason: pacman -F database not available or file not indexed\n\
                                        Missing: File database may need updating\n\n\
                                        To enable file search:\n\
                                        [COMMANDS]\n\
                                        $ sudo pacman -Fy\n\
                                        $ pacman -F {}",
                                        potential_file,
                                        potential_file.trim_start_matches('/')
                                    )
                                );
                            }
                        }
                    }
                    _ => {
                        return Some(
                            "[SUMMARY]\n\
                            Unable to query package database.\n\n\
                            [DETAILS]\n\
                            Reason: pacman command failed or not found\n\
                            Missing: pacman package manager\n\n\
                            [COMMANDS]\n\
                            $ which pacman\n\
                            $ pacman --version"
                                .to_string(),
                        );
                    }
                }
            } else {
                // Binary or filename (no leading /): try pacman -F directly
                match std::process::Command::new("pacman")
                    .arg("-F")
                    .arg(potential_file)
                    .output()
                {
                    Ok(output) if output.status.success() => {
                        if let Ok(search_result) = String::from_utf8(output.stdout) {
                            let matches: Vec<&str> = search_result.trim().lines().take(10).collect();
                            if !matches.is_empty() {
                                return Some(format!(
                                    "[SUMMARY]\n\
                                    Found {} package(s) providing {}.\n\n\
                                    [DETAILS]\n\
                                    {}\n\n\
                                    [COMMANDS]\n\
                                    $ pacman -F {}",
                                    matches.len(),
                                    potential_file,
                                    matches.join("\n"),
                                    potential_file
                                ));
                            } else {
                                return Some(format!(
                                    "[SUMMARY]\n\
                                    No packages found providing {}.\n\n\
                                    [DETAILS]\n\
                                    The file database contains no matches for this filename.\n\
                                    It may be:\n\
                                    - A file not provided by any Arch package\n\
                                    - A typo in the filename\n\
                                    - An AUR package (not indexed by pacman -F)\n\n\
                                    [COMMANDS]\n\
                                    $ pacman -F {}\n\
                                    $ sudo pacman -Fy  # Update file database",
                                    potential_file, potential_file
                                ));
                            }
                        }
                    }
                    _ => {
                        return Some(
                            format!(
                                "[SUMMARY]\n\
                                Unable to search package file database.\n\n\
                                [DETAILS]\n\
                                Reason: pacman -F command failed or file database not available\n\
                                Missing: File database may need initialization\n\n\
                                To enable file search:\n\
                                [COMMANDS]\n\
                                $ sudo pacman -Fy\n\
                                $ pacman -F {}",
                                potential_file
                            )
                        );
                    }
                }
            }
        }
    }

    None
}

/// Determine if query should use ActionPlan mode
fn should_use_action_plan(user_text: &str) -> bool {
    let input_lower = user_text.to_lowercase();

    // Action keywords
    let action_keywords = [
        "install",
        "setup",
        "configure",
        "fix",
        "repair",
        "update",
        "upgrade",
        "enable",
        "disable",
        "start",
        "stop",
        "restart",
        "create",
        "delete",
        "remove",
        "change",
        "modify",
    ];

    action_keywords
        .iter()
        .any(|keyword| input_lower.contains(keyword))
}

/// Build conversational prompt for streaming LLM
/// Beta.280: Made public for TUI streaming support
pub fn build_conversational_prompt_for_tui(user_text: &str, telemetry: &SystemTelemetry) -> String {
    let hostname = std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "localhost".to_string());

    format!(
        "You are Anna, an Arch Linux system administrator assistant.\n\n\
         System Context:\n\
         - CPU: {}\n\
         - RAM: {:.1} GB used / {:.1} GB total\n\
         - Hostname: {}\n\n\
         User Question: {}\n\n\
         Provide a helpful, concise answer. If you don't know, say so.",
        telemetry.hardware.cpu_model,
        telemetry.memory.used_mb as f64 / 1024.0,
        telemetry.memory.total_mb as f64 / 1024.0,
        hostname,
        user_text
    )
}

/// Convert SystemTelemetry to HashMap for recipes
fn telemetry_to_hashmap(telemetry: &SystemTelemetry) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();

    // Hardware
    map.insert("cpu_model".to_string(), telemetry.hardware.cpu_model.clone());
    map.insert("cpu_cores".to_string(), telemetry.cpu.cores.to_string());
    map.insert("total_ram_gb".to_string(), (telemetry.hardware.total_ram_mb as f64 / 1024.0).to_string());
    if let Some(ref gpu) = telemetry.hardware.gpu_info {
        map.insert("gpu_model".to_string(), gpu.clone());
    }

    // System (hostname and kernel from system calls)
    map.insert(
        "hostname".to_string(),
        std::process::Command::new("hostname")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "localhost".to_string()),
    );
    map.insert(
        "kernel".to_string(),
        std::process::Command::new("uname")
            .arg("-r")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string()),
    );
    map.insert("user".to_string(), std::env::var("USER").unwrap_or_else(|_| "user".to_string()));
    map.insert("home".to_string(), std::env::var("HOME").unwrap_or_else(|_| "~".to_string()));

    // Disk space (iterate directly on Vec, calculate available_gb)
    if let Some(root_disk) = telemetry.disks.iter().find(|d| d.mount_point == "/") {
        let available_gb = (root_disk.total_mb - root_disk.used_mb) as f64 / 1024.0;
        map.insert("disk_free_gb".to_string(), available_gb.to_string());
        map.insert("disk_used_percent".to_string(), root_disk.usage_percent.to_string());
    }

    // Internet connectivity
    map.insert("internet_connected".to_string(), telemetry.network.is_connected.to_string());

    // Desktop environment (use correct field names)
    if let Some(ref desktop) = telemetry.desktop {
        if let Some(ref de) = desktop.de_name {
            map.insert("desktop_environment".to_string(), de.clone());
        }
        if let Some(ref wm) = desktop.wm_name {
            map.insert("window_manager".to_string(), wm.clone());
        }
        if let Some(ref display) = desktop.display_server {
            map.insert("display_protocol".to_string(), display.clone());
        }
    }

    map
}

// Beta.208: normalize_answer() function removed - use output::normalizer module instead
// All normalization is now handled by:
//   - normalizer::normalize_for_cli() - for CLI output
//   - normalizer::normalize_for_tui() - for TUI output (before ANSI codes)
// This ensures consistent formatting across all answer types.

/// Beta.243: Normalize query text for consistent matching
/// Beta.254: Enhanced with punctuation, emoji, and polite fluff removal
///
/// Applies bounded, conservative normalization:
/// - Converts to lowercase
/// - Strips repeated trailing punctuation (???, !!!, ..., etc.)
/// - Strips single trailing punctuation (?, ., !)
/// - Strips trailing emojis (🙂, 😊, 😅, 😉, 🤔, 👍, ✅)
/// - Strips polite fluff at start (please, hey, hi, hello)
/// - Strips polite fluff at end (please, thanks, thank you)
/// - Collapses multiple whitespace to single space
/// - Trims leading/trailing whitespace
///
/// Scope: Only leading/trailing noise removed. No in-sentence words deleted.
/// Public to allow reuse by system_report.rs
pub fn normalize_query_for_intent(text: &str) -> String {
    // Convert to lowercase first
    let mut normalized = text.to_lowercase();

    // Beta.254: Strip repeated trailing punctuation (???, !!!, ..., etc.)
    while normalized.ends_with("???") || normalized.ends_with("!!!") || normalized.ends_with("...") ||
          normalized.ends_with("??") || normalized.ends_with("!!") || normalized.ends_with("..") ||
          normalized.ends_with("?!") || normalized.ends_with("!?") {
        normalized = normalized[..normalized.len()-2].to_string();
    }

    // Strip single trailing punctuation (?, ., !)
    normalized = normalized.trim_end_matches(|c| c == '?' || c == '.' || c == '!').to_string();

    // Beta.254: Strip trailing emojis (common simple ones)
    let trailing_emojis = ["🙂", "😊", "😅", "😉", "🤔", "👍", "✅"];
    for emoji in &trailing_emojis {
        if normalized.ends_with(emoji) {
            normalized = normalized[..normalized.len() - emoji.len()].trim_end().to_string();
        }
    }

    // Beta.254: Strip polite fluff at start (separated by whitespace)
    let polite_prefixes = ["please ", "hey ", "hi ", "hello "];
    for prefix in &polite_prefixes {
        if normalized.starts_with(prefix) {
            normalized = normalized[prefix.len()..].to_string();
        }
    }

    // Beta.254: Strip polite fluff at end (separated by whitespace)
    let polite_suffixes = [" please", " thanks", " thank you"];
    for suffix in &polite_suffixes {
        if normalized.ends_with(suffix) {
            normalized = normalized[..normalized.len() - suffix.len()].to_string();
        }
    }

    // Normalize hyphens and underscores to spaces for word boundary matching
    normalized = normalized.replace('-', " ").replace('_', " ");

    // Collapse multiple whitespace to single space
    let mut result = String::new();
    let mut prev_was_space = false;

    for c in normalized.chars() {
        if c.is_whitespace() {
            if !prev_was_space {
                result.push(' ');
                prev_was_space = true;
            }
        } else {
            result.push(c);
            prev_was_space = false;
        }
    }

    result.trim().to_string()
}

/// Beta.238: Detect if query is requesting full system diagnostics
/// Beta.239: Expanded phrase coverage
/// Beta.243: Whitespace/punctuation normalization, phrase variations, bi-directional matching
/// Beta.244: Conceptual question exclusions, contextual system references
///
/// Recognized phrases:
/// - "run a full diagnostic" / "run diagnostic" / "perform diagnostic" / "execute diagnostic"
/// - "check my system health" / "check the system health" / "check system health"
/// - "show any problems"
/// - "system health check"
/// - "full system diagnostic"
/// - "diagnose my system" / "diagnose the system"
/// - "health check" / "check health" (Beta.243: bi-directional)
/// - "system check" / "check system" (Beta.243: bi-directional)
/// - "health report"
/// - "system report"
/// - "is my system ok" / "is the system ok" (Beta.243: the/my variation)
/// - "my system ok" / "system ok" (Beta.243: terse patterns)
/// - "is everything ok with my system"
///
/// Beta.277: Ambiguity Resolution Framework
/// ========================================
///
/// Detects queries that contain diagnostic keywords but lack system context,
/// or contain human/existential language that makes intent ambiguous.
///
/// A query is ambiguous if:
/// - It contains diagnostic keywords ("problems", "issues", "errors")
/// - But NO system context keywords
/// - AND contains human-context or existential language
///
/// Examples:
/// - "Any problems?" → ambiguous (no system context)
/// - "Problems in my life?" → ambiguous (human context)
/// - "Any problems with my system?" → NOT ambiguous (has system context)
fn is_ambiguous_query(normalized: &str) -> bool {
    // System context keywords (Rule C requirement)
    let system_keywords = [
        "system", "machine", "computer", "health", "diagnostic", "check",
        "server", "host", "pc", "laptop", "hardware", "software",
    ];

    // Check if query has system context
    let has_system_context = system_keywords.iter().any(|kw| normalized.contains(kw));

    // If has clear system context, not ambiguous
    if has_system_context {
        return false;
    }

    // Diagnostic keywords that could be ambiguous without context
    let diagnostic_keywords = ["problems", "issues", "errors", "failures", "warnings"];
    let has_diagnostic_keyword = diagnostic_keywords.iter().any(|kw| normalized.contains(kw));

    // Human/existential context indicators
    let human_context = [
        "life", "my day", "my situation", "feeling", "i think", "i feel",
        "personally", "in general", "theoretically", "existential",
        "philosophical", "mentally", "emotionally",
    ];

    let has_human_context = human_context.iter().any(|ctx| normalized.contains(ctx));

    // Ambiguous if: has diagnostic keyword BUT no system context AND has human context
    if has_diagnostic_keyword && !has_system_context && has_human_context {
        return true;
    }

    // Short queries without system context are ambiguous ONLY if very short (2 words or less)
    // Examples: "Any problems?" → ambiguous, "Show problems" → NOT ambiguous (has "show" action verb)
    if has_diagnostic_keyword && !has_system_context {
        let word_count = normalized.split_whitespace().count();
        if word_count <= 2 {
            return true;
        }
    }

    false
}

/// Beta.278: Sysadmin Report v1 - Full System Briefing Detection
/// ==============================================================
///
/// Detects queries requesting a comprehensive sysadmin-style briefing
/// that combines health, diagnostics, proactive issues, and session info.
///
/// This is distinct from:
/// - Pure health checks ("check my system health") → diagnostic
/// - Domain-specific queries ("check my network") → sysadmin domain answers
/// - Educational queries ("what is a healthy system") → conversational
///
/// Sysadmin report queries ask for a **broad overview** of the system state,
/// like what a senior sysadmin would want at the start of a shift.
///
/// Examples that SHOULD match:
/// - "full system report"
/// - "sysadmin report"
/// - "overall situation on this system"
/// - "summarize the state of my machine"
/// - "what's the current situation on this box"
/// - "give me a full status report"
///
/// Examples that should NOT match:
/// - "check my system health" → goes to diagnostic
/// - "check my network" → goes to domain-specific sysadmin composer
/// - "what is a healthy system" → goes to conversational (educational)
/// - "status" → goes to system_report (lighter status check)
pub fn is_sysadmin_report_query(user_text: &str) -> bool {
    let normalized = normalize_query_for_intent(user_text);

    // Exact multi-word phrases indicating sysadmin report intent
    let exact_patterns = [
        // Core sysadmin report phrases
        "sysadmin report",
        "full sysadmin report",
        "complete sysadmin report",
        "sysadmin style report",
        "sysadmin briefing",
        "system admin report",
        "system administrator report",

        // Full system report variants
        "full system report",
        "complete system report",
        "full status report",
        "complete status report",
        "comprehensive system report",
        "detailed system report",

        // Overall/summary phrasing
        "overall situation",
        "overall system situation",
        "overall machine situation",
        "overall computer situation",
        "current situation",
        "system situation",
        "machine situation",

        // Summarize variants
        "summarize the system",
        "summarize my system",
        "summarize this system",
        "summarize the machine",
        "summarize my machine",
        "summarize this machine",
        "summarize the state",
        "summarize system state",
        "summarize machine state",

        // "Give me" imperatives
        "give me a full report",
        "give me a system report",
        "give me a full system report",
        "give me an overview",
        "give me a full overview",
        "give me the full picture",
        "give me everything",

        // "Show me" imperatives
        "show me a full report",
        "show me the full system report",
        "show me everything",
        "show me the full picture",
        "show me the complete picture",

        // What's going on (broad scope)
        "what is going on with this system",
        "what is going on with my system",
        "what is happening on this system",
        "what is happening on my system",
        "what's going on overall",

        // Overview phrases
        "overview of this system",
        "overview of my system",
        "overview of the system",
        "overview of problems",
        "system overview",
        "machine overview",
        "complete overview",
        "full overview",
    ];

    for pattern in &exact_patterns {
        if normalized.contains(pattern) {
            return true;
        }
    }

    // Combined keyword patterns (need both)
    // Example: "full" + "report" + "system"
    let has_full_or_complete = normalized.contains("full") ||
                                normalized.contains("complete") ||
                                normalized.contains("comprehensive") ||
                                normalized.contains("detailed");

    let has_report_or_briefing = normalized.contains("report") ||
                                   normalized.contains("briefing") ||
                                   normalized.contains("summary");

    let has_system_ref = normalized.contains("system") ||
                         normalized.contains("machine") ||
                         normalized.contains("computer") ||
                         normalized.contains("box");

    if has_full_or_complete && has_report_or_briefing && has_system_ref {
        return true;
    }

    false
}

fn is_full_diagnostic_query(user_text: &str) -> bool {
    // Beta.243: Apply normalization for robust matching
    let normalized = normalize_query_for_intent(user_text);

    // Beta.277: Rule C - Diagnostic Requires Clear System Intent
    // Check ambiguity BEFORE pattern matching
    if is_ambiguous_query(&normalized) {
        return false;
    }

    // Beta.244: Exclude conceptual/definition questions before checking patterns
    // These are questions about what system health IS, not about THIS system's health
    let conceptual_patterns = [
        "what is",
        "what does",
        "what's",
        "explain",
        "define",
        "definition of",
        "what are",
        "describe",
        "tell me about",
    ];

    let conceptual_subjects = [
        "healthy system",
        "system health",
        "a healthy",
        "good system health",
    ];

    // If query is asking "what is a healthy system" or similar, route to conversational
    for concept_pattern in &conceptual_patterns {
        if normalized.contains(concept_pattern) {
            for subject in &conceptual_subjects {
                if normalized.contains(subject) {
                    // This is a definition/conceptual question, not about this machine
                    return false;
                }
            }
        }
    }

    // Exact phrase matches first (highest specificity)
    let exact_matches = [
        "run a full diagnostic",
        "run full diagnostic",
        "full diagnostic",
        "check my system health",
        "check the system health",  // Beta.243: the/my variation
        "check system health",
        "show any problems",
        "show me any problems",
        "full system diagnostic",
        "system health check",
        // Beta.239: New exact phrases
        "health check",
        "check health",  // Beta.243: bi-directional
        "system check",
        "check system",  // Beta.243: bi-directional
        "health report",
        "system report",
        "is my system ok",
        "is the system ok",  // Beta.243: the/my variation
        "is the system okay",  // Beta.243: the/my + okay variant
        "is everything ok with my system",
        "is my system okay",
        "is everything okay with my system",
        // Beta.243: Terse patterns (auxiliary verb dropped)
        "my system ok",
        "my system okay",
        "the system ok",
        "the system okay",
        "system ok",
        "system okay",
        // Beta.243: Verb variations
        "run diagnostic",  // standalone with run
        "perform diagnostic",
        "execute diagnostic",
        // Beta.249: High-value health check patterns
        "is my system healthy",
        "is the system healthy",
        "is everything ok",  // standalone, without "with my system"
        "is everything okay",  // okay variant
        "everything ok",  // terse form
        "everything okay",  // terse okay variant
        "are there any problems",
        "are there any issues",
        "show me any issues",
        "anything wrong",
        "are any services failing",
        "how is my system",  // captures "how is my system today/doing/etc"
        "how is the system",
        // Beta.251: Troubleshooting and problem detection
        "what's wrong",
        "what is wrong",
        "whats wrong",  // no apostrophe variant
        "something wrong",
        "anything wrong with",
        // Beta.251: Compact health status patterns
        "health status",
        "status health",  // bi-directional
        // Beta.251: Service health patterns
        "services down",
        "services are down",
        "service down",
        "which services down",
        "which services are down",
        // Beta.251: System doing patterns
        "system doing",  // captures "how is [this/my] system doing"
        "my system doing",
        "the system doing",
        "this system doing",
        // Beta.251: Check machine/computer patterns
        "check this machine",
        "check my machine",
        "check this computer",
        "check my computer",
        "check this host",
        "check my host",
        // Beta.252: Resource health variants (Category A - trivial safe fix)
        "disk healthy",
        "cpu healthy",
        "memory healthy",
        "ram healthy",
        "network health",
        "machine healthy",
        "computer healthy",
        // Beta.253: Category B - "What's wrong" with system context
        "what's wrong with my system",
        "what is wrong with my system",
        "what's wrong with this system",
        "what's wrong with my machine",
        "what's wrong with this machine",
        "what's wrong with my computer",
        // Beta.253: Clear diagnostic commands
        "service health",
        "show me problems",
        "show problems",
        "display health",
        "check system",
        "diagnose system",
        "do diagnostic",
        "run diagnostic",
        // Beta.254: Resource-specific errors/problems/issues (question format)
        "journal errors",
        "package problems",
        "failed boot attempts",
        "boot attempts",
        "internet connectivity issues",
        "connectivity issues",
        "hardware problems",
        "overheating issues",
        "filesystem errors",
        "mount problems",
        "security issues",
        // Beta.254: Possessive health forms
        "computer's health",
        "machine's health",
        "system's health",
        // Beta.255: Temporal diagnostic patterns (time + health/error terms)
        "errors today",
        "errors recently",
        "errors lately",
        "critical errors today",
        "critical errors recently",
        "failed services today",
        "failed services recently",
        "any errors today",
        "any errors recently",
        "issues today",
        "issues recently",
        "problems today",
        "problems recently",
        "failures today",
        "failures recently",
        "morning system check",
        "morning check",
        "checking in on the system",
        "just checking the system",
        // Beta.256: Resource health variants + consolidation
        "is my machine healthy",
        "is my disk healthy",
        "machine healthy",
        "disk healthy",
        "full system check",
        "complete diagnostic",
        "system problem",  // singular form
        "service issue",   // singular form
        "no problems",     // negation pattern
        // Beta.256: Intent markers and polite requests
        "i want to know if my system is healthy",
        "i need a system check",
        "can you check my system",
        // Beta.275: High/medium priority router_bug fixes
        // Negative forms
        "nothing broken",
        "no problems right",
        "not having issues am i",
        "should i worry",
        // Short diagnostic commands
        "diagnose system",
        "generate diagnostic",
        "fetch health info",
        "in depth diagnostic",
        "deep diagnostic",
        "thorough system check",
        // System wellness and completeness
        "system wellness check",
        "complete diagnostic",
        // Abbreviated forms
        "sys health",
        "svc problems",
        "system problem",  // singular
        "service issue",   // singular
        // Possessive health forms
        "my system's health",
        "this computer's health",
        // Resource-specific question patterns
        "journal errors",
        "package problems",
        "broken packages",
        "orphaned packages",
        "network problems",
        "hardware problems",
        "overheating issues",
        "filesystem errors",
        "mount problems",
        "security issues",
        "failed boot attempts",
        "internet connectivity issues",
        "updates available",
        "performance problems",
        "resource problems",
        "configuration problems",
        "dependency issues",
        "permission problems",
        "access issues",
        "compatibility issues",
        "quota problems",
        "lock problems",
        "timeout issues",
        // Status forms that should be diagnostic
        "status of my system",
        "my computer status",
        "cpu health check",
        // Critical/priority forms
        "critical issues",
        "high priority problems",
        "very serious issues",
        // "Just checking" patterns
        "just checking in on the system",
        "morning system check",
        "morning check",
        // Diagnostic report requests
        "give me a diagnostic report",
        "show me if there are problems",
        "are there problems if so what",
        // Machine/PC variants
        "is my machine healthy",
        "is my disk healthy",
        // Negative question forms
        "system is healthy right",
        // Beta.276: Conditional diagnostic patterns
        "are there problems",
        "any problems",
    ];

    for phrase in &exact_matches {
        if normalized.contains(phrase) {
            // Beta.240: Log diagnostic phrase match (developer-only)
            crate::debug::log_diagnostic_phrase_match(phrase, "exact_match");
            return true;
        }
    }

    // Beta.276: Single-word "diagnostic" as exact match only (not substring)
    // This avoids false positives like "do diagnostic" while catching "diagnostic" alone
    if normalized == "diagnostic" {
        crate::debug::log_diagnostic_phrase_match("diagnostic", "exact_word");
        return true;
    }

    // Pattern-based matches (broader but still specific)
    // "diagnose" + "system" or "my system" or "the system"
    if normalized.contains("diagnose") {
        if normalized.contains("system") || normalized.contains("my system") || normalized.contains("the system") {
            // Beta.240: Log pattern match
            crate::debug::log_diagnostic_phrase_match(&normalized, "diagnose_system_pattern");
            return true;
        }
    }

    // "full" + "diagnostic" (even if not adjacent)
    if normalized.contains("full") && normalized.contains("diagnostic") {
        // Beta.240: Log pattern match
        crate::debug::log_diagnostic_phrase_match(&normalized, "full_diagnostic_pattern");
        return true;
    }

    // Beta.276: "system diagnostic" or "diagnostic analysis" patterns
    if normalized.contains("system diagnostic") || normalized.contains("diagnostic analysis") {
        crate::debug::log_diagnostic_phrase_match(&normalized, "system_diagnostic_pattern");
        return true;
    }

    // "system" + "health" (common combination)
    // Beta.244: Enhanced with contextual awareness
    // Positive indicators: this system, this machine, my system, on this computer, here
    // Negative indicators: in general, in theory, on linux, on arch linux (without "this"/"my")
    if normalized.contains("system") && normalized.contains("health") {
        // Check for positive contextual indicators
        let positive_indicators = [
            "this system",
            "this machine",
            "my system",
            "my machine",
            "on this computer",
            "on this system",
            "on my system",
            "here",
            "this computer",
        ];

        let negative_indicators = [
            "in general",
            "in theory",
            "on linux",
            "on arch linux",
            "in arch",
            "for linux",
        ];

        let has_positive = positive_indicators.iter().any(|ind| normalized.contains(ind));
        let has_negative = negative_indicators.iter().any(|ind| normalized.contains(ind));

        // If both positive and negative, prefer conversational (ambiguous)
        if has_positive && has_negative {
            return false;
        }

        // If positive indicators present, treat as diagnostic
        if has_positive {
            crate::debug::log_diagnostic_phrase_match(&normalized, "system_health_contextual");
            return true;
        }

        // If no conceptual question pattern detected earlier, allow the match
        // (This preserves Beta.243 behavior for simple "system health" queries)
        crate::debug::log_diagnostic_phrase_match(&normalized, "system_health_pattern");
        return true;
    }

    // Beta.249: Resource-specific health patterns
    // Pattern: "[resource] problems?" or "[resource] issues?"
    let resources = ["disk", "disk space", "cpu", "memory", "ram", "network", "service", "services", "boot", "package", "packages"];
    let health_terms = ["problems", "issues", "errors", "failures", "failing"];

    for resource in &resources {
        for term in &health_terms {
            let pattern = format!("{} {}", resource, term);
            if normalized.contains(&pattern) {
                crate::debug::log_diagnostic_phrase_match(&normalized, "resource_health_pattern");
                return true;
            }
        }
    }

    // Beta.249: "Is [resource] [health_state]?" patterns
    // Examples: "is my cpu overloaded?", "is disk full?"
    let overload_terms = ["overloaded", "full", "exhausted", "running out"];
    for resource in &resources {
        for term in &overload_terms {
            if normalized.contains(resource) && normalized.contains(term) {
                crate::debug::log_diagnostic_phrase_match(&normalized, "resource_overload_pattern");
                return true;
            }
        }
    }

    // Beta.249: "running out of [resource]" pattern
    if normalized.contains("running out") {
        for resource in &resources {
            if normalized.contains(resource) {
                crate::debug::log_diagnostic_phrase_match(&normalized, "running_out_pattern");
                return true;
            }
        }
    }

    false
}

/// Beta.238: Handle diagnostic query by calling internal brain analysis
/// Beta.250: Now uses canonical diagnostic formatter
///
/// Routes natural language diagnostic requests to the same diagnostic engine
/// that powers the hidden `annactl brain` command. Returns formatted diagnostic
/// report suitable for CLI or TUI display.
///
/// Beta.257: Accepts query parameter for temporal wording support
/// Beta.258: Use daily snapshot for temporal queries ("today", "recently")
async fn handle_diagnostic_query(query: &str) -> Result<UnifiedQueryResult> {
    use anna_common::ipc::{Method, ResponseData};
    use crate::rpc_client::RpcClient;
    use crate::diagnostic_formatter::{
        format_diagnostic_report_with_query, format_daily_snapshot,
        compute_daily_snapshot, SessionDelta, DiagnosticMode
    };
    use crate::startup::welcome::load_last_session;

    // Fetch brain analysis from daemon (same as brain command)
    let mut client = RpcClient::connect().await
        .map_err(|e| anyhow::anyhow!("Diagnostic engine unavailable: {}\n\nThe daemon (annad) must be running to perform diagnostics.", e))?;

    let response = client.call(Method::BrainAnalysis).await
        .map_err(|e| anyhow::anyhow!("Diagnostic analysis failed: {}", e))?;

    // Beta.240: Print RPC stats if debug enabled (developer-only)
    crate::debug::print_rpc_stats(client.get_stats());

    let analysis = match response {
        ResponseData::BrainAnalysis(data) => data,
        _ => return Err(anyhow::anyhow!("Unexpected response type from diagnostic engine")),
    };

    // Beta.258: Check if this is a temporal query
    let normalized_query = query.to_lowercase();
    let is_temporal = normalized_query.contains("today") || normalized_query.contains("recently");

    // Beta.258: For temporal queries, use daily snapshot format
    if is_temporal {
        // Try to load session metadata for delta computation
        let session_delta = match load_last_session() {
            Ok(Some(last_session)) => {
                // Fetch current telemetry using system_query module
                match crate::system_query::query_system_telemetry() {
                    Ok(current_telemetry) => {
                        use crate::startup::welcome::create_telemetry_snapshot;
                        let current_snapshot = create_telemetry_snapshot(&current_telemetry);
                        let kernel_changed = last_session.last_telemetry.kernel_version != current_snapshot.kernel_version;
                        let package_delta = (current_snapshot.package_count as i32) - (last_session.last_telemetry.package_count as i32);

                        SessionDelta {
                            kernel_changed,
                            old_kernel: if kernel_changed { Some(last_session.last_telemetry.kernel_version.clone()) } else { None },
                            new_kernel: if kernel_changed { Some(current_snapshot.kernel_version.clone()) } else { None },
                            package_delta,
                            boots_since_last: if kernel_changed { 1 } else { 0 },
                        }
                    }
                    Err(_) => {
                        // Telemetry fetch failed, use defaults
                        SessionDelta::default()
                    }
                }
            }
            _ => {
                // No last session or load failed, use defaults
                SessionDelta::default()
            }
        };

        let snapshot = compute_daily_snapshot(&analysis, session_delta);
        let report = format_daily_snapshot(&snapshot, true);

        Ok(UnifiedQueryResult::ConversationalAnswer {
            answer: report,
            confidence: AnswerConfidence::High,
            sources: vec!["internal diagnostic engine (9 deterministic checks) + session metadata".to_string()],
        })
    } else {
        // Beta.257: Use query-aware formatter for non-temporal queries
        let report = format_diagnostic_report_with_query(&analysis, DiagnosticMode::Full, query);

        Ok(UnifiedQueryResult::ConversationalAnswer {
            answer: report,
            confidence: AnswerConfidence::High,
            sources: vec!["internal diagnostic engine (9 deterministic checks)".to_string()],
        })
    }
}

/// Beta.278: Handle sysadmin report query
///
/// Generates a comprehensive sysadmin briefing combining:
/// - Health summary and diagnostics
/// - Daily snapshot
/// - Proactive correlated issues
/// - Key domain highlights
async fn handle_sysadmin_report_query(_query: &str) -> Result<UnifiedQueryResult> {
    use anna_common::ipc::{Method, ResponseData};
    use crate::rpc_client::RpcClient;
    use crate::diagnostic_formatter::{format_daily_snapshot, compute_daily_snapshot, SessionDelta};
    use crate::startup::welcome::load_last_session;

    // Fetch brain analysis from daemon
    let mut client = RpcClient::connect().await
        .map_err(|e| anyhow::anyhow!("Diagnostic engine unavailable: {}\n\nThe daemon (annad) must be running to perform diagnostics.", e))?;

    let response = client.call(Method::BrainAnalysis).await
        .map_err(|e| anyhow::anyhow!("Sysadmin report failed: {}", e))?;

    crate::debug::print_rpc_stats(client.get_stats());

    let analysis = match response {
        ResponseData::BrainAnalysis(data) => data,
        _ => return Err(anyhow::anyhow!("Unexpected response type from diagnostic engine")),
    };

    // Compute daily snapshot for session context
    let session_delta = match load_last_session() {
        Ok(Some(last_session)) => {
            // Fetch current telemetry using system_query module
            match crate::system_query::query_system_telemetry() {
                Ok(current_telemetry) => {
                    use crate::startup::welcome::create_telemetry_snapshot;
                    let current_snapshot = create_telemetry_snapshot(&current_telemetry);
                    let kernel_changed = last_session.last_telemetry.kernel_version != current_snapshot.kernel_version;
                    let package_delta = (current_snapshot.package_count as i32) - (last_session.last_telemetry.package_count as i32);

                    SessionDelta {
                        kernel_changed,
                        old_kernel: if kernel_changed { Some(last_session.last_telemetry.kernel_version.clone()) } else { None },
                        new_kernel: if kernel_changed { Some(current_snapshot.kernel_version.clone()) } else { None },
                        package_delta,
                        boots_since_last: if kernel_changed { 1 } else { 0 },
                    }
                }
                Err(_) => {
                    // Telemetry fetch failed, use defaults
                    SessionDelta::default()
                }
            }
        }
        _ => {
            // No last session or load failed, use defaults
            SessionDelta::default()
        }
    };

    let daily_snapshot = compute_daily_snapshot(&analysis, session_delta);
    let snapshot_text = format_daily_snapshot(&daily_snapshot, false);

    // Extract proactive data
    let proactive_issues = &analysis.proactive_issues;
    let proactive_health_score = analysis.proactive_health_score;

    // Compose the sysadmin report
    let report = crate::sysadmin_answers::compose_sysadmin_report_answer(
        &analysis,
        Some(&snapshot_text),
        proactive_issues,
        proactive_health_score,
    );

    Ok(UnifiedQueryResult::ConversationalAnswer {
        answer: report,
        confidence: AnswerConfidence::High,
        sources: vec!["system diagnostics, proactive analysis, session tracking".to_string()],
    })
}

// Beta.250: Old format_diagnostic_report function removed
// Now using canonical formatter from diagnostic_formatter module

/// Beta.272: Detect proactive remediation queries
///
/// Patterns focused on "what should I fix" and "top issues"
fn is_proactive_remediation_query(text: &str) -> bool {
    let normalized = normalize_query_for_intent(text);

    let proactive_patterns = [
        // "What should I fix" family
        "what should i fix first",
        "what should i fix",
        "what do i fix first",
        "what to fix first",
        "what needs fixing first",
        "what needs to be fixed first",

        // "Top issues" family
        "what are my top issues",
        "what are the top issues",
        "show me my top issues",
        "show top issues",
        "top issues",
        "top problems",
        "main issues",
        "main problems",

        // "Most important" family
        "what is the most important issue",
        "what's the most important issue",
        "most important issue",
        "most important problem",
        "biggest issue",
        "biggest problem",

        // "Main problem" family
        "what is the main problem",
        "what's the main problem",
        "main problem",

        // "Critical/urgent" family
        "any critical problems",
        "any critical issues",
        "critical problems",
        "critical issues",
        "any urgent issues",
        "any urgent problems",
        "urgent issues",
        "urgent problems",

        // "Priority" family
        "highest priority issue",
        "highest priority problem",
        "priority issues",
        "priority problems",
    ];

    for pattern in &proactive_patterns {
        if normalized == *pattern || normalized.contains(pattern) {
            return true;
        }
    }

    false
}

/// Beta.272: Handle proactive remediation query
///
/// Returns remediation answer for the top proactive issue, or "no issues" message
async fn handle_proactive_remediation_query(_user_text: &str) -> Result<UnifiedQueryResult> {
    // Fetch brain/health data via RPC
    let mut client = crate::rpc_client::RpcClient::connect_quick(None).await?;
    let response = client.call(anna_common::ipc::Method::BrainAnalysis).await?;

    let brain_data = match response {
        anna_common::ipc::ResponseData::BrainAnalysis(data) => data,
        _ => return Err(anyhow::anyhow!("Unexpected RPC response type")),
    };

    // Check if there are any proactive issues
    if brain_data.proactive_issues.is_empty() {
        // No proactive issues detected
        let answer = format!(
            "[SUMMARY]\n\
            No correlated issues detected. System health is determined by individual checks only.\n\n\
            [DETAILS]\n\
            The proactive engine did not find any high confidence correlations between network, \
            disk, services, CPU, memory, or processes.\n\n\
            Individual diagnostic insights may still be present. Run a full health check for details.\n\n\
            [COMMANDS]\n\
            $ annactl \"check my system health\"\n\
            $ annactl status"
        );

        return Ok(UnifiedQueryResult::ConversationalAnswer {
            answer,
            confidence: AnswerConfidence::High,
            sources: vec!["proactive engine (deterministic)".to_string()],
        });
    }

    // Sort by severity and get top issue
    let mut sorted_issues = brain_data.proactive_issues.clone();
    sorted_issues.sort_by(|a, b| {
        let a_priority = crate::diagnostic_formatter::severity_priority_proactive(&a.severity);
        let b_priority = crate::diagnostic_formatter::severity_priority_proactive(&b.severity);
        b_priority.cmp(&a_priority) // Descending
    });

    let top_issue = &sorted_issues[0];

    // Generate remediation answer
    let answer = crate::sysadmin_answers::compose_top_proactive_remediation(top_issue, &brain_data)
        .unwrap_or_else(|| {
            format!(
                "[SUMMARY]\n\
                Top correlated issue: {}\n\n\
                [DETAILS]\n\
                Specific remediation guidance not yet available for this issue type (root cause: {}).\n\
                Confidence: {:.0}%\n\n\
                [COMMANDS]\n\
                $ annactl \"check my system health\"\n\
                $ annactl status",
                top_issue.summary,
                top_issue.root_cause,
                top_issue.confidence * 100.0
            )
        });

    Ok(UnifiedQueryResult::ConversationalAnswer {
        answer,
        confidence: AnswerConfidence::High,
        sources: vec!["proactive engine + deterministic remediation".to_string()],
    })
}

/// Beta.273: Detect proactive status summary queries
///
/// Patterns focused on "show proactive status", "summarize top issues", "summarize findings"
fn is_proactive_status_query(text: &str) -> bool {
    let normalized = normalize_query_for_intent(text);

    let status_patterns = [
        "show proactive status",
        "proactive status",
        "summarize top issues",
        "summarize issues",
        "what problems do you see",
        "what problems can you see",
        "summarize correlations",
        "summarize findings",
        "top correlated issues",
        "show correlations",
        "show top correlations",
        "list proactive issues",
        "list correlations",
        "proactive summary",
        "correlation summary",
    ];

    for pattern in &status_patterns {
        if normalized == *pattern || normalized.contains(pattern) {
            return true;
        }
    }

    false
}

/// Beta.273: Handle proactive status summary query
///
/// Returns a summary of all proactive issues with scores, or "no issues" message
async fn handle_proactive_status_query(_user_text: &str) -> Result<UnifiedQueryResult> {
    // Fetch brain/health data via RPC
    let mut client = crate::rpc_client::RpcClient::connect_quick(None).await?;
    let response = client.call(anna_common::ipc::Method::BrainAnalysis).await?;

    let brain_data = match response {
        anna_common::ipc::ResponseData::BrainAnalysis(data) => data,
        _ => return Err(anyhow::anyhow!("Unexpected RPC response type")),
    };

    // Check if there are any proactive issues
    if brain_data.proactive_issues.is_empty() {
        // No proactive issues detected
        let answer = format!(
            "[SUMMARY]\n\
            No correlated issues found.\n\n\
            [DETAILS]\n\
            The proactive engine did not detect any high-confidence correlations. \
            System health is determined by individual diagnostic checks only.\n\n\
            [COMMANDS]\n\
            $ annactl status\n\
            $ annactl \"check my system health\""
        );

        return Ok(UnifiedQueryResult::ConversationalAnswer {
            answer,
            confidence: AnswerConfidence::High,
            sources: vec!["proactive engine (deterministic)".to_string()],
        });
    }

    // Build summary answer
    let issue_count = brain_data.proactive_issues.len();
    let health_score = brain_data.proactive_health_score;

    let mut answer = format!(
        "[SUMMARY]\n\
        Proactive engine detected {} correlated issue(s).\n\
        System health score: {}/100\n\n\
        [DETAILS]\n",
        issue_count,
        health_score
    );

    // Sort by severity
    let mut sorted_issues = brain_data.proactive_issues.clone();
    sorted_issues.sort_by(|a, b| {
        let a_priority = crate::diagnostic_formatter::severity_priority_proactive(&a.severity);
        let b_priority = crate::diagnostic_formatter::severity_priority_proactive(&b.severity);
        b_priority.cmp(&a_priority) // Descending
    });

    // Show all issues (up to 10)
    sorted_issues.truncate(10);
    for (idx, issue) in sorted_issues.iter().enumerate() {
        let score = (issue.confidence * 100.0) as u8;
        let severity_marker = match issue.severity.to_lowercase().as_str() {
            "critical" => "✗",
            "warning" => "⚠",
            "info" => "ℹ",
            "trend" => "~",
            _ => "•",
        };

        answer.push_str(&format!(
            "{}. {} {} (severity: {}, score: {})\n",
            idx + 1,
            severity_marker,
            issue.root_cause,
            issue.severity,
            score
        ));
    }

    answer.push_str(
        "\n[COMMANDS]\n\
        $ annactl status\n\
        $ annactl \"check my system health\""
    );

    Ok(UnifiedQueryResult::ConversationalAnswer {
        answer,
        confidence: AnswerConfidence::High,
        sources: vec!["proactive engine (deterministic)".to_string()],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_use_action_plan() {
        assert!(should_use_action_plan("install docker"));
        assert!(should_use_action_plan("fix broken packages"));
        assert!(should_use_action_plan("setup neovim"));

        assert!(!should_use_action_plan("what is my CPU?"));
        assert!(!should_use_action_plan("how much RAM do I have?"));
    }

    #[test]
    fn test_is_full_diagnostic_query() {
        // Beta.238: Test diagnostic query detection
        // Beta.239: Extended with new phrase patterns

        // Exact matches - Beta.238
        assert!(is_full_diagnostic_query("run a full diagnostic"));
        assert!(is_full_diagnostic_query("check my system health"));
        assert!(is_full_diagnostic_query("show any problems"));
        assert!(is_full_diagnostic_query("full system diagnostic"));

        // Beta.239: New exact matches
        assert!(is_full_diagnostic_query("health check"));
        assert!(is_full_diagnostic_query("system check"));
        assert!(is_full_diagnostic_query("health report"));
        assert!(is_full_diagnostic_query("system report"));
        assert!(is_full_diagnostic_query("is my system ok"));
        assert!(is_full_diagnostic_query("is everything ok with my system"));
        assert!(is_full_diagnostic_query("is my system okay"));
        assert!(is_full_diagnostic_query("is everything okay with my system"));

        // Case insensitive
        assert!(is_full_diagnostic_query("RUN A FULL DIAGNOSTIC"));
        assert!(is_full_diagnostic_query("Check System Health"));
        assert!(is_full_diagnostic_query("HEALTH CHECK"));
        assert!(is_full_diagnostic_query("System Report"));

        // Pattern matches
        assert!(is_full_diagnostic_query("diagnose my system"));
        assert!(is_full_diagnostic_query("can you diagnose the system"));
        assert!(is_full_diagnostic_query("please check system health"));

        // Non-diagnostic queries (should NOT trigger)
        assert!(!is_full_diagnostic_query("what is my CPU?"));
        assert!(!is_full_diagnostic_query("install docker"));
        assert!(!is_full_diagnostic_query("check disk space"));
        assert!(!is_full_diagnostic_query("health insurance"));  // "health" alone shouldn't trigger
        assert!(!is_full_diagnostic_query("system update"));     // "system" alone shouldn't trigger
    }
}
