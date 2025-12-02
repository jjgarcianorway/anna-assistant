//! Anna Boot Snapshot v7.23.0 - Boot-Aware Incident Summary
//!
//! Provides boot-anchored views with incident summaries.
//! All data comes from journalctl and pattern engine.
//!
//! Rules:
//! - Incidents are deduplicated log patterns at warning and above
//! - Uses pattern IDs from the log pattern engine
//! - Current boot only for status display

use chrono::{DateTime, Local, TimeZone};
use std::process::Command;

/// Boot snapshot data
#[derive(Debug, Clone)]
pub struct BootSnapshot {
    pub boot_started: DateTime<Local>,
    pub uptime: String,
    pub anna_started: Option<DateTime<Local>>,
    pub anna_uptime: Option<String>,
    pub incidents: Vec<IncidentPattern>,
}

/// A deduplicated incident pattern
#[derive(Debug, Clone)]
pub struct IncidentPattern {
    pub pattern_id: String,
    pub component: String,
    pub message: String,
    pub count: u32,
}

impl BootSnapshot {
    /// Get current boot snapshot
    pub fn current() -> Self {
        let boot_started = get_boot_start_time();
        let uptime = get_system_uptime();
        let anna_started = get_anna_start_time();
        let anna_uptime = anna_started.map(|_| get_anna_uptime());
        let incidents = get_current_boot_incidents();

        BootSnapshot {
            boot_started,
            uptime,
            anna_started,
            anna_uptime,
            incidents,
        }
    }
}

/// Get system boot start time
fn get_boot_start_time() -> DateTime<Local> {
    // Try journalctl --list-boots first
    let output = Command::new("journalctl")
        .args(["--list-boots", "-o", "short-iso"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // Parse first line like: " 0 abc123 2025-12-02 07:13:21 CET—2025-12-02 10:34:21 CET"
            if let Some(line) = stdout.lines().find(|l| l.trim().starts_with("0 ") || l.trim().starts_with(" 0 ")) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    // Try to parse date+time
                    let date_str = format!("{} {}", parts[2], parts[3]);
                    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%d %H:%M:%S") {
                        return Local.from_local_datetime(&dt).unwrap();
                    }
                }
            }
        }
    }

    // Fallback: use uptime to calculate boot time
    if let Some(uptime_secs) = get_uptime_seconds() {
        let now = Local::now();
        return now - chrono::Duration::seconds(uptime_secs as i64);
    }

    Local::now()
}

/// Get system uptime as human-readable string
fn get_system_uptime() -> String {
    if let Some(secs) = get_uptime_seconds() {
        format_duration(secs)
    } else {
        "unknown".to_string()
    }
}

/// Get uptime in seconds from /proc/uptime
fn get_uptime_seconds() -> Option<u64> {
    let content = std::fs::read_to_string("/proc/uptime").ok()?;
    let secs_str = content.split_whitespace().next()?;
    secs_str.parse::<f64>().ok().map(|s| s as u64)
}

