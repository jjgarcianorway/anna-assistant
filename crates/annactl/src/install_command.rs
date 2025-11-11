//! Installation command handler
//!
//! Phase 0.8: Interactive installation dialogue
//! Citation: [archwiki:Installation_guide]

use crate::errors::*;
use crate::logging::LogEntry;
use crate::rpc_client::RpcClient;
use anna_common::ipc::{DiskSetupData, InstallConfigData, ResponseData};
use anyhow::{Context, Result};
use std::io::{self, Write};
use std::time::Instant;

/// Execute interactive installation command
pub async fn execute_install_command(
    dry_run: bool,
    req_id: &str,
    state: &str,
    start_time: Instant,
) -> Result<()> {
    // Verify we're in iso_live state
    if state != "iso_live" {
        eprintln!("[anna] Installation only available in iso_live state");
        eprintln!("[anna] Current state: {}", state);
        eprintln!("Citation: [archwiki:Installation_guide]");
        std::process::exit(EXIT_COMMAND_NOT_AVAILABLE);
    }

    println!("[anna] Arch Linux Installation");
    println!();

    // Interactive dialogue
    let config = if dry_run {
        // Use defaults for dry-run
        create_default_config()
    } else {
        gather_installation_config().await?
    };

    // Connect to daemon
    let mut client = RpcClient::connect()
        .await
        .context("Failed to connect to daemon")?;

    // Perform installation
    println!();
    if dry_run {
        println!("[anna] Installation simulation (dry-run)");
    } else {
        println!("[anna] Starting installation...");
    }
    println!();

    let response = client.perform_install(config.clone(), dry_run).await?;

    let data = match response {
        ResponseData::InstallResult(data) => data,
        _ => {
            eprintln!("[anna] Invalid response from daemon");
            std::process::exit(EXIT_INVALID_RESPONSE);
        }
    };

    // Print results
    print_install_results(&data);

    // Determine exit code
    let exit_code = if data.success {
        EXIT_SUCCESS
    } else {
        1
    };

    // Log to ctl.jsonl
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let log_entry = LogEntry {
        ts: LogEntry::now(),
        req_id: req_id.to_string(),
        state: state.to_string(),
        command: "install".to_string(),
        allowed: Some(true),
        args: if dry_run {
            vec!["--dry-run".to_string()]
        } else {
            vec![]
        },
        exit_code,
        citation: "[archwiki:Installation_guide]".to_string(),
        duration_ms,
        ok: data.success,
        error: None,
    };
    let _ = log_entry.write();

    std::process::exit(exit_code);
}

/// Gather installation configuration from user
async fn gather_installation_config() -> Result<InstallConfigData> {
    let mut config = InstallConfigData {
        disk_setup: DiskSetupData::Manual {
            root_partition: String::new(),
            boot_partition: String::new(),
            swap_partition: None,
        },
        bootloader: "systemd-boot".to_string(),
        hostname: String::new(),
        username: String::new(),
        timezone: "UTC".to_string(),
        locale: "en_US.UTF-8".to_string(),
        extra_packages: vec![],
    };

    // Disk setup
    println!("[anna] Disk Setup");
    println!("Available partitions:");
    let _ = std::process::Command::new("lsblk")
        .args(&["-o", "NAME,SIZE,TYPE,MOUNTPOINT"])
        .status();
    println!();

    config.disk_setup = prompt_disk_setup()?;

    // Bootloader
    println!();
    println!("[anna] Bootloader");
    config.bootloader = prompt_choice(
        "Select bootloader",
        &[("systemd-boot", "Modern, simple"), ("grub", "Traditional")],
        "systemd-boot",
    )?;

    // Hostname
    println!();
    config.hostname = prompt_string("Hostname", Some("archlinux"))?;

    // Username
    config.username = prompt_string("Username", Some("user"))?;

    // Timezone
    println!();
    config.timezone = prompt_string("Timezone", Some("UTC"))?;

    // Locale
    config.locale = prompt_string("Locale", Some("en_US.UTF-8"))?;

    // Extra packages
    println!();
    println!("[anna] Additional packages (space-separated, or Enter to skip):");
    let extra = prompt_string("Packages", None)?;
    if !extra.is_empty() {
        config.extra_packages = extra.split_whitespace().map(|s| s.to_string()).collect();
    }

    Ok(config)
}

