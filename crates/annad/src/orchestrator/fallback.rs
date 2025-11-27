//! Fallback Answer Extraction v0.12.2
//!
//! Extracts basic facts from raw evidence when LLM fails to produce an answer.
//! This is a last-resort mechanism to avoid refusals when we have evidence.

use anna_common::ProbeEvidenceV10;

/// Extract a fallback answer from raw evidence
/// Returns None if no relevant extraction is possible
pub fn extract_fallback_answer(question: &str, evidence: &[ProbeEvidenceV10]) -> Option<String> {
    let q_lower = question.to_lowercase();

    for ev in evidence {
        let raw = ev.raw.as_deref().unwrap_or("");

        // CPU-related questions
        if (q_lower.contains("cpu")
            || q_lower.contains("processor")
            || q_lower.contains("core")
            || q_lower.contains("thread"))
            && ev.probe_id == "cpu.info"
        {
            return extract_cpu_fact(&q_lower, raw);
        }

        // Memory questions
        if (q_lower.contains("ram") || q_lower.contains("memory")) && ev.probe_id == "mem.info" {
            return extract_memory_fact(raw);
        }

        // Disk questions
        if (q_lower.contains("disk") || q_lower.contains("storage") || q_lower.contains("drive"))
            && ev.probe_id == "disk.lsblk"
        {
            return Some(format!(
                "Disk information from system:\n{}",
                truncate_raw(raw, 500)
            ));
        }

        // Network questions
        if (q_lower.contains("network") || q_lower.contains("interface") || q_lower.contains("ip"))
            && (ev.probe_id == "net.addr" || ev.probe_id == "net.links")
        {
            return Some(format!("Network information:\n{}", truncate_raw(raw, 500)));
        }

        // Kernel questions
        if q_lower.contains("kernel") && ev.probe_id == "system.kernel" {
            return Some(format!("Kernel: {}", raw.trim()));
        }

        // DNS questions
        if q_lower.contains("dns") && ev.probe_id == "dns.resolv" {
            return Some(format!("DNS configuration:\n{}", raw.trim()));
        }

        // Route questions
        if (q_lower.contains("route") || q_lower.contains("routing")) && ev.probe_id == "net.routes"
        {
            return Some(format!("Routing table:\n{}", truncate_raw(raw, 500)));
        }

        // Update questions
        if (q_lower.contains("update") || q_lower.contains("pacman"))
            && ev.probe_id == "pkg.pacman_updates"
        {
            let lines: Vec<&str> = raw.lines().collect();
            if lines.is_empty() || raw.trim().is_empty() {
                return Some("No pacman updates are currently available.".to_string());
            }
            return Some(format!("{} pacman updates available.", lines.len()));
        }

        // Anna health questions
        if (q_lower.contains("healthy") || q_lower.contains("health") || q_lower.contains("anna"))
            && ev.probe_id == "anna.self_health"
        {
            return Some(format!("Anna self-health: {}", raw.trim()));
        }

        // Journal/logs questions
        if (q_lower.contains("log") || q_lower.contains("journal"))
            && ev.probe_id == "system.journal_slice"
        {
            return Some(format!("Recent system logs:\n{}", truncate_raw(raw, 800)));
        }

        // Home directory questions
        if q_lower.contains("home") || q_lower.contains("folder") || q_lower.contains("directory") {
            // No specific probe for this, but we might have it in evidence
            if !raw.is_empty() {
                return Some(format!("Directory listing:\n{}", truncate_raw(raw, 500)));
            }
        }
    }

    None
}

