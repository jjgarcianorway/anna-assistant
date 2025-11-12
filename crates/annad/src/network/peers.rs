//! Peer management for distributed consensus (Phase 1.11)
//!
//! Handles peer discovery, configuration, TLS/mTLS, and resilient HTTP client communication.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

/// TLS configuration for mTLS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Path to CA certificate (for verifying peers)
    pub ca_cert: PathBuf,
    /// Path to server certificate
    pub server_cert: PathBuf,
    /// Path to server private key
    pub server_key: PathBuf,
    /// Path to client certificate
    pub client_cert: PathBuf,
    /// Path to client private key
    pub client_key: PathBuf,
}

impl TlsConfig {
    /// Validate TLS configuration files exist and have correct permissions
    pub async fn validate(&self) -> Result<()> {
        // Check all files exist
        for (path, name) in [
            (&self.ca_cert, "CA certificate"),
            (&self.server_cert, "server certificate"),
            (&self.server_key, "server key"),
            (&self.client_cert, "client certificate"),
            (&self.client_key, "client key"),
        ] {
            if !path.exists() {
                return Err(anyhow!("TLS {} not found: {}", name, path.display()));
            }
        }

        // Check key file permissions (must be 0600)
        for (path, name) in [
            (&self.server_key, "server key"),
            (&self.client_key, "client key"),
        ] {
            self.check_key_permissions(path, name).await?;
        }

        info!("✓ TLS configuration validated");
        Ok(())
    }

    #[cfg(unix)]
    async fn check_key_permissions(&self, path: &Path, name: &str) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let metadata = fs::metadata(path).await
            .with_context(|| format!("Failed to stat {}", name))?;

        let mode = metadata.permissions().mode() & 0o777;
        if mode != 0o600 {
            return Err(anyhow!(
                "TLS {} has insecure permissions {:o} (must be 0600): {}",
                name,
                mode,
                path.display()
            ));
        }

        Ok(())
    }

    #[cfg(not(unix))]
    async fn check_key_permissions(&self, _path: &Path, _name: &str) -> Result<()> {
        // Non-Unix systems: skip permission check
        Ok(())
    }

    /// Load rustls server config
    pub async fn load_server_config(&self) -> Result<Arc<rustls::ServerConfig>> {
        use rustls::ServerConfig;
        use rustls_pemfile::{certs, pkcs8_private_keys};
        use std::io::BufReader;

        // Load CA cert
        let ca_cert_pem = fs::read(&self.ca_cert).await
            .with_context(|| format!("Failed to read CA cert: {}", self.ca_cert.display()))?;
        let mut ca_reader = BufReader::new(&ca_cert_pem[..]);
        let ca_certs = certs(&mut ca_reader)
            .collect::<Result<Vec<_>, _>>()
            .with_context(|| "Failed to parse CA certificate")?;

        if ca_certs.is_empty() {
            return Err(anyhow!("No certificates found in CA file"));
        }

        // Create root store
        let mut root_store = rustls::RootCertStore::empty();
        for cert in ca_certs {
            root_store.add(cert)
                .with_context(|| "Failed to add CA cert to root store")?;
        }

        // Load server cert chain
        let server_cert_pem = fs::read(&self.server_cert).await
            .with_context(|| format!("Failed to read server cert: {}", self.server_cert.display()))?;
        let mut server_cert_reader = BufReader::new(&server_cert_pem[..]);
        let server_certs = certs(&mut server_cert_reader)
            .collect::<Result<Vec<_>, _>>()
            .with_context(|| "Failed to parse server certificate")?;

        if server_certs.is_empty() {
            return Err(anyhow!("No certificates found in server cert file"));
        }

        // Load server private key
        let server_key_pem = fs::read(&self.server_key).await
            .with_context(|| format!("Failed to read server key: {}", self.server_key.display()))?;
        let mut server_key_reader = BufReader::new(&server_key_pem[..]);
        let mut keys = pkcs8_private_keys(&mut server_key_reader)
            .collect::<Result<Vec<_>, _>>()
            .with_context(|| "Failed to parse server private key")?;

        if keys.is_empty() {
            return Err(anyhow!("No private keys found in server key file"));
        }

        let private_key = rustls::pki_types::PrivateKeyDer::Pkcs8(keys.remove(0));

        // Build server config with mTLS
        let client_verifier = rustls::server::WebPkiClientVerifier::builder(Arc::new(root_store))
            .build()
            .with_context(|| "Failed to build client verifier")?;

        let config = ServerConfig::builder()
            .with_client_cert_verifier(client_verifier)
            .with_single_cert(server_certs, private_key)
            .with_context(|| "Failed to build server config")?;

        info!("✓ TLS server config loaded (mTLS enabled)");
        Ok(Arc::new(config))
    }

    /// Load rustls client config
    pub async fn load_client_config(&self) -> Result<Arc<rustls::ClientConfig>> {
        use rustls::ClientConfig;
        use rustls_pemfile::{certs, pkcs8_private_keys};
        use std::io::BufReader;

        // Load CA cert
        let ca_cert_pem = fs::read(&self.ca_cert).await
            .with_context(|| format!("Failed to read CA cert: {}", self.ca_cert.display()))?;
        let mut ca_reader = BufReader::new(&ca_cert_pem[..]);
        let ca_certs = certs(&mut ca_reader)
            .collect::<Result<Vec<_>, _>>()
            .with_context(|| "Failed to parse CA certificate")?;

        if ca_certs.is_empty() {
            return Err(anyhow!("No certificates found in CA file"));
        }

        // Create root store
        let mut root_store = rustls::RootCertStore::empty();
        for cert in ca_certs {
            root_store.add(cert)
                .with_context(|| "Failed to add CA cert to root store")?;
        }

        // Load client cert chain
        let client_cert_pem = fs::read(&self.client_cert).await
            .with_context(|| format!("Failed to read client cert: {}", self.client_cert.display()))?;
        let mut client_cert_reader = BufReader::new(&client_cert_pem[..]);
        let client_certs = certs(&mut client_cert_reader)
            .collect::<Result<Vec<_>, _>>()
            .with_context(|| "Failed to parse client certificate")?;

        if client_certs.is_empty() {
            return Err(anyhow!("No certificates found in client cert file"));
        }

        // Load client private key
        let client_key_pem = fs::read(&self.client_key).await
            .with_context(|| format!("Failed to read client key: {}", self.client_key.display()))?;
        let mut client_key_reader = BufReader::new(&client_key_pem[..]);
        let mut keys = pkcs8_private_keys(&mut client_key_reader)
            .collect::<Result<Vec<_>, _>>()
            .with_context(|| "Failed to parse client private key")?;

        if keys.is_empty() {
            return Err(anyhow!("No private keys found in client key file"));
        }

        let private_key = rustls::pki_types::PrivateKeyDer::Pkcs8(keys.remove(0));

        // Build client config with mTLS
        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_client_auth_cert(client_certs, private_key)
            .with_context(|| "Failed to build client config")?;

        info!("✓ TLS client config loaded (mTLS enabled)");
        Ok(Arc::new(config))
    }
}

