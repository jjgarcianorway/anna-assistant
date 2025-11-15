// Upgrade Command - Auto-Update Implementation
// Phase 3.10: AUR-Aware Auto-Upgrade System
//
// Handles checking for updates, downloading, verifying, and installing new versions

use anna_common::github_releases::{GitHubClient, is_update_available};
use anna_common::installation_source::{detect_current_installation, InstallationSource};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

const GITHUB_OWNER: &str = "jjgarcianorway";
const GITHUB_REPO: &str = "anna-assistant";
const BACKUP_DIR: &str = "/var/lib/anna/backup";

/// Execute upgrade command
pub async fn execute_upgrade_command(auto_yes: bool, check_only: bool) -> Result<()> {
    println!("🔄 Anna Assistant Upgrade System");
    println!();

    // Step 1: Detect installation source
    let source = detect_current_installation();
    println!("📦 Installation Source: {}", source.description());

    if !source.allows_auto_update() {
        print_aur_refusal_message(&source);
        std::process::exit(1);
    }

    println!("✅ Auto-update: Enabled");
    println!();

    // Step 2: Get current version
    let current_version = env!("CARGO_PKG_VERSION");
    println!("📌 Current Version: v{}", current_version);

    // Step 3: Check for updates
    println!("🌐 Checking GitHub for updates...");
    let client = GitHubClient::new(GITHUB_OWNER, GITHUB_REPO);

    let latest_release = match client.get_latest_release().await {
        Ok(release) => release,
        Err(e) => {
            eprintln!("❌ Failed to check for updates: {}", e);
            eprintln!("   Network issue or GitHub API unavailable");
            std::process::exit(1);
        }
    };

    let latest_version = latest_release.version();
    println!("📌 Latest Version: v{}", latest_version);
    println!();

    // Step 4: Compare versions
    if !is_update_available(current_version, latest_version) {
        println!("✅ You are already running the latest version!");
        return Ok(());
    }

    println!("🆕 Update available: v{} → v{}", current_version, latest_version);
    println!();

    if check_only {
        println!("Run 'sudo annactl upgrade' to install the update.");
        return Ok(());
    }

    // Step 5: Confirm upgrade
    if !auto_yes {
        print!("Do you want to upgrade? [y/N] ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
            println!("Upgrade cancelled.");
            return Ok(());
        }
    }

    println!("📥 Downloading v{}...", latest_version);

    // Step 6: Download binaries
    let temp_dir = std::env::temp_dir().join(format!("anna-upgrade-{}", latest_version));
    std::fs::create_dir_all(&temp_dir)?;

    let annactl_asset = latest_release
        .find_asset("annactl")
        .context("annactl binary not found in release")?;

    let annad_asset = latest_release
        .find_asset("annad")
        .context("annad binary not found in release")?;

    let checksum_asset = latest_release
        .find_asset("SHA256SUMS")
        .context("SHA256SUMS not found in release")?;

    // Download files
    let annactl_path = temp_dir.join("annactl");
    let annad_path = temp_dir.join("annad");
    let checksum_path = temp_dir.join("SHA256SUMS");

    client.download_asset(annactl_asset, &annactl_path.to_string_lossy()).await?;
    client.download_asset(annad_asset, &annad_path.to_string_lossy()).await?;
    client.download_asset(checksum_asset, &checksum_path.to_string_lossy()).await?;

    println!("✅ Downloaded binaries");

    // Step 7: Verify checksums
    println!("🔐 Verifying checksums...");
    verify_checksums(&temp_dir)?;
    println!("✅ Checksums verified");

    // Step 8: Backup current binaries
    println!("💾 Backing up current version...");
    backup_current_binaries()?;
    println!("✅ Backup complete");

    // Step 9: Stop daemon before replacing binaries
    println!("⏸️  Stopping daemon...");
    stop_daemon()?;
    println!("✅ Daemon stopped");

    // Step 10: Install new binaries
    println!("📦 Installing v{}...", latest_version);
    install_binaries(&annactl_path, &annad_path)?;
    println!("✅ Installation complete");

    // Step 11: Start daemon with new binaries
    println!("🔄 Starting daemon...");
    restart_daemon()?;
    println!("✅ Daemon restarted");

    println!();
    println!("🎉 Successfully upgraded to v{}", latest_version);
    println!();
    println!("Verify: annactl --version");

    Ok(())
}

/// Print AUR refusal message
fn print_aur_refusal_message(source: &InstallationSource) {
    println!();
    println!("⚠️  Auto-update disabled for AUR installations");
    println!();
    println!("Your Anna installation is managed by pacman/AUR.");
    println!("Auto-updates would conflict with your package manager.");
    println!();
    println!("To update Anna, use your package manager:");
    println!("  {}", source.update_command());
    println!();
    println!("Why? AUR packages:");
    println!("  • Track dependencies");
    println!("  • Maintain file ownership");
    println!("  • Integrate with system upgrades");
    println!("  • Provide rollback via pacman");
    println!();
}

