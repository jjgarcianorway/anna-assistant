//! Auto-update system for Anna Assistant
//!
//! Checks for new releases on GitHub and can automatically update binaries

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::path::Path;
use tracing::{info, warn, error};

const REPO: &str = "jjgarcianorway/anna-assistant";
const INSTALL_DIR: &str = "/usr/local/bin";
const BACKUP_DIR: &str = "/var/lib/anna/backup";

/// Information about an available update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub is_update_available: bool,
    pub download_url_annad: String,
    pub download_url_annactl: String,
    pub release_notes_url: String,
    pub published_at: String,
}

/// Get current Anna version from binary
pub fn get_current_version() -> Result<String> {
    let output = Command::new("/usr/local/bin/annad")
        .arg("--version")
        .output()
        .context("Failed to get current version")?;

    let version_output = String::from_utf8_lossy(&output.stdout);

    // Extract version number (format: "annad v1.0.0-beta.41")
    if let Some(version) = version_output.split_whitespace().nth(1) {
        Ok(version.to_string())
    } else {
        anyhow::bail!("Could not parse version from: {}", version_output);
    }
}

/// Check GitHub for the latest release
pub async fn check_for_updates() -> Result<UpdateInfo> {
    info!("Checking for updates from GitHub");

    let current_version = get_current_version()?;
    info!("Current version: {}", current_version);

    // Fetch latest release (including prereleases)
    let client = reqwest::Client::builder()
        .user_agent("anna-assistant")
        .build()?;

    let url = format!("https://api.github.com/repos/{}/releases", REPO);
    let response = client.get(&url)
        .send()
        .await
        .context("Failed to fetch releases from GitHub")?;

    let releases: Vec<serde_json::Value> = response.json().await
        .context("Failed to parse GitHub API response")?;

    if releases.is_empty() {
        anyhow::bail!("No releases found on GitHub");
    }

    // Get first release (most recent)
    let latest = &releases[0];
    let latest_version = latest["tag_name"].as_str()
        .context("No tag_name in release")?
        .to_string();

    info!("Latest version: {}", latest_version);

    // Find asset URLs
    let assets = latest["assets"].as_array()
        .context("No assets in release")?;

    let annad_asset = assets.iter()
        .find(|a| a["name"].as_str() == Some("annad"))
        .context("annad binary not found in release")?;

    let annactl_asset = assets.iter()
        .find(|a| a["name"].as_str() == Some("annactl"))
        .context("annactl binary not found in release")?;

    let download_url_annad = annad_asset["browser_download_url"].as_str()
        .context("No download URL for annad")?
        .to_string();

    let download_url_annactl = annactl_asset["browser_download_url"].as_str()
        .context("No download URL for annactl")?
        .to_string();

    let release_notes_url = latest["html_url"].as_str()
        .context("No release notes URL")?
        .to_string();

    let published_at = latest["published_at"].as_str()
        .unwrap_or("unknown")
        .to_string();

    let is_update_available = latest_version != current_version;

    Ok(UpdateInfo {
        current_version,
        latest_version,
        is_update_available,
        download_url_annad,
        download_url_annactl,
        release_notes_url,
        published_at,
    })
}

/// Download and verify a binary
async fn download_binary(url: &str, dest_path: &Path) -> Result<()> {
    info!("Downloading from {} to {}", url, dest_path.display());

    let client = reqwest::Client::builder()
        .user_agent("anna-assistant")
        .build()?;

    let response = client.get(url)
        .send()
        .await
        .context("Failed to download binary")?;

    let bytes = response.bytes().await
        .context("Failed to read binary data")?;

    std::fs::write(dest_path, bytes)
        .context("Failed to write binary to disk")?;

    // Make executable
    let mut perms = std::fs::metadata(dest_path)?.permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o755);
    }
    std::fs::set_permissions(dest_path, perms)?;

    info!("Downloaded and made executable: {}", dest_path.display());
    Ok(())
}

/// Verify a downloaded binary by checking its version
fn verify_binary(binary_path: &Path, expected_version: &str) -> Result<()> {
    info!("Verifying binary: {}", binary_path.display());

    let output = Command::new(binary_path)
        .arg("--version")
        .output()
        .context("Failed to run binary")?;

    if !output.status.success() {
        anyhow::bail!("Binary failed to execute");
    }

    let version_output = String::from_utf8_lossy(&output.stdout);

    // Strip 'v' prefix from expected version for comparison
    // Expected: "v1.0.0-beta.58", Binary outputs: "annad 1.0.0-beta.58"
    let version_without_v = expected_version.strip_prefix('v').unwrap_or(expected_version);

    if !version_output.contains(version_without_v) {
        anyhow::bail!(
            "Version mismatch: expected {}, got {}",
            expected_version,
            version_output.trim()
        );
    }

    info!("Binary verified successfully");
    Ok(())
}

