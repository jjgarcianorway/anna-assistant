//! Anna Brain v10.0.2 - Fallback Answer Extraction
//!
//! When the LLM fails to follow the protocol, these functions extract
//! answers directly from the gathered evidence using pattern matching.

use crate::brain_v10::contracts::{BrainSession, ReliabilityLabel};
use crate::brain_v10::orchestrator::BrainResult;

/// Try to extract an obvious answer from evidence when LLM fails
pub fn try_fallback_answer(query: &str, session: &BrainSession) -> Option<BrainResult> {
    let q = query.to_lowercase();

    // Package queries - "is X installed"
    if q.contains("installed") {
        if let Some(result) = fallback_package_query(&q, session) {
            return Some(result);
        }
    }

    // RAM queries - "how much RAM", "memory", "free"
    if q.contains("ram") || q.contains("memory") || (q.contains("free") && !q.contains("disk")) {
        if let Some(result) = fallback_ram_query(session) {
            return Some(result);
        }
    }

    // CPU queries - "what CPU", "cores", "threads", "processor"
    if q.contains("cpu") || q.contains("processor") || q.contains("core") || q.contains("thread") {
        if let Some(result) = fallback_cpu_query(&q, session) {
            return Some(result);
        }
    }

    // SSE/AVX queries
    if q.contains("sse") || q.contains("avx") {
        if let Some(result) = fallback_cpu_features_query(session) {
            return Some(result);
        }
    }

    // Disk space queries
    if q.contains("disk") || q.contains("space") || q.contains("storage") || q.contains("filesystem") || q.contains("root") {
        if let Some(result) = fallback_disk_query(session) {
            return Some(result);
        }
    }

    // GPU queries
    if q.contains("gpu") || q.contains("graphics") || q.contains("video") || q.contains("nvidia") || q.contains("amd") {
        if let Some(result) = fallback_gpu_query(session) {
            return Some(result);
        }
    }

    // Desktop/WM queries
    if q.contains("desktop") || q.contains("window manager") || q.contains(" wm") || q.contains(" de ") {
        if let Some(result) = fallback_desktop_query(session) {
            return Some(result);
        }
    }

    // Games queries
    if q.contains("game") {
        if let Some(result) = fallback_games_query(session) {
            return Some(result);
        }
    }

    // File manager queries
    if q.contains("file manager") || (q.contains("file") && q.contains("manager")) {
        if let Some(result) = fallback_file_manager_query(session) {
            return Some(result);
        }
    }

    // Orphan packages
    if q.contains("orphan") {
        if let Some(result) = fallback_orphan_query(session) {
            return Some(result);
        }
    }

    // Network queries
    if q.contains("network") || q.contains("wifi") || q.contains("ethernet") || q.contains("interface") || q.contains("connected") {
        if let Some(result) = fallback_network_query(session) {
            return Some(result);
        }
    }

    // DNS queries
    if q.contains("dns") || q.contains("resolver") || q.contains("nameserver") {
        if let Some(result) = fallback_dns_query(session) {
            return Some(result);
        }
    }

    // Updates queries
    if q.contains("update") && !q.contains("upgrade") {
        if let Some(result) = fallback_updates_query(session) {
            return Some(result);
        }
    }

    // Big folders queries
    if (q.contains("folder") || q.contains("director")) && (q.contains("big") || q.contains("large") || q.contains("size") || q.contains("top")) {
        if let Some(result) = fallback_big_folders_query(session) {
            return Some(result);
        }
    }

    // Big files queries
    if q.contains("file") && (q.contains("big") || q.contains("large") || q.contains("size") || q.contains("top")) && !q.contains("manager") {
        if let Some(result) = fallback_big_files_query(session) {
            return Some(result);
        }
    }

    // System summary / overview
    if q.contains("summary") || q.contains("overview") || q.contains("know about") || q.contains("how are you") {
        if let Some(result) = fallback_system_summary(session) {
            return Some(result);
        }
    }

    // Issues / problems / fire
    if q.contains("issue") || q.contains("problem") || q.contains("worried") || q.contains("fire") {
        if let Some(result) = fallback_system_issues(session) {
            return Some(result);
        }
    }

    // Kernel
    if q.contains("kernel") {
        if let Some(result) = fallback_kernel_query(session) {
            return Some(result);
        }
    }

    None
}

