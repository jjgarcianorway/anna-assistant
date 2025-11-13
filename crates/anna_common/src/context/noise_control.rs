// Noise Control - Reduces repetition and nagging from low-priority issues
// Phase 4.6: Profiles, Noise Control, and Stable Feel

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use rusqlite::Connection;
use tracing::{debug, info};

use crate::caretaker_brain::{CaretakerIssue, IssueSeverity};

/// Issue tracking state for noise control
#[derive(Debug, Clone)]
pub struct IssueState {
    pub issue_key: String,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub last_shown: Option<DateTime<Utc>>,
    pub times_shown: i32,
    pub times_ignored: i32,
    pub last_repair_attempt: Option<DateTime<Utc>>,
    pub repair_success: Option<bool>,
    pub severity: IssueSeverity,
}

/// Noise control configuration
#[derive(Debug, Clone)]
pub struct NoiseControlConfig {
    /// Days after which Info issues are de-emphasized
    pub info_deemphasis_days: i64,

    /// Days after which Warning issues are de-emphasized (longer than Info)
    pub warning_deemphasis_days: i64,

    /// Critical issues are never de-emphasized
    pub never_deemphasize_critical: bool,

    /// Show all issues on first run (no tracking data exists)
    pub show_all_on_first_run: bool,
}

impl Default for NoiseControlConfig {
    fn default() -> Self {
        Self {
            info_deemphasis_days: 7,
            warning_deemphasis_days: 14,
            never_deemphasize_critical: true,
            show_all_on_first_run: true,
        }
    }
}

/// Update issue tracking state in database
pub fn update_issue_state(conn: &Connection, issue: &CaretakerIssue) -> Result<()> {
    let issue_key = issue_key_from_title(&issue.title);
    let severity_str = severity_to_string(&issue.severity);
    let now = Utc::now();

    // Try to insert or update
    conn.execute(
        "INSERT INTO issue_tracking (issue_key, first_seen, last_seen, severity, last_details)
         VALUES (?1, ?2, ?2, ?3, ?4)
         ON CONFLICT(issue_key) DO UPDATE SET
             last_seen = ?2,
             severity = ?3,
             last_details = ?4",
        rusqlite::params![
            &issue_key,
            now.to_rfc3339(),
            &severity_str,
            &issue.explanation,
        ],
    )
    .context("Failed to update issue tracking state")?;

    debug!("Updated issue tracking state for: {}", issue_key);
    Ok(())
}

/// Mark issue as shown to user
pub fn mark_issue_shown(conn: &Connection, issue_key: &str) -> Result<()> {
    let now = Utc::now();

    conn.execute(
        "UPDATE issue_tracking
         SET last_shown = ?1,
             times_shown = times_shown + 1
         WHERE issue_key = ?2",
        rusqlite::params![now.to_rfc3339(), issue_key],
    )
    .context("Failed to mark issue as shown")?;

    debug!("Marked issue as shown: {}", issue_key);
    Ok(())
}

/// Mark issue as ignored (user saw it but didn't act)
pub fn mark_issue_ignored(conn: &Connection, issue_key: &str) -> Result<()> {
    conn.execute(
        "UPDATE issue_tracking
         SET times_ignored = times_ignored + 1
         WHERE issue_key = ?1",
        rusqlite::params![issue_key],
    )
    .context("Failed to mark issue as ignored")?;

    debug!("Marked issue as ignored: {}", issue_key);
    Ok(())
}

/// Mark issue as repaired
pub fn mark_issue_repaired(
    conn: &Connection,
    issue_key: &str,
    success: bool,
) -> Result<()> {
    let now = Utc::now();

    conn.execute(
        "UPDATE issue_tracking
         SET last_repair_attempt = ?1,
             repair_success = ?2
         WHERE issue_key = ?3",
        rusqlite::params![now.to_rfc3339(), success, issue_key],
    )
    .context("Failed to mark issue as repaired")?;

    info!(
        "Marked issue as repaired (success={}): {}",
        success, issue_key
    );
    Ok(())
}

