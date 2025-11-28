//! Auto-Update Manager v0.26.0
//!
//! Reliable background updates with zero-downtime installation.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::protocol_v26::{
    BinaryChecksum, DownloadState, InstallStrategy, UpdateEvent, UpdateEventType, UpdateProgress,
    UpdateResultV26, UpdateTrace,
};

// ============================================================================
// CONFIGURATION
// ============================================================================

/// Default download directory
pub const DEFAULT_DOWNLOAD_DIR: &str = "/var/lib/anna/updates";

/// Default staging directory for zero-downtime install
pub const DEFAULT_STAGING_DIR: &str = "/var/lib/anna/staging";

/// Maximum download retries
pub const DEFAULT_MAX_RETRIES: u32 = 3;

/// Download chunk size (bytes)
pub const DEFAULT_CHUNK_SIZE: usize = 65536;

/// Auto-update manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoUpdateConfig {
    /// Enable background downloads
    pub background_download: bool,
    /// Download directory
    pub download_dir: PathBuf,
    /// Staging directory for atomic installs
    pub staging_dir: PathBuf,
    /// Installation strategy
    pub install_strategy: InstallStrategy,
    /// Maximum download retries
    pub max_retries: u32,
    /// Verify checksums before install
    pub verify_checksums: bool,
    /// Auto-install after download
    pub auto_install: bool,
    /// Create backup before install
    pub backup_before_install: bool,
}

impl Default for AutoUpdateConfig {
    fn default() -> Self {
        Self {
            background_download: true,
            download_dir: PathBuf::from(DEFAULT_DOWNLOAD_DIR),
            staging_dir: PathBuf::from(DEFAULT_STAGING_DIR),
            install_strategy: InstallStrategy::ZeroDowntime,
            max_retries: DEFAULT_MAX_RETRIES,
            verify_checksums: true,
            auto_install: true,
            backup_before_install: true,
        }
    }
}

// ============================================================================
// UPDATE MANAGER
// ============================================================================

/// Auto-update manager state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AutoUpdateManager {
    /// Current progress
    pub progress: UpdateProgress,
    /// Active trace (if any)
    pub active_trace: Option<UpdateTrace>,
    /// Downloaded binaries
    pub downloaded_binaries: Vec<DownloadedBinary>,
    /// Checksums for verification
    pub checksums: Vec<BinaryChecksum>,
    /// Retry count for current download
    pub retry_count: u32,
}

/// Information about a downloaded binary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadedBinary {
    /// Binary name (annad, annactl)
    pub name: String,
    /// Path to downloaded file
    pub path: PathBuf,
    /// Size in bytes
    pub size: u64,
    /// SHA256 checksum
    pub sha256: Option<String>,
    /// Verified flag
    pub verified: bool,
}

impl AutoUpdateManager {
    /// Create new manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Start update check
    pub fn start_check(&mut self, current_version: &str) {
        let trace_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        self.active_trace = Some(UpdateTrace {
            trace_id,
            started_at: now,
            ended_at: None,
            state: DownloadState::Checking,
            from_version: current_version.to_string(),
            to_version: String::new(),
            events: vec![UpdateEvent {
                timestamp: now,
                event_type: UpdateEventType::CheckStarted,
                version: None,
                details: None,
            }],
            result: None,
        });

        self.progress.state = DownloadState::Checking;
    }

    /// Record new version found
    pub fn found_version(&mut self, version: &str, total_bytes: u64) {
        self.progress.target_version = Some(version.to_string());
        self.progress.total_bytes = Some(total_bytes);

        if let Some(ref mut trace) = self.active_trace {
            trace.to_version = version.to_string();
            trace.events.push(UpdateEvent {
                timestamp: chrono::Utc::now().timestamp(),
                event_type: UpdateEventType::NewVersionFound,
                version: Some(version.to_string()),
                details: Some(format!("{} bytes", total_bytes)),
            });
        }
    }

    /// Start download
    pub fn start_download(&mut self) {
        let now = chrono::Utc::now().timestamp();
        self.progress.state = DownloadState::Downloading { progress_percent: 0 };
        self.progress.started_at = Some(now);
        self.retry_count = 0;

        if let Some(ref mut trace) = self.active_trace {
            trace.state = DownloadState::Downloading { progress_percent: 0 };
            trace.events.push(UpdateEvent {
                timestamp: now,
                event_type: UpdateEventType::DownloadStarted,
                version: self.progress.target_version.clone(),
                details: None,
            });
        }
    }