fn fallback_package_query(query_lower: &str, session: &BrainSession) -> Option<BrainResult> {
    let package_name = query_lower
        .split_whitespace()
        .find(|w| !["is", "installed", "installed?", "?", "do", "i", "have", "any", "a", "an", "graphical"].contains(w))?;

    for evidence in &session.evidence {
        if evidence.source == "run_shell" {
            let is_relevant = evidence.description.to_lowercase().contains(package_name)
                || evidence.content.to_lowercase().contains(package_name);
            if !is_relevant { continue; }

            if evidence.content.contains("local/") && evidence.is_success() {
                let pkg_line = evidence.content.lines().next()?;
                return Some(BrainResult::Answer {
                    text: format!("Yes, {} is installed [{}].\nVersion: {}", package_name, evidence.id, pkg_line.trim()),
                    reliability: 0.9,
                    label: ReliabilityLabel::High,
                });
            }
            if evidence.content.trim().is_empty() || !evidence.is_success() {
                return Some(BrainResult::Answer {
                    text: format!("No, {} is not installed [{}].", package_name, evidence.id),
                    reliability: 0.9,
                    label: ReliabilityLabel::High,
                });
            }
        }
    }
    None
}

fn fallback_ram_query(session: &BrainSession) -> Option<BrainResult> {
    for evidence in &session.evidence {
        // Handle free -h output (with Gi/Mi suffixes)
        if evidence.content.contains("Mem:") {
            for line in evidence.content.lines() {
                if line.starts_with("Mem:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        let total = parts[1];
                        let used = parts[2];
                        let free = parts[3];
                        return Some(BrainResult::Answer {
                            text: format!("RAM: {} total, {} used, {} free [{}].", total, used, free, evidence.id),
                            reliability: 0.95,
                            label: ReliabilityLabel::High,
                        });
                    }
                }
            }
        }
    }
    None
}

fn fallback_cpu_query(query: &str, session: &BrainSession) -> Option<BrainResult> {
    let mut model = String::new();
    let mut cores = String::new();
    let mut threads = String::new();
    let mut evidence_id = String::new();

    for evidence in &session.evidence {
        for line in evidence.content.lines() {
            if line.contains("Model name:") {
                if let Some(name) = line.split(':').nth(1) {
                    model = name.trim().to_string();
                    evidence_id = evidence.id.clone();
                }
            }
            if line.starts_with("CPU(s):") {
                if let Some(c) = line.split(':').nth(1) {
                    threads = c.trim().to_string();
                }
            }
            if line.contains("Core(s) per socket:") {
                if let Some(c) = line.split(':').nth(1) {
                    cores = c.trim().to_string();
                }
            }
        }
    }

    if !model.is_empty() {
        let mut answer = format!("CPU: {} [{}]", model, evidence_id);
        if !cores.is_empty() || !threads.is_empty() {
            if query.contains("core") || query.contains("thread") {
                answer = format!("{}\nCores: {}, Threads: {}", answer, cores, threads);
            }
        }
        return Some(BrainResult::Answer {
            text: answer,
            reliability: 0.95,
            label: ReliabilityLabel::High,
        });
    }
    None
}

fn fallback_cpu_features_query(session: &BrainSession) -> Option<BrainResult> {
    for evidence in &session.evidence {
        let content = &evidence.content;
        if content.contains("sse") || content.contains("avx") {
            // Look for flags line or grep output
            let features: Vec<&str> = content.lines()
                .filter(|l| l.contains("sse") || l.contains("avx"))
                .collect();

            let has_sse2 = content.contains("sse2");
            let has_avx2 = content.contains("avx2");

            return Some(BrainResult::Answer {
                text: format!(
                    "SSE2: {}, AVX2: {} [{}]\nFeatures found: {}",
                    if has_sse2 { "Yes" } else { "No" },
                    if has_avx2 { "Yes" } else { "No" },
                    evidence.id,
                    features.join(", ")
                ),
                reliability: 0.95,
                label: ReliabilityLabel::High,
            });
        }
    }
    None
}

