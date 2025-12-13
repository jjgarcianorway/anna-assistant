// v0.0.540: Installation Date Tracker (Phase 116)
// Tracks installation date and anniversaries per VISION.md fun stats

use chrono::{DateTime, Datelike, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Installation method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum InstallMethod {
    #[default]
    CurlScript,
    Manual,
    Package,
    Development,
    Unknown,
}

impl std::fmt::Display for InstallMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CurlScript => write!(f, "Curl Script"),
            Self::Manual => write!(f, "Manual"),
            Self::Package => write!(f, "Package Manager"),
            Self::Development => write!(f, "Development Build"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Installation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum InstallStatus {
    #[default]
    Active,
    Upgraded,
    Reinstalled,
    Paused,
}

impl std::fmt::Display for InstallStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "Active"),
            Self::Upgraded => write!(f, "Upgraded"),
            Self::Reinstalled => write!(f, "Reinstalled"),
            Self::Paused => write!(f, "Paused"),
        }
    }
}

/// Installation info record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationInfo {
    pub install_date: DateTime<Utc>,
    pub first_run: DateTime<Utc>,
    pub last_upgrade: Option<DateTime<Utc>>,
    pub install_method: InstallMethod,
    pub status: InstallStatus,
    pub upgrade_count: u32,
    pub initial_version: String,
    pub current_version: String,
}

impl Default for InstallationInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl InstallationInfo {
    /// Create new installation info (marks install date as now)
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            install_date: now,
            first_run: now,
            last_upgrade: None,
            install_method: InstallMethod::default(),
            status: InstallStatus::default(),
            upgrade_count: 0,
            initial_version: String::new(),
            current_version: String::new(),
        }
    }

    /// Create with specific install date
    pub fn with_date(date: DateTime<Utc>) -> Self {
        Self {
            install_date: date,
            first_run: date,
            last_upgrade: None,
            install_method: InstallMethod::default(),
            status: InstallStatus::default(),
            upgrade_count: 0,
            initial_version: String::new(),
            current_version: String::new(),
        }
    }

    /// Set install method
    pub fn with_method(mut self, method: InstallMethod) -> Self {
        self.install_method = method;
        self
    }

    /// Set versions
    pub fn with_versions(mut self, initial: impl Into<String>, current: impl Into<String>) -> Self {
        self.initial_version = initial.into();
        self.current_version = current.into();
        self
    }

    /// Record upgrade
    pub fn record_upgrade(&mut self, new_version: impl Into<String>) {
        self.last_upgrade = Some(Utc::now());
        self.current_version = new_version.into();
        self.upgrade_count += 1;
        self.status = InstallStatus::Upgraded;
    }

    /// Days since installation
    pub fn days_installed(&self) -> i64 {
        (Utc::now() - self.install_date).num_days()
    }

    /// Weeks since installation
    pub fn weeks_installed(&self) -> i64 {
        self.days_installed() / 7
    }

    /// Months since installation (approximate)
    pub fn months_installed(&self) -> i64 {
        self.days_installed() / 30
    }

    /// Check if anniversary (yearly)
    pub fn is_anniversary(&self) -> bool {
        let now = Utc::now();
        now.month() == self.install_date.month() && now.day() == self.install_date.day()
    }

    /// Check if monthly milestone
    pub fn is_monthly_milestone(&self) -> bool {
        Utc::now().day() == self.install_date.day()
    }

    /// Days until next anniversary
    pub fn days_until_anniversary(&self) -> i64 {
        let now = Utc::now();
        let this_year = now.year();

        // Calculate anniversary this year
        let anniversary_this_year = self.install_date
            .with_year(this_year)
            .unwrap_or(self.install_date);

        if anniversary_this_year > now {
            (anniversary_this_year - now).num_days()
        } else {
            // Anniversary already passed this year
            let next_year_anniversary = self.install_date
                .with_year(this_year + 1)
                .unwrap_or(self.install_date);
            (next_year_anniversary - now).num_days()
        }
    }

    /// Human-readable uptime
    pub fn uptime_string(&self) -> String {
        let days = self.days_installed();
        if days == 0 {
            "Just installed today!".to_string()
        } else if days == 1 {
            "1 day".to_string()
        } else if days < 7 {
            format!("{} days", days)
        } else if days < 30 {
            format!("{} weeks", self.weeks_installed())
        } else if days < 365 {
            format!("{} months", self.months_installed())
        } else {
            let years = days / 365;
            let remaining_months = (days % 365) / 30;
            if remaining_months > 0 {
                format!("{} years, {} months", years, remaining_months)
            } else {
                format!("{} years", years)
            }
        }
    }

    /// Time since last upgrade
    pub fn time_since_upgrade(&self) -> Option<Duration> {
        self.last_upgrade.map(|upgrade| Utc::now() - upgrade)
    }
}

