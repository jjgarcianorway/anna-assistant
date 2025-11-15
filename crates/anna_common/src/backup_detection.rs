use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupDetection {
    pub installed_tools: Vec<BackupTool>,
    pub last_backups: HashMap<String, LastBackup>,
    pub integrity_errors: Vec<BackupIntegrityError>,
    pub missing_snapshots: Vec<MissingSnapshot>,
    pub overall_status: BackupStatus,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupTool {
    pub name: String,
    pub tool_type: BackupToolType,
    pub installed: bool,
    pub version: Option<String>,
    pub config_path: Option<String>,
    pub config_exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackupToolType {
    Snapshot,     // timeshift, snapper
    Incremental,  // rsnapshot, duplicity
    Deduplication, // borg, restic
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastBackup {
    pub tool: String,
    pub timestamp: Option<String>,
    pub age_hours: Option<u64>,
    pub status: String,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupIntegrityError {
    pub tool: String,
    pub error_type: String,
    pub message: String,
    pub severity: ErrorSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingSnapshot {
    pub tool: String,
    pub expected_interval_hours: u64,
    pub last_seen_hours: Option<u64>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackupStatus {
    Healthy,          // Backups exist and are recent
    Warning,          // Backups exist but are old
    Critical,         // No backups or severe errors
    NoBackupTool,     // No backup tools installed
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorSeverity {
    Critical,
    Warning,
    Info,
}

impl BackupDetection {
    pub fn detect() -> Self {
        let installed_tools = detect_backup_tools();
        let last_backups = detect_last_backups(&installed_tools);
        let integrity_errors = check_backup_integrity(&installed_tools);
        let missing_snapshots = detect_missing_snapshots(&installed_tools, &last_backups);

        let overall_status = calculate_backup_status(&installed_tools, &last_backups, &integrity_errors);
        let recommendations = generate_recommendations(&overall_status, &installed_tools, &last_backups);

        BackupDetection {
            installed_tools,
            last_backups,
            integrity_errors,
            missing_snapshots,
            overall_status,
            recommendations,
        }
    }

    pub fn has_any_backup_tool(&self) -> bool {
        self.installed_tools.iter().any(|t| t.installed)
    }

    pub fn has_recent_backup(&self, max_age_hours: u64) -> bool {
        self.last_backups.values().any(|backup| {
            backup.age_hours.map(|age| age <= max_age_hours).unwrap_or(false)
        })
    }

    pub fn health_score(&self) -> u8 {
        match self.overall_status {
            BackupStatus::Healthy => 100,
            BackupStatus::Warning => 60,
            BackupStatus::Critical => 20,
            BackupStatus::NoBackupTool => 0,
        }
    }
}

fn detect_backup_tools() -> Vec<BackupTool> {
    let tools = vec![
        ("timeshift", BackupToolType::Snapshot, "/etc/timeshift/timeshift.json"),
        ("snapper", BackupToolType::Snapshot, "/etc/snapper/configs"),
        ("rsnapshot", BackupToolType::Incremental, "/etc/rsnapshot.conf"),
        ("borg", BackupToolType::Deduplication, ""),
        ("restic", BackupToolType::Deduplication, ""),
        ("duplicity", BackupToolType::Incremental, ""),
    ];

    tools.iter().map(|(name, tool_type, config)| {
        let installed = is_command_available(name);
        let version = if installed {
            get_tool_version(name)
        } else {
            None
        };

        let config_path = if !config.is_empty() {
            Some(config.to_string())
        } else {
            None
        };

        let config_exists = config_path.as_ref()
            .map(|p| Path::new(p).exists())
            .unwrap_or(false);

        BackupTool {
            name: name.to_string(),
            tool_type: tool_type.clone(),
            installed,
            version,
            config_path,
            config_exists,
        }
    }).collect()
}

fn is_command_available(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn get_tool_version(tool: &str) -> Option<String> {
    let version_output = match tool {
        "timeshift" => Command::new(tool).arg("--version").output().ok()?,
        "snapper" => Command::new(tool).arg("--version").output().ok()?,
        "rsnapshot" => return None, // rsnapshot doesn't have a simple version flag
        "borg" => Command::new(tool).arg("--version").output().ok()?,
        "restic" => Command::new(tool).arg("version").output().ok()?,
        "duplicity" => Command::new(tool).arg("--version").output().ok()?,
        _ => return None,
    };

    String::from_utf8(version_output.stdout)
        .ok()
        .map(|s| s.lines().next().unwrap_or("").trim().to_string())
        .filter(|s| !s.is_empty())
}

fn detect_last_backups(tools: &[BackupTool]) -> HashMap<String, LastBackup> {
    let mut backups = HashMap::new();

    for tool in tools.iter().filter(|t| t.installed) {
        if let Some(last_backup) = get_last_backup_for_tool(&tool.name) {
            backups.insert(tool.name.clone(), last_backup);
        }
    }

    backups
}

fn get_last_backup_for_tool(tool: &str) -> Option<LastBackup> {
    match tool {
        "timeshift" => detect_timeshift_last_backup(),
        "snapper" => detect_snapper_last_backup(),
        "rsnapshot" => detect_rsnapshot_last_backup(),
        "borg" => detect_borg_last_backup(),
        "restic" => detect_restic_last_backup(),
        "duplicity" => None, // Would need to check backup destination
        _ => None,
    }
}

fn detect_timeshift_last_backup() -> Option<LastBackup> {
    let output = Command::new("timeshift")
        .arg("--list")
        .arg("--scripted")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let list = String::from_utf8(output.stdout).ok()?;
    // Parse timeshift list output for most recent snapshot
    let first_line = list.lines()
        .filter(|line| !line.is_empty() && !line.starts_with("Name"))
        .next()?;

    Some(LastBackup {
        tool: "timeshift".to_string(),
        timestamp: Some(first_line.to_string()),
        age_hours: None, // Would need to parse timestamp
        status: "exists".to_string(),
        location: Some("/run/timeshift/backup".to_string()),
    })
}

fn detect_snapper_last_backup() -> Option<LastBackup> {
    let output = Command::new("snapper")
        .arg("list")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let list = String::from_utf8(output.stdout).ok()?;
    let last_snapshot = list.lines()
        .filter(|line| !line.is_empty() && !line.starts_with("#") && !line.contains("Type"))
        .last()?;

    Some(LastBackup {
        tool: "snapper".to_string(),
        timestamp: Some(last_snapshot.to_string()),
        age_hours: None,
        status: "exists".to_string(),
        location: Some("/.snapshots".to_string()),
    })
}

fn detect_rsnapshot_last_backup() -> Option<LastBackup> {
    // Check for rsnapshot backup directories
    let snapshot_root = Path::new("/var/cache/rsnapshot");
    if !snapshot_root.exists() {
        return None;
    }

    // Look for most recent backup directory
    let entries = std::fs::read_dir(snapshot_root).ok()?;
    let mut most_recent: Option<std::fs::DirEntry> = None;

    for entry in entries.flatten() {
        if let Ok(metadata) = entry.metadata() {
            if metadata.is_dir() {
                if most_recent.is_none() {
                    most_recent = Some(entry);
                } else if let Some(ref current) = most_recent {
                    if let (Ok(entry_time), Ok(current_time)) =
                        (metadata.modified(), current.metadata().and_then(|m| m.modified())) {
                        if entry_time > current_time {
                            most_recent = Some(entry);
                        }
                    }
                }
            }
        }
    }

    most_recent.map(|entry| LastBackup {
        tool: "rsnapshot".to_string(),
        timestamp: entry.file_name().to_string_lossy().to_string().into(),
        age_hours: None,
        status: "exists".to_string(),
        location: Some(snapshot_root.to_string_lossy().to_string()),
    })
}

fn detect_borg_last_backup() -> Option<LastBackup> {
    // Would need to know the repository location
    // Check common locations or config files
    None
}

fn detect_restic_last_backup() -> Option<LastBackup> {
    // Would need to know the repository location
    // Check common locations or config files
    None
}

fn check_backup_integrity(tools: &[BackupTool]) -> Vec<BackupIntegrityError> {
    let mut errors = Vec::new();

    for tool in tools.iter().filter(|t| t.installed) {
        match tool.name.as_str() {
            "timeshift" => {
                // Check if timeshift can access backup location
                if let Some(timeshift_errors) = check_timeshift_integrity() {
                    errors.extend(timeshift_errors);
                }
            },
            "snapper" => {
                // Check snapper configs
                if let Some(snapper_errors) = check_snapper_integrity() {
                    errors.extend(snapper_errors);
                }
            },
            _ => {}
        }
    }

    errors
}

fn check_timeshift_integrity() -> Option<Vec<BackupIntegrityError>> {
    let backup_path = Path::new("/run/timeshift/backup");
    if !backup_path.exists() {
        return Some(vec![BackupIntegrityError {
            tool: "timeshift".to_string(),
            error_type: "missing_backup_location".to_string(),
            message: "Timeshift backup location does not exist".to_string(),
            severity: ErrorSeverity::Warning,
        }]);
    }
    None
}

fn check_snapper_integrity() -> Option<Vec<BackupIntegrityError>> {
    let snapshots_path = Path::new("/.snapshots");
    if !snapshots_path.exists() {
        return Some(vec![BackupIntegrityError {
            tool: "snapper".to_string(),
            error_type: "missing_snapshots".to_string(),
            message: "Snapper snapshots directory does not exist".to_string(),
            severity: ErrorSeverity::Warning,
        }]);
    }
    None
}

fn detect_missing_snapshots(
    tools: &[BackupTool],
    last_backups: &HashMap<String, LastBackup>,
) -> Vec<MissingSnapshot> {
    let mut missing = Vec::new();

    // Check for tools that are installed but have no recent backups
    for tool in tools.iter().filter(|t| t.installed) {
        if !last_backups.contains_key(&tool.name) {
            missing.push(MissingSnapshot {
                tool: tool.name.clone(),
                expected_interval_hours: match tool.tool_type {
                    BackupToolType::Snapshot => 24,      // Daily snapshots
                    BackupToolType::Incremental => 168,  // Weekly incrementals
                    BackupToolType::Deduplication => 168, // Weekly deduplicated
                },
                last_seen_hours: None,
                description: format!("{} is installed but no backups detected", tool.name),
            });
        }
    }

    missing
}

fn calculate_backup_status(
    tools: &[BackupTool],
    last_backups: &HashMap<String, LastBackup>,
    errors: &[BackupIntegrityError],
) -> BackupStatus {
    let has_installed_tools = tools.iter().any(|t| t.installed);

    if !has_installed_tools {
        return BackupStatus::NoBackupTool;
    }

    let has_critical_errors = errors.iter().any(|e| e.severity == ErrorSeverity::Critical);
    if has_critical_errors {
        return BackupStatus::Critical;
    }

    if last_backups.is_empty() {
        return BackupStatus::Critical;
    }

    // Check if any backup is recent (within 7 days)
    let has_recent = last_backups.values().any(|backup| {
        backup.age_hours.map(|age| age <= 168).unwrap_or(false)
    });

    if has_recent {
        BackupStatus::Healthy
    } else {
        BackupStatus::Warning
    }
}

fn generate_recommendations(
    status: &BackupStatus,
    tools: &[BackupTool],
    last_backups: &HashMap<String, LastBackup>,
) -> Vec<String> {
    let mut recommendations = Vec::new();

    match status {
        BackupStatus::NoBackupTool => {
            recommendations.push("No backup tools installed. Consider installing timeshift or snapper for system snapshots".to_string());
        },
        BackupStatus::Critical => {
            if tools.iter().any(|t| t.installed) {
                recommendations.push("Backup tools are installed but no backups found. Create an initial backup immediately".to_string());
            } else {
                recommendations.push("Critical: No backup system configured. Your data is at risk".to_string());
            }
        },
        BackupStatus::Warning => {
            recommendations.push("Backups exist but may be outdated. Consider running a fresh backup".to_string());
        },
        BackupStatus::Healthy => {
            if last_backups.len() == 1 {
                recommendations.push("Consider setting up a secondary backup tool for redundancy".to_string());
            }
        },
    }

    // Check for tools with configs but not running
    for tool in tools.iter().filter(|t| t.installed && t.config_exists) {
        if !last_backups.contains_key(&tool.name) {
            recommendations.push(format!("{} is configured but not creating backups", tool.name));
        }
    }

    recommendations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_detection() {
        let detection = BackupDetection::detect();
        assert!(detection.health_score() <= 100);
    }

    #[test]
    fn test_backup_status_calculation() {
        let tools = vec![BackupTool {
            name: "timeshift".to_string(),
            tool_type: BackupToolType::Snapshot,
            installed: true,
            version: Some("1.0".to_string()),
            config_path: None,
            config_exists: false,
        }];

        let backups = HashMap::new();
        let errors = Vec::new();

        let status = calculate_backup_status(&tools, &backups, &errors);
        assert_eq!(status, BackupStatus::Critical);
    }
}