    /// Update download progress
    pub fn update_progress(&mut self, bytes: u64) {
        self.progress.bytes_downloaded = bytes;
        let percent = self.progress.percentage();
        self.progress.state = DownloadState::Downloading {
            progress_percent: percent,
        };

        // Update ETA based on download speed
        if let (Some(started), Some(total)) = (self.progress.started_at, self.progress.total_bytes)
        {
            let elapsed = chrono::Utc::now().timestamp() - started;
            if elapsed > 0 && bytes > 0 {
                let speed = bytes as f64 / elapsed as f64;
                let remaining = total.saturating_sub(bytes);
                self.progress.eta_seconds = Some((remaining as f64 / speed) as u64);
            }
        }

        if let Some(ref mut trace) = self.active_trace {
            trace.state = self.progress.state.clone();
        }
    }

    /// Complete download
    pub fn download_complete(&mut self) {
        let now = chrono::Utc::now().timestamp();
        self.progress.state = DownloadState::Ready;

        if let Some(ref mut trace) = self.active_trace {
            trace.state = DownloadState::Ready;
            trace.events.push(UpdateEvent {
                timestamp: now,
                event_type: UpdateEventType::DownloadCompleted,
                version: self.progress.target_version.clone(),
                details: Some(format!("{} bytes", self.progress.bytes_downloaded)),
            });
        }
    }

    /// Record checksum verification
    pub fn checksum_verified(&mut self, success: bool) {
        self.progress.checksum_verified = Some(success);

        let event_type = if success {
            UpdateEventType::ChecksumVerified
        } else {
            UpdateEventType::ChecksumFailed
        };

        if let Some(ref mut trace) = self.active_trace {
            trace.events.push(UpdateEvent {
                timestamp: chrono::Utc::now().timestamp(),
                event_type,
                version: self.progress.target_version.clone(),
                details: None,
            });
        }
    }

    /// Start installation
    pub fn start_install(&mut self) {
        self.progress.state = DownloadState::Installing;

        if let Some(ref mut trace) = self.active_trace {
            trace.state = DownloadState::Installing;
            trace.events.push(UpdateEvent {
                timestamp: chrono::Utc::now().timestamp(),
                event_type: UpdateEventType::InstallStarted,
                version: self.progress.target_version.clone(),
                details: None,
            });
        }
    }

    /// Complete installation
    pub fn install_complete(&mut self) {
        let now = chrono::Utc::now().timestamp();
        self.progress.state = DownloadState::PendingRestart;

        if let Some(ref mut trace) = self.active_trace {
            trace.state = DownloadState::PendingRestart;
            trace.ended_at = Some(now);
            trace.result = Some(UpdateResultV26::Success);
            trace.events.push(UpdateEvent {
                timestamp: now,
                event_type: UpdateEventType::InstallCompleted,
                version: self.progress.target_version.clone(),
                details: None,
            });
        }
    }

    /// Record failure
    pub fn record_failure(&mut self, reason: &str) {
        let now = chrono::Utc::now().timestamp();
        self.progress.state = DownloadState::Failed {
            reason: reason.to_string(),
        };

        if let Some(ref mut trace) = self.active_trace {
            trace.state = self.progress.state.clone();
            trace.ended_at = Some(now);
            trace.result = Some(UpdateResultV26::Failed {
                reason: reason.to_string(),
            });
            trace.events.push(UpdateEvent {
                timestamp: now,
                event_type: UpdateEventType::DownloadFailed,
                version: self.progress.target_version.clone(),
                details: Some(reason.to_string()),
            });
        }
    }

    /// Increment retry
    pub fn retry(&mut self) -> bool {
        self.retry_count += 1;
        self.retry_count <= DEFAULT_MAX_RETRIES
    }

    /// Add downloaded binary
    pub fn add_binary(&mut self, binary: DownloadedBinary) {
        self.downloaded_binaries.push(binary);
    }

    /// Reset state
    pub fn reset(&mut self) {
        self.progress = UpdateProgress::default();
        self.active_trace = None;
        self.downloaded_binaries.clear();
        self.checksums.clear();
        self.retry_count = 0;
    }

    /// Get current trace for logging
    pub fn get_trace(&self) -> Option<&UpdateTrace> {
        self.active_trace.as_ref()
    }

    /// Check if update is in progress
    pub fn is_in_progress(&self) -> bool {
        !matches!(
            self.progress.state,
            DownloadState::Idle | DownloadState::Failed { .. }
        )
    }

    /// Check if ready to install
    pub fn is_ready_to_install(&self) -> bool {
        matches!(self.progress.state, DownloadState::Ready)
            && self.progress.checksum_verified == Some(true)
    }
}

