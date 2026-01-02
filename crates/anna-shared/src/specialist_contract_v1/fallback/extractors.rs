//! Fallback extractors - v0.0.440.
//!
//! Functions to extract answers from probe output.

/// Extract memory information from `free -h` output.
pub fn extract_memory_from_free(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.starts_with("Mem:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let total = parts.get(1)?;
                let available = parts.get(6).or(parts.get(3))?;
                return Some(format!("Memory: {} total, {} available.", total, available));
            }
        }
    }
    None
}

/// Extract boot time from `systemd-analyze` output.
pub fn extract_boot_time(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.contains("Startup finished") {
            return Some(line.trim().to_string());
        }
    }
    output
        .lines()
        .next()
        .map(|l| format!("Boot time: {}", l.trim()))
}

/// Extract disk usage from `df -h` output.
pub fn extract_disk_usage(output: &str) -> Option<String> {
    let mut results = Vec::new();
    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            let mount = parts.get(5)?;
            let use_pct = parts.get(4)?;
            let avail = parts.get(3)?;
            if *mount == "/" || mount.starts_with("/home") {
                results.push(format!("{}: {} used, {} available", mount, use_pct, avail));
            }
        }
    }
    if results.is_empty() {
        None
    } else {
        Some(results.join(". "))
    }
}

/// Extract failed services from `systemctl --failed` output.
pub fn extract_failed_services(output: &str) -> Option<String> {
    let lines: Vec<&str> = output.lines().collect();
    if lines.is_empty() || output.contains("0 loaded units") {
        return Some("No failed services.".to_string());
    }

    let count = lines.len().saturating_sub(1);
    if count == 0 {
        Some("No failed services.".to_string())
    } else {
        Some(format!("{} failed service(s).", count))
    }
}

/// Extract load average from `uptime` output.
pub fn extract_load_average(output: &str) -> Option<String> {
    if let Some(idx) = output.find("load average:") {
        return Some(output[idx..].trim().to_string());
    }
    None
}

/// Extract GPU information from `lspci | grep -i vga` output.
pub fn extract_gpu_info(output: &str) -> Option<String> {
    let gpu_lines: Vec<&str> = output
        .lines()
        .filter(|l| l.contains("VGA") || l.contains("3D") || l.contains("Display"))
        .collect();

    if gpu_lines.is_empty() {
        Some("No GPU detected.".to_string())
    } else {
        // Extract just the device name
        Some(gpu_lines.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_failed_services() {
        assert_eq!(
            extract_failed_services(""),
            Some("No failed services.".to_string())
        );

        assert_eq!(
            extract_failed_services("0 loaded units"),
            Some("No failed services.".to_string())
        );
    }
}
