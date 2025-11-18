// Phase 1.8: Cryptographic Layer - Ed25519 Signatures (REAL IMPLEMENTATION)
// Status: PoC - Full Ed25519 support with key management

use anyhow::{anyhow, Result};
use ed25519_dalek::{Signature as DalekSignature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::fs;
use tracing::{debug, info, warn};

// ============================================================================
// KEY TYPES
// ============================================================================

/// Ed25519 public key (32 bytes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKey {
    bytes: [u8; 32],
}

impl PublicKey {
    /// Create from 32-byte array
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }

    /// Get bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Get node ID fingerprint (first 16 hex chars)
    pub fn fingerprint(&self) -> String {
        let hex = hex::encode(&self.bytes);
        format!("node_{}", &hex[0..16])
    }

    /// Parse from hex string
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let bytes = hex::decode(hex_str)?;
        if bytes.len() != 32 {
            return Err(anyhow!("Invalid public key length: {}", bytes.len()));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self::from_bytes(arr))
    }

    /// Encode to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }

    /// Convert to ed25519-dalek VerifyingKey
    pub fn to_verifying_key(&self) -> Result<VerifyingKey> {
        VerifyingKey::from_bytes(&self.bytes).map_err(|e| anyhow!("Invalid public key: {}", e))
    }
}

/// Ed25519 secret key (32 bytes seed)
#[derive(Debug, Clone)]
pub struct SecretKey {
    bytes: [u8; 32],
}

impl SecretKey {
    /// Create from 32-byte array
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }

    /// Get bytes (dangerous - exposes secret material)
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Parse from hex string
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let bytes = hex::decode(hex_str)?;
        if bytes.len() != 32 {
            return Err(anyhow!("Invalid secret key length: {}", bytes.len()));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self::from_bytes(arr))
    }

    /// Encode to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }

    /// Convert to ed25519-dalek SigningKey
    pub fn to_signing_key(&self) -> SigningKey {
        SigningKey::from_bytes(&self.bytes)
    }
}

impl Drop for SecretKey {
    fn drop(&mut self) {
        // Zero out secret key material on drop (basic security)
        self.bytes.fill(0);
    }
}

/// Ed25519 signature (64 bytes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    bytes: Vec<u8>,
}

impl Signature {
    /// Create from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 64 {
            return Err(anyhow!("Invalid signature length: {}", bytes.len()));
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    /// Get bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Convert to ed25519-dalek Signature
    pub fn to_dalek_signature(&self) -> Result<DalekSignature> {
        let arr: [u8; 64] = self
            .bytes
            .as_slice()
            .try_into()
            .map_err(|_| anyhow!("Invalid signature length"))?;
        Ok(DalekSignature::from_bytes(&arr))
    }
}

/// Keypair (public + secret)
#[derive(Debug, Clone)]
pub struct Keypair {
    pub public: PublicKey,
    pub secret: SecretKey,
}

impl Keypair {
    /// Create from secret key (derives public key)
    pub fn from_secret(secret: SecretKey) -> Self {
        let signing_key = secret.to_signing_key();
        let verifying_key = signing_key.verifying_key();
        let public = PublicKey::from_bytes(verifying_key.to_bytes());
        Self { public, secret }
    }
}

// ============================================================================
// KEY GENERATION
// ============================================================================

/// Generate new Ed25519 keypair using OsRng
pub fn generate_keypair() -> Result<Keypair> {
    info!("Generating Ed25519 keypair");

    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    let secret = SecretKey::from_bytes(signing_key.to_bytes());
    let public = PublicKey::from_bytes(verifying_key.to_bytes());

    debug!(
        "Generated keypair with fingerprint: {}",
        public.fingerprint()
    );

    Ok(Keypair { public, secret })
}

// ============================================================================
// SIGNING
// ============================================================================

/// Sign a message with secret key
pub fn sign(message: &[u8], secret: &SecretKey) -> Result<Signature> {
    debug!("Signing message (len: {})", message.len());

    let signing_key = secret.to_signing_key();
    let signature_dalek = signing_key.sign(message);

    Signature::from_bytes(&signature_dalek.to_bytes())
}

