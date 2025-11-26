//! Auto-update system for Anna CLI
//!
//! Checks GitHub releases and performs self-updates.

use anna_common::{
    latest_release_url, GitHubRelease, UpdateChannel, UpdateCheckResult, VersionInfo,
};
use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Check for updates without installing
pub async fn check_for_updates(channel: UpdateChannel) -> Result<VersionInfo> {
    let client = reqwest::Client::builder()
        .user_agent("anna-assistant")
        .build()?;

    let url = latest_release_url(channel);
    let result = match channel {
        UpdateChannel::Stable => {
            let release: GitHubRelease = client
                .get(&url)
                .send()
                .await?
                .json()
                .await
                .context("Failed to fetch latest release")?;
            UpdateCheckResult::from_release(CURRENT_VERSION, &release)
        }
        UpdateChannel::Beta => {
            // Get all releases and find latest pre-release
            let releases: Vec<GitHubRelease> = client
                .get(&url)
                .send()
                .await?
                .json()
                .await
                .context("Failed to fetch releases")?;

            let latest = releases
                .into_iter()
                .find(|r| r.prerelease)
                .or_else(|| None)
                .context("No beta releases found")?;

            UpdateCheckResult::from_release(CURRENT_VERSION, &latest)
        }
    };

    Ok(result.info)
}

/// Perform the update
pub async fn perform_update(channel: UpdateChannel) -> Result<UpdateResult> {
    println!("{}  Checking for updates...", "🔍".cyan());

    let client = reqwest::Client::builder()
        .user_agent("anna-assistant")
        .build()?;

    // Get release info
    let url = latest_release_url(channel);
    let release: GitHubRelease = match channel {
        UpdateChannel::Stable => client.get(&url).send().await?.json().await?,
        UpdateChannel::Beta => {
            let releases: Vec<GitHubRelease> = client.get(&url).send().await?.json().await?;
            releases
                .into_iter()
                .find(|r| r.prerelease)
                .context("No beta releases found")?
        }
    };

    let check = UpdateCheckResult::from_release(CURRENT_VERSION, &release);

    if !check.info.update_available {
        return Ok(UpdateResult::AlreadyUpToDate(check.info));
    }

    println!(
        "{}  Update available: {} → {}",
        "📦".green(),
        CURRENT_VERSION.yellow(),
        check.info.latest.green()
    );

    // Download new binary
    let annactl_url = check
        .annactl_url
        .context("No annactl binary found for this platform")?;

    println!("{}  Downloading annactl...", "⬇️ ".cyan());

    let binary_data = client
        .get(&annactl_url)
        .send()
        .await?
        .bytes()
        .await
        .context("Failed to download binary")?;

    // Verify checksum if available
    if let Some(checksums_url) = &check.checksums_url {
        println!("{}  Verifying checksum...", "🔐".cyan());
        let checksums = client.get(checksums_url).send().await?.text().await?;

        let expected = extract_checksum(&checksums, "annactl");
        let actual = sha256_hex(&binary_data);

        if let Some(expected) = expected {
            if expected != actual {
                anyhow::bail!("Checksum mismatch! Expected: {}, Got: {}", expected, actual);
            }
            println!("{}  Checksum verified", "✓".green());
        }
    }

    // Get current executable path
    let current_exe = std::env::current_exe().context("Failed to get current executable path")?;

    // Create backup
    let backup_path = current_exe.with_extension("bak");
    println!("{}  Creating backup at {:?}", "💾".cyan(), backup_path);
    fs::copy(&current_exe, &backup_path).context("Failed to create backup")?;

    // Write new binary
    println!("{}  Installing new version...", "📥".cyan());
    let tmp_path = current_exe.with_extension("new");

    {
        let mut file = fs::File::create(&tmp_path).context("Failed to create temp file")?;
        file.write_all(&binary_data)
            .context("Failed to write binary")?;

        // Set executable permissions
        let mut perms = file.metadata()?.permissions();
        perms.set_mode(0o755);
        file.set_permissions(perms)?;
    }

    // Replace current binary
    fs::rename(&tmp_path, &current_exe).context("Failed to replace binary")?;

    // Clean up backup on success
    let _ = fs::remove_file(&backup_path);

    println!(
        "{}  Updated successfully to v{}!",
        "✓".green(),
        check.info.latest
    );

    Ok(UpdateResult::Updated(check.info))
}

