//! Deterministic Answer Engine - v6.41.0
//!
//! CRITICAL: This module intercepts queries that can be answered deterministically
//! WITHOUT LLM hallucination. If a query matches a pattern here, we return
//! the REAL answer from system_info, not LLM-generated text.
//!
//! Philosophy:
//! - If we can read it from /proc, sysfs, or commands -> USE THAT
//! - If we can detect it deterministically -> USE THAT
//! - NEVER let LLM generate: CPU info, RAM, disk space, VRAM, processes, etc.
//!
//! Query patterns we intercept:
//! - RAM queries: "how much ram", "ram do I have", "memory", "how much free"
//! - CPU queries: "what CPU", "CPU info", "does my CPU have SSE/AVX"
//! - GPU queries: "what GPU", "VRAM", "graphics card"
//! - Disk queries: "disk space", "how much free space", "biggest folders"
//! - DE/WM queries: "what DE", "what WM", "desktop environment"
//! - System report: "system report", "hardware report", "system info"

use anna_common::{de_wm_detector, system_info, system_report};
use anyhow::Result;

/// Deterministic answer - returned when we can answer without LLM
pub struct DeterministicAnswer {
    pub answer: String,
    pub source: String,
}

/// Try to answer query deterministically. Returns Some if we can answer without LLM.
pub fn try_deterministic_answer(query: &str) -> Option<DeterministicAnswer> {
    let query_lower = query.to_lowercase();

    // RAM queries
    if query_lower.contains("how much ram")
        || query_lower.contains("ram do i have")
        || (query_lower.contains("memory") && !query_lower.contains("swap"))
    {
        return answer_ram_query(&query_lower);
    }

    // Free memory queries
    if (query_lower.contains("how much") || query_lower.contains("how many"))
        && query_lower.contains("free")
        && !query_lower.contains("disk")
        && !query_lower.contains("space")
    {
        return answer_free_memory_query();
    }

    // CPU queries
    if query_lower.contains("what cpu")
        || query_lower.contains("cpu do i have")
        || query_lower.contains("processor")
        || query_lower.contains("what's my cpu")
    {
        return answer_cpu_query();
    }

    // CPU features (SSE, AVX, etc.)
    if (query_lower.contains("cpu") || query_lower.contains("processor"))
        && (query_lower.contains("sse")
            || query_lower.contains("avx")
            || query_lower.contains("features")
            || query_lower.contains("flags"))
    {
        return answer_cpu_features_query(&query_lower);
    }

    // GPU queries
    if query_lower.contains("what gpu")
        || query_lower.contains("graphics card")
        || query_lower.contains("video card")
        || query_lower.contains("what's my gpu")
    {
        return answer_gpu_query();
    }

    // VRAM queries
    if query_lower.contains("vram") || query_lower.contains("video memory") {
        return answer_vram_query();
    }

    // Disk space queries
    if (query_lower.contains("disk space")
        || query_lower.contains("free space")
        || query_lower.contains("how much space")
        || query_lower.contains("disk usage"))
        && !query_lower.contains("folder")
        && !query_lower.contains("directory")
    {
        return answer_disk_space_query();
    }

    // Biggest directories queries
    if (query_lower.contains("biggest")
        || query_lower.contains("largest")
        || query_lower.contains("top"))
        && (query_lower.contains("folder") || query_lower.contains("director"))
    {
        return answer_biggest_dirs_query(&query_lower);
    }

    // DE/WM queries
    if (query_lower.contains("desktop environment")
        || query_lower.contains("window manager")
        || (query_lower.contains("what") && (query_lower.contains(" de ") || query_lower.contains(" wm ")))
        || query_lower.contains("what de ")
        || query_lower.contains("what wm ")
        || query_lower.contains("de or wm")
        || query_lower.contains("wm or de"))
        && !query_lower.contains("installed")
    {
        if query_lower.contains("recommend") || query_lower.contains("suggest") || query_lower.contains("should") {
            return answer_de_wm_recommendation_query();
        } else {
            return answer_de_wm_query();
        }
    }

    // System report queries
    if query_lower.contains("system report")
        || query_lower.contains("hardware report")
        || (query_lower.contains("system") && query_lower.contains("info"))
    {
        return answer_system_report_query();
    }

    // LLM upgrade queries
    if (query_lower.contains("upgrade") || query_lower.contains("update"))
        && (query_lower.contains("llm")
            || query_lower.contains("brain")
            || query_lower.contains("model"))
    {
        return answer_llm_upgrade_query();
    }

    // Orphaned packages queries
    if query_lower.contains("orphan") && query_lower.contains("package") {
        return answer_orphaned_packages_query();
    }

    // Package presence queries - v6.41.0: "Do I have X installed?"
    if (query_lower.contains("do i have") || query_lower.contains("any")  || query_lower.contains("is there"))
        && query_lower.contains("install")
    {
        if query_lower.contains("game") {
            return answer_package_presence_query("games");
        } else if query_lower.contains("file manager") || query_lower.contains("file browser") {
            return answer_package_presence_query("file_managers");
        } else if query_lower.contains("browser") && !query_lower.contains("file") {
            return answer_package_presence_query("browsers");
        } else if query_lower.contains(" de ") || query_lower.contains(" wm ")
            || query_lower.contains("desktop environment")
            || query_lower.contains("window manager")
        {
            return answer_package_presence_query("de_wm");
        } else if query_lower.contains("terminal") || query_lower.contains("term emulator") {
            return answer_package_presence_query("terminals");
        }
    }

    None
}

