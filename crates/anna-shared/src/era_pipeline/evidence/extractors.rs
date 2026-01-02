//! Standard fact extractors for common probes.

use super::fact_value::FactValue;
use std::collections::HashMap;

/// Extract memory facts from `free -h` output.
pub fn extract_memory(output: &str) -> HashMap<String, FactValue> {
    let mut facts = HashMap::new();

    for line in output.lines() {
        if line.starts_with("Mem:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                if let Some(total) = parse_size_gib(parts.get(1).unwrap_or(&"")) {
                    facts.insert("memory.total_gib".to_string(), FactValue::Number(total));
                }
                if let Some(used) = parse_size_gib(parts.get(2).unwrap_or(&"")) {
                    facts.insert("memory.used_gib".to_string(), FactValue::Number(used));
                }
                if let Some(free) = parse_size_gib(parts.get(3).unwrap_or(&"")) {
                    facts.insert("memory.free_gib".to_string(), FactValue::Number(free));
                }
                if let Some(available) = parse_size_gib(parts.get(6).unwrap_or(&"")) {
                    facts.insert(
                        "memory.available_gib".to_string(),
                        FactValue::Number(available),
                    );
                }
            }
        }
    }

    facts
}

/// Extract boot facts from `systemd-analyze` output.
pub fn extract_boot_time(output: &str) -> HashMap<String, FactValue> {
    let mut facts = HashMap::new();

    // Parse "Startup finished in Xs (firmware) + Xs (loader) + Xs (kernel) + Xs (userspace) = Xs"
    if let Some(total) = extract_total_seconds(output) {
        facts.insert("boot.total_time_s".to_string(), FactValue::Number(total));
    }

    facts
}

/// Extract blame list from `systemd-analyze blame` output.
pub fn extract_blame(output: &str) -> HashMap<String, FactValue> {
    let mut facts = HashMap::new();
    let mut services = Vec::new();

    for line in output.lines().take(10) {
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        if parts.len() >= 2 {
            let time = parts[0];
            let service = parts[1];
            services.push(format!("{} ({})", service, time));
        }
    }

    if !services.is_empty() {
        // First one is slowest
        if let Some(first) = services.first() {
            facts.insert(
                "boot.slowest_service".to_string(),
                FactValue::String(first.clone()),
            );
        }
        facts.insert("boot.blame".to_string(), FactValue::List(services));
    }

    facts
}

/// Extract disk facts from `df -h` output.
pub fn extract_disk(output: &str) -> HashMap<String, FactValue> {
    let mut facts = HashMap::new();

    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            let mount = parts[5];
            let used_pct = parts[4].trim_end_matches('%');

            if mount == "/" {
                if let Ok(pct) = used_pct.parse::<f64>() {
                    facts.insert("disk.root_used_pct".to_string(), FactValue::Number(pct));
                }
                if let Some(avail) = parse_size_gib(parts[3]) {
                    facts.insert("disk.root_free_gib".to_string(), FactValue::Number(avail));
                }
            }
        }
    }

    facts
}

/// Extract failed services from `systemctl --failed` output.
pub fn extract_failed_services(output: &str) -> HashMap<String, FactValue> {
    let mut facts = HashMap::new();
    let mut failed = Vec::new();

    for line in output.lines() {
        if line.contains("failed") && line.contains(".service") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(service) = parts.first() {
                failed.push(service.to_string());
            }
        }
    }

    facts.insert(
        "services.failed_count".to_string(),
        FactValue::Number(failed.len() as f64),
    );
    facts.insert("services.failed_list".to_string(), FactValue::List(failed));

    facts
}

/// Parse size string (e.g., "16Gi", "500Mi") to GiB.
fn parse_size_gib(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.ends_with("Gi") || s.ends_with("G") {
        s.trim_end_matches("Gi").trim_end_matches("G").parse().ok()
    } else if s.ends_with("Mi") || s.ends_with("M") {
        s.trim_end_matches("Mi")
            .trim_end_matches("M")
            .parse::<f64>()
            .ok()
            .map(|m| m / 1024.0)
    } else if s.ends_with("Ki") || s.ends_with("K") {
        s.trim_end_matches("Ki")
            .trim_end_matches("K")
            .parse::<f64>()
            .ok()
            .map(|k| k / (1024.0 * 1024.0))
    } else {
        s.parse().ok()
    }
}

/// Extract total seconds from systemd-analyze output.
fn extract_total_seconds(output: &str) -> Option<f64> {
    // Look for "= Xs" or "= Xmin Xs"
    if let Some(idx) = output.find('=') {
        let after_eq = &output[idx + 1..];
        let total_str = after_eq.trim();

        // Parse "Xmin Ys" or "Xs"
        if total_str.contains("min") {
            let parts: Vec<&str> = total_str.split_whitespace().collect();
            let mut total = 0.0;
            for part in parts {
                if part.ends_with("min") {
                    if let Ok(mins) = part.trim_end_matches("min").parse::<f64>() {
                        total += mins * 60.0;
                    }
                } else if part.ends_with('s') && !part.ends_with("ms") {
                    if let Ok(secs) = part.trim_end_matches('s').parse::<f64>() {
                        total += secs;
                    }
                }
            }
            return Some(total);
        } else if total_str.ends_with('s') {
            return total_str.trim_end_matches('s').trim().parse().ok();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_extraction() {
        let output =
            "              total        used        free      shared  buff/cache   available
Mem:           31Gi       8.2Gi        15Gi       1.2Gi       7.8Gi        21Gi";

        let facts = extract_memory(output);
        assert!(facts.get("memory.total_gib").is_some());
        assert!(facts.get("memory.free_gib").is_some());
    }
}
