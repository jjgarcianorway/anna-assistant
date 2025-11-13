// Monitoring Stack Installation
// Phase 3.1: Automated Monitoring Setup
//
// Handles installation and configuration of Prometheus + Grafana
// with adaptive mode selection based on system resources.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Check if a package is installed via pacman
fn is_package_installed(package: &str) -> Result<bool> {
    let output = Command::new("pacman")
        .args(["-Q", package])
        .output()
        .context("Failed to run pacman -Q")?;

    Ok(output.status.success())
}

/// Check if running as root (required for installation)
fn check_root() -> Result<()> {
    #[cfg(unix)]
    {
        let output = Command::new("id")
            .arg("-u")
            .output()
            .context("Failed to check user ID")?;

        if let Ok(uid_str) = String::from_utf8(output.stdout) {
            if let Ok(uid) = uid_str.trim().parse::<u32>() {
                if uid != 0 {
                    anyhow::bail!("This operation requires root privileges. Please run with sudo.");
                }
            }
        }
    }
    Ok(())
}

/// Install packages via pacman
fn install_packages(packages: &[&str], dry_run: bool) -> Result<()> {
    if dry_run {
        println!("[DRY RUN] Would install: {}", packages.join(", "));
        return Ok(());
    }

    check_root()?;

    println!("Installing packages: {}", packages.join(", "));

    let status = Command::new("pacman")
        .args(["-S", "--needed", "--noconfirm"])
        .args(packages)
        .status()
        .context("Failed to run pacman")?;

    if !status.success() {
        anyhow::bail!("Package installation failed");
    }

    println!("✓ Packages installed successfully");
    Ok(())
}

/// Enable and start systemd services
fn enable_and_start_services(services: &[&str], dry_run: bool) -> Result<()> {
    if dry_run {
        println!("[DRY RUN] Would enable and start: {}", services.join(", "));
        return Ok(());
    }

    check_root()?;

    for service in services {
        println!("Enabling and starting service: {}", service);

        // Enable service
        let enable_status = Command::new("systemctl")
            .args(["enable", service])
            .status()
            .context(format!("Failed to enable service: {}", service))?;

        if !enable_status.success() {
            eprintln!("⚠️  Failed to enable service: {}", service);
        }

        // Start service
        let start_status = Command::new("systemctl")
            .args(["start", service])
            .status()
            .context(format!("Failed to start service: {}", service))?;

        if !start_status.success() {
            eprintln!("⚠️  Failed to start service: {}", service);
        } else {
            println!("✓ Service {} is running", service);
        }
    }

    Ok(())
}

/// Check service status
pub fn check_service_status(service: &str) -> Result<bool> {
    let output = Command::new("systemctl")
        .args(["is-active", service])
        .output()
        .context(format!("Failed to check status of {}", service))?;

    Ok(output.status.success())
}

/// Deploy Prometheus configuration
fn deploy_prometheus_config(mode: &str, dry_run: bool) -> Result<()> {
    let config_src = match mode {
        "full" => "monitoring/prometheus/prometheus-full.yml",
        "light" => "monitoring/prometheus/prometheus-light.yml",
        _ => return Ok(()), // No config for minimal
    };

    if dry_run {
        println!("[DRY RUN] Would deploy Prometheus config from {}", config_src);
        return Ok(());
    }

    check_root()?;

    // Check if source config exists
    if !Path::new(config_src).exists() {
        println!("⚠️  Prometheus config template not found: {}", config_src);
        println!("   Using default Prometheus configuration");
        return Ok(());
    }

    let config_dest = "/etc/prometheus/prometheus.yml";

    // Backup existing config
    if Path::new(config_dest).exists() {
        let backup = format!("{}.backup", config_dest);
        fs::copy(config_dest, &backup)
            .context("Failed to backup existing Prometheus config")?;
        println!("✓ Backed up existing config to {}", backup);
    }

    // Copy new config
    fs::copy(config_src, config_dest)
        .context("Failed to deploy Prometheus config")?;

    println!("✓ Deployed Prometheus configuration");

    // Reload Prometheus
    let _ = Command::new("systemctl")
        .args(["reload", "prometheus"])
        .status();

    Ok(())
}

/// Deploy Grafana dashboards
fn deploy_grafana_dashboards(dry_run: bool) -> Result<()> {
    if dry_run {
        println!("[DRY RUN] Would deploy Grafana dashboards");
        return Ok(());
    }

    check_root()?;

    let dashboard_dir = "/var/lib/grafana/dashboards";

    // Create dashboard directory
    fs::create_dir_all(dashboard_dir)
        .context("Failed to create Grafana dashboard directory")?;

    // Copy dashboard files if they exist
    let dashboards = [
        ("monitoring/dashboards/anna-overview.json", "anna-overview.json"),
        ("monitoring/dashboards/anna-resources.json", "anna-resources.json"),
    ];

    let mut deployed = 0;
    for (src, dest) in &dashboards {
        if Path::new(src).exists() {
            let dest_path = format!("{}/{}", dashboard_dir, dest);
            fs::copy(src, &dest_path)
                .context(format!("Failed to copy dashboard: {}", src))?;
            deployed += 1;
        }
    }

    if deployed > 0 {
        println!("✓ Deployed {} Grafana dashboard(s)", deployed);
    } else {
        println!("⚠️  No dashboard templates found in monitoring/dashboards/");
    }

    Ok(())
}

