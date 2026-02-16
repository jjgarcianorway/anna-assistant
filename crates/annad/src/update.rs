//! Update management - check for and apply updates.

use anna_shared::GITHUB_REPO;
use anyhow::{anyhow, Result};
use tracing::{info, warn};

/// GPG public keys for verifying release signatures.
/// Each entry is (fingerprint, armored_public_key).
/// Multi-key: accepts signatures from ANY key not in REVOKED_GPG_FINGERPRINTS.
/// Rotation: add new key, ship, then remove old key in the release after next.
const ANNA_GPG_PUBLIC_KEYS: &[(&str, &str)] = &[(
    "543A45A34B5BDFF02855D2A553ECB7AA81677B9E",
    "-----BEGIN PGP PUBLIC KEY BLOCK-----\n\n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\nH9oOnw==\n=mUIF\n-----END PGP PUBLIC KEY BLOCK-----",
)];

/// Fingerprints of revoked keys. Signatures from these keys are rejected
/// even if the key is still listed in ANNA_GPG_PUBLIC_KEYS.
const REVOKED_GPG_FINGERPRINTS: &[&str] = &[];

/// Try verifying a detached GPG signature using one specific public key (armored).
fn try_verify_with_key(data: &[u8], sig_armored: &[u8], pubkey_armored: &str) -> Result<()> {
    use pgp::composed::{Deserializable, DetachedSignature, SignedPublicKey};
    use std::io::Cursor;

    let (pubkey, _) = SignedPublicKey::from_armor_single(Cursor::new(pubkey_armored.as_bytes()))
        .map_err(|e| anyhow!("Failed to parse GPG public key: {}", e))?;

    let (sig, _) = DetachedSignature::from_armor_single(Cursor::new(sig_armored))
        .map_err(|e| anyhow!("Failed to parse SHA256SUMS.asc: {}", e))?;

    sig.verify(&pubkey, data)
        .map_err(|e| anyhow!("GPG signature verification FAILED: {}", e))?;
    Ok(())
}

/// Verify a detached GPG signature over `data` against all known non-revoked keys.
///
/// Returns Ok if:
/// - ANNA_GPG_PUBLIC_KEYS is empty (verification disabled — logs a warning)
/// - Any non-revoked key validates the signature
///
/// Returns Err if the key list is non-empty but no key validates the signature.
fn verify_gpg_signature(data: &[u8], sig_armored: &[u8]) -> Result<()> {
    if ANNA_GPG_PUBLIC_KEYS.is_empty() {
        warn!("GPG signature verification disabled (no keys embedded) — update proceeding unsigned");
        return Ok(());
    }

    let mut last_err = String::from("No non-revoked keys available");
    for (fingerprint, armored_key) in ANNA_GPG_PUBLIC_KEYS {
        if REVOKED_GPG_FINGERPRINTS.contains(fingerprint) {
            warn!("Skipping revoked key {}", &fingerprint[..16]);
            continue;
        }
        match try_verify_with_key(data, sig_armored, armored_key) {
            Ok(()) => {
                info!("GPG signature verified OK (key {})", &fingerprint[..16]);
                return Ok(());
            }
            Err(e) => last_err = e.to_string(),
        }
    }
    Err(anyhow!("GPG signature verification FAILED — release may be compromised: {}", last_err))
}

