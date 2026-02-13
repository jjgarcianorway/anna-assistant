//! Update management - check for and apply updates.

use anna_shared::GITHUB_REPO;
use anyhow::{anyhow, Result};
use tracing::info;

pub use crate::update_ops::{
    download_file, get_arch_name, install_binary_pair, patch_service_unit_path,
    rollback_binaries, schedule_daemon_restart, verify_assets_exist, verify_binary_version,
    verify_checksum, verify_pair_consistency,
};

/// GitHub API response for releases
#[derive(Debug, serde::Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

/// Check GitHub for the latest version
pub async fn check_latest_version() -> Result<String> {
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        GITHUB_REPO
    );

    let client = reqwest::Client::builder()
        .user_agent("anna-assistant")
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow!("GitHub API error: {}", response.status()));
    }

    let release: GitHubRelease = response.json().await?;

    // Remove 'v' prefix if present
    let version = release.tag_name.trim_start_matches('v').to_string();

    // Verify that required assets are actually downloadable
    verify_assets_exist(&client, &version).await?;

    Ok(version)
}

/// Compare versions, returns true if remote is newer
pub fn is_newer_version(current: &str, remote: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> { v.split('.').filter_map(|s| s.parse().ok()).collect() };

    let current_parts = parse(current);
    let remote_parts = parse(remote);

    if current_parts.is_empty() || remote_parts.is_empty() {
        return false;
    }

    for i in 0..3 {
        let c = current_parts.get(i).unwrap_or(&0);
        let r = remote_parts.get(i).unwrap_or(&0);
        if r > c {
            return true;
        }
        if r < c {
            return false;
        }
    }
    false
}

/// Perform atomic pair update of both annactl and annad
pub async fn perform_update(new_version: &str) -> Result<()> {
    info!("Starting atomic pair update to version {}", new_version);

    let arch_name = get_arch_name()?;

    let base_url = format!(
        "https://github.com/{}/releases/download/v{}",
        GITHUB_REPO, new_version
    );

    let tmp_dir = std::env::temp_dir().join("anna-update");
    std::fs::create_dir_all(&tmp_dir)?;

    // Download both binaries before replacing anything
    info!("Downloading annactl...");
    let annactl_url = format!("{}/annactl-linux-{}", base_url, arch_name);
    let annactl_path = tmp_dir.join("annactl");
    download_file(&annactl_url, &annactl_path).await?;

    info!("Downloading annad...");
    let annad_url = format!("{}/annad-linux-{}", base_url, arch_name);
    let annad_path = tmp_dir.join("annad");
    download_file(&annad_url, &annad_path).await?;

    // Download and verify checksums
    info!("Verifying checksums...");
    let sums_url = format!("{}/SHA256SUMS", base_url);
    let sums_path = tmp_dir.join("SHA256SUMS");
    download_file(&sums_url, &sums_path).await?;

    verify_checksum(
        &annactl_path,
        &sums_path,
        &format!("annactl-linux-{}", arch_name),
    )?;
    verify_checksum(
        &annad_path,
        &sums_path,
        &format!("annad-linux-{}", arch_name),
    )?;

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&annactl_path, std::fs::Permissions::from_mode(0o755))?;
        std::fs::set_permissions(&annad_path, std::fs::Permissions::from_mode(0o755))?;
    }

    // Verify downloaded binaries report correct version
    info!("Verifying downloaded binary versions...");
    verify_binary_version(&annactl_path, new_version, "annactl")?;
    verify_binary_version(&annad_path, new_version, "annad")?;

    // Backup existing binaries for rollback
    info!("Backing up existing binaries...");
    let backup_annactl = tmp_dir.join("annactl.backup");
    let backup_annad = tmp_dir.join("annad.backup");
    std::fs::copy("/usr/local/bin/annactl", &backup_annactl).ok();
    std::fs::copy("/usr/local/bin/annad", &backup_annad).ok();

    // Atomic pair update - both or neither
    info!("Installing new binaries as atomic pair...");
    if let Err(e) = install_binary_pair(&annactl_path, &annad_path) {
        tracing::warn!("Update failed, rolling back: {}", e);
        rollback_binaries(&backup_annactl, &backup_annad);
        std::fs::remove_dir_all(&tmp_dir).ok();
        return Err(e);
    }

    // Verify installed versions match
    info!("Verifying pair consistency...");
    if let Err(e) = verify_pair_consistency(new_version) {
        tracing::warn!("Pair consistency check failed, rolling back: {}", e);
        rollback_binaries(&backup_annactl, &backup_annad);
        std::fs::remove_dir_all(&tmp_dir).ok();
        return Err(e);
    }

    // Patch service unit PATH if missing (auto-update doesn't reinstall the service file)
    patch_service_unit_path();

    // Schedule daemon restart
    info!("Scheduling daemon restart...");
    schedule_daemon_restart()?;

    // Cleanup
    std::fs::remove_dir_all(&tmp_dir).ok();

    info!(
        "Atomic pair update to {} complete, daemon will restart",
        new_version
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("0.0.1", "0.0.2"));
        assert!(is_newer_version("0.0.9", "0.1.0"));
        assert!(!is_newer_version("0.0.2", "0.0.1"));
        assert!(!is_newer_version("0.0.1", "0.0.1"));
    }
}
