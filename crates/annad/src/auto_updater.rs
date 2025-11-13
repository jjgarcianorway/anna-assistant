// Auto-Updater Service for Daemon
// Phase 3.10: AUR-Aware Auto-Upgrade System
//
// Daily background check for updates, with automatic upgrade for manual installations

use anna_common::github_releases::{GitHubClient, is_update_available};
use anna_common::installation_source::{detect_current_installation, InstallationSource};
use std::time::Duration;
use tokio::time;
use tracing::{error, info, warn};

const GITHUB_OWNER: &str = "jjgarcianorway";
const GITHUB_REPO: &str = "anna-assistant";
const CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60); // 24 hours
const LAST_CHECK_FILE: &str = "/var/lib/anna/last_update_check";

pub struct AutoUpdater {
    enabled: bool,
    source: InstallationSource,
}

impl AutoUpdater {
    pub fn new() -> Self {
        let source = detect_current_installation();
        let enabled = source.allows_auto_update();

        info!(
            "Auto-updater initialized: source={:?}, enabled={}",
            source, enabled
        );

        Self { enabled, source }
    }

    /// Start auto-update background service
    pub fn start(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            if !self.enabled {
                info!(
                    "Auto-update service disabled for installation source: {}",
                    self.source.description()
                );
                return;
            }

            info!("Auto-update service started (checks every 24h)");

            // Initial delay: 1 hour after startup
            time::sleep(Duration::from_secs(60 * 60)).await;

            loop {
                self.check_and_update().await;

                // Wait 24 hours before next check
                time::sleep(CHECK_INTERVAL).await;
            }
        })
    }

    /// Check for updates and install if available
    async fn check_and_update(&self) {
        info!("Checking for updates...");

        // Record check time
        if let Err(e) = self.record_check_time().await {
            warn!("Failed to record check time: {}", e);
        }

        let current_version = env!("CARGO_PKG_VERSION");

        // Fetch latest release
        let client = GitHubClient::new(GITHUB_OWNER, GITHUB_REPO);

        let latest_release = match client.get_latest_release().await {
            Ok(release) => release,
            Err(e) => {
                error!("Failed to fetch latest release: {}", e);
                return;
            }
        };

        let latest_version = latest_release.version();

        // Check if update is available
        if !is_update_available(current_version, latest_version) {
            info!("Already running latest version: v{}", current_version);
            return;
        }

        info!("Update available: v{} → v{}", current_version, latest_version);

        // For now, just log the update availability
        // Full automatic upgrade will be implemented after user approval mechanism
        info!("Auto-upgrade not yet fully implemented - user must run 'sudo annactl upgrade'");

        // TODO: Implement automatic download and installation
        // This would require:
        // 1. Download new binaries
        // 2. Verify checksums
        // 3. Backup current version
        // 4. Replace binaries
        // 5. Restart daemon
        //
        // For Phase 3.10, we'll log and notify, but require manual upgrade
    }

    /// Record last check time
    async fn record_check_time(&self) -> Result<(), std::io::Error> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        tokio::fs::write(LAST_CHECK_FILE, timestamp).await
    }
}

/// Get last update check time
pub async fn get_last_check_time() -> Option<String> {
    tokio::fs::read_to_string(LAST_CHECK_FILE).await.ok()
}