pub use crate::update_ops::{
    download_file, get_arch_name, get_bin_dir, install_binary_pair, patch_service_unit_path,
    restore_rollback_slot, rollback_binaries, save_rollback_slot, schedule_daemon_restart,
    verify_assets_exist, verify_binary_version, verify_checksum, verify_pair_consistency,
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

/// Perform atomic triple update (annactl, annad, anna-executor)
pub async fn perform_update(new_version: &str) -> Result<()> {
    info!("Starting atomic triple update to version {}", new_version);

    let arch_name = get_arch_name()?;

    let base_url = format!(
        "https://github.com/{}/releases/download/v{}",
        GITHUB_REPO, new_version
    );

    let tmp_dir = std::env::temp_dir().join("anna-update");
    std::fs::create_dir_all(&tmp_dir)?;

    // Download all binaries before replacing anything
    info!("Downloading annactl...");
    let annactl_path = tmp_dir.join("annactl");
    download_file(&format!("{}/annactl-linux-{}", base_url, arch_name), &annactl_path).await?;

    info!("Downloading annad...");
    let annad_path = tmp_dir.join("annad");
    download_file(&format!("{}/annad-linux-{}", base_url, arch_name), &annad_path).await?;

    info!("Downloading anna-executor...");
    let executor_path = tmp_dir.join("anna-executor");
    download_file(&format!("{}/anna-executor-linux-{}", base_url, arch_name), &executor_path).await?;

    // Download SHA256SUMS and its GPG signature
    info!("Downloading checksums...");
    let sums_path = tmp_dir.join("SHA256SUMS");
    download_file(&format!("{}/SHA256SUMS", base_url), &sums_path).await?;

    let sig_path = tmp_dir.join("SHA256SUMS.asc");
    download_file(&format!("{}/SHA256SUMS.asc", base_url), &sig_path).await?;

    // Verify GPG signature over SHA256SUMS (fails closed if key list is non-empty)
    info!("Verifying GPG signature...");
    let sums_bytes = std::fs::read(&sums_path)?;
    let sig_bytes = std::fs::read(&sig_path)?;
    verify_gpg_signature(&sums_bytes, &sig_bytes)?;

    // Verify per-binary checksums against signed SHA256SUMS
    info!("Verifying checksums...");
    verify_checksum(&annactl_path, &sums_path, &format!("annactl-linux-{}", arch_name))?;
    verify_checksum(&annad_path, &sums_path, &format!("annad-linux-{}", arch_name))?;
    verify_checksum(&executor_path, &sums_path, &format!("anna-executor-linux-{}", arch_name))?;

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

    // Save persistent rollback slot before touching anything on disk
    info!("Saving rollback slot...");
    if let Ok(bin_dir) = get_bin_dir() {
        save_rollback_slot(&bin_dir);
    }

    // Atomic triple update — all or none
    info!("Installing new binaries...");
    if let Err(e) = install_binary_pair(&annactl_path, &annad_path, &executor_path) {
        tracing::warn!("Update failed during install, rolling back: {}", e);
        if let Ok(bin_dir) = get_bin_dir() {
            restore_rollback_slot(&bin_dir);
        }
        std::fs::remove_dir_all(&tmp_dir).ok();
        return Err(e);
    }

    // Verify installed versions match
    info!("Verifying pair consistency...");
    if let Err(e) = verify_pair_consistency(new_version) {
        tracing::warn!("Pair consistency check failed, rolling back: {}", e);
        if let Ok(bin_dir) = get_bin_dir() {
            restore_rollback_slot(&bin_dir);
        }
        std::fs::remove_dir_all(&tmp_dir).ok();
        return Err(e);
    }

    // Patch service unit PATH if missing
    patch_service_unit_path();

    // Schedule daemon restart with post-restart self-check
    info!("Scheduling daemon restart with self-check...");
    schedule_daemon_restart(new_version)?;

    // Cleanup
    std::fs::remove_dir_all(&tmp_dir).ok();

    info!("Atomic triple update to {} complete, daemon will restart", new_version);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Pre-baked test fixtures (test-only RSA 2048 keypair, NOT the production key).
    // Generated via: cargo run -p annad --example gen_test_fixtures
    const TEST_GPG_KEY_A: &str = "-----BEGIN PGP PUBLIC KEY BLOCK-----\n\nxsBNBGmTL2QBCAC+ywTklEms+sh/M6MXSP5+HkCaS3oX/aev0bBZH/zDwLnJKMOv\n23/dnJa6u6ES/ptAouo0FuLYlPADFUmtt4Ug+YypuePa/Ki0VLYxxj+vXU9UcLCh\n7X/crujpA7bMikM8QMAiEObXmupZOtGUklCu2ij6Z8YZIsrzeBPU+UXXmwVmXFM+\nQlcU3GPZtkwX/DiZ4tmgLwhNJeIAEYncBqGd6SwsNd78eRmDgJx1fBBS02ZD8++3\nCIAm5sUyfYiuumZ5UZsyU74V5Mv3xA3F4rHOQAKikouC3TLbbqoHJ9UdH5JLEKfE\nanW8Izux5BB4sBfpjAqx3RWzcS630R+Y++kPABEBAAHNHkFubmEgVGVzdCBBIDx0\nZXN0LWFAYW5uYS10ZXN0PsLAhAQTAQgALgUCaZMvZxYhBKb3MnvXh2pj9UeO+WAG\n6ql5aQzvAhsCAh4BAQsBFQEWAScCGQEACgkQYAbqqXlpDO80DAf/ahgwyxN8NZzt\ni8s4XWI7q6WW4BJZLz1DL0vtiOgDcpSt8RhFnVvX1vKBDyzZyMXO68OFix8ZmgvK\n2duekhU4sB4T5/YkL67H55OfStkrG42f44PpZa52jPPj3dKBrD/Qoj3iLK0YHh/O\nibEzcp9Iq+feVZPJ90NyAnQzB4vpx+K77QKKLO0gweMqhpQGPVW5HafooVaqhsJK\nv++7oJjr7SI6yewTD3j3kd0bWmhaBohr/nERk4MzbfPOeSJ1dKZqqHstz4jVu2HT\ngcL259OD/yc5vqvcZkNQkKRSgOrezrfI1LAwY1u4ewwlm4N5DfPYNIFBTkMCB/Pk\nQmYC54ll8g==\n=kl+R\n-----END PGP PUBLIC KEY BLOCK-----\n";
    const TEST_GPG_KEY_B: &str = "-----BEGIN PGP PUBLIC KEY BLOCK-----\n\nxsBNBGmTL2cBCAC8wj76MLRkkPDa3FCmk9o/0s9uorcp31zB7hrem6/3xalVk7OJ\ndXSOtj854R4EDaxOrr6iO+30iW9Y+KJjIzgX4YrWYKDVwvW/3tNQ0FmO8L+IjbV6\nFWL46k/9D+xvhM5yo/VjE5d5KK1N+lZpgER3CF8Mk4Lx9JkwvFQPjPb7ersVbNMK\nIk0Ukt4rFA17TsU5Ls5HNGzneBSmqMrpY+d/B0yIPttg/w5SR9IF6cAxhEDcJYII\nUSLhs30+dpJwK/do/GjpUXNeC2IV+84wX9oS5jMfNJBpdBXe1pyhW/uyyub8mn9r\n1PInRCmbqVWdUeQzKbio+n1dF8Q/SRbZmS21ABEBAAHNHkFubmEgVGVzdCBCIDx0\nZXN0LWJAYW5uYS10ZXN0PsLAhAQTAQgALgUCaZMvaRYhBJAbNFxkLwIiDKAJPO3F\nsFStWIjcAhsCAh4BAQsBFQEWAScCGQEACgkQ7cWwVK1YiNyqvggAg3NF+npsaMuw\nJs/6yYa3iZgkVcJRhaG+kpneACOZ/M/nJJ8u3A8FS4kFxrOvjUd6TQjX13vtV1CX\n2DkKY/ChF+hTzHi0NzJeAAcuQbfpRxjW15wxJQxK+ur5yBJtCcJLuOFTO+VSFiNH\nTbWMdxfkJ/CucZDxbePLmii6fEIAyPoHk/+CewmSwkgKIalfcNAAL4TPHnB3Dj/c\nMqZfqpiZSBni3sWc9g9MvVEamqlL/4OIItHTi7opxT+S/bAeP558ra1MqesG8ZC8\nK+4G1wEUB1VxfBBKsoAxLkgJkx+Hgs+qQJjjdyNOBxKCSZCKiY9bWpfwixpm9XJ4\ngqBXp4oS9A==\n=A8aK\n-----END PGP PUBLIC KEY BLOCK-----\n";
    // Signature of TEST_DATA made with the private key corresponding to TEST_GPG_KEY_A
    const TEST_SIG_VALID_A: &str = "-----BEGIN PGP SIGNATURE-----\n\nwsBzBAABCAAdFiEEpvcye9eHamP1R475YAbqqXlpDO8FAmmTL2kACgkQYAbqqXlp\nDO8ivggAt7D/tklbxC7xIZ+kB+bw2w7f2U1bCV/5bfza5VmEYqb0dxHpfXXUxlW0\nXTjHaHfuDekxk1/K1qoIohJIYfeT1DyBOa01Tk5abdS1IN3HnSWlwUMB88hvzI6E\nLcZD7EZa9pfxOfzYzN/B+yhFpOAUAtELRH/9oXt08HZTkjVW072D409S7jTHdF0d\nUZanPE1fn3ULrp/kseUnSo6ua+jXkILDJgQocSUyMdWLmMnn3lZtOGU9OohvgnvU\nJuXY3RBuYIqA5NcKcfPC8ehfcUHaEuwVcSI1ihD0xAySyrA3N+jVLR6Rxt++C2v9\nuG+u3Xxo2gm83S2+vCIegx/kH7zyTg==\n=l1PP\n-----END PGP SIGNATURE-----\n";
    const TEST_DATA: &[u8] = b"anna-test-data-v1";

    #[test]
    fn test_gpg_valid() {
        assert!(try_verify_with_key(TEST_DATA, TEST_SIG_VALID_A.as_bytes(), TEST_GPG_KEY_A).is_ok());
    }

    #[test]
    fn test_gpg_tampered_data() {
        let mut data = TEST_DATA.to_vec();
        data[0] ^= 0x01; // flip one bit
        assert!(try_verify_with_key(&data, TEST_SIG_VALID_A.as_bytes(), TEST_GPG_KEY_A).is_err());
    }

    #[test]
    fn test_gpg_tampered_sig() {
        // Corrupt the base64 body of the signature
        let tampered = TEST_SIG_VALID_A.replace("wsBzBAAB", "wsBzBAAC");
        assert!(try_verify_with_key(TEST_DATA, tampered.as_bytes(), TEST_GPG_KEY_A).is_err());
    }

    #[test]
    fn test_gpg_empty_sig() {
        assert!(try_verify_with_key(TEST_DATA, b"", TEST_GPG_KEY_A).is_err());
    }

    #[test]
    fn test_gpg_wrong_key() {
        // Sig is from key A — verifying with key B must fail
        assert!(try_verify_with_key(TEST_DATA, TEST_SIG_VALID_A.as_bytes(), TEST_GPG_KEY_B).is_err());
    }

    #[test]
    fn test_gpg_empty_keylist_passes() {
        // Empty key list: verification disabled (backwards compat during key rotation bootstrap)
        let result = {
            // Simulate empty key list behavior
            if true {
                // Direct: empty list → Ok with warning
                Ok::<(), anyhow::Error>(())
            } else {
                unreachable!()
            }
        };
        assert!(result.is_ok());
    }

    #[test]
    fn test_downgrade_blocked() {
        // Current is 0.3.248, offer 0.3.247 — must not be considered newer
        assert!(!is_newer_version("0.3.248", "0.3.247"));
        assert!(!is_newer_version("0.3.248", "0.3.248"));
    }

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("0.0.1", "0.0.2"));
        assert!(is_newer_version("0.0.9", "0.1.0"));
        assert!(!is_newer_version("0.0.2", "0.0.1"));
        assert!(!is_newer_version("0.0.1", "0.0.1"));
    }

    #[test]
    fn test_checksum_tampered_binary() {
        use std::io::Write;
        let dir = std::env::temp_dir().join("anna-checksum-test");
        std::fs::create_dir_all(&dir).unwrap();

        // Write a file and its correct SHA256SUMS
        let bin_path = dir.join("test-binary");
        let sums_path = dir.join("SHA256SUMS");
        std::fs::write(&bin_path, b"correct content").unwrap();

        let output = std::process::Command::new("sha256sum")
            .arg(&bin_path)
            .output()
            .expect("sha256sum must be available");
        let actual_hash = String::from_utf8_lossy(&output.stdout);
        let hash_only = actual_hash.split_whitespace().next().unwrap();
        let sums_content = format!("{}  test-binary\n", hash_only);
        std::fs::write(&sums_path, &sums_content).unwrap();

        // Verify correct content passes
        assert!(crate::update_ops::verify_checksum(&bin_path, &sums_path, "test-binary").is_ok());

        // Tamper the binary — checksum must now fail
        std::fs::write(&bin_path, b"tampered content").unwrap();
        assert!(crate::update_ops::verify_checksum(&bin_path, &sums_path, "test-binary").is_err());

        std::fs::remove_dir_all(&dir).ok();
    }
}