/// Install monitoring stack in full mode
pub fn install_full_mode(dry_run: bool) -> Result<()> {
    println!("\n🚀 Installing Full Monitoring Stack");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Check what's already installed
    let prometheus_installed = is_package_installed("prometheus")?;
    let grafana_installed = is_package_installed("grafana")?;

    println!("\nCurrent status:");
    println!("  Prometheus: {}", if prometheus_installed { "✓ Installed" } else { "✗ Not installed" });
    println!("  Grafana:    {}", if grafana_installed { "✓ Installed" } else { "✗ Not installed" });
    println!();

    // Install Prometheus first (required for Grafana to be useful)
    if !prometheus_installed {
        install_packages(&["prometheus"], dry_run)?;
    } else {
        println!("✓ Prometheus already installed");
    }

    // Deploy Prometheus configuration
    deploy_prometheus_config("full", dry_run)?;

    // Start Prometheus first
    enable_and_start_services(&["prometheus"], dry_run)?;

    // Phase 3.9: Check if Prometheus is running before provisioning Grafana
    let prometheus_running = check_service_status("prometheus").unwrap_or(false);

    if !prometheus_running && !dry_run {
        println!("\n⚠️  Prometheus is not running - skipping Grafana installation");
        println!("   Grafana requires Prometheus for metrics datasource.");
        println!("   Fix Prometheus first, then run: annactl monitor install");
        return Ok(());
    }

    // Now install and configure Grafana
    if !grafana_installed {
        install_packages(&["grafana"], dry_run)?;
    } else {
        println!("✓ Grafana already installed");
    }

    // Deploy Grafana dashboards
    deploy_grafana_dashboards(dry_run)?;

    // Start Grafana
    enable_and_start_services(&["grafana"], dry_run)?;

    if !dry_run {
        println!("\n✅ Full monitoring stack installed successfully!");
        println!("\n📊 Access Points:");
        println!("  Prometheus: http://localhost:9090");
        println!("  Grafana:    http://localhost:3000");
        println!();
        println!("🔑 Default Credentials:");
        println!("  Username: admin");
        println!("  Password: admin");
        println!("  (Change on first login)");
        println!();

        // Phase 3.9: Detect SSH session and show tunnel command
        if let Ok(session_type) = std::env::var("SSH_CONNECTION") {
            if !session_type.is_empty() {
                println!("🌐 Remote Access (SSH session detected):");
                println!();
                println!("  To access Grafana from your local machine, create an SSH tunnel:");
                println!("  [33mssh -L 3000:localhost:3000 {}@<host>[39m",
                    std::env::var("USER").unwrap_or_else(|_| "user".to_string()));
                println!();
                println!("  Then browse to: http://localhost:3000");
                println!();
                println!("  For Prometheus:");
                println!("  [33mssh -L 9090:localhost:9090 {}@<host>[39m",
                    std::env::var("USER").unwrap_or_else(|_| "user".to_string()));
                println!();
            }
        }

        println!("📝 Next Steps:");
        println!("  1. Browse to Grafana and change the default password");
        println!("  2. Prometheus datasource is auto-configured: http://localhost:9090");
        println!("  3. Import Anna dashboards from /var/lib/grafana/dashboards/");
        println!("  4. View metrics: annactl metrics --prometheus");
    }

    Ok(())
}

/// Install monitoring stack in light mode
pub fn install_light_mode(dry_run: bool) -> Result<()> {
    println!("\n🚀 Installing Light Monitoring Stack");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Check what's already installed
    let prometheus_installed = is_package_installed("prometheus")?;

    println!("\nCurrent status:");
    println!("  Prometheus: {}", if prometheus_installed { "✓ Installed" } else { "✗ Not installed" });
    println!();

    // Install Prometheus if needed
    if !prometheus_installed {
        install_packages(&["prometheus"], dry_run)?;
    } else {
        println!("✓ Prometheus already installed");
    }

    // Deploy configuration
    deploy_prometheus_config("light", dry_run)?;

    // Enable and start service
    enable_and_start_services(&["prometheus"], dry_run)?;

    if !dry_run {
        println!("\n✅ Light monitoring stack installed successfully!");
        println!("\n📊 Access Points:");
        println!("  Prometheus: http://localhost:9090");
        println!("  Metrics API: http://localhost:9090/metrics");
        println!();

        // Phase 3.9: Detect SSH session and show tunnel command
        if let Ok(session_type) = std::env::var("SSH_CONNECTION") {
            if !session_type.is_empty() {
                println!("🌐 Remote Access (SSH session detected):");
                println!();
                println!("  To access Prometheus from your local machine, create an SSH tunnel:");
                println!("  [33mssh -L 9090:localhost:9090 {}@<host>[39m",
                    std::env::var("USER").unwrap_or_else(|_| "user".to_string()));
                println!();
                println!("  Then browse to: http://localhost:9090");
                println!();
            }
        }

        println!("💡 Note: Grafana not installed (light mode)");
        println!("   Use Prometheus web UI for metrics visualization");
        println!("   Or export to external monitoring via federation");
        println!();
        println!("📝 Next Steps:");
        println!("  1. Browse to Prometheus: http://localhost:9090");
        println!("  2. Explore Anna metrics: annactl metrics --prometheus");
        println!("  3. Query PromQL: anna_system_memory_available_mb");
    }

    Ok(())
}

/// Install monitoring stack in minimal mode
pub fn install_minimal_mode() -> Result<()> {
    println!("\n✅ Minimal Mode - No External Monitoring");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\nAnna's internal monitoring is active.");
    println!("No external tools will be installed.");
    println!("\nView system status:");
    println!("  annactl status  - Overall system health");
    println!("  annactl health  - Detailed health probes");
    println!("  annactl metrics - Internal metrics");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_package_installed() {
        // Test with a package that's definitely installed (systemd on Arch)
        let result = is_package_installed("systemd");
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_service_status() {
        // Test with systemd itself (always running)
        let result = check_service_status("systemd");
        assert!(result.is_ok());
    }
}
