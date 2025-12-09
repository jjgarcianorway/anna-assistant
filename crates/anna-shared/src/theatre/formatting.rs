//! Theatre formatting helpers (v0.0.226).

/// Describes what probes are checking in human-friendly terms
pub fn describe_check(probe_ids: &[String]) -> String {
    let mut checks = Vec::new();

    for id in probe_ids {
        let desc = match id.as_str() {
            "df" => "disk space",
            "free" => "memory",
            "lscpu" => "CPU info",
            "sensors" => "temperatures",
            "systemctl" | "systemctl_failed" => "services",
            "journalctl_errors" | "journalctl_warnings" => "system logs",
            "ip_addr" | "ip" => "network interfaces",
            "ss" | "listening_ports" => "network ports",
            "lspci_audio" | "pactl_cards" => "audio hardware",
            "lsblk" => "block devices",
            "uname" => "kernel info",
            "top_cpu" => "CPU usage",
            "top_memory" => "memory usage",
            "command_v" => "installed tools",
            _ if id.contains("editor") => "editors",
            _ => continue,
        };
        if !checks.contains(&desc) {
            checks.push(desc);
        }
    }

    if checks.is_empty() {
        "system data".to_string()
    } else {
        checks.join(", ")
    }
}

/// Format case ID in service desk style
pub fn format_case_id(request_id: &str) -> String {
    let short = &request_id[..8.min(request_id.len())];
    format!("CN-{}", short.to_uppercase())
}
