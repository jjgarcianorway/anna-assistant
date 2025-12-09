//! CPU answer handlers (v0.0.176).

use anna_shared::rpc::ProbeResult;

use super::DeterministicResult;
use crate::parsers::find_probe;

/// Answer CPU cores query using lscpu probe
pub fn answer_cpu_cores(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
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
            if let Some(c) = line
                .split(':')
                .nth(1)
                .and_then(|s| s.trim().parse::<u32>().ok())
            {
                cores = Some(cores.unwrap_or(0) + c);
            }
        } else if line.starts_with("Socket(s):") {
            if let Some(s) = line
                .split(':')
                .nth(1)
                .and_then(|s| s.trim().parse::<u32>().ok())
            {
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
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer CPU temperature query using sensors probe
pub fn answer_cpu_temp(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
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
        avg_temp,
        max_temp,
        cpu_temps.len()
    );

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: cpu_temps.len(),
        route_class: route_class.to_string(),
    })
}

/// Extract temperature from a sensors output line
fn extract_temperature(line: &str) -> Option<f32> {
    // Look for patterns like "+45.0°C" or "45.0 C"
    for part in line.split_whitespace() {
        let cleaned = part
            .trim_start_matches('+')
            .trim_end_matches('°')
            .trim_end_matches('C');
        if let Ok(temp) = cleaned.parse::<f32>() {
            if temp > 0.0 && temp < 150.0 {
                return Some(temp);
            }
        }
    }
    None
}
