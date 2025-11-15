//! Report Display - Generate professional system reports
//!
//! Phase 5.1: Conversational UX
//! Create reports suitable for managers or documentation

use anna_common::display::UI;
use anna_common::ipc::{Method, ResponseData};
use chrono::Local;
use crate::rpc_client::RpcClient;

/// Generate a professional system report using actual system data
pub fn generate_professional_report() {
    let ui = UI::auto();

    // Fetch real system facts from daemon
    let facts = match fetch_system_facts() {
        Some(f) => f,
        None => {
            ui.error("Unable to fetch system data from daemon");
            return;
        }
    };

    println!();
    ui.section_header("📋", "System Report");
    ui.info(&format!("Generated: {}", Local::now().format("%Y-%m-%d %H:%M:%S")));
    ui.info(&format!("Machine: {}", facts.hostname));
    println!();

    // Executive Summary
    ui.section_header("📊", "Executive Summary");

    let health_status = if !facts.failed_services.is_empty() {
        "functioning with some issues that require attention"
    } else {
        "functioning well"
    };

    ui.info(&format!("This {} workstation ({}) is {}.",
        if facts.desktop_environment.is_some() || facts.window_manager.is_some() {
            "desktop"
        } else {
            "server"
        },
        facts.hostname,
        health_status
    ));

    if let Some(ref de) = facts.desktop_environment {
        ui.info(&format!("Desktop environment: {}", de));
    } else if let Some(ref wm) = facts.window_manager {
        ui.info(&format!("Window manager: {}", wm));
    } else {
        ui.info("Headless/server configuration");
    }

    println!();

    // Machine Overview
    ui.section_header("💻", "Machine Overview");

    ui.info("Operating System:");
    ui.bullet_list(&[
        &format!("Distribution: Arch Linux ({})", facts.kernel),
        &format!("Hostname: {}", facts.hostname),
        &format!("Shell: {}", facts.shell),
        &format!("Installed Packages: {}", facts.installed_packages),
    ]);
    println!();

    ui.info("Hardware Configuration:");
    let mut hw_info = vec![
        format!("CPU: {} ({} cores)", facts.cpu_model, facts.cpu_cores),
        format!("RAM: {:.1} GB total memory", facts.total_memory_gb),
    ];

    if let Some(ref gpu) = facts.gpu_model {
        if let Some(vram_mb) = facts.gpu_vram_mb {
            hw_info.push(format!("GPU: {} ({} MB VRAM)", gpu, vram_mb));
        } else {
            hw_info.push(format!("GPU: {}", gpu));
        }
    } else if let Some(ref vendor) = facts.gpu_vendor {
        hw_info.push(format!("GPU: {} (vendor)", vendor));
    }

    if !facts.storage_devices.is_empty() {
        let total_storage: f64 = facts.storage_devices.iter().map(|d| d.size_gb).sum();
        hw_info.push(format!("Storage: {:.0} GB total across {} device(s)",
            total_storage, facts.storage_devices.len()));
    }

    ui.bullet_list(&hw_info.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    println!();

    if let Some(ref display) = facts.display_server {
        ui.info("Display Configuration:");
        let mut display_info = vec![format!("Display Server: {}", display)];
        if let Some(ref de) = facts.desktop_environment {
            display_info.push(format!("Desktop Environment: {}", de));
        }
        if let Some(ref wm) = facts.window_manager {
            display_info.push(format!("Window Manager: {}", wm));
        }
        ui.bullet_list(&display_info.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        println!();
    }

    // System Health
    ui.section_header("🏥", "System Health & Status");

    if facts.failed_services.is_empty() {
        ui.success("Overall Status: Healthy - No failed services");
    } else {
        ui.warning(&format!("Overall Status: {} failed service(s) detected",
            facts.failed_services.len()));

        ui.info("Failed Services:");
        for service in &facts.failed_services {
            ui.bullet_list(&[&format!("❌ {}", service)]);
        }
        println!();
    }

    ui.info("Resource Utilization:");
    let mut resources = vec![];

    // Disk space
    for disk in &facts.storage_devices {
        let usage_pct = (disk.used_gb / disk.size_gb) * 100.0;
        let status = if usage_pct > 90.0 { "⚠️  CRITICAL" }
                    else if usage_pct > 80.0 { "⚠️  Warning" }
                    else { "✓ Good" };
        resources.push(format!("{}: {:.1}/{:.1} GB ({:.0}%) - {}",
            disk.mount_point, disk.used_gb, disk.size_gb, usage_pct, status));
    }

    // Boot time
    if let Some(boot_time) = facts.boot_time_seconds {
        let status = if boot_time > 60.0 { "⚠️  Slow" }
                    else if boot_time > 30.0 { "Moderate" }
                    else { "✓ Fast" };
        resources.push(format!("Boot Time: {:.1}s - {}", boot_time, status));
    }

    // Package cache
    if facts.package_cache_size_gb > 5.0 {
        resources.push(format!("Package Cache: {:.1} GB - Consider cleanup",
            facts.package_cache_size_gb));
    }

    ui.bullet_list(&resources.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    println!();

    // Dev Tools
    if !facts.dev_tools_detected.is_empty() {
        ui.info("Development Tools Detected:");
        ui.bullet_list(&facts.dev_tools_detected.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        println!();
    }

    // Recommendations
    ui.section_header("📝", "Recommended Actions");

    let mut recommendations: Vec<String> = vec![];

    if !facts.failed_services.is_empty() {
        recommendations.push("Investigate and fix failed services".to_string());
    }

    for disk in &facts.storage_devices {
        let usage_pct = (disk.used_gb / disk.size_gb) * 100.0;
        if usage_pct > 80.0 {
            recommendations.push(format!("Free up space on {} ({:.0}% full)",
                disk.mount_point, usage_pct));
        }
    }

    if facts.package_cache_size_gb > 5.0 {
        recommendations.push("Clean package cache to reclaim disk space".to_string());
    }

    if !facts.orphan_packages.is_empty() {
        recommendations.push(format!("Review {} orphaned packages",
            facts.orphan_packages.len()));
    }

    if !facts.slow_services.is_empty() {
        recommendations.push("Review slow-starting services to improve boot time".to_string());
    }

    if recommendations.is_empty() {
        ui.success("No immediate actions required - system is running optimally!");
    } else {
        let refs: Vec<&str> = recommendations.iter().map(|s| s.as_str()).collect();
        ui.numbered_list(&refs);
    }
    println!();

    // Technical Notes
    ui.section_header("📋", "Technical Notes");

    ui.info("This report was generated by Anna Assistant using real-time");
    ui.info("system telemetry. All data is collected locally and reflects");
    ui.info("the actual state of this machine at the time of generation.");
    println!();

    ui.success("Report Complete");
    println!();
}

/// Fetch system facts from daemon
fn fetch_system_facts() -> Option<anna_common::types::SystemFacts> {
    tokio::runtime::Runtime::new().ok()?.block_on(async {
        let mut rpc = RpcClient::connect().await.ok()?;
        match rpc.call(Method::GetFacts).await {
            Ok(ResponseData::Facts(facts)) => Some(facts),
            _ => None,
        }
    })
}