/// Verify signature on message with public key
pub fn verify(message: &[u8], signature: &Signature, public: &PublicKey) -> Result<bool> {
    debug!(
        "Verifying signature (message len: {}, sig len: {})",
        message.len(),
        signature.as_bytes().len()
    );

    let verifying_key = public.to_verifying_key()?;
    let signature_dalek = signature.to_dalek_signature()?;

    match verifying_key.verify(message, &signature_dalek) {
        Ok(()) => {
            debug!("Signature verification: SUCCESS");
            Ok(true)
        }
        Err(e) => {
            warn!("Signature verification: FAILED - {}", e);
            Ok(false)
        }
    }
}

// ============================================================================
// KEY STORAGE
// ============================================================================

/// Load keypair from files (atomic read)
pub async fn load_keypair(base_path: &Path) -> Result<Keypair> {
    info!("Loading keypair from {:?}", base_path);

    let pub_path = base_path.join("node_id.pub");
    let sec_path = base_path.join("node_id.sec");

    // Check permissions on secret key (must be 400 or 600)
    let sec_metadata = fs::metadata(&sec_path).await?;
    let perms = sec_metadata.permissions();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = perms.mode() & 0o777;
        if mode != 0o400 && mode != 0o600 {
            return Err(anyhow!(
                "Secret key has insecure permissions: {:o} (expected 400 or 600)",
                mode
            ));
        }
    }

    // Read keys
    let pub_hex = fs::read_to_string(&pub_path).await?;
    let sec_hex = fs::read_to_string(&sec_path).await?;

    let public = PublicKey::from_hex(pub_hex.trim())?;
    let secret = SecretKey::from_hex(sec_hex.trim())?;

    // Verify keypair consistency
    let expected_public = Keypair::from_secret(secret.clone()).public;
    if expected_public.as_bytes() != public.as_bytes() {
        return Err(anyhow!(
            "Keypair mismatch: public key doesn't match secret key"
        ));
    }

    info!("Loaded keypair: {}", public.fingerprint());
    Ok(Keypair { public, secret })
}

/// Save keypair to files (atomic write with temp + rename)
pub async fn save_keypair(keypair: &Keypair, base_path: &Path) -> Result<()> {
    info!("Saving keypair to {:?}", base_path);

    // Create directory if it doesn't exist
    fs::create_dir_all(base_path).await?;

    let pub_path = base_path.join("node_id.pub");
    let sec_path = base_path.join("node_id.sec");

    // Write to temp files first
    let pub_temp = base_path.join("node_id.pub.tmp");
    let sec_temp = base_path.join("node_id.sec.tmp");

    fs::write(&pub_temp, keypair.public.to_hex()).await?;
    fs::write(&sec_temp, keypair.secret.to_hex()).await?;

    // Set permissions on secret key (400)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&sec_temp).await?.permissions();
        perms.set_mode(0o400);
        fs::set_permissions(&sec_temp, perms).await?;
    }

    // Atomic rename
    fs::rename(&pub_temp, &pub_path).await?;
    fs::rename(&sec_temp, &sec_path).await?;

    info!("Keypair saved: {}", keypair.public.fingerprint());
    Ok(())
}

/// Check if keypair exists
pub async fn keypair_exists(base_path: &Path) -> bool {
    let pub_path = base_path.join("node_id.pub");
    let sec_path = base_path.join("node_id.sec");

    tokio::fs::try_exists(&pub_path).await.unwrap_or(false)
        && tokio::fs::try_exists(&sec_path).await.unwrap_or(false)
}

// ============================================================================
// HASHING
// ============================================================================

/// Compute SHA-256 hash of data
pub fn sha256_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("sha256:{}", hex::encode(result))
}

/// Compute forecast hash for observation
pub fn compute_forecast_hash(
    forecast_id: &str,
    predicted: &crate::mirror_audit::types::SystemMetrics,
) -> String {
    // Canonical encoding: forecast_id || metrics in fixed order
    let canonical = format!(
        "{}|{}|{}|{}|{}|{}",
        forecast_id,
        predicted.health_score,
        predicted.empathy_index,
        predicted.strain_index,
        predicted.network_coherence,
        predicted.avg_trust_score
    );

    sha256_hash(canonical.as_bytes())
}

