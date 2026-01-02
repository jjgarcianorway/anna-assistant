//! Status conversion and snapshot generation.

use std::path::Path;
use std::process::Command;

use anna_shared::helpers::{known_helpers, load_helpers, HelperPackage, InstallSource};
use anna_shared::specialists::SpecialistRole;
use anna_shared::status::{DaemonStatus, UpdateStatus};
use anna_shared::status_snapshot::{
    ConfigInfo, DaemonInfo, HelpersInfo, ModelsInfo, PermissionsInfo, RoleModelBinding,
    StatusSnapshot, UpdateInfo, UpdateResult, VersionInfo,
};
use anna_shared::teams::Team;
use anna_shared::VERSION;

use super::types::DaemonStateInner;

impl DaemonStateInner {
    pub fn to_status(&self) -> DaemonStatus {
        use anna_shared::status::LatencyStatus;

        // v0.0.36: Include p50 and p90 percentiles
        let latency = if !self.latency.total.samples.is_empty() {
            Some(LatencyStatus {
                translator_avg_ms: self.latency.translator.avg_ms(),
                translator_p50_ms: self.latency.translator.p50_ms(),
                translator_p90_ms: self.latency.translator.p90_ms(),
                translator_p95_ms: self.latency.translator.p95_ms(),
                probes_avg_ms: self.latency.probes.avg_ms(),
                probes_p50_ms: self.latency.probes.p50_ms(),
                probes_p90_ms: self.latency.probes.p90_ms(),
                probes_p95_ms: self.latency.probes.p95_ms(),
                specialist_avg_ms: self.latency.specialist.avg_ms(),
                specialist_p50_ms: self.latency.specialist.p50_ms(),
                specialist_p90_ms: self.latency.specialist.p90_ms(),
                specialist_p95_ms: self.latency.specialist.p95_ms(),
                total_avg_ms: self.latency.total.avg_ms(),
                total_p50_ms: self.latency.total.p50_ms(),
                total_p90_ms: self.latency.total.p90_ms(),
                total_p95_ms: self.latency.total.p95_ms(),
                sample_count: self.latency.total.samples.len(),
            })
        } else {
            None
        };

        DaemonStatus {
            version: VERSION.to_string(),
            state: self.state.clone(),
            pid: Some(self.pid),
            uptime_seconds: self.started_at.elapsed().as_secs(),
            debug_mode: self.config.debug_mode(),
            update: UpdateStatus {
                enabled: self.update.enabled,
                check_interval_secs: self.update.check_interval_secs,
                last_check_at: self.update.last_check_at,
                next_check_at: self.update.next_check_at,
                latest_version: self.update.latest_version.clone(),
                latest_checked_at: self.update.latest_checked_at,
                update_available: false,
                check_state: anna_shared::status::UpdateCheckState::NeverChecked,
            },
            llm: self.llm.clone(),
            hardware: self.hardware.clone(),
            ledger: self.ledger.summary(),
            last_error: self.last_error.clone(),
            latency,
            teams: anna_shared::status::TeamRoster::new(),
            truthfulness_score: self.truthfulness_score,
        }
    }

    /// Build comprehensive status snapshot (v0.0.29)
    pub fn to_status_snapshot(&self) -> StatusSnapshot {
        let captured_at_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Version info
        let versions = VersionInfo::new(VERSION).with_remote(self.update.latest_version.clone());

        // Daemon info
        let mut daemon = DaemonInfo::running(self.pid, self.started_at.elapsed().as_secs());
        if let Some(err) = &self.last_error {
            daemon = daemon.with_error(err.clone());
        }

        // Permissions info (basic - can be enhanced)
        let (user, groups) = current_user_and_groups();
        let mut perms = PermissionsInfo::current()
            .with_groups(groups)
            .with_daemon_access(Path::new(anna_shared::SOCKET_PATH).exists())
            .with_data_dir_ok(Path::new("/var/lib/anna").is_dir());
        perms.user = user;

        // Update info
        let update = UpdateInfo {
            interval_s: self.update.check_interval_secs,
            last_check_ts: self.update.last_check_at.map(|dt| dt.timestamp() as u64),
            next_check_ts: self.update.next_check_at.map(|dt| dt.timestamp() as u64),
            last_result: if self.update.update_available {
                UpdateResult::UpdateAvailable {
                    version: self.update.latest_version.clone().unwrap_or_default(),
                }
            } else if self.update.last_check_at.is_some() {
                UpdateResult::UpToDate
            } else {
                UpdateResult::NotChecked
            },
        };

        // Helpers info
        let mut helpers_registry = load_helpers();
        for pkg in known_helpers().packages {
            if helpers_registry.get(&pkg.id).is_none() {
                helpers_registry.register(pkg);
            }
        }
        if let Some(ollama_pkg) = helpers_registry.get_mut("ollama") {
            ollama_pkg.available = self.ollama.installed;
            if ollama_pkg.install_source == InstallSource::Unknown && self.ollama.installed {
                ollama_pkg.install_source = InstallSource::User;
            }
        } else {
            let mut pkg = HelperPackage::new("ollama", "Ollama").required();
            pkg.available = self.ollama.installed;
            pkg.install_source = if self.ollama.installed {
                InstallSource::User
            } else {
                InstallSource::Unknown
            };
            helpers_registry.register(pkg);
        }
        let helpers = HelpersInfo::from_registry(&helpers_registry);

        // Models info
        let models = ModelsInfo {
            ollama_present: self.ollama.installed,
            ollama_running: self.ollama.running,
            ollama_version: self.ollama.version.clone(),
            roles: self
                .llm
                .models
                .iter()
                .map(|m| RoleModelBinding {
                    team: Team::General,
                    role: match m.role.as_str() {
                        "translator" => SpecialistRole::Translator,
                        "supervisor" => SpecialistRole::Senior,
                        _ => SpecialistRole::Junior,
                    },
                    model_name: m.name.clone(),
                    model_present: m.pulled,
                })
                .collect(),
            downloads: Vec::new(),
        };

        // Config info - v0.0.449: Enhanced per VISION.md
        let config = ConfigInfo {
            debug_mode: self.config.debug_mode(),
            repl_clean_mode: !self.config.debug_mode(),
            autonomy_level: 0, // Conservative default
            auto_update: self.config.daemon.auto_update,
            learning_mode: false, // TODO: Add to config when implemented
            fast_path_enabled: self.config.daemon.fast_path_enabled,
            internal_comms: false, // TODO: Add to config when implemented
            request_timeout_secs: self.config.daemon.request_timeout_secs,
            update_check_interval_secs: self.config.daemon.update_interval,
        };

        // Teams info - v0.0.454: Dynamic team availability per VISION.md
        let teams = anna_shared::status_snapshot::TeamsInfo::detect();

        StatusSnapshot {
            captured_at_ts,
            versions,
            daemon,
            perms,
            update,
            helpers,
            models,
            config,
            teams,
            truthfulness_score: self.truthfulness_score,
        }
    }
}

fn current_user_and_groups() -> (String, Vec<String>) {
    let user = std::env::var("SUDO_USER")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "unknown".to_string());

    let groups = Command::new("id")
        .args(["-Gn", &user])
        .output()
        .ok()
        .map(|out| {
            String::from_utf8_lossy(&out.stdout)
                .split_whitespace()
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();

    (user, groups)
}
