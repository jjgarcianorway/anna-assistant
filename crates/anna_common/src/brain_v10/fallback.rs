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
    let mut cores_per_socket = String::new();
    let mut sockets = String::new();
    let mut threads_per_core = String::new();
    let mut total_cpus = String::new();
    let mut evidence_id = String::new();

    for evidence in &session.evidence {
        for line in evidence.content.lines() {
            if line.contains("Model name:") {
                if let Some(name) = line.split(':').nth(1) {
                    model = name.trim().to_string();
                    evidence_id = evidence.id.clone();
                }
            }
            // v10.2.1: Parse all relevant lscpu fields for accurate core/thread reporting
            if line.starts_with("CPU(s):") && !line.contains("NUMA") && !line.contains("list") {
                if let Some(c) = line.split(':').nth(1) {
                    total_cpus = c.trim().to_string();
                }
            }
            if line.contains("Core(s) per socket:") {
                if let Some(c) = line.split(':').nth(1) {
                    cores_per_socket = c.trim().to_string();
                }
            }
            if line.contains("Socket(s):") {
                if let Some(c) = line.split(':').nth(1) {
                    sockets = c.trim().to_string();
                }
            }
            if line.contains("Thread(s) per core:") {
                if let Some(c) = line.split(':').nth(1) {
                    threads_per_core = c.trim().to_string();
                }
            }
        }
    }

    if !model.is_empty() {
        let mut answer = format!("🖥️  CPU: {} [{}]", model, evidence_id);

        // v10.2.1: Be honest about core/thread counts - only report what we can prove
        if query.contains("core") || query.contains("thread") {
            answer.push_str("\n\n");

            // Calculate physical cores if we have all the data
            let can_calc_physical = !cores_per_socket.is_empty() && !sockets.is_empty();
            let physical_cores = if can_calc_physical {
                cores_per_socket.parse::<u32>().ok()
                    .and_then(|c| sockets.parse::<u32>().ok().map(|s| c * s))
            } else {
                None
            };

            if !total_cpus.is_empty() {
                answer.push_str(&format!("📊  Logical CPUs: {} [{}]\n", total_cpus, evidence_id));
            }

            if let Some(cores) = physical_cores {
                answer.push_str(&format!("📊  Physical cores: {} ({} socket(s) × {} cores/socket) [{}]\n",
                    cores, sockets, cores_per_socket, evidence_id));
            } else if !cores_per_socket.is_empty() {
                answer.push_str(&format!("📊  Cores per socket: {} [{}]\n", cores_per_socket, evidence_id));
                if sockets.is_empty() {
                    answer.push_str("⚠️  Socket count not in evidence - cannot calculate total physical cores\n");
                }
            } else {
                answer.push_str(&format!(
                    "⚠️  I see {} logical CPUs [{}] but cannot reliably determine physical core count.\n\
                     Core(s) per socket and Socket(s) fields are missing from evidence.",
                    if total_cpus.is_empty() { "unknown" } else { &total_cpus },
                    evidence_id
                ));
            }

            if !threads_per_core.is_empty() {
                let smt = threads_per_core.parse::<u32>().unwrap_or(1) > 1;
                answer.push_str(&format!("📊  Threads per core: {} (SMT/HT: {}) [{}]",
                    threads_per_core,
                    if smt { "enabled" } else { "disabled" },
                    evidence_id
                ));
            }
        }

        return Some(BrainResult::Answer {
            text: answer,
            reliability: if cores_per_socket.is_empty() { 0.7 } else { 0.95 },
            label: if cores_per_socket.is_empty() { ReliabilityLabel::Medium } else { ReliabilityLabel::High },
        });
    }
    None
}