/// Get issue state from database
pub fn get_issue_state(conn: &Connection, issue_key: &str) -> Result<Option<IssueState>> {
    let mut stmt = conn.prepare(
        "SELECT issue_key, first_seen, last_seen, last_shown, times_shown, times_ignored,
                last_repair_attempt, repair_success, severity
         FROM issue_tracking
         WHERE issue_key = ?1",
    )?;

    let result = stmt.query_row([issue_key], |row| {
        Ok(IssueState {
            issue_key: row.get(0)?,
            first_seen: parse_datetime(&row.get::<_, String>(1)?),
            last_seen: parse_datetime(&row.get::<_, String>(2)?),
            last_shown: row
                .get::<_, Option<String>>(3)?
                .map(|s| parse_datetime(&s)),
            times_shown: row.get(4)?,
            times_ignored: row.get(5)?,
            last_repair_attempt: row
                .get::<_, Option<String>>(6)?
                .map(|s| parse_datetime(&s)),
            repair_success: row.get(7)?,
            severity: string_to_severity(&row.get::<_, String>(8)?),
        })
    });

    match result {
        Ok(state) => Ok(Some(state)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Check if issue should be de-emphasized based on noise control rules
pub fn should_deemphasize(
    state: &IssueState,
    config: &NoiseControlConfig,
) -> bool {
    let now = Utc::now();

    // Never de-emphasize Critical issues
    if state.severity == IssueSeverity::Critical && config.never_deemphasize_critical {
        return false;
    }

    // If successfully repaired, de-emphasize
    if let Some(true) = state.repair_success {
        return true;
    }

    // Check if issue has been shown recently
    if let Some(last_shown) = state.last_shown {
        let days_since_shown = (now - last_shown).num_days();

        match state.severity {
            IssueSeverity::Critical => false, // Never de-emphasize Critical
            IssueSeverity::Warning => {
                // De-emphasize Warning after warning_deemphasis_days
                days_since_shown < config.warning_deemphasis_days && state.times_shown > 0
            }
            IssueSeverity::Info => {
                // De-emphasize Info after info_deemphasis_days
                days_since_shown < config.info_deemphasis_days && state.times_shown > 0
            }
        }
    } else {
        // Never shown before, don't de-emphasize
        false
    }
}

/// Filter issues based on noise control rules
pub fn filter_issues_by_noise_control(
    conn: &Connection,
    issues: Vec<CaretakerIssue>,
    config: &NoiseControlConfig,
) -> Result<(Vec<CaretakerIssue>, Vec<CaretakerIssue>)> {
    let mut show_issues = Vec::new();
    let mut suppressed_issues = Vec::new();

    for issue in issues {
        let issue_key = issue_key_from_title(&issue.title);

        // Update tracking state for this issue
        update_issue_state(conn, &issue)?;

        // Get current state
        let state = get_issue_state(conn, &issue_key)?;

        let should_suppress = if let Some(state) = state {
            should_deemphasize(&state, config)
        } else {
            // No state yet (first run), show everything
            false
        };

        if should_suppress {
            debug!("Suppressing issue due to noise control: {}", issue_key);
            suppressed_issues.push(issue);
        } else {
            // Mark as shown
            mark_issue_shown(conn, &issue_key)?;
            show_issues.push(issue);
        }
    }

    Ok((show_issues, suppressed_issues))
}

/// Generate a stable issue key from the issue title
fn issue_key_from_title(title: &str) -> String {
    // Normalize title to create stable key
    // Remove common prefixes and normalize spacing
    title
        .trim()
        .to_lowercase()
        .replace("detected", "")
        .replace("found", "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

/// Convert severity to string
fn severity_to_string(severity: &IssueSeverity) -> String {
    match severity {
        IssueSeverity::Critical => "Critical".to_string(),
        IssueSeverity::Warning => "Warning".to_string(),
        IssueSeverity::Info => "Info".to_string(),
    }
}

/// Convert string to severity
fn string_to_severity(s: &str) -> IssueSeverity {
    match s {
        "Critical" => IssueSeverity::Critical,
        "Warning" => IssueSeverity::Warning,
        "Info" => IssueSeverity::Info,
        _ => IssueSeverity::Info, // Default to Info
    }
}

/// Parse datetime from RFC3339 string
fn parse_datetime(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_db() -> Result<Connection> {
        let conn = Connection::open_in_memory()?;

        // Create issue_tracking table
        conn.execute(
            "CREATE TABLE issue_tracking (
                issue_key TEXT PRIMARY KEY,
                first_seen DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                last_seen DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                last_shown DATETIME,
                times_shown INTEGER DEFAULT 0,
                times_ignored INTEGER DEFAULT 0,
                last_repair_attempt DATETIME,
                repair_success BOOLEAN,
                severity TEXT NOT NULL,
                last_details TEXT
            )",
            [],
        )?;

        Ok(conn)
    }

    #[test]
    fn test_issue_key_generation() {
        let key1 = issue_key_from_title("Disk Space Warning");
        let key2 = issue_key_from_title("disk space warning");
        assert_eq!(key1, key2);

        let key3 = issue_key_from_title("Service Failed: sshd");
        assert!(key3.contains("service"));
        assert!(key3.contains("failed"));
    }

    #[test]
    fn test_update_issue_state() {
        let conn = create_test_db().unwrap();

        let issue = CaretakerIssue::new(
            IssueSeverity::Warning,
            "Disk Space Low",
            "Your disk is 95% full",
            "Clean up disk space"
        );

        update_issue_state(&conn, &issue).unwrap();

        let state = get_issue_state(&conn, "disk-space-low").unwrap();
        assert!(state.is_some());

        let state = state.unwrap();
        assert_eq!(state.severity, IssueSeverity::Warning);
        assert_eq!(state.times_shown, 0); // Not marked as shown yet
    }

    #[test]
    fn test_mark_issue_shown() {
        let conn = create_test_db().unwrap();

        let issue = CaretakerIssue::new(
            IssueSeverity::Info,
            "Backup Reminder",
            "No backup tools detected",
            "Install timeshift"
        );

        update_issue_state(&conn, &issue).unwrap();
        mark_issue_shown(&conn, "backup-reminder").unwrap();

        let state = get_issue_state(&conn, "backup-reminder").unwrap().unwrap();
        assert_eq!(state.times_shown, 1);
        assert!(state.last_shown.is_some());
    }

    #[test]
    fn test_mark_issue_repaired() {
        let conn = create_test_db().unwrap();

        let issue = CaretakerIssue::new(
            IssueSeverity::Warning,
            "Time Sync Disabled",
            "NTP is not enabled",
            "Enable systemd-timesyncd"
        ).with_repair_action("time-sync-enable");

        update_issue_state(&conn, &issue).unwrap();
        mark_issue_repaired(&conn, "time-sync-disabled", true).unwrap();

        let state = get_issue_state(&conn, "time-sync-disabled")
            .unwrap()
            .unwrap();
        assert_eq!(state.repair_success, Some(true));
        assert!(state.last_repair_attempt.is_some());
    }

    #[test]
    fn test_should_deemphasize_critical_never() {
        let config = NoiseControlConfig::default();

        let state = IssueState {
            issue_key: "critical-issue".to_string(),
            first_seen: Utc::now() - Duration::days(30),
            last_seen: Utc::now(),
            last_shown: Some(Utc::now() - Duration::days(10)),
            times_shown: 5,
            times_ignored: 3,
            last_repair_attempt: None,
            repair_success: None,
            severity: IssueSeverity::Critical,
        };

        assert!(!should_deemphasize(&state, &config));
    }

    #[test]
    fn test_should_deemphasize_info_after_days() {
        let config = NoiseControlConfig::default();

        // Info issue shown 8 days ago (past threshold)
        let state_old = IssueState {
            issue_key: "info-issue".to_string(),
            first_seen: Utc::now() - Duration::days(20),
            last_seen: Utc::now(),
            last_shown: Some(Utc::now() - Duration::days(8)),
            times_shown: 3,
            times_ignored: 2,
            last_repair_attempt: None,
            repair_success: None,
            severity: IssueSeverity::Info,
        };

        assert!(!should_deemphasize(&state_old, &config));

        // Info issue shown 5 days ago (within threshold)
        let state_recent = IssueState {
            issue_key: "info-issue".to_string(),
            first_seen: Utc::now() - Duration::days(10),
            last_seen: Utc::now(),
            last_shown: Some(Utc::now() - Duration::days(5)),
            times_shown: 3,
            times_ignored: 2,
            last_repair_attempt: None,
            repair_success: None,
            severity: IssueSeverity::Info,
        };

        assert!(should_deemphasize(&state_recent, &config));
    }

    #[test]
    fn test_should_deemphasize_repaired() {
        let config = NoiseControlConfig::default();

        let state = IssueState {
            issue_key: "repaired-issue".to_string(),
            first_seen: Utc::now() - Duration::days(5),
            last_seen: Utc::now(),
            last_shown: Some(Utc::now() - Duration::days(1)),
            times_shown: 2,
            times_ignored: 0,
            last_repair_attempt: Some(Utc::now() - Duration::hours(1)),
            repair_success: Some(true),
            severity: IssueSeverity::Warning,
        };

        assert!(should_deemphasize(&state, &config));
    }
}
