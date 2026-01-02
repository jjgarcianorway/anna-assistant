//! Extraction helper functions for parsing probe outputs.

/// Extract boot time fact from systemd-analyze output.
pub fn extract_boot_time_fact(analyze_output: &str) -> String {
    // Extract the summary line from systemd-analyze
    for line in analyze_output.lines() {
        if line.contains("Startup finished") {
            return line.to_string();
        }
    }
    format!(
        "Boot analysis: {}",
        analyze_output.lines().next().unwrap_or("unavailable")
    )
}

/// Extract top offenders from systemd-analyze blame output.
pub fn extract_top_offenders(blame_output: &str, count: usize) -> Vec<String> {
    blame_output
        .lines()
        .filter(|l| !l.is_empty())
        .take(count)
        .map(|l| l.trim().to_string())
        .collect()
}

/// Extract memory fact from free -h output.
pub fn extract_memory_fact(free_output: &str) -> String {
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

/// Extract top memory-consuming processes from ps output.
pub fn extract_top_mem_processes(ps_output: &str, count: usize) -> Vec<String> {
    ps_output
        .lines()
        .skip(1) // Skip header
        .take(count)
        .map(|l| l.trim().to_string())
        .collect()
}

/// Extract disk usage fact from df -h output.
pub fn extract_disk_fact(df_output: &str) -> String {
    let mut facts = Vec::new();
    for line in df_output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            let mount = parts.get(5).unwrap_or(&"?");
            let use_pct = parts.get(4).unwrap_or(&"?");
            let avail = parts.get(3).unwrap_or(&"?");
            if *mount == "/" || mount.starts_with("/home") || mount.starts_with("/boot") {
                facts.push(format!("{}: {} used, {} available", mount, use_pct, avail));
            }
        }
    }
    if facts.is_empty() {
        "Disk information unavailable".to_string()
    } else {
        facts.join("\n")
    }
}

/// Extract top directories from du output.
pub fn extract_top_directories(du_output: &str, count: usize) -> Vec<String> {
    du_output
        .lines()
        .take(count)
        .map(|l| l.trim().to_string())
        .collect()
}

/// Extract load average from uptime output.
pub fn extract_load_fact(uptime_output: &str) -> String {
    if let Some(idx) = uptime_output.find("load average:") {
        return uptime_output[idx..].to_string();
    }
    format!("Load: {}", uptime_output.trim())
}

/// Extract top CPU-consuming processes from top output.
pub fn extract_top_cpu_processes(top_output: &str, count: usize) -> Vec<String> {
    top_output
        .lines()
        .skip(1) // Skip header
        .take(count)
        .map(|l| l.trim().to_string())
        .collect()
}

/// Extract GPU hardware information from lspci output.
pub fn extract_gpu_fact(lspci_output: &str) -> String {
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

/// Extract kernel driver from lspci -k output.
pub fn extract_kernel_driver(lspci_k_output: &str) -> Option<String> {
    for line in lspci_k_output.lines() {
        if line.contains("Kernel driver in use:") {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 2 {
                return Some(parts[1].trim().to_string());
            }
        }
    }
    None
}

/// Extract GPU modules from lsmod output.
pub fn extract_gpu_modules(lsmod_output: &str) -> Vec<String> {
    lsmod_output
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.split_whitespace().next().unwrap_or("").to_string())
        .filter(|s| !s.is_empty())
        .take(5)
        .collect()
}
