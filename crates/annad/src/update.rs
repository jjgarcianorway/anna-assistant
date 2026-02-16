//! Update management - check for and apply updates.

use anna_shared::GITHUB_REPO;
use anyhow::{anyhow, Result};
use tracing::{info, warn};

/// GPG public key for verifying release signatures.
///
/// Empty string = signature verification disabled (pre-key state).
/// Replace with the ASCII-armored public key once generated:
///   gpg --gen-key
///   gpg --armor --export <fingerprint>
/// Then set GPG_PRIVATE_KEY + GPG_PASSPHRASE in GitHub Actions secrets.
const ANNA_GPG_PUBLIC_KEY: &str = "";

/// Verify a detached GPG signature over `data`.
///
/// Returns Ok if:
/// - `ANNA_GPG_PUBLIC_KEY` is empty (verification disabled — logs a warning)
/// - The signature is cryptographically valid
///
/// Returns Err if the key is present but the signature is invalid.
fn verify_gpg_signature(data: &[u8], sig_armored: &[u8]) -> Result<()> {
    if ANNA_GPG_PUBLIC_KEY.is_empty() {
        warn!("GPG signature verification disabled: no public key embedded. Skipping.");
        return Ok(());
    }

    use pgp::composed::{Deserializable, DetachedSignature, SignedPublicKey};
    use std::io::Cursor;

    let (pubkey, _) = SignedPublicKey::from_armor_single(Cursor::new(ANNA_GPG_PUBLIC_KEY.as_bytes()))
        .map_err(|e| anyhow!("Failed to parse embedded GPG public key: {}", e))?;

    let (sig, _) = DetachedSignature::from_armor_single(Cursor::new(sig_armored))
        .map_err(|e| anyhow!("Failed to parse SHA256SUMS.asc: {}", e))?;

    sig.verify(&pubkey, data)
        .map_err(|e| anyhow!("GPG signature verification FAILED — release may be compromised: {}", e))?;

    info!("GPG signature verified OK");
    Ok(())
}

pub use crate::update_ops::{
    download_file, get_arch_name, get_bin_dir, install_binary_pair, patch_service_unit_path,
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

    // Download all binaries before replacing anything
    info!("Downloading annactl...");
    let annactl_url = format!("{}/annactl-linux-{}", base_url, arch_name);
    let annactl_path = tmp_dir.join("annactl");
    download_file(&annactl_url, &annactl_path).await?;

    info!("Downloading annad...");
    let annad_url = format!("{}/annad-linux-{}", base_url, arch_name);
    let annad_path = tmp_dir.join("annad");
    download_file(&annad_url, &annad_path).await?;

    info!("Downloading anna-executor...");
    let executor_url = format!("{}/anna-executor-linux-{}", base_url, arch_name);
    let executor_path = tmp_dir.join("anna-executor");
    download_file(&executor_url, &executor_path).await?;

    // Download SHA256SUMS and its GPG signature
    info!("Downloading checksums...");
    let sums_url = format!("{}/SHA256SUMS", base_url);
    let sums_path = tmp_dir.join("SHA256SUMS");
    download_file(&sums_url, &sums_path).await?;

    let sig_url = format!("{}/SHA256SUMS.asc", base_url);
    let sig_path = tmp_dir.join("SHA256SUMS.asc");
    download_file(&sig_url, &sig_path).await?;

    // Verify GPG signature over SHA256SUMS (fails closed if key present)
    info!("Verifying GPG signature...");
    let sums_bytes = std::fs::read(&sums_path)?;
    let sig_bytes = std::fs::read(&sig_path)?;
    verify_gpg_signature(&sums_bytes, &sig_bytes)?;

    // Verify per-binary checksums against signed SHA256SUMS
    info!("Verifying checksums...");
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
    verify_checksum(
        &executor_path,
        &sums_path,
        &format!("anna-executor-linux-{}", arch_name),
    )?;

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&annactl_path, std::fs::Permissions::from_mode(0o755))?;
        std::fs::set_permissions(&annad_path, std::fs::Permissions::from_mode(0o755))?;
        std::fs::set_permissions(&executor_path, std::fs::Permissions::from_mode(0o755))?;
    }

    // Verify downloaded binaries report correct version
    info!("Verifying downloaded binary versions...");
    verify_binary_version(&annactl_path, new_version, "annactl")?;
    verify_binary_version(&annad_path, new_version, "annad")?;
    verify_binary_version(&executor_path, new_version, "anna-executor")?;

    // Backup existing binaries for rollback
    info!("Backing up existing binaries...");
    let backup_annactl = tmp_dir.join("annactl.backup");
    let backup_annad = tmp_dir.join("annad.backup");
    let backup_executor = tmp_dir.join("anna-executor.backup");
    if let Ok(bin_dir) = get_bin_dir() {
        std::fs::copy(bin_dir.join("annactl"), &backup_annactl).ok();
        std::fs::copy(bin_dir.join("annad"), &backup_annad).ok();
        std::fs::copy(bin_dir.join("anna-executor"), &backup_executor).ok();
    }

    // Atomic triple update — all or none
    info!("Installing new binaries...");
    if let Err(e) = install_binary_pair(&annactl_path, &annad_path, &executor_path) {
        tracing::warn!("Update failed, rolling back: {}", e);
        rollback_binaries(&backup_annactl, &backup_annad, &backup_executor);
        std::fs::remove_dir_all(&tmp_dir).ok();
        return Err(e);
    }

    // Verify installed versions match
    info!("Verifying pair consistency...");
    if let Err(e) = verify_pair_consistency(new_version) {
        tracing::warn!("Pair consistency check failed, rolling back: {}", e);
        rollback_binaries(&backup_annactl, &backup_annad, &backup_executor);
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
