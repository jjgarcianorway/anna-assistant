//! Formatting functions for deterministic answers.

use super::domain::QueryDomain;

/// Format deterministic answer from probe output.
pub fn format_deterministic_answer(domain: QueryDomain, probe_output: &str) -> String {
    match domain {
        QueryDomain::Ram => format_ram_answer(probe_output),
        QueryDomain::Swap => format_swap_answer(probe_output),
        QueryDomain::Uptime => format_uptime_answer(probe_output),
        QueryDomain::Kernel => format_kernel_answer(probe_output),
        QueryDomain::Desktop => format_desktop_answer(probe_output),
        _ => probe_output.trim().to_string(),
    }
}

fn format_ram_answer(output: &str) -> String {
    // Parse free -h output
    for line in output.lines() {
        if line.starts_with("Mem:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                return format!("{} available", parts[6]);
            }
        }
    }
    output.trim().to_string()
}

fn format_swap_answer(output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.is_empty() || trimmed.contains("no swap") {
        "No, swap is not enabled.".to_string()
    } else {
        let first_line = trimmed.lines().next().unwrap_or("");
        if first_line.contains("NAME") {
            // swapon --show header, get next line
            if let Some(data) = trimmed.lines().nth(1) {
                format!("Yes, swap is enabled: {}", data)
            } else {
                "Yes, swap is enabled.".to_string()
            }
        } else {
            format!("Yes, swap is enabled: {}", first_line)
        }
    }
}

fn format_uptime_answer(output: &str) -> String {
    // uptime -p output is already human-readable
    output.trim().to_string()
}

fn format_kernel_answer(output: &str) -> String {
    output.trim().to_string()
}

fn format_desktop_answer(output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        "No desktop environment detected (might be running in TTY).".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_swap_answer() {
        assert_eq!(format_swap_answer(""), "No, swap is not enabled.");
        assert!(format_swap_answer("NAME TYPE SIZE\n/dev/sda2 partition 8G")
            .contains("Yes, swap is enabled"));
    }
}
