//! Evidence Extractors - v0.0.439.
//!
//! Helper functions to extract structured information from probe outputs.

/// Extract memory summary from free -h output.
pub fn extract_memory_summary(free_output: &str) -> String {
    // Parse "free -h" output
    for line in free_output.lines() {
        if line.starts_with("Mem:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let total = parts.get(1).unwrap_or(&"?");
                let used = parts.get(2).unwrap_or(&"?");
                let available = parts.get(6).unwrap_or(parts.get(3).unwrap_or(&"?"));
                return format!(
                    "Memory: {} total, {} used, {} available",
                    total, used, available
                );
            }
        }
    }
    "Memory information unavailable".to_string()
}

/// Extract disk summary from df -h output.
pub fn extract_disk_summary(df_output: &str) -> String {
    let mut summaries = Vec::new();
    for line in df_output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            let mount = parts.get(5).unwrap_or(&"?");
            let use_pct = parts.get(4).unwrap_or(&"?");
            let avail = parts.get(3).unwrap_or(&"?");
            if *mount == "/" || mount.starts_with("/home") {
                summaries.push(format!("{}: {} used, {} available", mount, use_pct, avail));
            }
        }
    }
    if summaries.is_empty() {
        "Disk information unavailable".to_string()
    } else {
        summaries.join("\n")
    }
}

/// Extract boot time from systemd-analyze output.
pub fn extract_boot_time(analyze_output: &str) -> String {
    // systemd-analyze output like "Startup finished in 3.5s (kernel) + 5.2s (userspace) = 8.7s"
    if let Some(line) = analyze_output.lines().next() {
        if line.contains("Startup finished") {
            return line.to_string();
        }
    }
    format!(
        "Boot analysis: {}",
        analyze_output.lines().next().unwrap_or("unavailable")
    )
}

/// Extract top blame entries from systemd-analyze blame output.
pub fn extract_top_blame(blame_output: &str, count: usize) -> String {
    blame_output
        .lines()
        .take(count)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Extract load average from uptime output.
pub fn extract_load_average(uptime_output: &str) -> String {
    // Parse "uptime" output for load averages
    if let Some(idx) = uptime_output.find("load average:") {
        let load_part = &uptime_output[idx..];
        return load_part.to_string();
    }
    format!("Load: {}", uptime_output.trim())
}

/// Extract failed services from systemctl output.
pub fn extract_failed_services(failed_output: &str) -> String {
    let lines: Vec<&str> = failed_output.lines().collect();
    if lines.is_empty() || failed_output.contains("0 loaded units") {
        return "No failed services.".to_string();
    }
    let count = lines.len().saturating_sub(1); // Exclude header
    if count == 0 {
        "No failed services.".to_string()
    } else {
        format!("{} failed service(s):\n{}", count, failed_output.trim())
    }
}

/// Extract GPU information from lspci output.
pub fn extract_gpu_info(lspci_output: &str) -> String {
    let gpu_lines: Vec<&str> = lspci_output
        .lines()
        .filter(|l| l.contains("VGA") || l.contains("3D") || l.contains("Display"))
        .collect();
    if gpu_lines.is_empty() {
        "No GPU detected".to_string()
    } else {
        gpu_lines.join("\n")
    }
}

/// Extract GPU driver information from lspci -k and lsmod output.
pub fn extract_gpu_driver(lspci_k_output: &str, lsmod: Option<&str>) -> String {
    let mut result = String::new();

    // Extract kernel driver from lspci -k
    for line in lspci_k_output.lines() {
        if line.contains("Kernel driver") || line.contains("Kernel modules") {
            result.push_str(line.trim());
            result.push('\n');
        }
    }

    if let Some(lsmod_out) = lsmod {
        if !lsmod_out.is_empty() {
            result.push_str("Loaded modules: ");
            result.push_str(
                lsmod_out
                    .lines()
                    .take(3)
                    .collect::<Vec<_>>()
                    .join(", ")
                    .as_str(),
            );
        }
    }

    if result.is_empty() {
        "GPU driver information unavailable".to_string()
    } else {
        result.trim().to_string()
    }
}

/// Extract DNS status from resolvectl output.
pub fn extract_dns_status(resolvectl_output: &str) -> String {
    // Extract key DNS info
    let mut servers = Vec::new();
    for line in resolvectl_output.lines() {
        if line.contains("DNS Servers") || line.contains("Current DNS") {
            servers.push(line.trim());
        }
    }
    if servers.is_empty() {
        format!(
            "DNS status:\n{}",
            resolvectl_output
                .lines()
                .take(5)
                .collect::<Vec<_>>()
                .join("\n")
        )
    } else {
        servers.join("\n")
    }
}

/// Extract WiFi status from iw output.
pub fn extract_wifi_status(iw_output: &str) -> String {
    if iw_output.contains("Not connected") || iw_output.contains("No such device") {
        return "WiFi: Not connected".to_string();
    }

    let mut info = Vec::new();
    for line in iw_output.lines() {
        if line.contains("SSID") || line.contains("signal") || line.contains("freq") {
            info.push(line.trim());
        }
    }
    if info.is_empty() {
        iw_output.lines().take(3).collect::<Vec<_>>().join("\n")
    } else {
        format!("WiFi: {}", info.join(", "))
    }
}

/// Extract sensors summary from sensors output.
pub fn extract_sensors_summary(sensors_output: &str) -> String {
    let mut temps = Vec::new();
    for line in sensors_output.lines() {
        if line.contains("°C") || line.contains("Core") || line.contains("temp") {
            temps.push(line.trim());
        }
    }
    if temps.is_empty() {
        "No temperature sensors detected".to_string()
    } else {
        temps.into_iter().take(5).collect::<Vec<_>>().join("\n")
    }
}

/// Extract recent errors from journalctl output.
pub fn extract_recent_errors(logs_output: &str) -> String {
    let lines: Vec<&str> = logs_output.lines().collect();
    if lines.is_empty() {
        return "No recent errors in logs.".to_string();
    }
    format!("{} recent error(s):\n{}", lines.len(), logs_output.trim())
}

/// Extract firewall status from firewall command output.
pub fn extract_firewall_status(fw_output: &str) -> String {
    if fw_output.contains("inactive") || fw_output.contains("not running") {
        "Firewall: Inactive".to_string()
    } else if fw_output.contains("active") || fw_output.contains("running") {
        "Firewall: Active".to_string()
    } else {
        format!(
            "Firewall status: {}",
            fw_output.lines().next().unwrap_or("unknown")
        )
    }
}

/// Extract package updates summary from checkupdates output.
pub fn extract_updates_summary(updates_output: &str) -> String {
    let lines: Vec<&str> = updates_output.lines().collect();
    if lines.is_empty() {
        "System is up to date (no pending updates).".to_string()
    } else {
        format!("{} package update(s) available.", lines.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_memory_summary() {
        let output = "              total        used        free      shared  buff/cache   available\nMem:           31Gi       8.2Gi        15Gi       1.2Gi       7.8Gi        21Gi";
        let summary = extract_memory_summary(output);
        assert!(summary.contains("31Gi"));
        assert!(summary.contains("available"));
    }

    #[test]
    fn test_extract_failed_services() {
        let empty = "";
        assert_eq!(
            extract_failed_services(empty),
            "No failed services."
        );

        let with_failed = "  UNIT                    LOAD   ACTIVE SUB    DESCRIPTION\n● foo.service           loaded failed failed Foo Service";
        let summary = extract_failed_services(with_failed);
        assert!(summary.contains("failed"));
    }
}