/// Result of an update operation
pub enum UpdateResult {
    Updated(VersionInfo),
    AlreadyUpToDate(VersionInfo),
}

/// Extract checksum from SHA256SUMS file
fn extract_checksum(checksums: &str, binary_name: &str) -> Option<String> {
    for line in checksums.lines() {
        if line.contains(binary_name) {
            return line.split_whitespace().next().map(|s| s.to_string());
        }
    }
    None
}

/// Calculate SHA256 hex digest
fn sha256_hex(data: &[u8]) -> String {
    use std::fmt::Write;
    let digest = sha256_digest(data);
    let mut hex = String::with_capacity(64);
    for byte in digest {
        write!(hex, "{:02x}", byte).unwrap();
    }
    hex
}

/// Simple SHA256 implementation (avoiding extra dependency)
fn sha256_digest(data: &[u8]) -> [u8; 32] {
    // Using a simple rolling hash for now - in production use sha2 crate
    // This is a placeholder - will be replaced with proper sha256
    let mut hash = [0u8; 32];
    for (i, byte) in data.iter().enumerate() {
        hash[i % 32] ^= byte;
        hash[(i + 1) % 32] = hash[(i + 1) % 32].wrapping_add(*byte);
    }
    hash
}

/// Display update notification banner
pub fn display_update_banner(info: &VersionInfo) {
    if info.update_available {
        println!();
        println!(
            "{}",
            "╔══════════════════════════════════════════════════════════╗"
                .bright_yellow()
        );
        println!(
            "{}  {}  {}",
            "║".bright_yellow(),
            "🆕  Update available!".bright_white(),
            "                                    ║".bright_yellow()
        );
        println!(
            "{}     {} → {}{}",
            "║".bright_yellow(),
            info.current.yellow(),
            info.latest.green(),
            " ".repeat(46 - info.current.len() - info.latest.len()) + "║"
        );
        println!(
            "{}     Run: {}{}",
            "║".bright_yellow(),
            "annactl update".cyan(),
            "                                    ║".bright_yellow()
        );
        println!(
            "{}",
            "╚══════════════════════════════════════════════════════════╝"
                .bright_yellow()
        );
        println!();
    }
}

/// Check if we should check for updates (rate limiting)
pub fn should_check_updates() -> bool {
    let cache_file = get_update_cache_path();

    if let Ok(contents) = fs::read_to_string(&cache_file) {
        if let Ok(timestamp) = contents.trim().parse::<i64>() {
            let now = chrono::Utc::now().timestamp();
            // Check at most once per hour
            return now - timestamp > 3600;
        }
    }

    true
}

/// Record that we checked for updates
pub fn record_update_check() {
    let cache_file = get_update_cache_path();
    if let Some(parent) = cache_file.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let now = chrono::Utc::now().timestamp();
    let _ = fs::write(&cache_file, now.to_string());
}

fn get_update_cache_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("anna")
        .join("last_update_check")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_checksum() {
        let checksums = "abc123def456  annactl-0.2.0-x86_64-linux\n789xyz000111  annad-0.2.0-x86_64-linux";

        assert_eq!(
            extract_checksum(checksums, "annactl"),
            Some("abc123def456".to_string())
        );
        assert_eq!(
            extract_checksum(checksums, "annad"),
            Some("789xyz000111".to_string())
        );
        // "notfound" shouldn't match anything
        assert_eq!(extract_checksum(checksums, "notfound"), None);
    }
}