fn fallback_disk_query(session: &BrainSession) -> Option<BrainResult> {
    for evidence in &session.evidence {
        if evidence.content.contains("Filesystem") && evidence.content.contains("Use%") {
            let lines: Vec<&str> = evidence.content.lines()
                .filter(|l| !l.contains("tmpfs") && !l.contains("efivarfs") && !l.contains("devtmpfs"))
                .collect();
            if lines.len() > 1 {
                return Some(BrainResult::Answer {
                    text: format!("Disk usage [{}]:\n```\n{}\n```", evidence.id, lines.join("\n")),
                    reliability: 0.95,
                    label: ReliabilityLabel::High,
                });
            }
        }
    }
    None
}

fn fallback_gpu_query(session: &BrainSession) -> Option<BrainResult> {
    for evidence in &session.evidence {
        let content_lower = evidence.content.to_lowercase();
        if content_lower.contains("vga") || content_lower.contains("3d") || content_lower.contains("display") {
            let gpu_lines: Vec<&str> = evidence.content.lines()
                .filter(|l| {
                    let ll = l.to_lowercase();
                    ll.contains("vga") || ll.contains("3d") || ll.contains("display")
                })
                .collect();
            if !gpu_lines.is_empty() {
                return Some(BrainResult::Answer {
                    text: format!("GPU [{}]:\n{}", evidence.id, gpu_lines.join("\n")),
                    reliability: 0.95,
                    label: ReliabilityLabel::High,
                });
            }
        }
    }
    None
}

fn fallback_desktop_query(session: &BrainSession) -> Option<BrainResult> {
    for evidence in &session.evidence {
        let content = &evidence.content;
        if content.contains("XDG") || content.contains("DESKTOP") {
            let mut de = "Unknown";
            let mut session_type = "Unknown";

            for line in content.lines() {
                if line.contains("XDG_CURRENT_DESKTOP=") {
                    if let Some(val) = line.split('=').nth(1) {
                        if !val.is_empty() { de = val; }
                    }
                }
                if line.contains("XDG_SESSION_TYPE=") {
                    if let Some(val) = line.split('=').nth(1) {
                        if !val.is_empty() { session_type = val; }
                    }
                }
            }

            return Some(BrainResult::Answer {
                text: format!("Desktop: {}, Session type: {} [{}]", de, session_type, evidence.id),
                reliability: 0.85,
                label: ReliabilityLabel::Medium,
            });
        }
    }
    None
}

fn fallback_games_query(session: &BrainSession) -> Option<BrainResult> {
    for evidence in &session.evidence {
        let content = &evidence.content;
        // Look for package names (either local/ format or just package names)
        let game_packages: Vec<&str> = content.lines()
            .filter(|l| !l.trim().is_empty())
            .take(10)
            .collect();

        if !game_packages.is_empty() {
            return Some(BrainResult::Answer {
                text: format!("Gaming packages [{}]:\n{}", evidence.id, game_packages.join("\n")),
                reliability: 0.9,
                label: ReliabilityLabel::High,
            });
        }
        if content.trim().is_empty() {
            return Some(BrainResult::Answer {
                text: format!("No gaming packages found [{}].", evidence.id),
                reliability: 0.9,
                label: ReliabilityLabel::High,
            });
        }
    }
    None
}

fn fallback_file_manager_query(session: &BrainSession) -> Option<BrainResult> {
    for evidence in &session.evidence {
        let content = &evidence.content;
        let fm_packages: Vec<&str> = content.lines()
            .filter(|l| !l.trim().is_empty())
            .collect();

        if !fm_packages.is_empty() {
            return Some(BrainResult::Answer {
                text: format!("File managers installed [{}]:\n{}", evidence.id, fm_packages.join("\n")),
                reliability: 0.9,
                label: ReliabilityLabel::High,
            });
        }
        if content.trim().is_empty() {
            return Some(BrainResult::Answer {
                text: format!("No graphical file managers found [{}].", evidence.id),
                reliability: 0.9,
                label: ReliabilityLabel::High,
            });
        }
    }
    None
}

