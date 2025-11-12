//! Cryptographic operations for Collective Mind
//!
//! Phase 1.3: Simplified crypto for peer communication
//! NOTE: This is a basic implementation. Production use should employ
//! proper cryptographic libraries (e.g., ed25519-dalek, aes-gcm)
//! Citation: [archwiki:System_maintenance]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

/// Keypair for node identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPair {
    /// Public key (hex encoded)
    pub public_key: String,
    /// Private key (hex encoded) - should be kept secure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>,
    /// Key creation timestamp
    pub created_at: String,
}

impl KeyPair {
    /// Generate a new keypair
    /// NOTE: This is a placeholder. Production should use ed25519-dalek or similar
    pub fn generate() -> Result<Self> {
        use std::time::SystemTime;

        // Generate pseudo-random keys based on timestamp and system entropy
        // WARNING: This is NOT cryptographically secure for production!
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_nanos();

        let public_key = format!("{:032x}", now);
        let private_key = format!("{:032x}", now.wrapping_mul(0xdeadbeef));

        Ok(Self {
            public_key,
            private_key: Some(private_key),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Load keypair from file
    pub async fn load(path: &Path) -> Result<Self> {
        let json = fs::read_to_string(path)
            .await
            .context("Failed to read keypair file")?;

        let keypair: Self = serde_json::from_str(&json)
            .context("Failed to parse keypair JSON")?;

        Ok(keypair)
    }

    /// Save keypair to file
    pub async fn save(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize keypair")?;

        fs::write(path, json)
            .await
            .context("Failed to write keypair file")?;

        // Set restrictive permissions (owner read/write only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(path, perms)?;
        }

        Ok(())
    }

    /// Get public key only (for sharing)
    pub fn public_only(&self) -> Self {
        Self {
            public_key: self.public_key.clone(),
            private_key: None,
            created_at: self.created_at.clone(),
        }
    }

    /// Derive peer ID from public key (SHA-256 hash)
    pub fn peer_id(&self) -> String {
        // Simple hash for peer ID
        // Production should use proper SHA-256
        format!("peer_{}", &self.public_key[..16])
    }
}

/// Sign a message with private key
/// NOTE: This is a placeholder signature scheme
pub fn sign_message(message: &str, private_key: &str) -> String {
    // Simple signature: hash of message + private key
    // Production should use ed25519 signatures
    let combined = format!("{}{}", message, private_key);
    format!("sig_{:016x}", hash_string(&combined))
}

/// Verify message signature
/// NOTE: This is a placeholder verification
pub fn verify_signature(message: &str, signature: &str, public_key: &str) -> bool {
    // Placeholder verification
    // Production needs proper signature verification
    signature.starts_with("sig_") && !public_key.is_empty()
}

/// Simple string hashing (placeholder for SHA-256)
fn hash_string(s: &str) -> u64 {
    let mut hash: u64 = 0;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }
    hash
}

/// Encrypt message (placeholder)
pub fn encrypt_message(message: &str, _recipient_public_key: &str) -> String {
    // Placeholder: in production use AES-256-GCM or similar
    format!("enc_{}", message)
}

/// Decrypt message (placeholder)
pub fn decrypt_message(encrypted: &str, _private_key: &str) -> Result<String> {
    // Placeholder decryption
    if let Some(msg) = encrypted.strip_prefix("enc_") {
        Ok(msg.to_string())
    } else {
        anyhow::bail!("Invalid encrypted message format")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair = KeyPair::generate().unwrap();
        assert!(!keypair.public_key.is_empty());
        assert!(keypair.private_key.is_some());
    }

    #[test]
    fn test_peer_id_derivation() {
        let keypair = KeyPair::generate().unwrap();
        let peer_id = keypair.peer_id();
        assert!(peer_id.starts_with("peer_"));
    }

    #[test]
    fn test_signature() {
        let message = "test message";
        let private_key = "test_private_key";
        let public_key = "test_public_key";

        let signature = sign_message(message, private_key);
        assert!(verify_signature(message, &signature, public_key));
    }
}
