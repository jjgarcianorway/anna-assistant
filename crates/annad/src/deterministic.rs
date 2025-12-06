//! Deterministic answerer - produces answers without LLM when data is available.
//!
//! Answers common queries by parsing hardware snapshots and probe outputs.
//! Used as primary answerer for known query classes, fallback for LLM timeout.

use anna_shared::rpc::{ProbeResult, RuntimeContext};

use crate::parsers::find_probe;
use crate::probe_answers;
use crate::router::{classify_query, QueryClass};

/// Result from deterministic answerer with metadata
pub struct DeterministicResult {
    pub answer: String,
    #[allow(dead_code)]
    pub grounded: bool,
    pub parsed_data_count: usize, // Number of parsed entries (0 = empty)
    pub route_class: String,      // Query class used for routing (for trace)
}

/// Try to produce a deterministic answer from available data
pub fn try_answer(
    query: &str,
    context: &RuntimeContext,
    probe_results: &[ProbeResult],
) -> Option<DeterministicResult> {
    let query_class = classify_query(query);
    let route_class = query_class.to_string();

    match query_class {
        // FAST PATH: SystemTriage - errors/warnings only, no specialist
        QueryClass::SystemTriage => crate::triage_answer::generate_triage_answer(probe_results),
        QueryClass::CpuInfo => probe_answers::answer_cpu_info(&context.hardware, probe_results)
            .map(|mut r| { r.route_class = route_class; r }),
        // v0.0.45: CpuCores - uses lscpu probe
        QueryClass::CpuCores => answer_cpu_cores(probe_results, &route_class),
        // v0.0.45: CpuTemp - uses sensors probe
        QueryClass::CpuTemp => answer_cpu_temp(probe_results, &route_class),
        QueryClass::RamInfo => probe_answers::answer_ram_info(&context.hardware, probe_results)
            .map(|mut r| { r.route_class = route_class; r }),
        QueryClass::GpuInfo => probe_answers::answer_gpu_info(&context.hardware)
            .map(|mut r| { r.route_class = route_class; r }),
        // v0.0.45: HardwareAudio - uses lspci_audio probe
        QueryClass::HardwareAudio => answer_hardware_audio(probe_results, &route_class),
        QueryClass::TopMemoryProcesses => probe_answers::answer_top_memory(probe_results)
            .map(|mut r| { r.route_class = route_class; r }),
        QueryClass::TopCpuProcesses => probe_answers::answer_top_cpu(probe_results)
            .map(|mut r| { r.route_class = route_class; r }),
        QueryClass::DiskSpace => probe_answers::answer_disk_space(probe_results)
            .map(|mut r| { r.route_class = route_class; r }),
        QueryClass::NetworkInterfaces => probe_answers::answer_network_interfaces(probe_results)
            .map(|mut r| { r.route_class = route_class; r }),
        QueryClass::Help => Some(answer_help(&route_class)),
        QueryClass::SystemSlow => probe_answers::answer_system_slow(probe_results)
            .map(|mut r| { r.route_class = route_class; r }),
        QueryClass::MemoryUsage => answer_memory_usage(probe_results, &route_class),
        // v0.0.45: MemoryFree - uses free probe (same as MemoryUsage)
        QueryClass::MemoryFree => answer_memory_free(probe_results, &route_class),
        QueryClass::DiskUsage => answer_disk_usage(probe_results, &route_class),
        QueryClass::ServiceStatus => answer_service_status(probe_results, &route_class),
        QueryClass::SystemHealthSummary => answer_system_health_summary(probe_results, &route_class),
        // RAG-first classes - handled by rag_answerer, not here
        QueryClass::BootTimeStatus |
        QueryClass::InstalledPackagesOverview |
        QueryClass::AppAlternatives => None,
        // v0.0.45: PackageCount - uses pacman_count probe
        QueryClass::PackageCount => answer_package_count(probe_results, &route_class),
        // v0.0.45: InstalledToolCheck - uses command_v probe
        QueryClass::InstalledToolCheck => answer_installed_tool_check(probe_results, &route_class),
        // v0.45.5: ConfigureEditor - needs clarification, cannot be answered deterministically
        QueryClass::ConfigureEditor => None,
        QueryClass::Unknown => None,
    }
}

/// Help response describing available commands
fn answer_help(route_class: &str) -> DeterministicResult {
    let answer = r#"**Anna - Linux System Assistant**

I can answer questions about your system:

**Hardware Information:**
- "What CPU do I have?" - Show CPU model and cores
- "How much RAM?" - Show total memory
- "What GPU?" - Show graphics card info

**Process Monitoring:**
- "Top memory processes" - Show processes using most RAM
- "What's using CPU?" - Show processes using most CPU

**Storage:**
- "Disk space" - Show filesystem usage with warnings

**Network:**
- "Network interfaces" - Show IPs and interface status

**System Health:**
- "System health" - Full system summary
- "Status report" - Overview of CPU, memory, disk

**Diagnostics:**
- "It's slow" - Run full diagnostic (CPU, memory, disk)

Ask a question to get started!"#;

    DeterministicResult {
        answer: answer.to_string(),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    }
}