// ============================================================================
// Answer Generators
// ============================================================================

fn answer_ram_query(query: &str) -> Option<DeterministicAnswer> {
    let mem = system_info::get_ram_info().ok()?;

    let total_gb = mem.total_mb as f64 / 1024.0;
    let answer = if query.contains("total") || query.contains("installed") {
        format!("You have {:.1} GB of RAM installed.", total_gb)
    } else {
        format!(
            "RAM: {:.1} GB total ({} MB used, {} MB available)",
            total_gb, mem.used_mb, mem.available_mb
        )
    };

    Some(DeterministicAnswer {
        answer,
        source: "Source: /proc/meminfo".to_string(),
    })
}

fn answer_free_memory_query() -> Option<DeterministicAnswer> {
    let mem = system_info::get_ram_info().ok()?;

    let answer = format!(
        "Free RAM: {} MB ({} MB total, {} MB used, {} MB available)",
        mem.available_mb, mem.total_mb, mem.used_mb, mem.available_mb
    );

    Some(DeterministicAnswer {
        answer,
        source: "Source: /proc/meminfo".to_string(),
    })
}

fn answer_cpu_query() -> Option<DeterministicAnswer> {
    let cpu = system_info::get_cpu_info().ok()?;

    let freq_str = if let Some(freq) = cpu.frequency_mhz {
        format!(" @ {:.2} GHz", freq / 1000.0)
    } else {
        String::new()
    };

    let answer = format!(
        "CPU: {} ({} cores, {} threads{}) - {} architecture",
        cpu.model, cpu.cores, cpu.threads, freq_str, cpu.architecture
    );

    Some(DeterministicAnswer {
        answer,
        source: "Source: lscpu".to_string(),
    })
}

fn answer_cpu_features_query(query: &str) -> Option<DeterministicAnswer> {
    use std::fs;
    use std::process::Command;

    // Try lscpu first
    let output = Command::new("lscpu").output().ok()?;
    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if line.starts_with("Flags:") {
                let flags = line.split(':').nth(1)?.trim();

                // Filter for requested features
                let mut matching_features = Vec::new();

                if query.contains("sse") {
                    let sse_features: Vec<&str> = flags
                        .split_whitespace()
                        .filter(|f| f.starts_with("sse"))
                        .collect();
                    if !sse_features.is_empty() {
                        matching_features.push(format!("SSE: {}", sse_features.join(", ")));
                    }
                }

                if query.contains("avx") {
                    let avx_features: Vec<&str> = flags
                        .split_whitespace()
                        .filter(|f| f.starts_with("avx"))
                        .collect();
                    if !avx_features.is_empty() {
                        matching_features.push(format!("AVX: {}", avx_features.join(", ")));
                    }
                }

                if matching_features.is_empty() {
                    // Show all flags if no specific feature requested
                    return Some(DeterministicAnswer {
                        answer: format!("CPU flags: {}", flags),
                        source: "Source: lscpu".to_string(),
                    });
                } else {
                    return Some(DeterministicAnswer {
                        answer: format!("CPU features: {}", matching_features.join("; ")),
                        source: "Source: lscpu".to_string(),
                    });
                }
            }
        }
    }

    // Fallback to /proc/cpuinfo
    let cpuinfo = fs::read_to_string("/proc/cpuinfo").ok()?;
    for line in cpuinfo.lines() {
        if line.starts_with("flags") {
            let flags = line.split(':').nth(1)?.trim();
            return Some(DeterministicAnswer {
                answer: format!("CPU flags: {}", flags),
                source: "Source: /proc/cpuinfo".to_string(),
            });
        }
    }

    None
}

