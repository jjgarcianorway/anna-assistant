//! System resource queries (swap, disk, memory)

use anna_shared::parsers::{parse_probe_result, ParsedProbeData};
use anna_shared::rpc::ProbeResult;
use tracing::info;

use super::DirectAnswerResult;

/// Swap answer
pub(crate) fn try_swap_answer(query: &str, probes: &[ProbeResult]) -> Option<DirectAnswerResult> {
    if !query.contains("swap") {
        return None;
    }

    for probe in probes {
        let cmd = probe.command.to_lowercase();

        // /proc/swaps output
        if cmd.contains("/proc/swap") || cmd.contains("swapon") {
            let lines: Vec<&str> = probe.stdout.lines().collect();

            // Just header = no swap
            if lines.len() <= 1 {
                info!("v0.0.403: Direct swap answer - no swap");
                return Some(DirectAnswerResult {
                    answer: "**No swap** is configured on this system.".to_string(),
                    confidence: 95,
                });
            }

            // Has swap entries
            let mut answer = "**Swap is configured:**\n".to_string();
            for line in lines.iter().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let filename = parts[0];
                    let size_kb: u64 = parts[2].parse().unwrap_or(0);
                    let size_mb = size_kb / 1024;
                    answer.push_str(&format!("- {} ({} MB)\n", filename, size_mb));
                }
            }
            return Some(DirectAnswerResult {
                answer,
                confidence: 95,
            });
        }

        // free -h output
        if cmd.contains("free") {
            for line in probe.stdout.lines() {
                if line.starts_with("Swap:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let total = parts[1];
                        if total == "0" || total == "0B" || total == "0K" || total == "0M" {
                            return Some(DirectAnswerResult {
                                answer: "**No swap** is configured on this system.".to_string(),
                                confidence: 90,
                            });
                        }
                        return Some(DirectAnswerResult {
                            answer: format!("**Swap:** {} total", total),
                            confidence: 90,
                        });
                    }
                }
            }
        }
    }

    None
}

/// Disk answer
pub(crate) fn try_disk_answer(query: &str, probes: &[ProbeResult]) -> Option<DirectAnswerResult> {
    if !query.contains("disk") && !query.contains("space") && !query.contains("storage") {
        return None;
    }

    for probe in probes {
        let cmd = probe.command.to_lowercase();
        if cmd.contains("df") {
            let parsed = parse_probe_result(probe);
            if let ParsedProbeData::Disk(disks) = parsed {
                let mut answer = "**Disk Usage:**\n".to_string();
                for disk in &disks {
                    let status = if disk.percent_used >= 90 {
                        " [CRITICAL]"
                    } else if disk.percent_used >= 80 {
                        " [WARNING]"
                    } else {
                        ""
                    };
                    answer.push_str(&format!(
                        "- {} - {}% used{}\n",
                        disk.mount, disk.percent_used, status
                    ));
                }
                info!("v0.0.403: Direct disk answer");
                return Some(DirectAnswerResult {
                    answer,
                    confidence: 95,
                });
            }
        }
    }

    None
}

/// Memory answer
pub(crate) fn try_memory_answer(query: &str, probes: &[ProbeResult]) -> Option<DirectAnswerResult> {
    if !query.contains("memory") && !query.contains("ram") {
        return None;
    }

    for probe in probes {
        let cmd = probe.command.to_lowercase();
        if cmd.contains("free") {
            let parsed = parse_probe_result(probe);
            if let ParsedProbeData::Memory(mem) = parsed {
                let used_gb = mem.used_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
                let total_gb = mem.total_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
                let avail_gb = mem.available_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
                let used_pct = (mem.used_bytes as f64 / mem.total_bytes as f64 * 100.0) as u8;

                let status = if used_pct >= 90 {
                    " [HIGH]"
                } else if used_pct >= 75 {
                    " [MODERATE]"
                } else {
                    ""
                };

                let answer = format!(
                    "**Memory Usage:**\n- Used: {:.1} GB / {:.1} GB ({}%){}\n- Available: {:.1} GB",
                    used_gb, total_gb, used_pct, status, avail_gb
                );
                info!("v0.0.403: Direct memory answer");
                return Some(DirectAnswerResult {
                    answer,
                    confidence: 95,
                });
            }
        }
    }

    None
}
