//! Installation subsystem for guided Arch Linux setup
//!
//! Phase 0.8: System Installer - state-aware installation with structured dialogue
//! Citation: [archwiki:Installation_guide]

mod types;
mod disk;
mod packages;
mod bootloader;
mod users;
mod logging;

pub use types::{
    InstallConfig, InstallResult, InstallStep, DiskSetupMode, BootloaderType,
    InstallationState, StepResult,
};

use anyhow::{Context, Result};
use tracing::{info, warn, error};

/// Perform guided Arch Linux installation
///
/// This function orchestrates the full installation process:
/// 1. Validate environment (must be iso_live)
/// 2. Disk setup (partition, format, mount)
/// 3. Base system installation (pacstrap)
/// 4. System configuration (fstab, locale, timezone)
/// 5. Bootloader installation
/// 6. User creation
///
/// # Arguments
/// * `config` - Installation configuration from user dialogue
/// * `dry_run` - If true, simulates installation without executing
///
/// # Returns
/// * `InstallResult` - Detailed results of each installation step
///
/// # Security
/// - Must run as root
/// - Only executes in iso_live state
/// - Uses arch-chroot and pacstrap (no shell injection)
/// - All operations logged to /var/log/anna/install.jsonl
pub async fn perform_installation(
    config: &InstallConfig,
    dry_run: bool,
) -> Result<InstallResult> {
    info!("Starting installation: dry_run={}", dry_run);

    // Validate environment
    let env_state = detect_environment().await?;
    if env_state != InstallationState::IsoLive {
        return Err(anyhow::anyhow!(
            "Installation only available in iso_live state (detected: {:?})",
            env_state
        ));
    }

    // Check root privileges
    if !is_root() {
        return Err(anyhow::anyhow!(
            "Installation requires root privileges. Run with sudo."
        ));
    }

    let mut steps = Vec::new();
    let mut overall_success = true;

    // Step 1: Disk setup
    info!("Step 1: Disk setup");
    let disk_result = disk::setup_disks(config, dry_run).await;
    let disk_success = disk_result.is_ok();
    let disk_details = match &disk_result {
        Ok(details) => details.clone(),
        Err(e) => format!("Failed: {}", e),
    };
    steps.push(InstallStep {
        name: "disk_setup".to_string(),
        description: "Partition, format, and mount disks".to_string(),
        success: disk_success,
        details: disk_details.clone(),
        citation: "[archwiki:Installation_guide#Partition_the_disks]".to_string(),
    });

    // Log disk setup
    let log_entry = logging::InstallLogEntry::new(
        "disk_setup".to_string(),
        "partition_format_mount".to_string(),
        disk_success,
        disk_details,
        "[archwiki:Installation_guide#Partition_the_disks]".to_string(),
        dry_run,
    );
    let _ = log_entry.write().await;

    if !disk_success {
        overall_success = false;
        warn!("Disk setup failed, aborting installation");
        return Ok(InstallResult {
            success: false,
            steps,
            message: "Installation aborted: disk setup failed".to_string(),
            citation: "[archwiki:Installation_guide]".to_string(),
        });
    }

    // Step 2: Base system installation
    info!("Step 2: Base system installation");
    let packages_result = packages::install_base_system(config, dry_run).await;
    let packages_success = packages_result.is_ok();
    steps.push(InstallStep {
        name: "base_system".to_string(),
        description: "Install base packages with pacstrap".to_string(),
        success: packages_success,
        details: match &packages_result {
            Ok(details) => details.clone(),
            Err(e) => format!("Failed: {}", e),
        },
        citation: "[archwiki:Installation_guide#Install_essential_packages]".to_string(),
    });
    if !packages_success {
        overall_success = false;
        warn!("Base system installation failed");
    }

    // Step 3: System configuration
    info!("Step 3: System configuration");
    let config_result = packages::configure_system(config, dry_run).await;
    let config_success = config_result.is_ok();
    steps.push(InstallStep {
        name: "system_config".to_string(),
        description: "Configure fstab, locale, timezone, hostname".to_string(),
        success: config_success,
        details: match &config_result {
            Ok(details) => details.clone(),
            Err(e) => format!("Failed: {}", e),
        },
        citation: "[archwiki:Installation_guide#Configure_the_system]".to_string(),
    });
    if !config_success {
        overall_success = false;
    }

    // Step 4: Bootloader installation
    info!("Step 4: Bootloader installation");
    let bootloader_result = bootloader::install_bootloader(config, dry_run).await;
    let bootloader_success = bootloader_result.is_ok();
    steps.push(InstallStep {
        name: "bootloader".to_string(),
        description: format!("Install {} bootloader", config.bootloader),
        success: bootloader_success,
        details: match &bootloader_result {
            Ok(details) => details.clone(),
            Err(e) => format!("Failed: {}", e),
        },
        citation: match config.bootloader {
            BootloaderType::SystemdBoot => "[archwiki:Systemd-boot]".to_string(),
            BootloaderType::Grub => "[archwiki:GRUB]".to_string(),
        },
    });
    if !bootloader_success {
        overall_success = false;
    }

    // Step 5: User creation
    info!("Step 5: User creation");
    let user_result = users::create_user(config, dry_run).await;
    let user_success = user_result.is_ok();
    steps.push(InstallStep {
        name: "user_creation".to_string(),
        description: format!("Create user '{}' and configure permissions", config.username),
        success: user_success,
        details: match &user_result {
            Ok(details) => details.clone(),
            Err(e) => format!("Failed: {}", e),
        },
        citation: "[archwiki:Users_and_groups]".to_string(),
    });
    if !user_success {
        overall_success = false;
    }

    let message = if dry_run {
        "Installation simulation completed".to_string()
    } else if overall_success {
        "Installation completed successfully".to_string()
    } else {
        "Installation completed with errors".to_string()
    };

    Ok(InstallResult {
        success: overall_success,
        steps,
        message,
        citation: "[archwiki:Installation_guide]".to_string(),
    })
}

/// Detect installation environment state
async fn detect_environment() -> Result<InstallationState> {
    // Check if running from Arch ISO
    if std::path::Path::new("/run/archiso").exists() {
        return Ok(InstallationState::IsoLive);
    }

    // Check if system has been installed but not fully configured
    if std::path::Path::new("/etc/fstab").exists()
        && !std::path::Path::new("/etc/hostname").exists()
    {
        return Ok(InstallationState::PostInstallMinimal);
    }

    // Check if system is fully configured
    if std::path::Path::new("/etc/hostname").exists()
        && std::path::Path::new("/etc/locale.conf").exists()
    {
        return Ok(InstallationState::Configured);
    }

    Ok(InstallationState::Unknown)
}

/// Check if running as root
fn is_root() -> bool {
    #[cfg(unix)]
    {
        use nix::unistd::Uid;
        Uid::effective().is_root()
    }
    #[cfg(not(unix))]
    {
        false
    }
}