fn answer_gpu_query() -> Option<DeterministicAnswer> {
    let gpus = system_info::get_gpu_info().ok()?;

    if gpus.is_empty() {
        return Some(DeterministicAnswer {
            answer: "No dedicated GPU detected.".to_string(),
            source: "Source: lspci".to_string(),
        });
    }

    let gpu_list: Vec<String> = gpus
        .iter()
        .map(|g| {
            let mut desc = format!("{} {}", g.vendor, g.model);
            if let Some(driver) = &g.driver {
                desc.push_str(&format!(" (driver: {})", driver));
            }
            desc
        })
        .collect();

    let answer = if gpus.len() == 1 {
        format!("GPU: {}", gpu_list[0])
    } else {
        format!("GPUs:\n{}", gpu_list.iter().map(|g| format!("  - {}", g)).collect::<Vec<_>>().join("\n"))
    };

    Some(DeterministicAnswer {
        answer,
        source: "Source: lspci".to_string(),
    })
}

fn answer_vram_query() -> Option<DeterministicAnswer> {
    let gpus = system_info::get_gpu_info().ok()?;

    let mut vram_info = Vec::new();

    for gpu in gpus.iter() {
        if let Some(vram_total) = gpu.vram_total_mb {
            let vram_gb = vram_total as f64 / 1024.0;
            let usage = if let Some(vram_used) = gpu.vram_used_mb {
                let vram_used_gb = vram_used as f64 / 1024.0;
                let vram_free_gb = (vram_total - vram_used) as f64 / 1024.0;
                format!(
                    "{:.1} GB total ({:.1} GB used, {:.1} GB free)",
                    vram_gb, vram_used_gb, vram_free_gb
                )
            } else {
                format!("{:.1} GB total", vram_gb)
            };

            vram_info.push(format!("{} {}: {}", gpu.vendor, gpu.model, usage));
        } else if gpu.vendor == "Intel" {
            vram_info.push(format!(
                "{} {}: Shared with system RAM (integrated graphics)",
                gpu.vendor, gpu.model
            ));
        } else {
            vram_info.push(format!(
                "{} {}: VRAM not reported by hardware",
                gpu.vendor, gpu.model
            ));
        }
    }

    let answer = if vram_info.is_empty() {
        "No GPU detected or VRAM information unavailable.".to_string()
    } else {
        format!("VRAM:\n{}", vram_info.iter().map(|v| format!("  {}", v)).collect::<Vec<_>>().join("\n"))
    };

    Some(DeterministicAnswer {
        answer,
        source: "Source: nvidia-smi / sysfs".to_string(),
    })
}

fn answer_disk_space_query() -> Option<DeterministicAnswer> {
    let disks = system_info::get_disk_usage().ok()?;

    let mut disk_lines = Vec::new();
    for disk in disks.iter() {
        disk_lines.push(format!(
            "{} on {}: {:.1} GB total, {:.1} GB used, {:.1} GB free ({}% used)",
            disk.filesystem,
            disk.mount_point,
            disk.total_gb,
            disk.used_gb,
            disk.available_gb,
            disk.use_percent
        ));
    }

    let answer = format!("Disk space:\n{}", disk_lines.iter().map(|l| format!("  {}", l)).collect::<Vec<_>>().join("\n"));

    Some(DeterministicAnswer {
        answer,
        source: "Source: df -h".to_string(),
    })
}