/// Compute outcome hash for observation
pub fn compute_outcome_hash(
    forecast_id: &str,
    actual: &crate::mirror_audit::types::SystemMetrics,
) -> String {
    // Canonical encoding: forecast_id || metrics in fixed order
    let canonical = format!(
        "{}|{}|{}|{}|{}|{}",
        forecast_id,
        actual.health_score,
        actual.empathy_index,
        actual.strain_index,
        actual.network_coherence,
        actual.avg_trust_score
    );

    sha256_hash(canonical.as_bytes())
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair = generate_keypair().unwrap();
        assert_eq!(keypair.public.as_bytes().len(), 32);
        assert_eq!(keypair.secret.as_bytes().len(), 32);
    }

    #[test]
    fn test_public_key_fingerprint() {
        let keypair = generate_keypair().unwrap();
        let fingerprint = keypair.public.fingerprint();
        assert!(fingerprint.starts_with("node_"));
        assert_eq!(fingerprint.len(), 21); // "node_" + 16 hex chars
    }

    #[test]
    fn test_signature_roundtrip() {
        let keypair = generate_keypair().unwrap();
        let message = b"test message for signing";

        let signature = sign(message, &keypair.secret).unwrap();
        assert_eq!(signature.as_bytes().len(), 64);

        let verified = verify(message, &signature, &keypair.public).unwrap();
        assert!(verified);
    }

    #[test]
    fn test_signature_verification_fails_wrong_message() {
        let keypair = generate_keypair().unwrap();
        let message = b"original message";
        let wrong_message = b"tampered message";

        let signature = sign(message, &keypair.secret).unwrap();
        let verified = verify(wrong_message, &signature, &keypair.public).unwrap();
        assert!(!verified);
    }

    #[test]
    fn test_signature_verification_fails_wrong_key() {
        let keypair1 = generate_keypair().unwrap();
        let keypair2 = generate_keypair().unwrap();
        let message = b"test message";

        let signature = sign(message, &keypair1.secret).unwrap();
        let verified = verify(message, &signature, &keypair2.public).unwrap();
        assert!(!verified);
    }

    #[test]
    fn test_public_key_hex_roundtrip() {
        let keypair = generate_keypair().unwrap();
        let hex = keypair.public.to_hex();
        let parsed = PublicKey::from_hex(&hex).unwrap();
        assert_eq!(keypair.public.as_bytes(), parsed.as_bytes());
    }

    #[test]
    fn test_secret_key_hex_roundtrip() {
        let keypair = generate_keypair().unwrap();
        let hex = keypair.secret.to_hex();
        let parsed = SecretKey::from_hex(&hex).unwrap();
        assert_eq!(keypair.secret.as_bytes(), parsed.as_bytes());
    }

    #[test]
    fn test_sha256_hash() {
        let data = b"test data";
        let hash = sha256_hash(data);
        assert!(hash.starts_with("sha256:"));
        assert_eq!(hash.len(), 7 + 64); // "sha256:" + 64 hex chars
    }

    #[tokio::test]
    async fn test_keypair_save_and_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let keypair = generate_keypair().unwrap();

        // Save
        save_keypair(&keypair, temp_dir.path()).await.unwrap();

        // Load
        let loaded = load_keypair(temp_dir.path()).await.unwrap();

        // Verify
        assert_eq!(keypair.public.as_bytes(), loaded.public.as_bytes());
        assert_eq!(keypair.secret.as_bytes(), loaded.secret.as_bytes());
    }

    #[tokio::test]
    async fn test_keypair_atomic_rotation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let keypair1 = generate_keypair().unwrap();
        let keypair2 = generate_keypair().unwrap();

        // Save first keypair
        save_keypair(&keypair1, temp_dir.path()).await.unwrap();

        // Rotate to second keypair (atomic)
        save_keypair(&keypair2, temp_dir.path()).await.unwrap();

        // Load should get second keypair
        let loaded = load_keypair(temp_dir.path()).await.unwrap();
        assert_eq!(keypair2.public.as_bytes(), loaded.public.as_bytes());
        assert_ne!(keypair1.public.as_bytes(), loaded.public.as_bytes());
    }
}
