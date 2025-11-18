// Phase 2: Certificate Pinning - Custom ServerCertVerifier
//
// Implements SHA256 fingerprint-based certificate pinning to prevent
// MITM attacks in distributed consensus scenarios.
//
// References:
// - OWASP: Certificate and Public Key Pinning
// - rustls: ServerCertVerifier trait

use rustls::client::danger::{ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName};
use rustls::Error as TlsError;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, warn};

/// Certificate pinning configuration
#[derive(Debug, Clone)]
pub struct PinningConfig {
    /// Map of peer hostnames to expected SHA256 fingerprints
    /// Format: "hostname" -> "sha256:HEXDIGEST"
    pub pinned_fingerprints: HashMap<String, String>,

    /// Whether to enforce pinning (fail-closed) or just log violations
    pub enforce: bool,
}

impl PinningConfig {
    pub fn new(enforce: bool) -> Self {
        Self {
            pinned_fingerprints: HashMap::new(),
            enforce,
        }
    }

    pub fn add_pin(&mut self, hostname: String, fingerprint: String) {
        self.pinned_fingerprints.insert(hostname, fingerprint);
    }

    pub fn get_pin(&self, hostname: &str) -> Option<&String> {
        self.pinned_fingerprints.get(hostname)
    }
}

/// Custom certificate verifier that enforces SHA256 fingerprint pinning
pub struct PinningVerifier {
    config: Arc<PinningConfig>,
    fallback_verifier: Arc<dyn ServerCertVerifier>,
    metrics: Option<Arc<super::metrics::ConsensusMetrics>>,
}

impl std::fmt::Debug for PinningVerifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PinningVerifier")
            .field("config", &self.config)
            .field("fallback_verifier", &"<dyn ServerCertVerifier>")
            .field("metrics", &self.metrics.is_some())
            .finish()
    }
}

impl PinningVerifier {
    pub fn new(
        config: Arc<PinningConfig>,
        fallback: Arc<dyn ServerCertVerifier>,
        metrics: Option<Arc<super::metrics::ConsensusMetrics>>,
    ) -> Self {
        Self {
            config,
            fallback_verifier: fallback,
            metrics,
        }
    }

    /// Compute SHA256 fingerprint of a certificate in DER format
    pub fn compute_fingerprint(cert_der: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(cert_der);
        let hash = hasher.finalize();
        format!("sha256:{}", hex::encode(hash))
    }

    /// Mask fingerprint for logging (show first 8 and last 8 chars)
    fn mask_fingerprint(fp: &str) -> String {
        if fp.len() < 20 {
            return fp.to_string();
        }
        format!("{}...{}", &fp[..15], &fp[fp.len() - 8..])
    }
}

impl ServerCertVerifier for PinningVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: rustls::pki_types::UnixTime,
    ) -> Result<ServerCertVerified, TlsError> {
        // First, perform standard validation using fallback verifier
        self.fallback_verifier.verify_server_cert(
            end_entity,
            intermediates,
            server_name,
            ocsp_response,
            now,
        )?;

        // Then, perform pinning validation
        let hostname = match server_name {
            ServerName::DnsName(dns) => dns.as_ref(),
            ServerName::IpAddress(ip) => {
                warn!("Certificate pinning: IP address in server name not supported, skipping");
                return Ok(ServerCertVerified::assertion());
            }
            _ => {
                warn!("Certificate pinning: unknown server name type, skipping");
                return Ok(ServerCertVerified::assertion());
            }
        };

        // Check if we have a pin configured for this peer
        if let Some(expected_fp) = self.config.get_pin(hostname) {
            let actual_fp = Self::compute_fingerprint(end_entity.as_ref());

            if &actual_fp != expected_fp {
                let masked_expected = Self::mask_fingerprint(expected_fp);
                let masked_actual = Self::mask_fingerprint(&actual_fp);

                error!(
                    peer = hostname,
                    expected = masked_expected,
                    actual = masked_actual,
                    "Certificate pinning violation detected"
                );

                // Emit metric
                if let Some(ref metrics) = self.metrics {
                    metrics.record_pinning_violation(hostname);
                }

                if self.config.enforce {
                    return Err(TlsError::General(format!(
                        "Certificate pinning violation: peer {}",
                        hostname
                    )));
                } else {
                    warn!(
                        peer = hostname,
                        "Certificate pinning violation (enforcement disabled, allowing connection)"
                    );
                }
            }
        } else {
            // No pin configured for this peer
            warn!(
                peer = hostname,
                "No certificate pin configured for peer, allowing connection"
            );
        }

        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, TlsError> {
        self.fallback_verifier
            .verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, TlsError> {
        self.fallback_verifier
            .verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.fallback_verifier.supported_verify_schemes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_fingerprint() {
        // Test vector: empty certificate (for testing hash computation)
        let cert_der = b"test certificate data";
        let fp = PinningVerifier::compute_fingerprint(cert_der);

        // Should be sha256: prefix + 64 hex chars
        assert!(fp.starts_with("sha256:"));
        assert_eq!(fp.len(), 7 + 64); // "sha256:" + 64 hex chars
    }

    #[test]
    fn test_mask_fingerprint() {
        let fp = "sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let masked = PinningVerifier::mask_fingerprint(fp);

        assert!(masked.contains("..."));
        assert!(masked.len() < fp.len());
        // Should show first 15 chars and last 8 chars
        assert!(masked.starts_with("sha256:12345678"));
        assert!(masked.ends_with("90abcdef"));
    }

    #[test]
    fn test_pinning_config() {
        let mut config = PinningConfig::new(true);
        config.add_pin("node1.example.com".to_string(), "sha256:abc123".to_string());

        assert_eq!(
            config.get_pin("node1.example.com"),
            Some(&"sha256:abc123".to_string())
        );
        assert_eq!(config.get_pin("node2.example.com"), None);
        assert!(config.enforce);
    }
}