/// Backup current binaries
fn backup_current_binaries() -> Result<()> {
    info!("Backing up current binaries");

    // Create backup directory with sudo
    let status = Command::new("sudo")
        .args(&["mkdir", "-p", BACKUP_DIR])
        .status()
        .context("Failed to execute mkdir")?;

    if !status.success() {
        anyhow::bail!("Failed to create backup directory with sudo");
    }

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");

    let annad_src = Path::new(INSTALL_DIR).join("annad");
    let annactl_src = Path::new(INSTALL_DIR).join("annactl");

    let annad_backup = Path::new(BACKUP_DIR).join(format!("annad.{}", timestamp));
    let annactl_backup = Path::new(BACKUP_DIR).join(format!("annactl.{}", timestamp));

    // Copy with sudo to backup directory
    let status_annad = Command::new("sudo")
        .args(&["cp", annad_src.to_str().unwrap(), annad_backup.to_str().unwrap()])
        .status()
        .context("Failed to backup annad")?;

    if !status_annad.success() {
        anyhow::bail!("Failed to backup annad");
    }

    let status_annactl = Command::new("sudo")
        .args(&["cp", annactl_src.to_str().unwrap(), annactl_backup.to_str().unwrap()])
        .status()
        .context("Failed to backup annactl")?;

    if !status_annactl.success() {
        anyhow::bail!("Failed to backup annactl");
    }

    info!("Binaries backed up to {}", BACKUP_DIR);
    Ok(())
}

/// Perform the update
pub async fn perform_update(update_info: &UpdateInfo) -> Result<()> {
    info!("Performing update from {} to {}",
          update_info.current_version, update_info.latest_version);

    // Create temp directory for download
    let temp_dir = std::env::temp_dir().join("anna-update");
    std::fs::create_dir_all(&temp_dir)
        .context("Failed to create temp directory")?;

    let temp_annad = temp_dir.join("annad");
    let temp_annactl = temp_dir.join("annactl");

    // Download binaries
    info!("Downloading new binaries...");
    download_binary(&update_info.download_url_annad, &temp_annad).await?;
    download_binary(&update_info.download_url_annactl, &temp_annactl).await?;

    // Verify downloads
    info!("Verifying downloads...");
    verify_binary(&temp_annad, &update_info.latest_version)?;
    verify_binary(&temp_annactl, &update_info.latest_version)?;

    // Backup current binaries
    backup_current_binaries()?;

    // Stop daemon (will be restarted by systemd)
    info!("Stopping daemon for update...");
    let _ = Command::new("sudo")
        .args(&["systemctl", "stop", "annad"])
        .output();

    // Replace binaries (use sudo cp to avoid permission issues)
    info!("Installing new binaries...");
    let annad_dest = Path::new(INSTALL_DIR).join("annad");
    let annactl_dest = Path::new(INSTALL_DIR).join("annactl");

    let status_annad = Command::new("sudo")
        .args(&["cp", temp_annad.to_str().unwrap(), annad_dest.to_str().unwrap()])
        .status()
        .context("Failed to install annad")?;

    if !status_annad.success() {
        anyhow::bail!("Failed to install annad binary");
    }

    let status_annactl = Command::new("sudo")
        .args(&["cp", temp_annactl.to_str().unwrap(), annactl_dest.to_str().unwrap()])
        .status()
        .context("Failed to install annactl")?;

    if !status_annactl.success() {
        anyhow::bail!("Failed to install annactl binary");
    }

    // Restart daemon
    info!("Restarting daemon...");
    let _ = Command::new("sudo")
        .args(&["systemctl", "start", "annad"])
        .output();

    // Clean up temp files
    let _ = std::fs::remove_dir_all(&temp_dir);

    info!("Update complete!");
    Ok(())
}

/// Check for updates and return info (non-blocking)
pub async fn check_updates_background() -> Option<UpdateInfo> {
    match check_for_updates().await {
        Ok(info) => {
            if info.is_update_available {
                info!("Update available: {} -> {}", info.current_version, info.latest_version);
                Some(info)
            } else {
                info!("Already on latest version: {}", info.current_version);
                None
            }
        }
        Err(e) => {
            warn!("Failed to check for updates: {}", e);
            None
        }
    }
}

/// Perform update with error handling
pub async fn update_if_available() -> Result<bool> {
    match check_for_updates().await {
        Ok(info) => {
            if info.is_update_available {
                info!("Update available, performing update...");
                perform_update(&info).await?;
                Ok(true)
            } else {
                info!("Already on latest version");
                Ok(false)
            }
        }
        Err(e) => {
            error!("Update check failed: {}", e);
            Err(e)
        }
    }
}