fn answer_biggest_dirs_query(query: &str) -> Option<DeterministicAnswer> {
    use std::env;

    // Extract count from query (e.g., "top 10 folders")
    let count = if query.contains("10") {
        10
    } else if query.contains("20") {
        20
    } else if query.contains("5") {
        5
    } else {
        10 // default
    };

    // Determine path - v6.41.0: Default to $HOME, not current directory or /
    let path = if query.contains("in /") {
        query
            .split("in /")
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .map(|p| format!("/{}", p))
            .unwrap_or_else(|| env::var("HOME").unwrap_or_else(|_| ".".to_string()))
    } else if query.contains("root") || query.contains("system") {
        "/".to_string()
    } else {
        // Default to $HOME for safety and usefulness
        env::var("HOME").unwrap_or_else(|_| ".".to_string())
    };

    // v6.41.0: Suppress permission errors with 2>/dev/null
    use std::process::Command;

    let output = Command::new("du")
        .args(&["-xh", "--max-depth=1", &path])
        .stderr(std::process::Stdio::null()) // Suppress permission denied errors
        .output();

    match output {
        Ok(output) if output.status.success() || !output.stdout.is_empty() => {
            let text = String::from_utf8_lossy(&output.stdout);
            let mut dirs = Vec::new();

            for line in text.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let size_str = parts[0];
                    let dir_path = parts[1..].join(" ");

                    // Skip the total line (same as input path)
                    if dir_path != path {
                        let size_gb = parse_du_size_to_gb(size_str);
                        dirs.push((size_gb, dir_path));
                    }
                }
            }

            // Sort by size descending
            dirs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

            // Take top N
            dirs.truncate(count);

            if dirs.is_empty() {
                return Some(DeterministicAnswer {
                    answer: format!("No subdirectories found in {}\n\nNote: Some directories may have been skipped due to permissions.", path),
                    source: "Source: du -xh --max-depth=1".to_string(),
                });
            }

            let mut answer = format!("Top {} largest directories in {}:\n\n", count, path);
            answer.push_str("  Size    Path\n");
            answer.push_str("  ------  ----\n");

            for (idx, (size, dir)) in dirs.iter().enumerate() {
                answer.push_str(&format!("  {:5.1} GB  {}\n", size, dir));
                if idx >= count - 1 {
                    break;
                }
            }

            if path == "/" || query.contains("root") || query.contains("system") {
                answer.push_str("\nNote: Some system directories could not be read (permission denied).");
            }

            Some(DeterministicAnswer {
                answer,
                source: format!("Source: du -xh --max-depth=1 {} 2>/dev/null", path),
            })
        }
        Ok(_) => Some(DeterministicAnswer {
            answer: format!("No output from du for path: {}\n\nThis could mean:\n  - The directory does not exist\n  - You do not have permission to read it\n  - The directory is empty", path),
            source: "Source: du (no output)".to_string(),
        }),
        Err(e) => Some(DeterministicAnswer {
            answer: format!("Failed to run du command: {}", e),
            source: "Source: du (command failed)".to_string(),
        }),
    }
}

fn parse_du_size_to_gb(size_str: &str) -> f64 {
    let size_str = size_str.trim();
    if size_str.is_empty() || size_str == "-" {
        return 0.0;
    }

    let (num_str, unit) = if size_str.ends_with('T') {
        (size_str.trim_end_matches('T'), 1024.0)
    } else if size_str.ends_with('G') {
        (size_str.trim_end_matches('G'), 1.0)
    } else if size_str.ends_with('M') {
        (size_str.trim_end_matches('M'), 1.0 / 1024.0)
    } else if size_str.ends_with('K') {
        (size_str.trim_end_matches('K'), 1.0 / (1024.0 * 1024.0))
    } else {
        // Assume bytes if no unit
        (size_str, 1.0 / (1024.0 * 1024.0 * 1024.0))
    };

    num_str.parse::<f64>().unwrap_or(0.0) * unit
}