/// Extract CPU facts from lscpu output
fn extract_cpu_fact(question: &str, raw: &str) -> Option<String> {
    let mut facts = Vec::new();

    for line in raw.lines() {
        let line_lower = line.to_lowercase();

        // Thread count (CPU(s) in lscpu is total threads)
        if question.contains("thread") && line_lower.starts_with("cpu(s):") {
            if let Some(val) = line.split(':').nth(1) {
                facts.push(format!("Total threads: {}", val.trim()));
            }
        }

        // Core count
        if question.contains("core") {
            if line_lower.starts_with("core(s) per socket:") {
                if let Some(val) = line.split(':').nth(1) {
                    facts.push(format!("Cores per socket: {}", val.trim()));
                }
            }
            // Also grab total CPU(s) as it represents logical cores/threads
            if line_lower.starts_with("cpu(s):") && !question.contains("thread") {
                if let Some(val) = line.split(':').nth(1) {
                    facts.push(format!("Total CPU(s)/cores: {}", val.trim()));
                }
            }
        }

        // Model name
        if question.contains("model") && line_lower.starts_with("model name:") {
            if let Some(val) = line.split(':').nth(1) {
                facts.push(format!("CPU model: {}", val.trim()));
            }
        }

        // AVX/SSE flags
        if (question.contains("avx") || question.contains("sse"))
            && line_lower.starts_with("flags:")
        {
            if let Some(flags) = line.split(':').nth(1) {
                let flags_lower = flags.to_lowercase();
                let flag_list: Vec<&str> = flags_lower.split_whitespace().collect();

                if question.contains("avx2") {
                    let has_avx2 = flag_list.contains(&"avx2");
                    facts.push(format!(
                        "AVX2 support: {}",
                        if has_avx2 { "Yes" } else { "No" }
                    ));
                } else if question.contains("avx") {
                    let has_avx = flag_list.contains(&"avx");
                    facts.push(format!(
                        "AVX support: {}",
                        if has_avx { "Yes" } else { "No" }
                    ));
                }

                if question.contains("sse2") {
                    let has_sse2 = flag_list.contains(&"sse2");
                    facts.push(format!(
                        "SSE2 support: {}",
                        if has_sse2 { "Yes" } else { "No" }
                    ));
                } else if question.contains("sse") {
                    let has_sse = flag_list.contains(&"sse");
                    facts.push(format!(
                        "SSE support: {}",
                        if has_sse { "Yes" } else { "No" }
                    ));
                }
            }
        }
    }

    if facts.is_empty() {
        // Return general CPU info
        Some(format!("CPU information:\n{}", truncate_raw(raw, 500)))
    } else {
        Some(facts.join("\n"))
    }
}

/// Extract memory facts from /proc/meminfo
fn extract_memory_fact(raw: &str) -> Option<String> {
    for line in raw.lines() {
        if line.starts_with("MemTotal:") {
            if let Some(val) = line.split(':').nth(1) {
                let kb_str = val.trim().replace(" kB", "").replace(" KB", "");
                if let Ok(kb) = kb_str.trim().parse::<u64>() {
                    let gb = kb / 1024 / 1024;
                    return Some(format!("Total RAM: {} GB ({} kB)", gb, kb));
                }
            }
        }
    }
    Some(format!("Memory information:\n{}", truncate_raw(raw, 300)))
}

/// Truncate raw output to a max length
fn truncate_raw(raw: &str, max_len: usize) -> String {
    if raw.len() <= max_len {
        raw.to_string()
    } else {
        format!("{}...", &raw[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_common::EvidenceStatus;

    fn make_evidence(probe_id: &str, raw: &str) -> ProbeEvidenceV10 {
        ProbeEvidenceV10 {
            probe_id: probe_id.to_string(),
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            status: EvidenceStatus::Ok,
            command: "test".to_string(),
            raw: Some(raw.to_string()),
            parsed: None,
        }
    }

    #[test]
    fn test_extract_cpu_threads() {
        let evidence = vec![make_evidence(
            "cpu.info",
            "Architecture: x86_64\nCPU(s): 24\nThread(s) per core: 2",
        )];
        let result = extract_fallback_answer("How many threads does my CPU have?", &evidence);
        assert!(result.is_some());
        assert!(result.unwrap().contains("24"));
    }

    #[test]
    fn test_extract_avx2() {
        let evidence = vec![make_evidence(
            "cpu.info",
            "Flags: fpu vme sse sse2 avx avx2 avx512f",
        )];
        let result = extract_fallback_answer("Does my CPU support AVX2?", &evidence);
        assert!(result.is_some());
        assert!(result.unwrap().contains("Yes"));
    }

    #[test]
    fn test_extract_memory() {
        let evidence = vec![make_evidence(
            "mem.info",
            "MemTotal: 32768000 kB\nMemFree: 1000000 kB",
        )];
        let result = extract_fallback_answer("How much RAM do I have?", &evidence);
        assert!(result.is_some());
        assert!(result.unwrap().contains("31 GB"));
    }

    #[test]
    fn test_no_relevant_evidence() {
        let evidence = vec![make_evidence("disk.lsblk", "sda 500G")];
        let result = extract_fallback_answer("What is my CPU model?", &evidence);
        assert!(result.is_none());
    }
}
