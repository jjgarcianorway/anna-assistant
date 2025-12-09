//! Best-effort summary generator from probe results.
//!
//! Extracted from answers.rs (v0.0.157) for modularization.
//! Generates summaries when specialist times out but evidence exists.

use anna_shared::parsers::{parse_df, parse_free, parse_lsblk, parse_lscpu, BlockDeviceType};
use anna_shared::rpc::ProbeResult;

use crate::answers::format_bytes_short;
use crate::parsers::{parse_df_h, parse_ip_addr, parse_ps_aux};

/// Generate best-effort summary from whatever probe results exist.
/// This is used when specialist times out but we have evidence.
/// Returns (answer, parsed_count) where parsed_count > 0 means useful data.
pub fn generate_best_effort_summary(probe_results: &[ProbeResult]) -> Option<(String, usize)> {
    let mut sections: Vec<String> = Vec::new();
    let mut parsed_count = 0;

    // Try to parse each probe type we know about
    for probe in probe_results {
        if probe.exit_code != 0 {
            continue; // Skip failed probes
        }

        // Memory from free (anna-shared parser)
        if probe.command.starts_with("free") {
            if let Ok(mem) = parse_free("free", &probe.stdout) {
                let used_pct = (mem.used_bytes as f64 / mem.total_bytes as f64 * 100.0) as u8;
                sections.push(format!(
                    "Memory: {} used of {} total ({}% used)",
                    format_bytes_short(mem.used_bytes),
                    format_bytes_short(mem.total_bytes),
                    used_pct
                ));
                parsed_count += 1;
            }
        }

        // Disk from df (anna-shared parser for typed data)
        if probe.command.starts_with("df") {
            if let Ok(disks) = parse_df("df", &probe.stdout) {
                if !disks.is_empty() {
                    let disk_lines: Vec<String> = disks
                        .iter()
                        .map(|d| {
                            let status = if d.percent_used >= 90 {
                                " [CRITICAL]"
                            } else if d.percent_used >= 80 {
                                " [WARNING]"
                            } else {
                                ""
                            };
                            format!("  {} is {}% full{}", d.mount, d.percent_used, status)
                        })
                        .collect();
                    sections.push(format!("Disk Usage:\n{}", disk_lines.join("\n")));
                    parsed_count += 1;
                }
            } else {
                // Fallback to simpler parser
                let disks = parse_df_h(&probe.stdout);
                if !disks.is_empty() {
                    let disk_lines: Vec<String> = disks
                        .iter()
                        .map(|d| {
                            let status = if d.use_percent >= 90 {
                                " [CRITICAL]"
                            } else if d.use_percent >= 80 {
                                " [WARNING]"
                            } else {
                                ""
                            };
                            format!("  {} is {}% full{}", d.mount, d.use_percent, status)
                        })
                        .collect();
                    sections.push(format!("Disk Usage:\n{}", disk_lines.join("\n")));
                    parsed_count += 1;
                }
            }
        }

        // Block devices from lsblk
        if probe.command.starts_with("lsblk") {
            if let Ok(devices) = parse_lsblk("lsblk", &probe.stdout) {
                let total: u64 = devices
                    .iter()
                    .filter(|d| d.device_type == BlockDeviceType::Disk)
                    .map(|d| d.size_bytes)
                    .sum();
                if total > 0 {
                    sections.push(format!("Storage: {} total", format_bytes_short(total)));
                    parsed_count += 1;
                }
            }
        }

        // CPU from lscpu
        if probe.command.starts_with("lscpu") {
            if let Ok(cpu) = parse_lscpu("lscpu", &probe.stdout) {
                sections.push(format!("CPU: {} ({} cores)", cpu.model_name, cpu.cpu_count));
                parsed_count += 1;
            }
        }

        // Network from ip addr (annad parser)
        if probe.command.starts_with("ip addr") || probe.command.starts_with("ip a") {
            let ifaces = parse_ip_addr(&probe.stdout);
            let up_ifaces: Vec<_> = ifaces.iter().filter(|i| i.state == "UP").collect();
            if !up_ifaces.is_empty() {
                let iface_lines: Vec<String> = up_ifaces
                    .iter()
                    .map(|i| {
                        let addr = i.ipv4.as_deref().unwrap_or("no IPv4");
                        format!("  {}: {}", i.name, addr)
                    })
                    .collect();
                sections.push(format!(
                    "Network Interfaces (UP):\n{}",
                    iface_lines.join("\n")
                ));
                parsed_count += 1;
            }
        }

        // Top memory/CPU processes from ps aux (annad parser)
        if probe.command.contains("ps") && probe.command.contains("--sort") {
            let procs = parse_ps_aux(&probe.stdout, 5);
            if !procs.is_empty() {
                let is_mem_sort = probe.command.contains("-%mem") || probe.command.contains("-rss");
                let label = if is_mem_sort { "Top Memory" } else { "Top CPU" };
                let proc_lines: Vec<String> = procs
                    .iter()
                    .take(3)
                    .map(|p| {
                        let rss = p.rss.as_deref().unwrap_or("?");
                        format!(
                            "  {} - {}% CPU, {}% MEM, {}",
                            p.command, p.cpu_percent, p.mem_percent, rss
                        )
                    })
                    .collect();
                sections.push(format!("{} Processes:\n{}", label, proc_lines.join("\n")));
                parsed_count += 1;
            }
        }
    }

    if parsed_count == 0 {
        return None;
    }

    // v0.0.141: Build final summary with friendlier message
    let header = "**System Information**";
    let body = sections.join("\n\n");
    let footer = "\n---\n*Based on system probe data. For more details, try a specific question like \"what cpu\" or \"disk space\".*";

    Some((format!("{}\n\n{}{}", header, body, footer), parsed_count))
}