/// Peer configuration entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConfig {
    pub node_id: String,
    pub address: String,
    pub port: u16,
}

impl PeerConfig {
    pub fn url(&self, tls_enabled: bool) -> String {
        let scheme = if tls_enabled { "https" } else { "http" };
        format!("{}://{}:{}", scheme, self.address, self.port)
    }
}

/// Peer list configuration (Phase 1.11)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerList {
    #[serde(default)]
    pub allow_insecure_peers: bool,

    #[serde(default)]
    pub tls: Option<TlsConfig>,

    pub peers: Vec<PeerConfig>,
}

impl PeerList {
    /// Load peer list from YAML file with TLS validation
    pub async fn load_from_file(path: &Path) -> Result<Self> {
        info!("Loading peer list from: {}", path.display());

        let content = fs::read_to_string(path).await
            .with_context(|| format!("Failed to read peers file: {}", path.display()))?;

        let peer_list: PeerList = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse peers YAML: {}", path.display()))?;

        // Validate TLS config if not in insecure mode
        if !peer_list.allow_insecure_peers {
            if peer_list.tls.is_none() {
                return Err(anyhow!(
                    "TLS configuration required when allow_insecure_peers=false. \
                     Set allow_insecure_peers=true to run without TLS (NOT RECOMMENDED)."
                ));
            }

            // Validate TLS files and permissions
            if let Some(ref tls_config) = peer_list.tls {
                tls_config.validate().await
                    .with_context(|| "TLS configuration validation failed")?;
            }
        } else {
            warn!("⚠️  WARNING: Running with allow_insecure_peers=true - TLS DISABLED");
            warn!("⚠️  This is NOT RECOMMENDED for production deployments");
        }

        // Deduplicate peers by node_id
        let mut seen = std::collections::HashSet::new();
        let unique_peers: Vec<_> = peer_list.peers.into_iter()
            .filter(|p| seen.insert(p.node_id.clone()))
            .collect();

        if unique_peers.len() != seen.len() {
            warn!("Deduplicated {} duplicate peer entries", seen.len() - unique_peers.len());
        }

        info!("Loaded {} unique peers", unique_peers.len());
        for peer in &unique_peers {
            debug!("  Peer: {} @ {}:{}", peer.node_id, peer.address, peer.port);
        }

        Ok(PeerList {
            allow_insecure_peers: peer_list.allow_insecure_peers,
            tls: peer_list.tls,
            peers: unique_peers,
        })
    }

