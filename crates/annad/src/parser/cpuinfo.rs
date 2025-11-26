//! Parser for /proc/cpuinfo

use anyhow::Result;
use regex::Regex;
use serde_json::{json, Value};
use std::collections::HashSet;

/// Parse /proc/cpuinfo output
pub fn parse(raw: &str) -> Result<Value> {
    let mut model_name = String::new();
    let mut vendor_id = String::new();
    let mut cpu_family = String::new();
    let mut physical_ids: HashSet<String> = HashSet::new();
    let mut core_ids: HashSet<String> = HashSet::new();
    let mut processor_count = 0u32;
    let mut cpu_mhz: Option<f64> = None;
    let mut cache_size = String::new();
    let mut flags: Vec<String> = Vec::new();

    let kv_re = Regex::new(r"^([^:]+)\s*:\s*(.*)$").unwrap();

    for line in raw.lines() {
        if let Some(caps) = kv_re.captures(line) {
            let key = caps.get(1).map_or("", |m| m.as_str().trim());
            let value = caps.get(2).map_or("", |m| m.as_str().trim());

            match key {
                "processor" => processor_count += 1,
                "model name" if model_name.is_empty() => model_name = value.to_string(),
                "vendor_id" if vendor_id.is_empty() => vendor_id = value.to_string(),
                "cpu family" if cpu_family.is_empty() => cpu_family = value.to_string(),
                "physical id" => {
                    physical_ids.insert(value.to_string());
                }
                "core id" => {
                    core_ids.insert(value.to_string());
                }
                "cpu MHz" if cpu_mhz.is_none() => {
                    cpu_mhz = value.parse().ok();
                }
                "cache size" if cache_size.is_empty() => cache_size = value.to_string(),
                "flags" if flags.is_empty() => {
                    flags = value.split_whitespace().map(String::from).collect();
                }
                _ => {}
            }
        }
    }

    // Calculate physical cores (unique core_ids per physical_id)
    let physical_cores = if physical_ids.is_empty() {
        // Single socket system - core_ids represent physical cores
        core_ids.len().max(1) as u32
    } else {
        // Multi-socket - estimate from unique combinations
        (physical_ids.len() * core_ids.len().max(1)) as u32
    };

    let logical_cores = processor_count;
    let sockets = physical_ids.len().max(1) as u32;
    let threads_per_core = if physical_cores > 0 {
        logical_cores / physical_cores
    } else {
        1
    };

    Ok(json!({
        "model_name": model_name,
        "vendor_id": vendor_id,
        "cpu_family": cpu_family,
        "sockets": sockets,
        "physical_cores": physical_cores,
        "logical_cores": logical_cores,
        "threads_per_core": threads_per_core,
        "cpu_mhz": cpu_mhz,
        "cache_size": cache_size,
        "flags": flags,
        "has_hyperthreading": threads_per_core > 1,
        "has_avx": flags.contains(&"avx".to_string()),
        "has_avx2": flags.contains(&"avx2".to_string()),
        "has_avx512": flags.iter().any(|f| f.starts_with("avx512")),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cpuinfo() {
        let sample = r#"processor	: 0
vendor_id	: GenuineIntel
cpu family	: 6
model name	: Intel(R) Core(TM) i7-10700K CPU @ 3.80GHz
physical id	: 0
core id	: 0
cpu MHz		: 3800.000
cache size	: 16384 KB
flags		: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr avx avx2

processor	: 1
vendor_id	: GenuineIntel
cpu family	: 6
model name	: Intel(R) Core(TM) i7-10700K CPU @ 3.80GHz
physical id	: 0
core id	: 1
cpu MHz		: 3800.000
"#;
        let result = parse(sample).unwrap();
        assert_eq!(result["vendor_id"], "GenuineIntel");
        assert_eq!(result["logical_cores"], 2);
        assert!(result["has_avx"].as_bool().unwrap());
    }
}
