//! Pretty rendering of profile data

use super::collector::ProfileData;
use super::checks::{Check, CheckStatus};
use anna_common::{anna_box, anna_narrative, MessageType};
use anyhow::Result;

pub fn render_profile(data: &ProfileData) -> Result<()> {
    anna_box(&["Here's what I've learned about your system"], MessageType::Narrative);
    println!();

    // Hardware section
    if !data.hardware.is_empty() {
        println!("💻 Hardware");
        if let Some(cpu) = data.hardware.get("cpu.model_name") {
            println!("  CPU: {}", cpu);
        }
        if let Some(mem) = data.hardware.get("memory.info") {
            let parts: Vec<_> = mem.split_whitespace().collect();
            if parts.len() >= 2 {
                println!("  Memory: {}", parts[1]);
            }
        }
        if let Some(kernel) = data.hardware.get("kernel.version") {
            println!("  Kernel: {}", kernel);
        }
        println!();
    }

    // Graphics section
    if !data.graphics.is_empty() {
        println!("🎨 Graphics");
        if let Some(gpu) = data.graphics.get("gpu.device") {
            // Truncate long GPU strings
            let gpu_short = if gpu.len() > 60 {
                format!("{}...", &gpu[..57])
            } else {
                gpu.clone()
            };
            println!("  GPU: {}", gpu_short);
        }
        if let Some(session) = data.graphics.get("session.type") {
            println!("  Session: {}", session);
        }
        if let Some(desktop) = data.graphics.get("desktop.current") {
            println!("  Desktop: {}", desktop);
        }
        if let Some(vaapi) = data.graphics.get("vaapi.available") {
            let status = if vaapi == "true" { "✅ Available" } else { "❌ Not available" };
            println!("  VA-API: {}", status);
        }
        println!();
    }

    // Audio section
    if !data.audio.is_empty() {
        println!("🔊 Audio");
        if let Some(server) = data.audio.get("audio.server") {
            println!("  Server: {}", server);
        }
        println!();
    }

    // Network section
    if !data.network.is_empty() {
        println!("🌐 Network");
        if let Some(interfaces) = data.network.get("network.interfaces") {
            let iface_list: Vec<_> = interfaces.split(';').take(3).collect();
            for iface in iface_list {
                if !iface.trim().is_empty() {
                    println!("  {}", iface.trim());
                }
            }
        }
        println!();
    }

    // Boot section
    if !data.boot.is_empty() {
        println!("⚡ Boot");
        if let Some(time) = data.boot.get("boot.time") {
            println!("  {}", time.lines().next().unwrap_or(time));
        }
        if let Some(failed) = data.boot.get("boot.failed_units") {
            if failed != "0" {
                println!("  ⚠️  {} failed units", failed);
            }
        }
        println!();
    }

    // Software section
    if !data.software.is_empty() {
        println!("📦 Software");
        if let Some(pkg_mgr) = data.software.get("pkg.manager") {
            print!("  Package manager: {}", pkg_mgr);
            if let Some(aur) = data.software.get("pkg.aur_helper") {
                print!(" + {} (AUR)", aur);
            }
            println!();
        }
        if let Some(shell) = data.software.get("shell.default") {
            println!("  Shell: {}", shell);
        }

        // List available tools
        let tools: Vec<_> = data.software.iter()
            .filter(|(k, v)| k.starts_with("tool.") && v == &"present")
            .map(|(k, _)| k.strip_prefix("tool.").unwrap())
            .collect();
        if !tools.is_empty() {
            println!("  Tools: {}", tools.join(", "));
        }
        println!();
    }

    // Anna's take
    anna_narrative(generate_annas_take(data));

    Ok(())
}

pub fn render_checks(checks: &[Check], json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(checks)?);
        return Ok(());
    }

    anna_box(&["I've run some health checks for you"], MessageType::Info);
    println!();

    let mut pass_count = 0;
    let mut warn_count = 0;
    let mut error_count = 0;

    for check in checks {
        match check.status {
            CheckStatus::Pass => pass_count += 1,
            CheckStatus::Warn => warn_count += 1,
            CheckStatus::Error => error_count += 1,
            CheckStatus::Info => {}
        }

        println!("{} {} - {}", check.status.emoji(), check.title, check.message);

        if let Some(rem) = &check.remediation {
            println!("     💡 {}", rem);
        }
        println!();
    }

    // Summary
    println!("Summary: {} passing, {} warnings, {} errors",
        pass_count, warn_count, error_count);

    if warn_count > 0 || error_count > 0 {
        anna_narrative("I found a few things we could improve. Want me to help?");
    } else {
        anna_narrative("Everything looks great! Your system is in good shape.");
    }

    Ok(())
}

fn generate_annas_take(data: &ProfileData) -> String {
    let mut observations = Vec::new();

    // Check for modern setup
    if data.graphics.get("session.type").map(|s| s.as_str()) == Some("wayland") {
        observations.push("running a modern Wayland session");
    }

    if data.audio.get("audio.server").map(|s| s.as_str()) == Some("PipeWire") {
        observations.push("using PipeWire for low-latency audio");
    }

    if data.software.contains_key("pkg.aur_helper") {
        observations.push("set up with AUR access");
    }

    if data.graphics.get("vaapi.available").map(|s| s.as_str()) == Some("true") {
        observations.push("GPU acceleration available");
    }

    if observations.is_empty() {
        "Your system looks stable and functional!".to_string()
    } else {
        format!("Nice setup! I see you're {} — that's a solid choice.", observations.join(", "))
    }
}
