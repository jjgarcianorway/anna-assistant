//! Unified Query Handler - 3-Tier Architecture
//!
//! Beta.144: Extract 3-tier query logic for reuse by CLI and TUI
//!
//! Architecture:
//! 1. Template Matching (instant, accurate, 40+ templates)
//! 2. RecipePlanner (LLM with critic validation)
//! 3. Generic LLM (conversational fallback)
//!
//! This fixes the TUI quality issue where TUI bypassed template matching.

use anna_common::command_recipe::Recipe;
use anna_common::llm::LlmConfig;
use anna_common::template_library::TemplateLibrary;
use anyhow::Result;
use std::collections::HashMap;
use std::process::Command;

/// Result of query processing
#[derive(Debug)]
pub enum QueryResult {
    /// Template matched - instant shell command result
    Template {
        template_id: String,
        command: String,
        output: String,
    },
    /// Recipe planned and validated by critic
    Recipe(Recipe),
    /// Generic LLM response (needs streaming)
    LlmFallback { config: LlmConfig, prompt: String },
}

/// Try template matching first (Tier 1)
pub fn try_template_match(user_text: &str) -> Option<(&'static str, HashMap<String, String>)> {
    let input_lower = user_text.to_lowercase();

    // Helper function for word-boundary keyword matching
    let contains_word = |text: &str, keyword: &str| {
        text.split(|c: char| !c.is_alphanumeric())
            .any(|word| word == keyword)
    };

    // Pattern matching for template selection (40+ templates)
    if contains_word(&input_lower, "swap") {
        Some(("check_swap_status", HashMap::new()))
    } else if contains_word(&input_lower, "gpu") || contains_word(&input_lower, "vram") {
        Some(("check_gpu_memory", HashMap::new()))
    } else if contains_word(&input_lower, "kernel") {
        Some(("check_kernel_version", HashMap::new()))
    } else if contains_word(&input_lower, "disk")
        || input_lower.contains("space")
        || input_lower.contains("storage")
    {
        Some(("check_disk_space", HashMap::new()))
    } else if contains_word(&input_lower, "ram")
        || contains_word(&input_lower, "memory")
        || contains_word(&input_lower, "mem")
    {
        Some(("check_memory", HashMap::new()))
    } else if input_lower.contains("uptime") {
        Some(("check_uptime", HashMap::new()))
    } else if input_lower.contains("cpu model") || input_lower.contains("processor") {
        Some(("check_cpu_model", HashMap::new()))
    } else if input_lower.contains("cpu load")
        || input_lower.contains("cpu usage")
        || input_lower.contains("load average")
    {
        Some(("check_cpu_load", HashMap::new()))
    } else if input_lower.contains("distro")
        || input_lower.contains("distribution")
        || input_lower.contains("os-release")
    {
        Some(("check_distro", HashMap::new()))
    } else if input_lower.contains("failed services")
        || (input_lower.contains("systemctl") && input_lower.contains("failed"))
    {
        Some(("check_failed_services", HashMap::new()))
    } else if input_lower.contains("journal")
        || (input_lower.contains("system") && input_lower.contains("errors"))
    {
        Some(("check_journal_errors", HashMap::new()))
    } else if input_lower.contains("wifi")
        || input_lower.contains("wireless")
        || (input_lower.contains("network")
            && (input_lower.contains("slow")
                || input_lower.contains("issue")
                || input_lower.contains("problem")))
    {
        Some(("wifi_diagnostics", HashMap::new()))

    // PACKAGE MANAGEMENT
    } else if input_lower.contains("orphan")
        || (input_lower.contains("unused") && input_lower.contains("package"))
    {
        Some(("list_orphaned_packages", HashMap::new()))
    } else if input_lower.contains("aur") {
        Some(("list_aur_packages", HashMap::new()))
    } else if input_lower.contains("pacman")
        && (input_lower.contains("cache") || input_lower.contains("size"))
    {
        Some(("check_pacman_cache_size", HashMap::new()))
    } else if (input_lower.contains("clean") || input_lower.contains("clear"))
        && input_lower.contains("cache")
    {
        Some(("clean_package_cache", HashMap::new()))
    } else if (input_lower.contains("search") || input_lower.contains("find"))
        && input_lower.contains("package")
        && (input_lower.contains("file") || input_lower.contains("provides"))
    {
        // Beta.204: arch-019 - Package file search
        Some(("search_package_file", HashMap::new()))
    } else if input_lower.contains("mirror")
        || (input_lower.contains("pacman") && input_lower.contains("server"))
    {
        Some(("check_pacman_mirrors", HashMap::new()))
    } else if input_lower.contains("update")
        || input_lower.contains("upgrade")
        || input_lower.contains("syu")
    {
        Some(("check_pending_updates", HashMap::new()))
    } else if input_lower.contains("pacman lock") || input_lower.contains("db.lck") {
        Some(("check_pacman_locks", HashMap::new()))
    } else if input_lower.contains("keyring")
        || input_lower.contains("pgp")
        || input_lower.contains("signature")
    {
        Some(("check_archlinux_keyring", HashMap::new()))
    } else if input_lower.contains("explicit") && input_lower.contains("package") {
        Some(("list_explicit_packages", HashMap::new()))
    } else if input_lower.contains("depends") || input_lower.contains("dependency") {
        Some(("package_depends", HashMap::new()))
    } else if input_lower.contains("required by") || input_lower.contains("reverse depend") {
        Some(("package_reverse_depends", HashMap::new()))
    } else if input_lower.contains("package integrity") || input_lower.contains("corrupt") {
        Some(("check_package_integrity", HashMap::new()))
    } else if input_lower.contains("recent")
        && (input_lower.contains("install") || input_lower.contains("pacman"))
    {
        Some(("show_recent_pacman_operations", HashMap::new()))

    // BOOT & SYSTEMD
    } else if input_lower.contains("boot")
        && (input_lower.contains("time") || input_lower.contains("slow"))
    {
        Some(("analyze_boot_time", HashMap::new()))
    } else if input_lower.contains("boot") && input_lower.contains("error") {
        Some(("check_boot_errors", HashMap::new()))
    } else if input_lower.contains("boot log") || input_lower.contains("dmesg") {
        Some(("show_boot_log", HashMap::new()))
    } else if input_lower.contains("critical chain") {
        Some(("analyze_boot_critical_chain", HashMap::new()))
    } else if input_lower.contains("timer")
        || (input_lower.contains("systemd") && input_lower.contains("scheduled"))
    {
        Some(("check_systemd_timers", HashMap::new()))
    } else if input_lower.contains("journal size") {
        Some(("analyze_journal_size", HashMap::new()))
    } else if input_lower.contains("systemd") && input_lower.contains("version") {
        Some(("check_systemd_version", HashMap::new()))
    } else if input_lower.contains("recent")
        && (input_lower.contains("error") || input_lower.contains("journal"))
    {
        Some(("show_recent_journal_errors", HashMap::new()))

    // CPU & PERFORMANCE
    } else if input_lower.contains("cpu")
        && (input_lower.contains("freq") || input_lower.contains("speed"))
    {
        Some(("check_cpu_frequency", HashMap::new()))
    } else if input_lower.contains("governor")
        || (input_lower.contains("cpu") && input_lower.contains("scaling"))
    {
        Some(("check_cpu_governor", HashMap::new()))
    } else if input_lower.contains("cpu usage") || input_lower.contains("cpu percent") {
        Some(("analyze_cpu_usage", HashMap::new()))
    } else if (input_lower.contains("cpu") || input_lower.contains("processor"))
        && (input_lower.contains("temp") || input_lower.contains("hot"))
    {
        Some(("check_cpu_temperature", HashMap::new()))
    } else if input_lower.contains("throttl")
        || (input_lower.contains("cpu") && input_lower.contains("slow"))
    {
        Some(("detect_cpu_throttling", HashMap::new()))
    } else if input_lower.contains("top") && input_lower.contains("cpu") {
        Some(("show_top_cpu_processes", HashMap::new()))
    } else if input_lower.contains("load") && input_lower.contains("average") {
        Some(("check_load_average", HashMap::new()))
    } else if input_lower.contains("context switch") {
        Some(("analyze_context_switches", HashMap::new()))

    // MEMORY
    } else if input_lower.contains("memory") && input_lower.contains("usage") {
        Some(("check_memory_usage", HashMap::new()))
    } else if input_lower.contains("swap") && input_lower.contains("usage") {
        Some(("check_swap_usage", HashMap::new()))
    } else if input_lower.contains("memory pressure") || input_lower.contains("oom") {
        Some(("analyze_memory_pressure", HashMap::new()))
    } else if input_lower.contains("top") && input_lower.contains("memory") {
        Some(("show_top_memory_processes", HashMap::new()))
    } else if input_lower.contains("oom killer") {
        Some(("check_oom_killer", HashMap::new()))
    } else if input_lower.contains("swap activity") {
        Some(("analyze_swap_activity", HashMap::new()))

    // NETWORK
    } else if input_lower.contains("dns") {
        Some(("check_dns_resolution", HashMap::new()))
    } else if input_lower.contains("network") && input_lower.contains("interface") {
        Some(("check_network_interfaces", HashMap::new()))
    } else if input_lower.contains("route") || input_lower.contains("routing") {
        Some(("check_routing_table", HashMap::new()))
    } else if input_lower.contains("firewall") || input_lower.contains("iptables") {
        Some(("check_firewall_rules", HashMap::new()))
    } else if input_lower.contains("port")
        && (input_lower.contains("open") || input_lower.contains("listen"))
    {
        Some(("check_listening_ports", HashMap::new()))
    } else if input_lower.contains("latency") || input_lower.contains("ping") {
        Some(("check_network_latency", HashMap::new()))
    } else if input_lower.contains("networkmanager") || input_lower.contains("nmcli") {
        Some(("check_networkmanager_status", HashMap::new()))

    // GPU & DISPLAY
    } else if input_lower.contains("nvidia") && !input_lower.contains("install") {
        Some(("check_nvidia_status", HashMap::new()))
    } else if input_lower.contains("amd")
        && (input_lower.contains("gpu") || input_lower.contains("graphics"))
    {
        Some(("check_amd_gpu", HashMap::new()))
    } else if input_lower.contains("gpu")
        && (input_lower.contains("info") || input_lower.contains("detect"))
    {
        Some(("check_gpu_info", HashMap::new()))
    } else if input_lower.contains("gpu") && input_lower.contains("driver") {
        Some(("check_gpu_drivers", HashMap::new()))
    } else if input_lower.contains("gpu")
        && (input_lower.contains("temp") || input_lower.contains("hot"))
    {
        Some(("check_gpu_temperature", HashMap::new()))
    } else if input_lower.contains("display server")
        || input_lower.contains("x11")
        || input_lower.contains("wayland")
    {
        Some(("check_display_server", HashMap::new()))
    } else if input_lower.contains("desktop environment")
        || input_lower.contains("kde")
        || input_lower.contains("gnome")
    {
        Some(("check_desktop_environment", HashMap::new()))
    } else if input_lower.contains("xorg") && input_lower.contains("error") {
        Some(("analyze_xorg_errors", HashMap::new()))
    } else if input_lower.contains("wayland") && input_lower.contains("compositor") {
        Some(("check_wayland_compositor", HashMap::new()))

    // HARDWARE
    } else if input_lower.contains("temperature")
        || input_lower.contains("temp")
        || input_lower.contains("heat")
    {
        Some(("check_temperature", HashMap::new()))
    } else if input_lower.contains("usb") {
        Some(("check_usb_devices", HashMap::new()))
    } else if input_lower.contains("pci") || input_lower.contains("lspci") {
        Some(("check_pci_devices", HashMap::new()))
    } else if input_lower.contains("hostname") {
        Some(("check_hostname", HashMap::new()))
    } else {
        None
    }
}

