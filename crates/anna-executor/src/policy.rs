//! Executor policy — loaded from /etc/anna/policy.toml on each request.
//!
//! All fields default to permissive (same behaviour as pre-policy).
//! An absent /etc/anna/policy.toml is treated as an all-defaults policy.
//!
//! Example /etc/anna/policy.toml:
//!
//!   allow_restart_service    = false   # disable systemctl restart entirely
//!   allow_clean_journal      = true
//!   allow_clean_package_cache = true
//!   allow_clean_tmp_files    = true
//!   min_journal_keep_days    = 7       # never vacuum more aggressively than 7 days
//!   min_package_keep_versions = 2      # always keep at least 2 package versions

use serde::Deserialize;
use tracing::warn;

const POLICY_PATH: &str = "/etc/anna/policy.toml";

/// Policy schema version this executor understands.
/// Executors reject policy files with a higher version (fail closed).
const CURRENT_POLICY_VERSION: u32 = 1;

fn default_policy_version() -> u32 { 1 }

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ExecutorPolicy {
    /// Policy schema version. Executor rejects versions > CURRENT_POLICY_VERSION.
    #[serde(default = "default_policy_version")]
    pub policy_version: u32,
    /// Allow RestartService RPCs. Default: true.
    pub allow_restart_service: bool,
    /// Allow CleanJournal RPCs. Default: true.
    pub allow_clean_journal: bool,
    /// Allow CleanPackageCache RPCs. Default: true.
    pub allow_clean_package_cache: bool,
    /// Allow CleanTmpFiles RPCs. Default: true.
    pub allow_clean_tmp_files: bool,
    /// Floor on CleanJournal keep_days — enforced even if caller asks for fewer.
    pub min_journal_keep_days: u32,
    /// Floor on CleanPackageCache keep_versions.
    pub min_package_keep_versions: u32,
}

impl Default for ExecutorPolicy {
    fn default() -> Self {
        Self {
            policy_version: 1,
            allow_restart_service: true,
            allow_clean_journal: true,
            allow_clean_package_cache: true,
            allow_clean_tmp_files: true,
            min_journal_keep_days: 0,
            min_package_keep_versions: 1,
        }
    }
}

impl ExecutorPolicy {
    /// Load policy from /etc/anna/policy.toml. Falls back to permissive defaults.
    /// Rejects (deny_all) if policy_version > CURRENT_POLICY_VERSION.
    pub fn load() -> Self {
        let policy = match std::fs::read_to_string(POLICY_PATH) {
            Ok(content) => toml::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        };
        if policy.policy_version > CURRENT_POLICY_VERSION {
            warn!(
                "policy.toml version {} > supported {}, denying all executor actions",
                policy.policy_version, CURRENT_POLICY_VERSION
            );
            return Self::deny_all();
        }
        policy
    }

    /// All-deny policy — used when the schema version is unrecognized.
    pub fn deny_all() -> Self {
        Self {
            policy_version: CURRENT_POLICY_VERSION,
            allow_restart_service: false,
            allow_clean_journal: false,
            allow_clean_package_cache: false,
            allow_clean_tmp_files: false,
            min_journal_keep_days: 0,
            min_package_keep_versions: 1,
        }
    }

    /// Return first 8 hex chars of MD5 of the raw policy.toml bytes.
    ///
    /// Used as a cheap drift fingerprint in audit log entries — not a security hash.
    /// Returns "00000000" if the file is absent (default policy in use).
    pub fn content_hash() -> String {
        let bytes = std::fs::read(POLICY_PATH).unwrap_or_default();
        format!("{:.8x}", md5::compute(&bytes))
    }
}