fn fallback_orphan_query(session: &BrainSession) -> Option<BrainResult> {
    for evidence in &session.evidence {
        if evidence.description.contains("orphan") || evidence.description.contains("Qdt") {
            if evidence.content.trim().is_empty() || evidence.content.contains("No orphans") {
                return Some(BrainResult::Answer {
                    text: format!("No orphan packages found [{}]. Your system is clean!", evidence.id),
                    reliability: 0.95,
                    label: ReliabilityLabel::High,
                });
            } else {
                let count = evidence.content.lines().filter(|l| !l.trim().is_empty()).count();
                return Some(BrainResult::Answer {
                    text: format!("{} orphan packages [{}]:\n```\n{}\n```", count, evidence.id, evidence.content),
                    reliability: 0.95,
                    label: ReliabilityLabel::High,
                });
            }
        }
    }
    None
}

fn fallback_network_query(session: &BrainSession) -> Option<BrainResult> {
    for evidence in &session.evidence {
        let content = &evidence.content;
        if content.contains("DEVICE") || content.contains("state UP") || content.contains("wlp") || content.contains("enp") || content.contains("eth") {
            // Try to identify the active interface
            let mut interface = "Unknown";
            let mut conn_type = "Unknown";

            for line in content.lines() {
                if line.contains("wifi") && line.contains("connected") {
                    interface = line.split_whitespace().next().unwrap_or("wifi");
                    conn_type = "WiFi";
                    break;
                }
                if line.contains("ethernet") && line.contains("connected") {
                    interface = line.split_whitespace().next().unwrap_or("ethernet");
                    conn_type = "Ethernet";
                    break;
                }
                if line.contains("state UP") {
                    if let Some(iface) = line.split(':').next() {
                        let parts: Vec<&str> = iface.split_whitespace().collect();
                        if parts.len() >= 2 {
                            interface = parts[1];
                            conn_type = if interface.starts_with("wl") { "WiFi" } else { "Ethernet" };
                        }
                    }
                }
            }

            return Some(BrainResult::Answer {
                text: format!("Network: {} via {} [{}]\n```\n{}\n```", conn_type, interface, evidence.id, content),
                reliability: 0.9,
                label: ReliabilityLabel::High,
            });
        }
    }
    None
}

fn fallback_dns_query(session: &BrainSession) -> Option<BrainResult> {
    for evidence in &session.evidence {
        if evidence.content.contains("nameserver") || evidence.content.contains("resolv") {
            let nameservers: Vec<&str> = evidence.content.lines()
                .filter(|l| l.contains("nameserver"))
                .collect();

            let issues: Vec<&str> = vec![];
            // Could check for obvious issues like 0.0.0.0

            return Some(BrainResult::Answer {
                text: format!("DNS configuration [{}]:\n{}\n{}",
                    evidence.id,
                    nameservers.join("\n"),
                    if issues.is_empty() { "No obvious issues." } else { &issues.join(", ") }
                ),
                reliability: 0.9,
                label: ReliabilityLabel::High,
            });
        }
    }
    None
}

fn fallback_updates_query(session: &BrainSession) -> Option<BrainResult> {
    for evidence in &session.evidence {
        if evidence.description.contains("update") || evidence.description.contains("checkupdates") {
            if evidence.content.trim().is_empty() || evidence.content.contains("No updates") || evidence.content.contains("not available") {
                return Some(BrainResult::Answer {
                    text: format!("System is up to date. No pending updates [{}].", evidence.id),
                    reliability: 0.95,
                    label: ReliabilityLabel::High,
                });
            } else {
                let count = evidence.content.lines().filter(|l| !l.trim().is_empty()).count();
                return Some(BrainResult::Answer {
                    text: format!("{} updates available [{}]:\n```\n{}\n```", count, evidence.id, evidence.content),
                    reliability: 0.95,
                    label: ReliabilityLabel::High,
                });
            }
        }
    }
    None
}