/// Execute a template and return the result
pub fn execute_template(
    template_id: &str,
    params: &HashMap<String, String>,
) -> Result<QueryResult> {
    let library = TemplateLibrary::default();

    let template = library
        .get(template_id)
        .ok_or_else(|| anyhow::anyhow!("Template not found: {}", template_id))?;

    let recipe = template
        .instantiate(params)
        .map_err(|e| anyhow::anyhow!("Template instantiation failed: {}", e))?;

    // Execute command
    let output = Command::new("sh").arg("-c").arg(&recipe.command).output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(QueryResult::Template {
        template_id: template_id.to_string(),
        command: recipe.command.clone(),
        output: stdout,
    })
}

/// Try RecipePlanner (Tier 2)
pub async fn try_recipe_planner(user_text: &str, config: &LlmConfig) -> Result<Option<Recipe>> {
    use anna_common::recipe_planner::{PlanningResult, RecipePlanner};

    let planner = RecipePlanner::new(config.clone());
    let telemetry_summary = "Arch Linux system".to_string();

    match planner.plan_recipe(user_text, &telemetry_summary).await {
        Ok(PlanningResult::Success(recipe)) => Ok(Some(recipe)),
        Ok(PlanningResult::Failed { .. }) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Get LLM config from Ollama
pub fn get_llm_config() -> LlmConfig {
    let model_name = match Command::new("ollama").arg("list").output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .nth(1)
                .and_then(|line| line.split_whitespace().next())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "llama3.1:8b".to_string())
        }
        _ => "llama3.1:8b".to_string(),
    };

    LlmConfig::local("http://127.0.0.1:11434/v1", &model_name)
}
