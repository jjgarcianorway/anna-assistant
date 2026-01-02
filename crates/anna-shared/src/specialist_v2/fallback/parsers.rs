//! Parsing helpers for extracting structured data from probe outputs.

/// Parse memory info from `free` command output
///
/// Returns: (available_bytes, total_bytes, used_bytes)
pub(super) fn parse_memory_from_free(output: &str) -> Option<(u64, u64, u64)> {
    for line in output.lines() {
        if line.to_lowercase().starts_with("mem:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let total = parse_size(parts[1])?;
                let used = parse_size(parts[2])?;
                let available = if parts.len() >= 7 {
                    parse_size(parts[6])? // free -h format
                } else {
                    parse_size(parts[3])? // older format
                };
                return Some((available, total, used));
            }
        }
    }
    None
}

/// Parse size string with units (K, M, G, T, Ki, Mi, Gi, Ti) into bytes
pub(super) fn parse_size(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    // Handle suffixes: K, M, G, T, Ki, Mi, Gi, Ti
    let (num_str, multiplier): (&str, u64) = if let Some(n) = s.strip_suffix("Gi") {
        (n, 1024 * 1024 * 1024)
    } else if let Some(n) = s.strip_suffix("Mi") {
        (n, 1024 * 1024)
    } else if let Some(n) = s.strip_suffix("Ki") {
        (n, 1024)
    } else if let Some(n) = s.strip_suffix('G') {
        (n, 1_000_000_000)
    } else if let Some(n) = s.strip_suffix('M') {
        (n, 1_000_000)
    } else if let Some(n) = s.strip_suffix('K') {
        (n, 1000)
    } else if let Some(n) = s.strip_suffix('T') {
        (n, 1_000_000_000_000_u64)
    } else {
        (s, 1)
    };

    num_str
        .parse::<f64>()
        .ok()
        .map(|n| (n * multiplier as f64) as u64)
}

/// Format bytes into human-readable string (MiB, GiB)
pub(super) fn format_bytes(bytes: u64) -> String {
    const GIB: u64 = 1024 * 1024 * 1024;
    const MIB: u64 = 1024 * 1024;

    if bytes >= GIB {
        format!("{:.1} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Parse failed services from systemctl output
pub(super) fn parse_failed_services(output: &str) -> Vec<String> {
    let mut failed = vec![];
    for line in output.lines() {
        let line_lower = line.to_lowercase();
        if line_lower.contains("failed") && line.contains(".service") {
            // Extract service name
            for word in line.split_whitespace() {
                if word.ends_with(".service") {
                    failed.push(word.to_string());
                    break;
                }
            }
        }
    }
    failed
}

/// Parse disk usage from df output
///
/// Returns: Vec<(mount_point, size, percent_used)>
pub(super) fn parse_disk_usage(output: &str) -> Vec<(String, String, u32)> {
    let mut partitions = vec![];
    for line in output.lines().skip(1) {
        // Skip header
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            if let Some(percent_str) = parts.iter().find(|s| s.ends_with('%')) {
                if let Ok(percent) = percent_str.trim_end_matches('%').parse::<u32>() {
                    let mount = parts.last().unwrap_or(&"/").to_string();
                    let size = parts.get(1).unwrap_or(&"?").to_string();
                    if !mount.starts_with("/dev") && !mount.starts_with("tmpfs") {
                        partitions.push((mount, size, percent));
                    }
                }
            }
        }
    }
    partitions
}

/// Parse network interfaces from ip addr output
///
/// Returns: Vec<(interface_name, state, ip_address)>
pub(super) fn parse_network_interfaces(output: &str) -> Vec<(String, String, String)> {
    let mut interfaces = vec![];
    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let name = parts[0].trim_end_matches(':').to_string();
            if name == "lo" {
                continue;
            }
            let state = if parts.len() >= 2 {
                parts[1].to_string()
            } else {
                "UNKNOWN".to_string()
            };
            let ip = parts.get(2).unwrap_or(&"").to_string();
            interfaces.push((name, state, ip));
        }
    }
    interfaces
}

/// Extract swap size from swapon or free output
pub(super) fn extract_swap_size(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.to_lowercase().contains("swap") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if part.to_lowercase().contains("swap") && i + 1 < parts.len() {
                    let size = parts[i + 1];
                    if size
                        .chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
                    {
                        return Some(size.to_string());
                    }
                }
            }
        }
    }
    None
}

/// Extract uptime string from uptime command output
pub(super) fn extract_uptime(output: &str) -> Option<String> {
    // "up X days, Y hours, Z minutes" format
    if let Some(start) = output.find("up ") {
        let rest = &output[start + 3..];
        if let Some(end) = rest.find(',') {
            return Some(rest[..end + 20.min(rest.len() - end)].trim().to_string());
        }
        return Some(
            rest.split_whitespace()
                .take(4)
                .collect::<Vec<_>>()
                .join(" "),
        );
    }
    None
}

/// Extract boot time from systemd-analyze output
pub(super) fn extract_boot_time(output: &str) -> Option<String> {
    // Look for "Xmin Ys" or "Xs" patterns
    for line in output.lines() {
        if line.contains("startup finished") || line.contains("=") {
            for word in line.split_whitespace() {
                if word.ends_with('s') || word.ends_with("min") {
                    return Some(word.to_string());
                }
            }
        }
    }
    None
}