    /// Get peer by node ID
    pub fn get_peer(&self, node_id: &str) -> Option<&PeerConfig> {
        self.peers.iter().find(|p| p.node_id == node_id)
    }

    /// Get all peers except self
    pub fn get_other_peers(&self, self_node_id: &str) -> Vec<&PeerConfig> {
        self.peers.iter()
            .filter(|p| p.node_id != self_node_id)
            .collect()
    }

    /// Check if TLS is enabled
    pub fn tls_enabled(&self) -> bool {
        !self.allow_insecure_peers && self.tls.is_some()
    }
}

/// Exponential backoff configuration
#[derive(Debug, Clone)]
pub struct BackoffConfig {
    pub base_ms: u64,
    pub factor: f64,
    pub jitter_percent: f64,
    pub max_ms: u64,
    pub max_attempts: usize,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            base_ms: 100,
            factor: 2.0,
            jitter_percent: 0.20,
            max_ms: 5000,
            max_attempts: 10,
        }
    }
}

impl BackoffConfig {
    /// Calculate backoff duration with jitter
    pub fn calculate_backoff(&self, attempt: usize) -> Duration {
        use rand::Rng;

        // Calculate base backoff: base * factor^attempt
        let base_backoff = (self.base_ms as f64 * self.factor.powi(attempt as i32)).min(self.max_ms as f64);

        // Add jitter: ±jitter_percent
        let mut rng = rand::thread_rng();
        let jitter_range = base_backoff * self.jitter_percent;
        let jitter = rng.gen_range(-jitter_range..=jitter_range);

        let backoff_ms = (base_backoff + jitter).max(0.0) as u64;
        Duration::from_millis(backoff_ms)
    }
}

/// Request result classification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestStatus {
    Success,
    NetworkError,
    TlsError,
    Http4xx,
    Http5xx,
    Timeout,
}

impl RequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::NetworkError => "network_error",
            Self::TlsError => "tls_error",
            Self::Http4xx => "http_4xx",
            Self::Http5xx => "http_5xx",
            Self::Timeout => "timeout",
        }
    }

    /// Determine if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::NetworkError | Self::Http5xx | Self::Timeout)
    }
}

/// HTTP client for peer communication with TLS and resilient retry
pub struct PeerClient {
    client: reqwest::Client,
    tls_enabled: bool,
    backoff: BackoffConfig,
    metrics: Option<Arc<super::metrics::ConsensusMetrics>>,
}