fn fallback_cpu_features_query(session: &BrainSession) -> Option<BrainResult> {
    // v10.2.1: Only answer based on actual CPU flags in evidence
    // NEVER answer from generic knowledge about CPU models
    for evidence in &session.evidence {
        let content = &evidence.content;

        // Look for actual flags - either from grep output or lscpu Flags line
        let flags_line = content.lines()
            .find(|l| l.starts_with("Flags:") || l.starts_with("flags"));

        let flags_content = if let Some(line) = flags_line {
            line.to_string()
        } else {
            // Check if this is grep output of individual flags
            let individual_flags: Vec<&str> = content.lines()
                .filter(|l| {
                    let trimmed = l.trim();
                    trimmed.starts_with("sse") || trimmed.starts_with("avx")
                })
                .collect();

            if individual_flags.is_empty() {
                continue;
            }
            individual_flags.join(" ")
        };

        // Search for specific flags in the evidence
        let has_sse2 = flags_content.contains("sse2");
        let has_avx = flags_content.contains("avx ") || flags_content.contains("avx\n") || flags_content.ends_with("avx");
        let has_avx2 = flags_content.contains("avx2");
        let has_avx512 = flags_content.contains("avx512");

        // Extract all SSE/AVX features found
        let mut features: Vec<&str> = Vec::new();
        for word in flags_content.split_whitespace() {
            if word.starts_with("sse") || word.starts_with("avx") {
                if !features.contains(&word) {
                    features.push(word);
                }
            }
        }

        let mut answer = String::new();
        answer.push_str(&format!("🔬  CPU Feature Support [{}]\n\n", evidence.id));

        if has_sse2 {
            answer.push_str("✅  SSE2: Yes (confirmed in CPU flags)\n");
        } else {
            answer.push_str("❓  SSE2: Not found in evidence\n");
        }

        if has_avx2 {
            answer.push_str("✅  AVX2: Yes (confirmed in CPU flags)\n");
        } else if has_avx {
            answer.push_str("⚠️  AVX2: No, but AVX (v1) is present\n");
        } else {
            answer.push_str("❓  AVX2: Not found in evidence\n");
        }

        if has_avx512 {
            answer.push_str("✅  AVX-512: Yes (confirmed in CPU flags)\n");
        }

        if !features.is_empty() {
            features.sort();
            answer.push_str(&format!("\n📋  All SSE/AVX features detected:\n    {}", features.join(", ")));
        }

        return Some(BrainResult::Answer {
            text: answer,
            reliability: if has_sse2 || has_avx2 { 0.95 } else { 0.6 },
            label: if has_sse2 || has_avx2 { ReliabilityLabel::High } else { ReliabilityLabel::Medium },
        });
    }

    // v10.2.1: If no flags evidence found, be honest about it
    Some(BrainResult::Answer {
        text: "⚠️  Cannot confirm SSE2/AVX2 support - CPU flags not in current evidence.\n\n\
               I need a probe that captures /proc/cpuinfo flags or lscpu output.\n\
               Run: grep -oE '(sse[^ ]*|avx[^ ]*)' /proc/cpuinfo | sort -u".to_string(),
        reliability: 0.2,
        label: ReliabilityLabel::VeryLow,
    })
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
        if content.contains("XDG") || content.contains("DESKTOP") || content.contains("SESSION") {
            let mut de = String::new();
            let mut session_type = String::new();
            let mut wm_processes: Vec<&str> = Vec::new();

            for line in content.lines() {
                // v10.3.0: Check XDG variables
                if line.contains("XDG_CURRENT_DESKTOP=") {
                    if let Some(val) = line.split('=').nth(1) {
                        if !val.is_empty() && val != "\"\"" {
                            de = val.trim_matches('"').to_string();
                        }
                    }
                }
                if line.contains("XDG_SESSION_TYPE=") {
                    if let Some(val) = line.split('=').nth(1) {
                        if !val.is_empty() && val != "\"\"" {
                            session_type = val.trim_matches('"').to_string();
                        }
                    }
                }
                // v10.3.0: Check for WM/DE processes
                let line_lower = line.to_lowercase();
                if line_lower.contains("hyprland") { wm_processes.push("Hyprland"); }
                if line_lower.contains("sway") { wm_processes.push("Sway"); }
                if line_lower.contains("gnome-shell") { wm_processes.push("GNOME"); }
                if line_lower.contains("kwin") { wm_processes.push("KDE/KWin"); }
                if line_lower.contains("xfce") { wm_processes.push("XFCE"); }
                if line_lower.contains("i3") && !line_lower.contains("i3status") { wm_processes.push("i3"); }
            }

            // v10.3.0: Handle SSH/TMUX sessions explicitly
            let is_tty = session_type == "tty" || session_type.is_empty();
            let no_de = de.is_empty() && wm_processes.is_empty();

            if is_tty && no_de {
                return Some(BrainResult::Answer {
                    text: format!(
                        "No graphical session detected [{}].\n\n\
                         Session type: {} (likely SSH or TTY)\n\
                         XDG_CURRENT_DESKTOP: not set\n\n\
                         Confidence: HIGH",
                        evidence.id,
                        if session_type.is_empty() { "tty" } else { &session_type }
                    ),
                    reliability: 0.95,
                    label: ReliabilityLabel::High,
                });
            }

            // Build answer from what we found
            let mut answer = String::new();
            if !de.is_empty() {
                answer.push_str(&format!("Desktop Environment: {} [{}]\n", de, evidence.id));
            }
            if !session_type.is_empty() {
                answer.push_str(&format!("Session Type: {} [{}]\n", session_type, evidence.id));
            }
            if !wm_processes.is_empty() {
                wm_processes.dedup();
                answer.push_str(&format!("WM/DE Processes: {} [{}]\n", wm_processes.join(", "), evidence.id));
            }

            if answer.is_empty() {
                answer = format!("Cannot determine DE/WM from evidence [{}].\n\nConfidence: LOW", evidence.id);
                return Some(BrainResult::Answer {
                    text: answer,
                    reliability: 0.4,
                    label: ReliabilityLabel::Low,
                });
            }

            answer.push_str("\nConfidence: HIGH");
            return Some(BrainResult::Answer {
                text: answer,
                reliability: 0.95,
                label: ReliabilityLabel::High,
            });
        }
    }
    None
}

