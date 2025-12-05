//! Health Brief Builder - Creates health briefs from probe results (v0.0.32).
//!
//! Uses existing parsers to extract actionable health items from probe outputs.

use anna_shared::health_brief::{BriefItemKind, BriefSeverity, HealthBrief, BriefItem};
use anna_shared::rpc::ProbeResult;

use crate::parsers::{find_probe, parse_df_h, parse_ps_aux};

/// Build a health brief from probe results
pub fn build_health_brief(probes: &[ProbeResult]) -> HealthBrief {
    let mut brief = HealthBrief::new();

    // Check disk usage
    check_disk_usage(probes, &mut brief);

    // Check memory usage
    check_memory_usage(probes, &mut brief);

    // Check failed services
    check_failed_services(probes, &mut brief);

    // Check high CPU processes
    check_high_cpu(probes, &mut brief);

    brief.finalize();
    brief
}

/// Check disk usage from df -h output
fn check_disk_usage(probes: &[ProbeResult], brief: &mut HealthBrief) {
    // Try "df -h" first, then just "df"
    let probe = find_probe(probes, "df -h").or_else(|| find_probe(probes, "df"));
    if let Some(probe) = probe {
        let filesystems = parse_df_h(&probe.stdout);
        for fs in filesystems {
            brief.add_disk(&fs.mount, fs.use_percent, &fs.avail);
        }
    }
}

/// Check memory usage from free output
fn check_memory_usage(probes: &[ProbeResult], brief: &mut HealthBrief) {
    if let Some(probe) = find_probe(probes, "free") {
        if let Some((used_percent, available)) = parse_free_usage(&probe.stdout) {
            brief.add_memory(used_percent, &available);
        }
    }
}

/// Parse free output for used percent and available memory
fn parse_free_usage(output: &str) -> Option<(u8, String)> {
    for line in output.lines() {
        if line.starts_with("Mem:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                // free -h format: Mem: total used free shared buff/cache available
                // free format: Mem: total used free shared buff/cache available (KB)
                let total: u64 = parts[1].trim_end_matches(|c: char| !c.is_ascii_digit())
                    .parse()
                    .ok()
                    .or_else(|| parse_human_size(parts[1]))?;
                let available: u64 = parts.get(6)
                    .and_then(|s| s.trim_end_matches(|c: char| !c.is_ascii_digit()).parse().ok())
                    .or_else(|| parts.get(6).and_then(|s| parse_human_size(s)))?;

                if total > 0 {
                    let used = total.saturating_sub(available);
                    let used_percent = ((used as f64 / total as f64) * 100.0) as u8;
                    let avail_str = format_size(available);
                    return Some((used_percent, avail_str));
                }
            }
        }
    }
    None
}

/// Parse human-readable size (e.g., "16G", "512M")
fn parse_human_size(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let (num_str, suffix) = if s.ends_with(|c: char| c.is_ascii_alphabetic()) {
        let idx = s.len() - 1;
        (&s[..idx], s.chars().last())
    } else {
        (s, None)
    };

    let num: f64 = num_str.parse().ok()?;
    let multiplier: u64 = match suffix {
        Some('G') | Some('g') => 1_073_741_824,
        Some('M') | Some('m') => 1_048_576,
        Some('K') | Some('k') => 1_024,
        Some('T') | Some('t') => 1_099_511_627_776,
        _ => 1,
    };

    Some((num * multiplier as f64) as u64)
}

/// Format size in human-readable form
fn format_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1}G", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.0}M", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.0}K", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}

/// Check failed services from systemctl --failed output
fn check_failed_services(probes: &[ProbeResult], brief: &mut HealthBrief) {
    if let Some(probe) = find_probe(probes, "systemctl --failed") {
        for line in probe.stdout.lines().skip(1) {
            let trimmed = line.trim();
            // Format: UNIT LOAD ACTIVE SUB DESCRIPTION
            // or: ● service.service loaded failed failed Description
            if trimmed.contains("failed") || trimmed.contains("●") {
                if let Some(service) = extract_service_name(trimmed) {
                    brief.add_failed_service(&service);
                }
            }
        }
    }
}