/// Format duration as human-readable
fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;

    if hours > 24 {
        let days = hours / 24;
        let rem_hours = hours % 24;
        format!("{}d {}h {}m", days, rem_hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

/// Get Anna daemon start time
fn get_anna_start_time() -> Option<DateTime<Local>> {
    let output = Command::new("systemctl")
        .args(["show", "annad.service", "-p", "ExecMainStartTimestamp", "--no-pager"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Parse "ExecMainStartTimestamp=Mon 2025-12-02 07:13:45 CET"
    let line = stdout.lines().next()?;
    let ts_str = line.strip_prefix("ExecMainStartTimestamp=")?;

    if ts_str.is_empty() || ts_str == "n/a" {
        return None;
    }

    // Parse the systemd timestamp format
    // "Mon 2025-12-02 07:13:45 CET"
    let parts: Vec<&str> = ts_str.split_whitespace().collect();
    if parts.len() >= 3 {
        let date_time_str = format!("{} {}", parts[1], parts[2]);
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&date_time_str, "%Y-%m-%d %H:%M:%S") {
            return Some(Local.from_local_datetime(&dt).unwrap());
        }
    }

    None
}

/// Get Anna uptime
fn get_anna_uptime() -> String {
    let output = Command::new("systemctl")
        .args(["show", "annad.service", "-p", "ExecMainStartTimestamp", "--no-pager"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if let Some(line) = stdout.lines().next() {
                if let Some(ts_str) = line.strip_prefix("ExecMainStartTimestamp=") {
                    if !ts_str.is_empty() && ts_str != "n/a" {
                        // Parse and calculate duration
                        let parts: Vec<&str> = ts_str.split_whitespace().collect();
                        if parts.len() >= 3 {
                            let date_time_str = format!("{} {}", parts[1], parts[2]);
                            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&date_time_str, "%Y-%m-%d %H:%M:%S") {
                                let start = Local.from_local_datetime(&dt).unwrap();
                                let now = Local::now();
                                let duration = now.signed_duration_since(start);
                                return format_duration(duration.num_seconds() as u64);
                            }
                        }
                    }
                }
            }
        }
    }

    "unknown".to_string()
}

/// Get incidents from current boot (warning and above)
fn get_current_boot_incidents() -> Vec<IncidentPattern> {
    let mut incidents = Vec::new();
    let mut pattern_counts: std::collections::HashMap<String, (String, String, u32)> = std::collections::HashMap::new();

    // Get all warning+ messages from current boot
    let output = Command::new("journalctl")
        .args(["-b", "-p", "warning..alert", "-o", "short", "--no-pager", "-q"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);

            for line in stdout.lines() {
                if line.trim().is_empty() {
                    continue;
                }

                // Parse line to extract component and message
                // Format: "Dec 02 10:30:15 hostname component[pid]: message"
                let parts: Vec<&str> = line.splitn(5, ' ').collect();
                if parts.len() >= 5 {
                    let component_msg = parts[4..].join(" ");

                    // Extract component name (before [pid]: or just before :)
                    let (component, message) = if let Some(bracket_pos) = component_msg.find('[') {
                        let colon_pos = component_msg.find(':').unwrap_or(component_msg.len());
                        let comp = &component_msg[..bracket_pos];
                        let msg = if colon_pos + 1 < component_msg.len() {
                            component_msg[colon_pos + 1..].trim()
                        } else {
                            ""
                        };
                        (comp, msg)
                    } else if let Some(colon_pos) = component_msg.find(':') {
                        let comp = &component_msg[..colon_pos];
                        let msg = if colon_pos + 1 < component_msg.len() {
                            component_msg[colon_pos + 1..].trim()
                        } else {
                            ""
                        };
                        (comp, msg)
                    } else {
                        continue;
                    };

                    // Normalize message to create pattern
                    let normalized = normalize_message(message);
                    let pattern_key = format!("{}:{}", component, normalized);

                    // Generate pattern ID from component
                    let pattern_id = generate_pattern_id(component, &normalized);

                    pattern_counts
                        .entry(pattern_key)
                        .and_modify(|e| e.2 += 1)
                        .or_insert((pattern_id, format!("{}: {}", component, truncate_message(&normalized, 50)), 1));
                }
            }
        }
    }

    // Convert to incident patterns, sorted by count
    let mut sorted: Vec<_> = pattern_counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.2.cmp(&a.1.2));

    for (_, (pattern_id, message, count)) in sorted.into_iter().take(10) {
        let component = message.split(':').next().unwrap_or("unknown").to_string();
        incidents.push(IncidentPattern {
            pattern_id,
            component,
            message,
            count,
        });
    }

    incidents
}