/// Prompt for disk setup configuration
fn prompt_disk_setup() -> Result<DiskSetupData> {
    let mode = prompt_choice(
        "Disk setup mode",
        &[
            ("manual", "Manual partitioning (existing partitions)"),
            ("auto", "Automatic with btrfs (not yet implemented)"),
        ],
        "manual",
    )?;

    match mode.as_str() {
        "manual" => {
            let root = prompt_string("Root partition (e.g., sda2)", None)?;
            let boot = prompt_string("Boot partition (e.g., sda1)", None)?;
            let swap = prompt_optional("Swap partition (optional, Enter to skip)", None)?;

            Ok(DiskSetupData::Manual {
                root_partition: root,
                boot_partition: boot,
                swap_partition: swap,
            })
        }
        "auto" => {
            eprintln!("[anna] Automatic btrfs setup not yet implemented");
            eprintln!("[anna] Please use manual mode");
            std::process::exit(1);
        }
        _ => unreachable!(),
    }
}

/// Prompt for a string value
fn prompt_string(prompt: &str, default: Option<&str>) -> Result<String> {
    print!("[anna] {}", prompt);
    if let Some(def) = default {
        print!(" [{}]", def);
    }
    print!(": ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        if let Some(def) = default {
            Ok(def.to_string())
        } else {
            eprintln!("[anna] Input required");
            std::process::exit(1);
        }
    } else {
        Ok(input.to_string())
    }
}

/// Prompt for optional string value
fn prompt_optional(prompt: &str, default: Option<&str>) -> Result<Option<String>> {
    print!("[anna] {}", prompt);
    if let Some(def) = default {
        print!(" [{}]", def);
    }
    print!(": ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        Ok(default.map(|s| s.to_string()))
    } else {
        Ok(Some(input.to_string()))
    }
}

/// Prompt for a choice from options
fn prompt_choice(prompt: &str, options: &[(&str, &str)], default: &str) -> Result<String> {
    println!("[anna] {}", prompt);
    for (value, description) in options {
        let marker = if *value == default { "*" } else { " " };
        println!("  {} {} - {}", marker, value, description);
    }
    print!("[anna] Choice [{}]: ", default);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        Ok(default.to_string())
    } else {
        // Validate choice
        if options.iter().any(|(v, _)| *v == input) {
            Ok(input.to_string())
        } else {
            eprintln!("[anna] Invalid choice: {}", input);
            std::process::exit(1);
        }
    }
}

/// Create default configuration for dry-run
fn create_default_config() -> InstallConfigData {
    InstallConfigData {
        disk_setup: DiskSetupData::Manual {
            root_partition: "sda2".to_string(),
            boot_partition: "sda1".to_string(),
            swap_partition: Some("sda3".to_string()),
        },
        bootloader: "systemd-boot".to_string(),
        hostname: "archlinux".to_string(),
        username: "user".to_string(),
        timezone: "UTC".to_string(),
        locale: "en_US.UTF-8".to_string(),
        extra_packages: vec![],
    }
}

/// Print installation results
fn print_install_results(data: &anna_common::ipc::InstallResultData) {
    for step in &data.steps {
        let status = if step.success { "OK" } else { "FAIL" };
        println!("[anna] {} — {} ({})", step.name, step.description, status);
        if !step.details.is_empty() {
            for line in step.details.lines() {
                println!("  {}", line);
            }
        }
        if !step.success || data.dry_run {
            println!("  Citation: {}", step.citation);
        }
    }

    println!();
    println!("[anna] {}", data.message);
    println!("Citation: {}", data.citation);

    if data.success && !data.dry_run {
        println!();
        println!("[anna] Installation complete!");
        println!("[anna] Next steps:");
        println!("  1. Reboot: umount -R /mnt && reboot");
        println!("  2. Log in with created user credentials");
        println!("  3. Change default passwords immediately");
    }
}
