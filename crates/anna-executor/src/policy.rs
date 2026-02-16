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

const POLICY_PATH: &str = "/etc/anna/policy.toml";

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ExecutorPolicy {
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
    pub fn load() -> Self {
        match std::fs::read_to_string(POLICY_PATH) {
            Ok(content) => toml::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }
}