/// Format installation info
pub fn format_installation_info(info: &InstallationInfo) -> String {
    let mut output = String::new();
    output.push_str("=== Installation Info ===\n\n");

    output.push_str(&format!("Install Date: {}\n", info.install_date.format("%Y-%m-%d")));
    output.push_str(&format!("Installed: {}\n", info.uptime_string()));
    output.push_str(&format!("Method: {}\n", info.install_method));
    output.push_str(&format!("Status: {}\n", info.status));

    if !info.initial_version.is_empty() {
        output.push_str(&format!("Initial Version: {}\n", info.initial_version));
    }

    if !info.current_version.is_empty() {
        output.push_str(&format!("Current Version: {}\n", info.current_version));
    }

    output.push_str(&format!("Upgrades: {}\n", info.upgrade_count));

    if let Some(last) = info.last_upgrade {
        output.push_str(&format!("Last Upgrade: {}\n", last.format("%Y-%m-%d")));
    }

    if info.is_anniversary() {
        output.push_str("\nHappy Anniversary!\n");
    } else {
        output.push_str(&format!("Days Until Anniversary: {}\n", info.days_until_anniversary()));
    }

    output
}

/// Format compact summary
pub fn format_installation_compact(info: &InstallationInfo) -> String {
    format!(
        "Installed {} ago ({}) - {} upgrades",
        info.uptime_string(),
        info.install_date.format("%Y-%m-%d"),
        info.upgrade_count
    )
}

/// Check if query is installation-related
pub fn is_installation_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("install date")
        || lower.contains("when was anna installed")
        || lower.contains("how long")
        || lower.contains("anniversary")
        || lower.contains("upgraded")
        || lower.contains("version history")
}

/// Fun fact about installation
pub fn installation_fun_fact() -> &'static str {
    "Anna remembers exactly when you first installed her! The 'installation date' stat celebrates your journey together."
}

/// Get anniversary message
pub fn anniversary_message(years: i64) -> String {
    match years {
        0 => "Welcome to Anna! Here's to a great journey together.".to_string(),
        1 => "Happy 1 year anniversary! Thanks for sticking with Anna.".to_string(),
        2 => "2 years together! Anna has learned a lot from you.".to_string(),
        3 => "3 years! You're a true Anna veteran now.".to_string(),
        _ => format!("{} years! You've been with Anna longer than most. Legendary!", years),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_install_method_default() {
        let method = InstallMethod::default();
        assert_eq!(method, InstallMethod::CurlScript);
    }

    #[test]
    fn test_install_status_default() {
        let status = InstallStatus::default();
        assert_eq!(status, InstallStatus::Active);
    }

    #[test]
    fn test_installation_info_new() {
        let info = InstallationInfo::new();
        assert_eq!(info.upgrade_count, 0);
        assert_eq!(info.status, InstallStatus::Active);
    }

    #[test]
    fn test_days_installed() {
        let past = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let info = InstallationInfo::with_date(past);
        assert!(info.days_installed() > 300);
    }

    #[test]
    fn test_record_upgrade() {
        let mut info = InstallationInfo::new();
        info.record_upgrade("0.0.100");
        assert_eq!(info.upgrade_count, 1);
        assert_eq!(info.current_version, "0.0.100");
        assert_eq!(info.status, InstallStatus::Upgraded);
    }

    #[test]
    fn test_uptime_string() {
        let info = InstallationInfo::new();
        assert!(info.uptime_string().contains("Just installed") || info.uptime_string().contains("day"));
    }

    #[test]
    fn test_with_method() {
        let info = InstallationInfo::new().with_method(InstallMethod::Package);
        assert_eq!(info.install_method, InstallMethod::Package);
    }

    #[test]
    fn test_is_installation_query() {
        assert!(is_installation_query("When was anna installed?"));
        assert!(is_installation_query("Show install date"));
        assert!(!is_installation_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = installation_fun_fact();
        assert!(fact.contains("install") || fact.contains("anniversary"));
    }

    #[test]
    fn test_anniversary_message() {
        let msg = anniversary_message(1);
        assert!(msg.contains("anniversary"));
    }
}
