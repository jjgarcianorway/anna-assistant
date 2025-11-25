//! System Report v2 - Fully Deterministic (NO LLM)
//!
//! v6.41.0: System reports must NEVER contain LLM-generated text.
//! All values come from real system data via system_info module.
//!
//! Structure:
//! - Header: "System Report"
//! - Hardware Summary
//! - Operating System
//! - CPU
//! - GPU
//! - Memory
//! - Storage
//! - Services
//! - Networking
//! - Diagnostics Summary
//!
//! NO emojis. Professional formatting only.

use crate::de_wm_detector;
use crate::system_info::*;
use anyhow::Result;
use std::process::Command;

/// Generate a complete system report (deterministic, no LLM)
pub fn generate_system_report() -> Result<String> {
    let mut report = String::new();

    // Header
    report.push_str("═══════════════════════════════════════════════════════════════\n");
    report.push_str("                        SYSTEM REPORT                          \n");
    report.push_str("═══════════════════════════════════════════════════════════════\n\n");

    // Operating System
    report.push_str(&generate_os_section()?);
    report.push_str("\n");

    // Hardware Summary
    report.push_str(&generate_hardware_summary()?);
    report.push_str("\n");

    // CPU Details
    report.push_str(&generate_cpu_section()?);
    report.push_str("\n");

    // GPU Details
    report.push_str(&generate_gpu_section()?);
    report.push_str("\n");

    // Memory Details
    report.push_str(&generate_memory_section()?);
    report.push_str("\n");

    // Storage Details
    report.push_str(&generate_storage_section()?);
    report.push_str("\n");

    // Desktop Environment
    report.push_str(&generate_desktop_section()?);
    report.push_str("\n");

    // Network Summary
    report.push_str(&generate_network_section()?);
    report.push_str("\n");

    // Services Status
    report.push_str(&generate_services_section()?);
    report.push_str("\n");

    // Diagnostics Summary
    report.push_str(&generate_diagnostics_section()?);
    report.push_str("\n");

    // Footer
    report.push_str("═══════════════════════════════════════════════════════════════\n");
    report.push_str("Report generated using deterministic system inspection\n");
    report.push_str("Source: /proc, sysfs, systemctl, lscpu, lspci, df, free\n");
    report.push_str("═══════════════════════════════════════════════════════════════\n");

    Ok(report)
}

// ============================================================================
// Section Generators
// ============================================================================

fn generate_os_section() -> Result<String> {
    let os = get_os_info()?;
    let uptime = get_uptime()?;

    Ok(format!(
        "─── Operating System ───────────────────────────────────────────\n\
         OS:           {}\n\
         Version:      {}\n\
         Kernel:       {}\n\
         Architecture: {}\n\
         Uptime:       {}",
        os.name, os.version, os.kernel, os.architecture, uptime
    ))
}

fn generate_hardware_summary() -> Result<String> {
    let cpu = get_cpu_info()?;
    let mem = get_ram_info()?;
    let gpus = get_gpu_info().unwrap_or_default();

    let gpu_summary = if gpus.is_empty() {
        "None detected".to_string()
    } else {
        gpus.iter()
            .map(|g| format!("{} {}", g.vendor, g.model))
            .collect::<Vec<_>>()
            .join(", ")
    };

    Ok(format!(
        "─── Hardware Summary ───────────────────────────────────────────\n\
         CPU:    {} ({} cores / {} threads)\n\
         Memory: {} MB total\n\
         GPU:    {}",
        cpu.model, cpu.cores, cpu.threads, mem.total_mb, gpu_summary
    ))
}

fn generate_cpu_section() -> Result<String> {
    let cpu = get_cpu_info()?;

    let freq_str = if let Some(freq) = cpu.frequency_mhz {
        format!("{:.2} MHz", freq)
    } else {
        "Unknown".to_string()
    };

    Ok(format!(
        "─── CPU Details ────────────────────────────────────────────────\n\
         Model:        {}\n\
         Vendor:       {}\n\
         Architecture: {}\n\
         Cores:        {}\n\
         Threads:      {}\n\
         Frequency:    {}",
        cpu.model, cpu.vendor, cpu.architecture, cpu.cores, cpu.threads, freq_str
    ))
}

fn generate_gpu_section() -> Result<String> {
    let gpus = get_gpu_info();

    let mut section = String::from("─── GPU Details ────────────────────────────────────────────────\n");

    match gpus {
        Ok(gpu_list) => {
            for (idx, gpu) in gpu_list.iter().enumerate() {
                if idx > 0 {
                    section.push('\n');
                }

                section.push_str(&format!("GPU {}:\n", idx + 1));
                section.push_str(&format!("  Vendor:  {}\n", gpu.vendor));
                section.push_str(&format!("  Model:   {}\n", gpu.model));

                if let Some(driver) = &gpu.driver {
                    section.push_str(&format!("  Driver:  {}\n", driver));
                }

                if let Some(vram_total) = gpu.vram_total_mb {
                    section.push_str(&format!("  VRAM:    {} MB", vram_total));
                    if let Some(vram_used) = gpu.vram_used_mb {
                        let vram_free = vram_total.saturating_sub(vram_used);
                        let usage_pct = (vram_used as f64 / vram_total as f64 * 100.0) as u8;
                        section.push_str(&format!(
                            " ({} MB used / {} MB free / {}% used)",
                            vram_used, vram_free, usage_pct
                        ));
                    }
                    section.push('\n');
                } else if gpu.vendor == "Intel" {
                    section.push_str("  VRAM:    Shared with system RAM (integrated graphics)\n");
                } else {
                    section.push_str("  VRAM:    Not reported by hardware\n");
                }
            }
        }
        Err(_) => {
            section.push_str("No GPU detected\n");
        }
    }

    Ok(section)
}