fn answer_de_wm_query() -> Option<DeterministicAnswer> {
    let detection = de_wm_detector::detect_de_wm();

    // v6.41.0: Handle "Unknown" / "Unable to detect" properly
    if detection.name == "Unable to detect" || detection.name == "Unknown" {
        let answer = "I could not reliably detect your desktop environment or window manager.\n\n\
I checked:\n\
  - XDG_CURRENT_DESKTOP and DESKTOP_SESSION environment variables\n\
  - Common WM processes (sway, i3, hyprland, etc.)\n\
  - Installed DE/WM packages\n\
  - Configuration directories\n\
  - X11 properties\n\n\
You may be:\n\
  - Running in a TTY (no graphical environment)\n\
  - Using a very minimal or custom window manager\n\
  - In a remote SSH session\n\n\
To check manually:\n\
  echo $XDG_CURRENT_DESKTOP\n\
  ps aux | grep -E 'sway|i3|hyprland|gnome|kde'".to_string();

        return Some(DeterministicAnswer {
            answer,
            source: "Source: DE/WM Detector v6.40.0".to_string(),
        });
    }

    let de_type_str = match detection.de_type {
        de_wm_detector::DeType::DesktopEnvironment => "Desktop Environment",
        de_wm_detector::DeType::WindowManager => "Window Manager",
        de_wm_detector::DeType::Compositor => "Compositor",
    };

    let confidence_str = match detection.confidence {
        de_wm_detector::Confidence::High => "High",
        de_wm_detector::Confidence::Medium => "Medium",
        de_wm_detector::Confidence::Low => "Low",
    };

    let session_type = std::env::var("XDG_SESSION_TYPE")
        .unwrap_or_else(|_| "tty".to_string());

    let answer = format!(
        "{}: {}\nSession type: {}\nConfidence: {} (detected via {})",
        de_type_str, detection.name, session_type, confidence_str, detection.detection_method
    );

    Some(DeterministicAnswer {
        answer,
        source: "Source: DE/WM Detector v6.40.0".to_string(),
    })
}

fn answer_de_wm_recommendation_query() -> Option<DeterministicAnswer> {
    // Get current system specs for recommendation
    let cpu = system_info::get_cpu_info().ok()?;
    let mem = system_info::get_ram_info().ok()?;
    let gpus = system_info::get_gpu_info().unwrap_or_default();

    let has_dgpu = gpus.iter().any(|g| g.vendor == "NVIDIA" || g.vendor == "AMD");
    let ram_gb = mem.total_mb / 1024;
    let is_powerful = cpu.cores >= 4 && ram_gb >= 8;

    let mut answer = String::from("Based on your system specs:\n");
    answer.push_str(&format!("  CPU: {} ({} cores)\n", cpu.model, cpu.cores));
    answer.push_str(&format!("  RAM: {} GB\n", ram_gb));
    if has_dgpu {
        answer.push_str(&format!("  GPU: Discrete graphics ({})\n",
            gpus.iter().find(|g| g.vendor != "Intel").map(|g| g.vendor.as_str()).unwrap_or("Unknown")));
    }
    answer.push_str("\nRecommendations:\n\n");

    if is_powerful && has_dgpu {
        answer.push_str("Your system is high-performance. Good options:\n\n");
        answer.push_str("Wayland Compositors (modern, smooth):\n");
        answer.push_str("  - Hyprland: Tiling, animations, highly customizable\n");
        answer.push_str("  - Sway: Tiling, i3-compatible, stable and minimal\n");
        answer.push_str("  - KDE Plasma (Wayland): Full DE, feature-rich\n\n");
        answer.push_str("X11 Options:\n");
        answer.push_str("  - i3: Classic tiling, rock-solid\n");
        answer.push_str("  - bspwm: Binary space partitioning, minimal\n");
    } else if is_powerful {
        answer.push_str("Your system is capable. Good options:\n\n");
        answer.push_str("Lightweight but feature-rich:\n");
        answer.push_str("  - Sway: Wayland tiling compositor\n");
        answer.push_str("  - i3: X11 tiling window manager\n");
        answer.push_str("  - XFCE: Lightweight desktop environment\n");
        answer.push_str("  - KDE Plasma: Full-featured, surprisingly efficient\n");
    } else {
        answer.push_str("Your system has limited resources. Best options:\n\n");
        answer.push_str("Very lightweight:\n");
        answer.push_str("  - i3: Minimal tiling WM (X11)\n");
        answer.push_str("  - Openbox: Floating WM, very light\n");
        answer.push_str("  - LXDE/LXQt: Minimal desktop environments\n");
        answer.push_str("  - dwm: Ultra-minimal (requires patching)\n");
    }

    answer.push_str("\nNote: This is a rules-based recommendation. Try a few and see what fits your workflow.");

    Some(DeterministicAnswer {
        answer,
        source: "Source: System telemetry + DE/WM database".to_string(),
    })
}