// === ROUTE Phase: Typed query class handlers ===
// Uses helper functions from answers module

/// Answer memory usage query using parsed free output
fn answer_memory_usage(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "free")?;
    let answer = crate::answers::answer_from_free_probe(probe)?;
    Some(DeterministicResult {
        answer, grounded: true, parsed_data_count: 1, route_class: route_class.to_string(),
    })
}

/// Answer disk usage query using parsed df output
fn answer_disk_usage(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "df")?;
    let answer = crate::answers::answer_from_df_probe(probe)?;
    Some(DeterministicResult {
        answer, grounded: true, parsed_data_count: 1, route_class: route_class.to_string(),
    })
}

/// Answer service status query using parsed systemctl output
fn answer_service_status(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    // Try is-active probe first
    if let Some(probe) = find_probe(probes, "systemctl is-active") {
        let service_name = probe.command.strip_prefix("systemctl is-active ").unwrap_or("service");
        if let Some(answer) = crate::answers::answer_from_is_active_probe(probe, service_name) {
            return Some(DeterministicResult {
                answer, grounded: true, parsed_data_count: 1, route_class: route_class.to_string(),
            });
        }
    }
    // Try failed units probe
    if let Some(probe) = find_probe(probes, "systemctl --failed") {
        if let Some(answer) = crate::answers::answer_from_failed_units_probe(probe) {
            return Some(DeterministicResult {
                answer, grounded: true, parsed_data_count: 1, route_class: route_class.to_string(),
            });
        }
    }
    None
}

/// Answer system health summary using health brief (v0.0.32: relevant-only, not full report)
fn answer_system_health_summary(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    use crate::health_brief_builder::build_health_brief;

    // Build health brief from probes (only shows warnings/errors)
    let brief = build_health_brief(probes);

    // Always return an answer - even if healthy
    let answer = brief.format_answer();
    let count = if brief.all_healthy { 1 } else { brief.items.len() };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: count,
        route_class: route_class.to_string(),
    })
}

// === v0.0.45: New query class handlers ===

/// Answer CPU cores query using lscpu probe
fn answer_cpu_cores(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "lscpu")?;
    if probe.exit_code != 0 {
        return None;
    }

    // Parse lscpu output for cores and threads
    let mut cores: Option<u32> = None;
    let mut threads: Option<u32> = None;

    for line in probe.stdout.lines() {
        if line.starts_with("CPU(s):") {
            threads = line.split(':').nth(1).and_then(|s| s.trim().parse().ok());
        } else if line.starts_with("Core(s) per socket:") {
            if let Some(c) = line.split(':').nth(1).and_then(|s| s.trim().parse::<u32>().ok()) {
                cores = Some(cores.unwrap_or(0) + c);
            }
        } else if line.starts_with("Socket(s):") {
            if let Some(s) = line.split(':').nth(1).and_then(|s| s.trim().parse::<u32>().ok()) {
                if let Some(c) = cores {
                    cores = Some(c * s);
                }
            }
        }
    }

    let answer = match (cores, threads) {
        (Some(c), Some(t)) => format!("Your CPU has {} cores ({} threads).", c, t),
        (Some(c), None) => format!("Your CPU has {} cores.", c),
        (None, Some(t)) => format!("Your CPU has {} logical processors.", t),
        (None, None) => return None,
    };

    Some(DeterministicResult {
        answer, grounded: true, parsed_data_count: 1, route_class: route_class.to_string(),
    })
}

/// Answer CPU temperature query using sensors probe
fn answer_cpu_temp(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "sensors")?;
    if probe.exit_code != 0 {
        return None;
    }

    // Parse sensors output for CPU temperature
    let mut cpu_temps = Vec::new();
    let mut in_cpu_section = false;

    for line in probe.stdout.lines() {
        // Look for CPU-related sections
        if line.contains("coretemp") || line.contains("k10temp") || line.contains("cpu") {
            in_cpu_section = true;
        } else if !line.starts_with(' ') && !line.starts_with('\t') && !line.is_empty() {
            in_cpu_section = false;
        }

        // Extract temperatures
        if in_cpu_section || line.to_lowercase().contains("core") {
            if let Some(temp) = extract_temperature(line) {
                cpu_temps.push(temp);
            }
        }
    }

    if cpu_temps.is_empty() {
        return None;
    }

    let avg_temp = cpu_temps.iter().sum::<f32>() / cpu_temps.len() as f32;
    let max_temp = cpu_temps.iter().cloned().fold(0.0f32, f32::max);

    let answer = format!(
        "CPU temperature: {:.1}°C average, {:.1}°C max across {} sensors.",
        avg_temp, max_temp, cpu_temps.len()
    );

    Some(DeterministicResult {
        answer, grounded: true, parsed_data_count: cpu_temps.len(), route_class: route_class.to_string(),
    })
}

