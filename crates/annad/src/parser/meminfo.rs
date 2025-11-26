//! Parser for /proc/meminfo

use anyhow::Result;
use regex::Regex;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Parse /proc/meminfo output
pub fn parse(raw: &str) -> Result<Value> {
    let kv_re = Regex::new(r"^([^:]+):\s*(\d+)\s*kB?$").unwrap();
    let mut values: HashMap<String, u64> = HashMap::new();

    for line in raw.lines() {
        if let Some(caps) = kv_re.captures(line) {
            let key = caps.get(1).map_or("", |m| m.as_str().trim());
            let value: u64 = caps
                .get(2)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0);
            values.insert(key.to_string(), value);
        }
    }

    let mem_total_kb = values.get("MemTotal").copied().unwrap_or(0);
    let mem_free_kb = values.get("MemFree").copied().unwrap_or(0);
    let mem_available_kb = values.get("MemAvailable").copied().unwrap_or(0);
    let buffers_kb = values.get("Buffers").copied().unwrap_or(0);
    let cached_kb = values.get("Cached").copied().unwrap_or(0);
    let swap_total_kb = values.get("SwapTotal").copied().unwrap_or(0);
    let swap_free_kb = values.get("SwapFree").copied().unwrap_or(0);

    // Convert to bytes and calculate derived values
    let mem_total = mem_total_kb * 1024;
    let mem_available = mem_available_kb * 1024;
    let mem_used = mem_total.saturating_sub(mem_available);
    let swap_total = swap_total_kb * 1024;
    let swap_used = swap_total_kb.saturating_sub(swap_free_kb) * 1024;

    let mem_percent = if mem_total > 0 {
        (mem_used as f64 / mem_total as f64) * 100.0
    } else {
        0.0
    };

    let swap_percent = if swap_total > 0 {
        (swap_used as f64 / swap_total as f64) * 100.0
    } else {
        0.0
    };

    Ok(json!({
        "mem_total_bytes": mem_total,
        "mem_available_bytes": mem_available,
        "mem_used_bytes": mem_used,
        "mem_free_bytes": mem_free_kb * 1024,
        "buffers_bytes": buffers_kb * 1024,
        "cached_bytes": cached_kb * 1024,
        "swap_total_bytes": swap_total,
        "swap_used_bytes": swap_used,
        "swap_free_bytes": swap_free_kb * 1024,
        "mem_percent": (mem_percent * 10.0).round() / 10.0,
        "swap_percent": (swap_percent * 10.0).round() / 10.0,
        "mem_total_gb": (mem_total as f64 / 1073741824.0 * 10.0).round() / 10.0,
        "mem_available_gb": (mem_available as f64 / 1073741824.0 * 10.0).round() / 10.0,
        "swap_total_gb": (swap_total as f64 / 1073741824.0 * 10.0).round() / 10.0,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_meminfo() {
        let sample = r#"MemTotal:       32768000 kB
MemFree:         8192000 kB
MemAvailable:   16384000 kB
Buffers:          512000 kB
Cached:          4096000 kB
SwapTotal:       8388608 kB
SwapFree:        8388608 kB
"#;
        let result = parse(sample).unwrap();
        // 32768000 KB = 31.25 GB (rounds to 31.3)
        assert_eq!(result["mem_total_gb"], 31.3);
        assert_eq!(result["swap_percent"], 0.0);
    }
}