fn generate_memory_section() -> Result<String> {
    let mem = get_ram_info()?;

    let mem_used_pct = (mem.used_mb as f64 / mem.total_mb as f64 * 100.0) as u8;
    let swap_status = if mem.swap_total_mb == 0 {
        "Not configured".to_string()
    } else {
        let swap_used_pct = if mem.swap_total_mb > 0 {
            (mem.swap_used_mb as f64 / mem.swap_total_mb as f64 * 100.0) as u8
        } else {
            0
        };
        format!(
            "{} MB total ({} MB used / {} MB free / {}% used)",
            mem.swap_total_mb,
            mem.swap_used_mb,
            mem.swap_total_mb.saturating_sub(mem.swap_used_mb),
            swap_used_pct
        )
    };

    Ok(format!(
        "─── Memory Details ─────────────────────────────────────────────\n\
         RAM Total:     {} MB\n\
         RAM Used:      {} MB\n\
         RAM Available: {} MB\n\
         RAM Usage:     {}%\n\
         Swap:          {}",
        mem.total_mb, mem.used_mb, mem.available_mb, mem_used_pct, swap_status
    ))
}

fn generate_storage_section() -> Result<String> {
    let disks = get_disk_usage()?;

    let mut section = String::from("─── Storage Details ────────────────────────────────────────────\n");

    for disk in disks.iter() {
        section.push_str(&format!(
            "{} on {}\n  Total: {:.1} GB | Used: {:.1} GB | Free: {:.1} GB | Usage: {}%\n\n",
            disk.filesystem,
            disk.mount_point,
            disk.total_gb,
            disk.used_gb,
            disk.available_gb,
            disk.use_percent
        ));
    }

    Ok(section)
}

fn generate_desktop_section() -> Result<String> {
    let de_wm = de_wm_detector::detect_de_wm();

    let session_type = std::env::var("XDG_SESSION_TYPE")
        .or_else(|_| {
            if std::env::var("WAYLAND_DISPLAY").is_ok() {
                Ok("wayland".to_string())
            } else if std::env::var("DISPLAY").is_ok() {
                Ok("x11".to_string())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        })
        .unwrap_or_else(|_| "Unknown".to_string());

    let de_type_str = match de_wm.de_type {
        de_wm_detector::DeType::DesktopEnvironment => "Desktop Environment",
        de_wm_detector::DeType::WindowManager => "Window Manager",
        de_wm_detector::DeType::Compositor => "Compositor",
    };

    let confidence_str = match de_wm.confidence {
        de_wm_detector::Confidence::High => "High",
        de_wm_detector::Confidence::Medium => "Medium",
        de_wm_detector::Confidence::Low => "Low",
    };

    Ok(format!(
        "─── Desktop Environment ────────────────────────────────────────\n\
         Type:         {}\n\
         Name:         {}\n\
         Session:      {}\n\
         Detection:    {} confidence via {}\n",
        de_type_str, de_wm.name, session_type, confidence_str, de_wm.detection_method
    ))
}

fn generate_network_section() -> Result<String> {
    let output = Command::new("ip")
        .args(&["addr", "show"])
        .output();

    let mut interfaces = Vec::new();

    if let Ok(output) = output {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                if !line.starts_with(' ') && line.contains(':') {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 2 {
                        let iface_name = parts[1].trim().split('@').next().unwrap_or("");
                        if !iface_name.is_empty() && iface_name != "lo" {
                            interfaces.push(iface_name.to_string());
                        }
                    }
                }
            }
        }
    }

    let iface_list = if interfaces.is_empty() {
        "None active (except loopback)".to_string()
    } else {
        interfaces.join(", ")
    };

    Ok(format!(
        "─── Network Summary ────────────────────────────────────────────\n\
         Active Interfaces: {}\n",
        iface_list
    ))
}