/// Extract temperature from a sensors output line
fn extract_temperature(line: &str) -> Option<f32> {
    // Look for patterns like "+45.0°C" or "45.0 C"
    for part in line.split_whitespace() {
        let cleaned = part.trim_start_matches('+').trim_end_matches('°').trim_end_matches('C');
        if let Ok(temp) = cleaned.parse::<f32>() {
            if temp > 0.0 && temp < 150.0 {
                return Some(temp);
            }
        }
    }
    None
}

/// Answer hardware audio query using lspci_audio probe
fn answer_hardware_audio(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "lspci")?;
    if probe.exit_code != 0 {
        return None;
    }

    // Filter for audio devices
    let audio_devices: Vec<&str> = probe.stdout.lines()
        .filter(|l| l.to_lowercase().contains("audio") || l.to_lowercase().contains("sound"))
        .collect();

    if audio_devices.is_empty() {
        return Some(DeterministicResult {
            answer: "No audio devices detected via lspci.".to_string(),
            grounded: true, parsed_data_count: 0, route_class: route_class.to_string(),
        });
    }

    let answer = if audio_devices.len() == 1 {
        format!("Audio device: {}", audio_devices[0].trim())
    } else {
        format!("Audio devices:\n{}", audio_devices.iter()
            .map(|d| format!("• {}", d.trim()))
            .collect::<Vec<_>>()
            .join("\n"))
    };

    Some(DeterministicResult {
        answer, grounded: true, parsed_data_count: audio_devices.len(), route_class: route_class.to_string(),
    })
}

/// Answer memory free query using free probe
fn answer_memory_free(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "free")?;
    let answer = crate::answers::answer_from_free_probe_available(probe)?;
    Some(DeterministicResult {
        answer, grounded: true, parsed_data_count: 1, route_class: route_class.to_string(),
    })
}

/// Answer package count query using pacman_count probe
fn answer_package_count(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "pacman")?;
    if probe.exit_code != 0 {
        return None;
    }

    // Parse package count from output
    let count: usize = probe.stdout.trim().parse().ok()?;

    let answer = format!("You have {} packages installed.", count);

    Some(DeterministicResult {
        answer, grounded: true, parsed_data_count: 1, route_class: route_class.to_string(),
    })
}

/// Answer installed tool check query using typed evidence (v0.45.7)
/// Handles both command_v probes AND pacman -Q probes.
/// Exit code 1 is VALID NEGATIVE EVIDENCE, not an error!
fn answer_installed_tool_check(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    use anna_shared::parsers::{parse_probe_result, ParsedProbeData, find_tool_evidence, find_package_evidence};

    // v0.45.7: Parse all probes to typed evidence
    let parsed: Vec<ParsedProbeData> = probes.iter()
        .map(|p| parse_probe_result(p))
        .collect();

    // Find tool existence evidence (from command -v, which, etc.)
    let tool_evidence: Vec<_> = parsed.iter()
        .filter_map(|p| p.as_tool())
        .collect();

    // Find package evidence (from pacman -Q, etc.)
    let package_evidence: Vec<_> = parsed.iter()
        .filter_map(|p| p.as_package())
        .collect();

    // If we have any tool or package evidence, we can answer
    if tool_evidence.is_empty() && package_evidence.is_empty() {
        return None;
    }

    // Build answer from evidence
    let mut answer_parts: Vec<String> = Vec::new();

    // Report tool evidence first (prefer this over package evidence)
    for tool in &tool_evidence {
        if tool.exists {
            let path_info = tool.path.as_ref()
                .map(|p| format!(" at `{}`", p))
                .unwrap_or_default();
            answer_parts.push(format!("Yes, **{}** is installed{}", tool.name, path_info));
        } else {
            answer_parts.push(format!("**{}** is not found in your PATH", tool.name));
        }
    }

    // Report package evidence
    for pkg in &package_evidence {
        if pkg.installed {
            let version_info = pkg.version.as_ref()
                .map(|v| format!(" (version {})", v))
                .unwrap_or_default();
            answer_parts.push(format!("**{}** is installed{}", pkg.name, version_info));
        } else {
            answer_parts.push(format!("**{}** package is not installed", pkg.name));
        }
    }

    // Combine evidence (may have both tool and package for same name)
    let answer = if answer_parts.len() == 1 {
        answer_parts[0].clone()
    } else {
        answer_parts.join("\n")
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: tool_evidence.len() + package_evidence.len(),
        route_class: route_class.to_string(),
    })
}

// Unit tests in tests/deterministic_tests.rs