fn fallback_games_query(session: &BrainSession) -> Option<BrainResult> {
    // v10.3.0: Check for actual Steam/game package evidence
    for evidence in &session.evidence {
        let content = &evidence.content;
        let desc_lower = evidence.description.to_lowercase();

        // Only process pacman output for games
        if !desc_lower.contains("steam") && !desc_lower.contains("game") && !desc_lower.contains("lutris") {
            continue;
        }

        // v10.3.0: Empty pacman output = not installed
        if content.trim().is_empty() {
            return Some(BrainResult::Answer {
                text: format!(
                    "No gaming packages found in pacman query [{}].\n\n\
                     Confidence: HIGH",
                    evidence.id
                ),
                reliability: 0.95,
                label: ReliabilityLabel::High,
            });
        }

        // Look for actual package names in local/ format
        let game_packages: Vec<&str> = content.lines()
            .filter(|l| {
                let trimmed = l.trim();
                !trimmed.is_empty() && (trimmed.contains("local/") || trimmed.contains("steam") || trimmed.contains("lutris"))
            })
            .take(10)
            .collect();

        if !game_packages.is_empty() {
            return Some(BrainResult::Answer {
                text: format!(
                    "Installed gaming packages [{}]:\n{}\n\nConfidence: HIGH",
                    evidence.id,
                    game_packages.join("\n")
                ),
                reliability: 0.95,
                label: ReliabilityLabel::High,
            });
        }
    }

    // v10.3.0: Check for filesystem evidence (Steam directories)
    for evidence in &session.evidence {
        let content = &evidence.content;
        if content.contains("steamapps") || content.contains(".steam") {
            let has_games = content.contains("common/") || content.contains("appmanifest");
            if has_games {
                return Some(BrainResult::Answer {
                    text: format!(
                        "Steam installation found [{}]:\n{}\n\nConfidence: HIGH",
                        evidence.id,
                        content.lines().take(10).collect::<Vec<_>>().join("\n")
                    ),
                    reliability: 0.95,
                    label: ReliabilityLabel::High,
                });
            }
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