impl PeerClient {
    /// Create new peer client (without TLS)
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(2500))
            .build()
            .unwrap();

        Self {
            client,
            tls_enabled: false,
            backoff: BackoffConfig::default(),
            metrics: None,
        }
    }

    /// Create peer client with TLS
    pub async fn new_with_tls(tls_config: &TlsConfig, metrics: Option<Arc<super::metrics::ConsensusMetrics>>) -> Result<Self> {
        // Load CA certificate for server verification
        let ca_pem = fs::read(&tls_config.ca_cert).await
            .with_context(|| format!("Failed to read CA cert: {}", tls_config.ca_cert.display()))?;
        let ca_cert = reqwest::Certificate::from_pem(&ca_pem)
            .with_context(|| "Failed to parse CA certificate")?;

        // Load client certificate and key for mTLS
        let client_cert_pem = fs::read(&tls_config.client_cert).await
            .with_context(|| format!("Failed to read client cert: {}", tls_config.client_cert.display()))?;
        let client_key_pem = fs::read(&tls_config.client_key).await
            .with_context(|| format!("Failed to read client key: {}", tls_config.client_key.display()))?;

        // Combine cert and key for reqwest identity
        let mut identity_pem = client_cert_pem.clone();
        identity_pem.extend_from_slice(&client_key_pem);

        let identity = reqwest::Identity::from_pem(&identity_pem)
            .with_context(|| "Failed to create client identity")?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(2500))
            .add_root_certificate(ca_cert)
            .identity(identity)
            .build()
            .with_context(|| "Failed to build TLS-enabled HTTP client")?;

        info!("✓ TLS-enabled peer client initialized (mTLS)");

        Ok(Self {
            client,
            tls_enabled: true,
            backoff: BackoffConfig::default(),
            metrics,
        })
    }

    /// Classify request error
    fn classify_error(&self, err: &reqwest::Error) -> RequestStatus {
        if err.is_timeout() {
            RequestStatus::Timeout
        } else if err.is_connect() || err.is_request() {
            // Check if TLS-related
            if err.to_string().contains("tls") || err.to_string().contains("certificate") {
                RequestStatus::TlsError
            } else {
                RequestStatus::NetworkError
            }
        } else if let Some(status) = err.status() {
            if status.is_client_error() {
                RequestStatus::Http4xx
            } else if status.is_server_error() {
                RequestStatus::Http5xx
            } else {
                RequestStatus::NetworkError
            }
        } else {
            RequestStatus::NetworkError
        }
    }

    /// Submit observation to peer with retry
    pub async fn submit_observation(
        &self,
        peer: &PeerConfig,
        observation: &crate::consensus::AuditObservation,
    ) -> Result<()> {
        let url = format!("{}/rpc/submit", peer.url(self.tls_enabled));

        for attempt in 0..self.backoff.max_attempts {
            debug!("Submitting observation to peer: {} (attempt {}/{})", peer.node_id, attempt + 1, self.backoff.max_attempts);

            match self.client.post(&url).json(observation).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Some(ref metrics) = self.metrics {
                            metrics.record_peer_request(&peer.node_id, RequestStatus::Success.as_str());
                        }
                        return Ok(());
                    } else {
                        let status = if response.status().is_client_error() {
                            RequestStatus::Http4xx
                        } else {
                            RequestStatus::Http5xx
                        };

                        if let Some(ref metrics) = self.metrics {
                            metrics.record_peer_request(&peer.node_id, status.as_str());
                        }

                        if !status.is_retryable() {
                            return Err(anyhow!("Peer {} returned non-retryable error: {}", peer.node_id, response.status()));
                        }

                        warn!("Peer {} returned {}, retrying...", peer.node_id, response.status());
                    }
                }
                Err(e) => {
                    let status = self.classify_error(&e);

                    if let Some(ref metrics) = self.metrics {
                        metrics.record_peer_request(&peer.node_id, status.as_str());
                    }

                    if !status.is_retryable() {
                        return Err(anyhow!("Non-retryable error for peer {}: {}", peer.node_id, e));
                    }

                    warn!("Error contacting peer {}: {}, retrying...", peer.node_id, e);
                }
            }

            // Backoff before retry (unless last attempt)
            if attempt < self.backoff.max_attempts - 1 {
                let backoff = self.backoff.calculate_backoff(attempt);
                debug!("Backing off for {:?} before retry", backoff);

                if let Some(ref metrics) = self.metrics {
                    metrics.record_backoff_duration(backoff.as_secs_f64());
                }

                sleep(backoff).await;
            }
        }

        Err(anyhow!("Failed to contact peer {} after {} attempts", peer.node_id, self.backoff.max_attempts))
    }

    /// Get consensus status from peer
    pub async fn get_status(
        &self,
        peer: &PeerConfig,
        round_id: Option<&str>,
    ) -> Result<serde_json::Value> {
        let mut url = format!("{}/rpc/status", peer.url(self.tls_enabled));
        if let Some(rid) = round_id {
            url = format!("{}?round_id={}", url, rid);
        }

        debug!("Getting status from peer: {}", peer.node_id);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to get status from {}: {}", peer.node_id, e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Peer {} returned error: {}",
                peer.node_id,
                response.status()
            ));
        }

        let status: serde_json::Value = response.json().await
            .map_err(|e| anyhow!("Failed to parse status from {}: {}", peer.node_id, e))?;

        Ok(status)
    }

    /// Broadcast observation to all peers with concurrent retry
    pub async fn broadcast_observation(
        &self,
        peers: &[&PeerConfig],
        observation: &crate::consensus::AuditObservation,
    ) -> Vec<(String, Result<()>)> {
        use tokio::task::JoinSet;

        let mut join_set = JoinSet::new();

        for peer in peers {
            let peer = (*peer).clone();
            let observation = observation.clone();
            let client = self.clone();

            join_set.spawn(async move {
                let node_id = peer.node_id.clone();
                let result = client.submit_observation(&peer, &observation).await;
                (node_id, result)
            });
        }

        let mut results = Vec::new();
        while let Some(result) = join_set.join_next().await {
            if let Ok((node_id, res)) = result {
                results.push((node_id, res));
            }
        }

        results
    }
}

impl Clone for PeerClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            tls_enabled: self.tls_enabled,
            backoff: self.backoff.clone(),
            metrics: self.metrics.clone(),
        }
    }
}

impl Default for PeerClient {
    fn default() -> Self {
        Self::new()
    }
}