fn answer_system_report_query() -> Option<DeterministicAnswer> {
    let report = system_report::generate_system_report().ok()?;

    Some(DeterministicAnswer {
        answer: report,
        source: "".to_string(), // Source is already in the report footer
    })
}

fn answer_llm_upgrade_query() -> Option<DeterministicAnswer> {
    use std::process::Command;

    // Check if ollama is available
    let ollama_check = Command::new("ollama").arg("list").output();

    match ollama_check {
        Ok(output) if output.status.success() => {
            let models = String::from_utf8_lossy(&output.stdout);
            let model_lines: Vec<&str> = models.lines().skip(1).collect(); // Skip header

            if model_lines.is_empty() {
                Some(DeterministicAnswer {
                    answer: "No Ollama models are currently installed.\n\nTo install a model:\n  ollama pull llama3.2:3b\n  ollama pull qwen2.5:3b\n\nNote: Anna does not automatically change models. Configure your preferred model in ~/.config/anna/config.toml".to_string(),
                    source: "Source: ollama list".to_string(),
                })
            } else {
                let mut answer = String::from("Installed Ollama models:\n");
                for line in model_lines.iter().take(10) {
                    answer.push_str(&format!("  {}\n", line));
                }
                answer.push_str("\nTo upgrade or add models:\n");
                answer.push_str("  ollama pull llama3.2:3b\n");
                answer.push_str("  ollama pull qwen2.5:3b\n\n");
                answer.push_str("Note: Anna does not automatically swap models. Update your model selection in ~/.config/anna/config.toml");

                Some(DeterministicAnswer {
                    answer,
                    source: "Source: ollama list".to_string(),
                })
            }
        }
        Ok(_) => Some(DeterministicAnswer {
            answer: "Ollama is installed but 'ollama list' failed.\n\nTry running:\n  systemctl status ollama\n  ollama --version\n\nIf Ollama is not running, start it:\n  systemctl start ollama".to_string(),
            source: "Source: ollama command check".to_string(),
        }),
        Err(_) => Some(DeterministicAnswer {
            answer: "Ollama is not installed or not in PATH.\n\nTo install Ollama:\n  curl -fsSL https://ollama.com/install.sh | sh\n\nOr on Arch Linux:\n  yay -S ollama\n\nAfter installation, pull a model:\n  ollama pull llama3.2:3b".to_string(),
            source: "Source: ollama availability check".to_string(),
        }),
    }
}

fn answer_package_presence_query(category: &str) -> Option<DeterministicAnswer> {
    use std::process::Command;

    // Define package lists for each category
    let (category_name, packages) = match category {
        "games" => ("games", vec![
            "0ad", "steam", "lutris", "wine", "playonlinux", "minecraft", "supertuxkart",
            "openttd", "hedgewars", "wesnoth", "minetest", "xonotic",
        ]),
        "file_managers" => ("file managers", vec![
            "thunar", "dolphin", "pcmanfm", "pcmanfm-qt", "nemo", "nautilus",
            "caja", "spacefm", "ranger", "vifm", "lf", "mc",
        ]),
        "browsers" => ("web browsers", vec![
            "firefox", "chromium", "google-chrome", "brave-bin", "vivaldi",
            "opera", "qutebrowser", "falkon", "epiphany", "midori",
        ]),
        "de_wm" => ("desktop environments / window managers", vec![
            "plasma", "gnome", "xfce4", "mate", "cinnamon", "lxde", "lxqt",
            "sway", "i3", "bspwm", "awesome", "dwm", "qtile", "hyprland",
            "openbox", "fluxbox", "xmonad",
        ]),
        "terminals" => ("terminal emulators", vec![
            "alacritty", "kitty", "konsole", "gnome-terminal", "xfce4-terminal",
            "terminator", "tilix", "st", "urxvt", "xterm", "foot", "wezterm",
        ]),
        _ => return None,
    };

    // Query installed packages
    let output = Command::new("pacman")
        .args(&["-Qq"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let installed = String::from_utf8_lossy(&output.stdout);
            let installed_set: std::collections::HashSet<&str> =
                installed.lines().collect();

            let mut found: Vec<&str> = packages
                .iter()
                .filter(|pkg| installed_set.contains(*pkg))
                .copied()
                .collect();

            if found.is_empty() {
                Some(DeterministicAnswer {
                    answer: format!("You do not have any known {} installed.", category_name),
                    source: "Source: pacman -Qq".to_string(),
                })
            } else {
                found.sort();
                let count = found.len();
                let list = if count <= 10 {
                    found.join(", ")
                } else {
                    format!("{}, ... and {} more", found[..10].join(", "), count - 10)
                };

                Some(DeterministicAnswer {
                    answer: format!(
                        "You have {} {} installed:\n  {}",
                        count,
                        if count == 1 { category_name.trim_end_matches('s') } else { category_name },
                        list
                    ),
                    source: "Source: pacman -Qq".to_string(),
                })
            }
        }
        Ok(_) => Some(DeterministicAnswer {
            answer: "Unable to query installed packages.\n\npacman -Qq failed. This could mean pacman is not configured correctly.".to_string(),
            source: "Source: pacman -Qq (failed)".to_string(),
        }),
        Err(_) => Some(DeterministicAnswer {
            answer: "Unable to query installed packages.\n\npacman is not available. This command only works on Arch Linux.".to_string(),
            source: "Source: pacman availability check".to_string(),
        }),
    }
}