// ============================================================================
// CHECKSUM VERIFICATION
// ============================================================================

/// Verify SHA256 checksum of a file
pub fn verify_checksum(path: &std::path::Path, expected: &str) -> std::io::Result<bool> {
    use std::io::Read;

    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let hash = hasher.finalize();
    let computed: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
    Ok(computed.eq_ignore_ascii_case(expected))
}

/// Simple SHA256 implementation for checksum verification
struct Sha256 {
    state: [u32; 8],
    data: Vec<u8>,
    len: u64,
}

impl Sha256 {
    fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
            data: Vec::new(),
            len: 0,
        }
    }

    fn update(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
        self.len += data.len() as u64;
    }

    fn finalize(mut self) -> [u8; 32] {
        // Padding
        self.data.push(0x80);
        while (self.data.len() % 64) != 56 {
            self.data.push(0);
        }
        let bit_len = self.len * 8;
        self.data.extend_from_slice(&bit_len.to_be_bytes());

        // Process blocks - collect chunks first to avoid borrow issues
        let chunks: Vec<Vec<u8>> = self.data.chunks(64).map(|c| c.to_vec()).collect();
        for chunk in chunks {
            self.process_block(&chunk);
        }

        // Output
        let mut output = [0u8; 32];
        for (i, &val) in self.state.iter().enumerate() {
            output[i * 4..(i + 1) * 4].copy_from_slice(&val.to_be_bytes());
        }
        output
    }

    fn process_block(&mut self, block: &[u8]) {
        const K: [u32; 64] = [
            0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
            0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
            0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
            0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
            0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
            0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
            0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
            0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
            0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
            0xc67178f2,
        ];

        let mut w = [0u32; 64];
        for (i, chunk) in block.chunks(4).enumerate().take(16) {
            w[i] = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }

        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_update_config_default() {
        let config = AutoUpdateConfig::default();
        assert!(config.background_download);
        assert!(config.verify_checksums);
        assert_eq!(config.max_retries, DEFAULT_MAX_RETRIES);
    }

    #[test]
    fn test_manager_lifecycle() {
        let mut manager = AutoUpdateManager::new();

        // Start check
        manager.start_check("0.25.0");
        assert!(manager.active_trace.is_some());
        assert!(matches!(manager.progress.state, DownloadState::Checking));

        // Found version
        manager.found_version("0.26.0", 10000);
        assert_eq!(
            manager.progress.target_version,
            Some("0.26.0".to_string())
        );

        // Start download
        manager.start_download();
        assert!(matches!(
            manager.progress.state,
            DownloadState::Downloading { .. }
        ));

        // Update progress
        manager.update_progress(5000);
        assert_eq!(manager.progress.bytes_downloaded, 5000);
        assert_eq!(manager.progress.percentage(), 50);

        // Complete download
        manager.download_complete();
        assert!(matches!(manager.progress.state, DownloadState::Ready));
    }

    #[test]
    fn test_manager_failure() {
        let mut manager = AutoUpdateManager::new();
        manager.start_check("0.25.0");
        manager.record_failure("Network error");

        assert!(manager.progress.is_failed());
        let trace = manager.get_trace().unwrap();
        assert!(matches!(trace.result, Some(UpdateResultV26::Failed { .. })));
    }

    #[test]
    fn test_manager_retry() {
        let mut manager = AutoUpdateManager::new();

        assert!(manager.retry()); // 1
        assert!(manager.retry()); // 2
        assert!(manager.retry()); // 3
        assert!(!manager.retry()); // Exceeded
    }

    #[test]
    fn test_sha256() {
        let mut hasher = Sha256::new();
        hasher.update(b"hello world");
        let result = hasher.finalize();

        let hex = result
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        assert_eq!(
            hex,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_is_ready_to_install() {
        let mut manager = AutoUpdateManager::new();
        manager.start_check("0.25.0");
        manager.found_version("0.26.0", 1000);
        manager.start_download();
        manager.update_progress(1000);
        manager.download_complete();

        // Not ready until checksum verified
        assert!(!manager.is_ready_to_install());

        manager.checksum_verified(true);
        assert!(manager.is_ready_to_install());
    }

    #[test]
    fn test_downloaded_binary() {
        let binary = DownloadedBinary {
            name: "annad".to_string(),
            path: PathBuf::from("/tmp/annad"),
            size: 1000,
            sha256: Some("abc123".to_string()),
            verified: false,
        };
        assert_eq!(binary.name, "annad");
        assert!(!binary.verified);
    }
}