fn fallback_big_folders_query(session: &BrainSession) -> Option<BrainResult> {
    for evidence in &session.evidence {
        if evidence.description.contains("du") || evidence.content.contains("G\t") || evidence.content.contains("M\t") {
            let folders: Vec<&str> = evidence.content.lines()
                .filter(|l| !l.trim().is_empty())
                .take(10)
                .collect();
            if !folders.is_empty() {
                return Some(BrainResult::Answer {
                    text: format!("Largest folders [{}]:\n```\n{}\n```", evidence.id, folders.join("\n")),
                    reliability: 0.95,
                    label: ReliabilityLabel::High,
                });
            }
        }
    }
    None
}

fn fallback_big_files_query(session: &BrainSession) -> Option<BrainResult> {
    for evidence in &session.evidence {
        if evidence.description.contains("find") || evidence.content.contains("M ") {
            let files: Vec<&str> = evidence.content.lines()
                .filter(|l| !l.trim().is_empty())
                .take(10)
                .collect();
            if !files.is_empty() {
                return Some(BrainResult::Answer {
                    text: format!("Largest files [{}]:\n```\n{}\n```", evidence.id, files.join("\n")),
                    reliability: 0.95,
                    label: ReliabilityLabel::High,
                });
            }
        }
    }
    None
}

fn fallback_system_summary(session: &BrainSession) -> Option<BrainResult> {
    let mut kernel = String::new();
    let mut ram = String::new();
    let mut disk = String::new();
    let mut network = String::new();
    let mut evidence_ids: Vec<String> = vec![];

    for evidence in &session.evidence {
        let content = &evidence.content;

        if content.contains("Kernel:") {
            for line in content.lines() {
                if line.contains("Kernel:") {
                    kernel = line.to_string();
                    evidence_ids.push(evidence.id.clone());
                }
            }
        }
        if content.contains("Mem:") {
            for line in content.lines() {
                if line.starts_with("Mem:") {
                    ram = line.to_string();
                    evidence_ids.push(evidence.id.clone());
                }
            }
        }
        if content.contains("default via") {
            network = content.lines().next().unwrap_or("").to_string();
            evidence_ids.push(evidence.id.clone());
        }
    }

    if !kernel.is_empty() || !ram.is_empty() {
        let refs = evidence_ids.iter().map(|s| format!("[{}]", s)).collect::<Vec<_>>().join(", ");
        return Some(BrainResult::Answer {
            text: format!("System summary {}:\n{}\n{}\n{}", refs, kernel, ram, network),
            reliability: 0.9,
            label: ReliabilityLabel::High,
        });
    }
    None
}

fn fallback_system_issues(session: &BrainSession) -> Option<BrainResult> {
    for evidence in &session.evidence {
        let content = &evidence.content;

        // Check for failed services
        if content.contains("UNIT") && content.contains("LOAD") && content.contains("ACTIVE") {
            let failed: Vec<&str> = content.lines()
                .filter(|l| l.contains("failed"))
                .collect();

            if failed.is_empty() {
                return Some(BrainResult::Answer {
                    text: format!("No failed services. System looks healthy [{}].", evidence.id),
                    reliability: 0.9,
                    label: ReliabilityLabel::High,
                });
            } else {
                return Some(BrainResult::Answer {
                    text: format!("Issues found [{}]:\n{}", evidence.id, failed.join("\n")),
                    reliability: 0.9,
                    label: ReliabilityLabel::High,
                });
            }
        }

        // Empty output means no issues
        if content.trim().is_empty() {
            return Some(BrainResult::Answer {
                text: format!("No critical issues found [{}]. System appears healthy.", evidence.id),
                reliability: 0.85,
                label: ReliabilityLabel::Medium,
            });
        }
    }
    None
}

fn fallback_kernel_query(session: &BrainSession) -> Option<BrainResult> {
    for evidence in &session.evidence {
        if evidence.content.contains("Linux") || evidence.description.contains("uname") {
            return Some(BrainResult::Answer {
                text: format!("Kernel [{}]: {}", evidence.id, evidence.content.lines().next().unwrap_or("")),
                reliability: 0.95,
                label: ReliabilityLabel::High,
            });
        }
    }
    None
}