fn generate_services_section() -> Result<String> {
    let output = Command::new("systemctl")
        .args(&["list-units", "--type=service", "--state=running", "--no-pager", "--plain"])
        .output();

    let mut service_count = 0;
    let mut important_services = Vec::new();

    if let Ok(output) = output {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                if line.contains(".service") {
                    service_count += 1;

                    // Track important system services
                    if line.contains("sshd")
                        || line.contains("NetworkManager")
                        || line.contains("docker")
                        || line.contains("postgres")
                        || line.contains("mysql")
                    {
                        if let Some(service_name) = line.split_whitespace().next() {
                            important_services.push(service_name.to_string());
                        }
                    }
                }
            }
        }
    }

    let mut section = format!(
        "─── Services Status ────────────────────────────────────────────\n\
         Running Services: {}\n",
        service_count
    );

    if !important_services.is_empty() {
        section.push_str("Important Services:\n");
        for service in important_services.iter() {
            section.push_str(&format!("  - {}\n", service));
        }
    }

    Ok(section)
}

fn generate_diagnostics_section() -> Result<String> {
    let mem = get_ram_info()?;
    let disks = get_disk_usage().unwrap_or_default();

    let mut diagnostics: Vec<String> = Vec::new();

    // Check memory pressure
    let mem_usage_pct = (mem.used_mb as f64 / mem.total_mb as f64 * 100.0) as u8;
    if mem_usage_pct > 90 {
        diagnostics.push("CRITICAL: Memory usage above 90%".to_string());
    } else if mem_usage_pct > 75 {
        diagnostics.push("WARNING: Memory usage above 75%".to_string());
    }

    // Check swap usage
    if mem.swap_total_mb > 0 && mem.swap_used_mb > 0 {
        let swap_usage_pct = (mem.swap_used_mb as f64 / mem.swap_total_mb as f64 * 100.0) as u8;
        if swap_usage_pct > 50 {
            diagnostics.push("WARNING: High swap usage detected".to_string());
        }
    }

    // Check disk space
    for disk in disks.iter() {
        if disk.use_percent > 95 {
            diagnostics.push(format!(
                "CRITICAL: {} is {}% full",
                disk.mount_point, disk.use_percent
            ));
        } else if disk.use_percent > 85 {
            diagnostics.push(format!(
                "WARNING: {} is {}% full",
                disk.mount_point, disk.use_percent
            ));
        }
    }

    let mut section = String::from("─── Diagnostics Summary ────────────────────────────────────────\n");

    if diagnostics.is_empty() {
        section.push_str("Status: All systems nominal\n");
    } else {
        section.push_str("Status: Issues detected\n\n");
        for diag in diagnostics.iter() {
            section.push_str(&format!("  {}\n", diag));
        }
    }

    Ok(section)
}

// ============================================================================
// Helper Functions
// ============================================================================

fn get_uptime() -> Result<String> {
    let uptime_secs = std::fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|s| s.split_whitespace().next().map(String::from))
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0) as u64;

    let days = uptime_secs / 86400;
    let hours = (uptime_secs % 86400) / 3600;
    let minutes = (uptime_secs % 3600) / 60;

    if days > 0 {
        Ok(format!("{} days {} hours {} minutes", days, hours, minutes))
    } else if hours > 0 {
        Ok(format!("{} hours {} minutes", hours, minutes))
    } else {
        Ok(format!("{} minutes", minutes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_system_report() {
        let result = generate_system_report();
        assert!(result.is_ok());
        let report = result.unwrap();

        // Verify sections are present
        assert!(report.contains("SYSTEM REPORT"));
        assert!(report.contains("Operating System"));
        assert!(report.contains("Hardware Summary"));
        assert!(report.contains("CPU Details"));
        assert!(report.contains("Memory Details"));
        assert!(report.contains("Storage Details"));

        // Verify NO markdown formatting
        assert!(!report.contains("**"));
        assert!(!report.contains("##"));
        assert!(!report.contains("```"));

        // Verify NO emojis
        assert!(!report.contains("🚀"));
        assert!(!report.contains("📊"));
        assert!(!report.contains("💾"));
    }

    #[test]
    fn test_os_section() {
        let result = generate_os_section();
        assert!(result.is_ok());
        let section = result.unwrap();
        assert!(section.contains("OS:"));
        assert!(section.contains("Kernel:"));
        assert!(section.contains("Uptime:"));
    }

    #[test]
    fn test_cpu_section() {
        let result = generate_cpu_section();
        assert!(result.is_ok());
        let section = result.unwrap();
        assert!(section.contains("Model:"));
        assert!(section.contains("Cores:"));
        assert!(section.contains("Threads:"));
    }

    #[test]
    fn test_memory_section() {
        let result = generate_memory_section();
        assert!(result.is_ok());
        let section = result.unwrap();
        assert!(section.contains("RAM Total:"));
        assert!(section.contains("RAM Used:"));
        assert!(section.contains("RAM Available:"));
    }

    #[test]
    fn test_storage_section() {
        let result = generate_storage_section();
        assert!(result.is_ok());
        let section = result.unwrap();
        assert!(section.contains("Storage Details"));
    }

    #[test]
    fn test_get_uptime() {
        let result = get_uptime();
        assert!(result.is_ok());
        let uptime = result.unwrap();
        // Should contain either "days", "hours", or "minutes"
        assert!(uptime.contains("days") || uptime.contains("hours") || uptime.contains("minutes"));
    }
}