/// Extract service name from systemctl --failed output line
fn extract_service_name(line: &str) -> Option<String> {
    let trimmed = line.trim_start_matches('●').trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if let Some(first) = parts.first() {
        if first.ends_with(".service") || first.ends_with(".socket") || first.ends_with(".timer") {
            return Some(first.to_string());
        }
    }
    None
}

/// Check high CPU processes from ps aux output
fn check_high_cpu(probes: &[ProbeResult], brief: &mut HealthBrief) {
    if let Some(probe) = find_probe(probes, "ps aux --sort=-%cpu") {
        let processes = parse_ps_aux(&probe.stdout, 5);
        for proc in processes {
            if let Ok(cpu) = proc.cpu_percent.parse::<f32>() {
                // Only report processes using > 80% CPU
                if cpu >= 80.0 {
                    brief.add_high_cpu(&proc.command, cpu);
                }
            }
        }
    }
}

/// Build a formatted health answer from probe results
pub fn build_health_answer(probes: &[ProbeResult]) -> String {
    let brief = build_health_brief(probes);
    brief.format_answer()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_probe(cmd: &str, stdout: &str) -> ProbeResult {
        ProbeResult {
            command: cmd.to_string(),
            exit_code: 0,
            stdout: stdout.to_string(),
            stderr: String::new(),
            timing_ms: 100,
        }
    }

    #[test]
    fn test_healthy_system() {
        let probes = vec![
            mock_probe("df -h", "Filesystem      Size  Used Avail Use% Mounted on
/dev/sda1       100G   50G   50G  50% /
/dev/sdb1        50G   20G   30G  40% /home"),
        ];

        let brief = build_health_brief(&probes);
        assert!(brief.all_healthy);
        assert_eq!(brief.items.len(), 0);
    }

    #[test]
    fn test_disk_warning() {
        let probes = vec![
            mock_probe("df -h", "Filesystem      Size  Used Avail Use% Mounted on
/dev/sda1       100G   87G   13G  87% /"),
        ];

        let brief = build_health_brief(&probes);
        assert!(!brief.all_healthy);
        assert_eq!(brief.items.len(), 1);
        assert_eq!(brief.items[0].kind, BriefItemKind::DiskSpace);
        assert_eq!(brief.items[0].severity, BriefSeverity::Warning);
    }

    #[test]
    fn test_disk_critical() {
        let probes = vec![
            mock_probe("df -h", "Filesystem      Size  Used Avail Use% Mounted on
/dev/sda1       100G   96G    4G  96% /"),
        ];

        let brief = build_health_brief(&probes);
        assert_eq!(brief.overall, BriefSeverity::Error);
    }

    #[test]
    fn test_failed_services() {
        let probes = vec![
            mock_probe("systemctl --failed", "UNIT           LOAD   ACTIVE SUB    DESCRIPTION
● nginx.service loaded failed failed A high performance web server"),
        ];

        let brief = build_health_brief(&probes);
        assert!(!brief.all_healthy);
        assert_eq!(brief.items.len(), 1);
        assert_eq!(brief.items[0].kind, BriefItemKind::Service);
    }

    #[test]
    fn test_parse_human_size() {
        assert_eq!(parse_human_size("16G"), Some(17_179_869_184));
        assert_eq!(parse_human_size("512M"), Some(536_870_912));
        assert_eq!(parse_human_size("1024K"), Some(1_048_576));
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(17_179_869_184), "16.0G");
        assert_eq!(format_size(536_870_912), "512M");
    }

    #[test]
    fn test_extract_service_name() {
        assert_eq!(
            extract_service_name("● nginx.service loaded failed failed"),
            Some("nginx.service".to_string())
        );
        assert_eq!(
            extract_service_name("docker.socket loaded active"),
            Some("docker.socket".to_string())
        );
        assert_eq!(extract_service_name("random text"), None);
    }

    // Golden test: health answer format
    #[test]
    fn golden_healthy_answer() {
        let probes = vec![
            mock_probe("df -h", "Filesystem Size Used Avail Use% Mounted on
/dev/sda1 100G 50G 50G 50% /"),
        ];
        let answer = build_health_answer(&probes);
        assert_eq!(answer, "Your system is healthy. No issues detected.");
    }
}