/// Verify SHA256 checksums
fn verify_checksums(temp_dir: &Path) -> Result<()> {
    let checksum_file = temp_dir.join("SHA256SUMS");
    let checksums = std::fs::read_to_string(&checksum_file)?;

    for line in checksums.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let expected_hash = parts[0];
        let filename = parts[1];

        let file_path = temp_dir.join(filename);
        if !file_path.exists() {
            continue;
        }

        let actual_hash = calculate_sha256(&file_path)?;

        if actual_hash != expected_hash {
            anyhow::bail!(
                "Checksum mismatch for {}: expected {}, got {}",
                filename,
                expected_hash,
                actual_hash
            );
        }
    }

    Ok(())
}

/// Calculate SHA256 hash of file
fn calculate_sha256(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Backup current binaries
fn backup_current_binaries() -> Result<()> {
    let backup_dir = PathBuf::from(BACKUP_DIR);
    std::fs::create_dir_all(&backup_dir)?;

    let current_exe = std::env::current_exe()?;
    let current_version = env!("CARGO_PKG_VERSION");

    // Determine binary paths
    let annactl_path = if current_exe.file_name().unwrap() == "annactl" {
        current_exe.clone()
    } else {
        current_exe.parent().unwrap().join("annactl")
    };

    let annad_path = current_exe.parent().unwrap().join("annad");

    // Backup with version suffix
    if annactl_path.exists() {
        let backup_path = backup_dir.join(format!("annactl-v{}", current_version));
        std::fs::copy(&annactl_path, &backup_path)?;
    }

    if annad_path.exists() {
        let backup_path = backup_dir.join(format!("annad-v{}", current_version));
        std::fs::copy(&annad_path, &backup_path)?;
    }

    Ok(())
}

/// Install new binaries
fn install_binaries(annactl_src: &Path, annad_src: &Path) -> Result<()> {
    let current_exe = std::env::current_exe()?;
    let install_dir = current_exe.parent().unwrap();

    let annactl_dest = install_dir.join("annactl");
    let annad_dest = install_dir.join("annad");

    // Set executable permissions on source files
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(annactl_src, perms.clone())?;
        std::fs::set_permissions(annad_src, perms)?;
    }

    // Use atomic rename strategy to avoid "Text file busy" error
    // Step 1: Copy to temporary files
    let annactl_tmp = install_dir.join(".annactl.new");
    let annad_tmp = install_dir.join(".annad.new");

    std::fs::copy(annactl_src, &annactl_tmp)?;
    std::fs::copy(annad_src, &annad_tmp)?;

    // Step 2: Atomically rename temporary files to final destination
    // This works even if the current binary is running
    std::fs::rename(&annactl_tmp, &annactl_dest)?;
    std::fs::rename(&annad_tmp, &annad_dest)?;

    Ok(())
}

/// Stop daemon before upgrade
fn stop_daemon() -> Result<()> {
    let output = Command::new("systemctl")
        .args(["stop", "annad"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Daemon might not be running, that's ok
        if !stderr.contains("not loaded") && !stderr.contains("not found") {
            anyhow::bail!("Failed to stop daemon: {}", stderr);
        }
    }

    // Wait for daemon to fully stop
    std::thread::sleep(std::time::Duration::from_secs(1));

    Ok(())
}

/// Restart daemon
fn restart_daemon() -> Result<()> {
    let output = Command::new("systemctl")
        .args(["start", "annad"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to start daemon: {}", stderr);
    }

    // Wait for daemon to start
    std::thread::sleep(std::time::Duration::from_secs(2));

    Ok(())
}

/// Rollback to previous version
pub fn rollback_upgrade() -> Result<()> {
    println!("🔄 Rolling back to previous version...");

    let backup_dir = PathBuf::from(BACKUP_DIR);
    if !backup_dir.exists() {
        anyhow::bail!("No backup found in {}", BACKUP_DIR);
    }

    // Find most recent backup
    let mut backups: Vec<PathBuf> = std::fs::read_dir(&backup_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect();

    backups.sort_by(|a, b| {
        let a_meta = a.metadata().unwrap();
        let b_meta = b.metadata().unwrap();
        b_meta.modified().unwrap().cmp(&a_meta.modified().unwrap())
    });

    if backups.is_empty() {
        anyhow::bail!("No backup files found");
    }

    let current_exe = std::env::current_exe()?;
    let install_dir = current_exe.parent().unwrap();

    // Restore from backup
    for backup_file in &backups {
        let filename = backup_file.file_name().unwrap().to_string_lossy();

        if filename.starts_with("annactl-") {
            let dest = install_dir.join("annactl");
            std::fs::copy(backup_file, dest)?;
            println!("✅ Restored annactl");
        } else if filename.starts_with("annad-") {
            let dest = install_dir.join("annad");
            std::fs::copy(backup_file, dest)?;
            println!("✅ Restored annad");
        }
    }

    restart_daemon()?;

    println!("✅ Rollback complete");

    Ok(())
}
