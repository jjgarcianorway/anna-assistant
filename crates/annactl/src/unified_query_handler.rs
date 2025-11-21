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
pub async fn handle_unified_query(
    user_text: &str,
    telemetry: &SystemTelemetry,
    llm_config: &LlmConfig,
) -> Result<UnifiedQueryResult> {
    // TIER 0: System Report (Version 150 - highest priority, no LLM)
    // Intercept "full report" queries to prevent hallucination
    if crate::system_report::is_system_report_query(user_text) {
        let report = crate::system_report::generate_full_report()
            .unwrap_or_else(|e| format!("Error generating system report: {}", e));

        return Ok(UnifiedQueryResult::ConversationalAnswer {
            answer: report,
            confidence: AnswerConfidence::High,
            sources: vec!["verified system telemetry".to_string()],
        });
    }

    // TIER 1: Beta.151 Deterministic Recipes (NEW - highest priority)
    // These are hard-coded, tested, zero-hallucination plans
    let telemetry_map = telemetry_to_hashmap(telemetry);
    if let Some(recipe_result) = crate::recipes::try_recipe_match(user_text, &telemetry_map) {
        let action_plan = recipe_result.map_err(|e| anyhow::anyhow!("Recipe build_plan failed: {}", e))?;

        // Validate recipe output
        action_plan
            .validate()
            .map_err(|e| anyhow::anyhow!("Recipe generated invalid ActionPlan: {}", e))?;

        let recipe_name = action_plan
            .meta
            .template_used
            .clone()
            .unwrap_or_else(|| "unknown_recipe".to_string());

        return Ok(UnifiedQueryResult::DeterministicRecipe {
            recipe_name,
            action_plan,
        });
    }

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
    }

    // TIER 3: V3 JSON Dialogue (structured action plans with LLM)
    // Version 150: RE-ENABLED - V3 dialogue now works with SystemTelemetry
    // Check if this looks like an actionable request
    if should_use_action_plan(user_text) {
        match dialogue_v3_json::run_dialogue_v3_json(user_text, telemetry, llm_config).await {
            Ok(result) => {
                return Ok(UnifiedQueryResult::ActionPlan {
                    action_plan: result.action_plan,
                    raw_json: result.raw_json,
                });
            }
            Err(e) => {
                // V3 dialogue failed, fall through to conversational answer
                eprintln!("V3 dialogue error (falling back to conversational): {}", e);
            }
        }
    }

    // TIER 4: Structured Conversational Answer (Version 150: NO STREAMING!)
    // Generate structured answer from telemetry or LLM
    generate_conversational_answer(user_text, telemetry, llm_config).await
}

/// Generate structured conversational answer (Version 150: enforces JSON)
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
            sources: vec!["system telemetry".to_string()],
        });
    }

    // Use LLM for complex conversational queries
    let client = Arc::new(LlmClient::from_config(llm_config)
        .map_err(|e| anyhow::anyhow!("LLM not available: {}", e))?);

    let prompt = build_conversational_prompt(user_text, telemetry);
    let llm_prompt = Arc::new(LlmPrompt {
        system: LlmClient::anna_system_prompt().to_string(),
        user: prompt,
        conversation_history: None,
    });

    // Get LLM response (blocking, not streaming!)
    // Beta.208: Apply canonical normalization to all answers
    // Beta.224: Run blocking LLM call in thread pool to avoid blocking async runtime
    let client_clone = Arc::clone(&client);
    let prompt_clone = Arc::clone(&llm_prompt);
    let response = tokio::task::spawn_blocking(move || {
        client_clone.chat(&prompt_clone)
    })
    .await
    .map_err(|e| anyhow::anyhow!("LLM task panicked: {}", e))?
    .map_err(|e| anyhow::anyhow!("LLM query failed: {}", e))?;

    Ok(UnifiedQueryResult::ConversationalAnswer {
        answer: output::normalize_for_cli(&response.text),
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
fn build_conversational_prompt(user_text: &str, telemetry: &SystemTelemetry) -> String {
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
}