/// Normalize a log message to create a pattern
fn normalize_message(msg: &str) -> String {
    let mut result = msg.to_string();

    // Replace IPs
    let ip_re = regex::Regex::new(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap();
    result = ip_re.replace_all(&result, "%IP%").to_string();

    // Replace PIDs
    let pid_re = regex::Regex::new(r"\[\d+\]").unwrap();
    result = pid_re.replace_all(&result, "[%PID%]").to_string();

    // Replace hex addresses
    let hex_re = regex::Regex::new(r"0x[0-9a-fA-F]+").unwrap();
    result = hex_re.replace_all(&result, "%ADDR%").to_string();

    // Replace paths
    let path_re = regex::Regex::new(r"/[a-zA-Z0-9/_.-]+").unwrap();
    result = path_re.replace_all(&result, "%PATH%").to_string();

    // Replace timestamps
    let ts_re = regex::Regex::new(r"\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}").unwrap();
    result = ts_re.replace_all(&result, "%TIME%").to_string();

    result
}

/// Generate a pattern ID based on component
fn generate_pattern_id(component: &str, message: &str) -> String {
    let prefix = match component.to_lowercase().as_str() {
        s if s.contains("network") || s.contains("nm-") || s.contains("wpa") => "NET",
        s if s.contains("nvidia") || s.contains("gpu") || s.contains("drm") => "GPU",
        s if s.contains("nvme") || s.contains("smart") || s.contains("ata") || s.contains("disk") => "STO",
        s if s.contains("audio") || s.contains("pulse") || s.contains("pipewire") => "AUD",
        s if s.contains("usb") => "USB",
        s if s.contains("power") || s.contains("battery") || s.contains("acpi") => "PWR",
        s if s.contains("bluetooth") || s.contains("bt") => "BLU",
        s if s.contains("kernel") => "KRN",
        s if s.contains("systemd") => "SYS",
        _ => "GEN",
    };

    // Create a simple hash from message using wrapping operations
    let hash: u32 = message.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32).wrapping_mul(31));
    format!("[{}{}]", prefix, format!("{:03}", hash % 1000))
}

/// Truncate message to max length
fn truncate_message(msg: &str, max_len: usize) -> String {
    if msg.len() <= max_len {
        msg.to_string()
    } else {
        format!("{}...", &msg[..max_len - 3])
    }
}

/// Format boot snapshot section for display
pub fn format_boot_snapshot_section(snapshot: &BootSnapshot) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push("[BOOT SNAPSHOT]".to_string());
    lines.push("  Current boot:".to_string());
    lines.push(format!("    started:        {}", snapshot.boot_started.format("%Y-%m-%d %H:%M:%S %Z")));
    lines.push(format!("    uptime:         {}", snapshot.uptime));

    if let Some(ref anna_start) = snapshot.anna_started {
        lines.push(format!("    Anna start:     {}", anna_start.format("%Y-%m-%d %H:%M:%S %Z")));
    }
    if let Some(ref anna_uptime) = snapshot.anna_uptime {
        lines.push(format!("    Anna uptime:    {}", anna_uptime));
    }
    lines.push(String::new());

    if snapshot.incidents.is_empty() {
        lines.push("  Incidents (current boot):".to_string());
        lines.push("    none recorded at warning or above".to_string());
    } else {
        lines.push("  Incidents (current boot, warning and above, grouped):".to_string());
        for incident in &snapshot.incidents {
            let count_str = if incident.count == 1 {
                "(seen 1 time)".to_string()
            } else {
                format!("(seen {} times)", incident.count)
            };
            lines.push(format!(
                "    {} {}  {}",
                incident.pattern_id,
                incident.message,
                count_str
            ));
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(60), "1m");
        assert_eq!(format_duration(3600), "1h 0m");
        assert_eq!(format_duration(3661), "1h 1m");
        assert_eq!(format_duration(90000), "1d 1h 0m");
    }

    #[test]
    fn test_normalize_message() {
        assert_eq!(
            normalize_message("connection to 192.168.1.1 failed"),
            "connection to %IP% failed"
        );
        assert_eq!(
            normalize_message("process[1234] exited"),
            "process[%PID%] exited"
        );
    }

    #[test]
    fn test_generate_pattern_id() {
        let id = generate_pattern_id("NetworkManager", "carrier lost");
        assert!(id.starts_with("[NET"));

        let id = generate_pattern_id("nvidia", "TDP cap reached");
        assert!(id.starts_with("[GPU"));

        let id = generate_pattern_id("nvme0n1", "reset controller");
        assert!(id.starts_with("[STO"));
    }

    #[test]
    fn test_truncate_message() {
        assert_eq!(truncate_message("short", 10), "short");
        assert_eq!(truncate_message("this is a very long message", 15), "this is a ve...");
    }
}
