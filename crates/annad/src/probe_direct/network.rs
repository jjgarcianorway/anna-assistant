//! Network and port queries

use anna_shared::rpc::ProbeResult;
use tracing::info;

use super::DirectAnswerResult;

/// Network answer
pub(crate) fn try_network_answer(query: &str, probes: &[ProbeResult]) -> Option<DirectAnswerResult> {
    if !query.contains("ip") && !query.contains("network") && !query.contains("address") {
        return None;
    }

    for probe in probes {
        let cmd = probe.command.to_lowercase();
        if cmd.contains("ip addr") || cmd.contains("ip a") {
            let mut interfaces: Vec<(String, Vec<String>)> = Vec::new();
            let mut current_iface = String::new();
            let mut current_ips: Vec<String> = Vec::new();

            for line in probe.stdout.lines() {
                if line
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
                {
                    if !current_iface.is_empty() {
                        interfaces.push((current_iface.clone(), current_ips.clone()));
                        current_ips.clear();
                    }
                    if let Some(name) = line.split(':').nth(1) {
                        current_iface = name.trim().to_string();
                    }
                } else if line.contains("inet ") && !line.contains("inet6") {
                    if let Some(addr) = line.split_whitespace().nth(1) {
                        current_ips.push(addr.to_string());
                    }
                }
            }
            if !current_iface.is_empty() {
                interfaces.push((current_iface, current_ips));
            }

            if !interfaces.is_empty() {
                let mut answer = "**Network Interfaces:**\n".to_string();
                for (iface, ips) in &interfaces {
                    if ips.is_empty() {
                        answer.push_str(&format!("- {}: no IPv4\n", iface));
                    } else {
                        answer.push_str(&format!("- {}: {}\n", iface, ips.join(", ")));
                    }
                }
                info!("v0.0.403: Direct network answer");
                return Some(DirectAnswerResult {
                    answer,
                    confidence: 90,
                });
            }
        }
    }

    None
}

/// v0.0.792: Port/listening answer
pub(crate) fn try_port_answer(query: &str, probes: &[ProbeResult]) -> Option<DirectAnswerResult> {
    if !query.contains("port") && !query.contains("listen") && !query.contains("ss") && !query.contains("netstat") {
        return None;
    }

    for probe in probes {
        let cmd = probe.command.to_lowercase();
        if cmd.contains("ss") || cmd.contains("netstat") {
            if probe.exit_code != 0 {
                continue;
            }

            let output = probe.stdout.trim();
            if output.is_empty() {
                return Some(DirectAnswerResult {
                    answer: "**No listening ports** found on this system.".to_string(),
                    confidence: 95,
                });
            }

            // Parse ss -tulpn output
            let lines: Vec<&str> = output.lines().collect();
            let port_count = lines.len().saturating_sub(1); // Subtract header

            if port_count == 0 {
                return Some(DirectAnswerResult {
                    answer: "**No listening ports** found on this system.".to_string(),
                    confidence: 95,
                });
            }

            // Format the answer with port information
            let mut answer = format!("**Listening Ports ({}):**\n", port_count);

            // Parse and format each listening port
            for line in lines.iter().skip(1).take(20) {
                // Skip header line
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    let proto = parts[0]; // tcp/udp
                    let local_addr = parts[4]; // Local Address:Port
                    let process = if parts.len() >= 7 {
                        // Extract process name from users:(("name",...))
                        parts[6..].join(" ")
                    } else {
                        String::new()
                    };

                    if process.is_empty() {
                        answer.push_str(&format!("- {} {}\n", proto, local_addr));
                    } else {
                        // Extract process name from users:(("name",pid=...))
                        let proc_name = process
                            .split("((\"")
                            .nth(1)
                            .and_then(|s| s.split('"').next())
                            .unwrap_or(&process);
                        answer.push_str(&format!("- {} {} ({})\n", proto, local_addr, proc_name));
                    }
                }
            }

            if port_count > 20 {
                answer.push_str(&format!("\n...and {} more\n", port_count - 20));
            }

            info!("v0.0.792: Direct port answer");
            return Some(DirectAnswerResult {
                answer,
                confidence: 95,
            });
        }
    }

    None
}