fn answer_orphaned_packages_query() -> Option<DeterministicAnswer> {
    use std::process::Command;

    // Check for orphaned packages using pacman -Qtdq
    let output = Command::new("pacman")
        .args(&["-Qtdq"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let orphans = String::from_utf8_lossy(&output.stdout);
            let orphan_list: Vec<&str> = orphans.lines().collect();

            if orphan_list.is_empty() {
                Some(DeterministicAnswer {
                    answer: "No orphaned packages found.\n\nYour system is clean - all installed packages are either explicitly installed or required as dependencies.".to_string(),
                    source: "Source: pacman -Qtdq".to_string(),
                })
            } else {
                let count = orphan_list.len();
                let mut answer = format!("Found {} orphaned package(s):\n\n", count);

                for (idx, pkg) in orphan_list.iter().enumerate().take(20) {
                    answer.push_str(&format!("  {}. {}\n", idx + 1, pkg));
                }

                if count > 20 {
                    answer.push_str(&format!("\n  ... and {} more\n", count - 20));
                }

                answer.push_str("\nTo remove them:\n");
                answer.push_str("  sudo pacman -Rns $(pacman -Qtdq)\n\n");
                answer.push_str("Note: Review the list before removing. Some packages may be intentionally installed.");

                Some(DeterministicAnswer {
                    answer,
                    source: "Source: pacman -Qtdq".to_string(),
                })
            }
        }
        Ok(_) => Some(DeterministicAnswer {
            answer: "Unable to check for orphaned packages.\n\nThe command 'pacman -Qtdq' failed. This could mean:\n  - pacman is not installed (unlikely on Arch)\n  - No orphaned packages exist (exit code 1)\n\nTry running manually:\n  pacman -Qtdq".to_string(),
            source: "Source: pacman -Qtdq (failed)".to_string(),
        }),
        Err(_) => Some(DeterministicAnswer {
            answer: "Unable to check for orphaned packages.\n\npacman is not available in PATH. This command only works on Arch Linux and Arch-based distributions.".to_string(),
            source: "Source: pacman availability check".to_string(),
        }),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ram_query() {
        let result = try_deterministic_answer("how much ram do I have?");
        assert!(result.is_some());
        let answer = result.unwrap();
        assert!(answer.answer.contains("GB"));
        assert!(answer.source.contains("/proc/meminfo"));
    }

    #[test]
    fn test_cpu_query() {
        let result = try_deterministic_answer("what CPU do I have?");
        assert!(result.is_some());
        let answer = result.unwrap();
        assert!(answer.answer.contains("CPU:"));
        assert!(answer.source.contains("lscpu"));
    }

    #[test]
    fn test_de_wm_query() {
        let result = try_deterministic_answer("what DE am I running?");
        assert!(result.is_some());
    }

    #[test]
    fn test_disk_space_query() {
        let result = try_deterministic_answer("how much disk space do I have?");
        assert!(result.is_some());
    }

    #[test]
    fn test_no_match() {
        let result = try_deterministic_answer("how do I install vim?");
        assert!(result.is_none());
    }
}
