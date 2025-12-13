//! Command Execution Logging - Phase 77
//!
//! Tracks commands executed by Anna for auditing, statistics, and learning.
//! VISION.md mentions Anna running commands and keeping track of actions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Execution result status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecStatus {
    Success,
    Failed,
    Timeout,
    Cancelled,
    Pending,
}

impl ExecStatus {
    pub fn symbol(&self) -> &'static str {
        match self {
            ExecStatus::Success => "+",
            ExecStatus::Failed => "x",
            ExecStatus::Timeout => "!",
            ExecStatus::Cancelled => "-",
            ExecStatus::Pending => "?",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ExecStatus::Success => "completed successfully",
            ExecStatus::Failed => "failed",
            ExecStatus::Timeout => "timed out",
            ExecStatus::Cancelled => "was cancelled",
            ExecStatus::Pending => "is pending",
        }
    }
}

/// Risk level for command execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandRisk {
    ReadOnly,
    LowRisk,
    MediumRisk,
    HighRisk,
    Critical,
}

impl CommandRisk {
    pub fn level(&self) -> u8 {
        match self {
            CommandRisk::ReadOnly => 0,
            CommandRisk::LowRisk => 1,
            CommandRisk::MediumRisk => 2,
            CommandRisk::HighRisk => 3,
            CommandRisk::Critical => 4,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CommandRisk::ReadOnly => "read-only",
            CommandRisk::LowRisk => "low risk",
            CommandRisk::MediumRisk => "medium risk",
            CommandRisk::HighRisk => "high risk",
            CommandRisk::Critical => "critical",
        }
    }
}

/// A single command execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// Unique execution ID
    pub id: String,
    /// The command that was executed
    pub command: String,
    /// Working directory
    pub working_dir: Option<String>,
    /// User who ran the command
    pub user: String,
    /// Whether elevated (sudo) was used
    pub elevated: bool,
    /// Execution status
    pub status: ExecStatus,
    /// Exit code if available
    pub exit_code: Option<i32>,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Timestamp when started
    pub started_at: u64,
    /// Associated ticket ID if any
    pub ticket_id: Option<String>,
    /// Risk level
    pub risk: CommandRisk,
    /// Output excerpt (truncated if long)
    pub output_excerpt: Option<String>,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Whether user confirmed before execution
    pub user_confirmed: bool,
}

/// Command execution tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionLog {
    /// All execution records
    pub records: Vec<ExecutionRecord>,
    /// Count by command pattern
    pub command_counts: HashMap<String, u64>,
    /// Success rate by command pattern
    pub success_counts: HashMap<String, u64>,
    /// Total elevated executions
    pub elevated_count: u64,
    /// Commands that failed most
    pub failure_counts: HashMap<String, u64>,
}

impl ExecutionLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a command execution
    pub fn record(&mut self, record: ExecutionRecord) {
        let pattern = extract_command_pattern(&record.command);

        *self.command_counts.entry(pattern.clone()).or_insert(0) += 1;

        if record.status == ExecStatus::Success {
            *self.success_counts.entry(pattern.clone()).or_insert(0) += 1;
        } else if record.status == ExecStatus::Failed {
            *self.failure_counts.entry(pattern.clone()).or_insert(0) += 1;
        }

        if record.elevated {
            self.elevated_count += 1;
        }

        self.records.push(record);
    }

    /// Get recent executions
    pub fn recent(&self, limit: usize) -> Vec<&ExecutionRecord> {
        self.records.iter().rev().take(limit).collect()
    }

    /// Get executions by status
    pub fn by_status(&self, status: ExecStatus) -> Vec<&ExecutionRecord> {
        self.records.iter().filter(|r| r.status == status).collect()
    }

    /// Get failed executions
    pub fn failed(&self) -> Vec<&ExecutionRecord> {
        self.by_status(ExecStatus::Failed)
    }

    /// Get elevated executions
    pub fn elevated(&self) -> Vec<&ExecutionRecord> {
        self.records.iter().filter(|r| r.elevated).collect()
    }

    /// Get high risk executions
    pub fn high_risk(&self) -> Vec<&ExecutionRecord> {
        self.records
            .iter()
            .filter(|r| r.risk.level() >= CommandRisk::HighRisk.level())
            .collect()
    }

    /// Total execution count
    pub fn total_count(&self) -> usize {
        self.records.len()
    }

    /// Success rate percentage
    pub fn success_rate(&self) -> f64 {
        let completed: usize = self
            .records
            .iter()
            .filter(|r| r.status == ExecStatus::Success || r.status == ExecStatus::Failed)
            .count();

        if completed == 0 {
            return 100.0;
        }

        let successful = self
            .records
            .iter()
            .filter(|r| r.status == ExecStatus::Success)
            .count();

        (successful as f64 / completed as f64) * 100.0
    }

    /// Average execution time in ms
    pub fn average_duration_ms(&self) -> u64 {
        if self.records.is_empty() {
            return 0;
        }
        let total: u64 = self.records.iter().map(|r| r.duration_ms).sum();
        total / self.records.len() as u64
    }

    /// Most used commands
    pub fn most_used(&self, limit: usize) -> Vec<(&str, u64)> {
        let mut commands: Vec<_> = self.command_counts.iter().collect();
        commands.sort_by(|a, b| b.1.cmp(a.1));
        commands.into_iter().take(limit).map(|(k, v)| (k.as_str(), *v)).collect()
    }

    /// Most failed commands
    pub fn most_failed(&self, limit: usize) -> Vec<(&str, u64)> {
        let mut commands: Vec<_> = self.failure_counts.iter().collect();
        commands.sort_by(|a, b| b.1.cmp(a.1));
        commands.into_iter().take(limit).map(|(k, v)| (k.as_str(), *v)).collect()
    }

    /// Commands for a specific ticket
    pub fn by_ticket(&self, ticket_id: &str) -> Vec<&ExecutionRecord> {
        self.records
            .iter()
            .filter(|r| r.ticket_id.as_deref() == Some(ticket_id))
            .collect()
    }
}

/// Extract command pattern (first word/command name)
fn extract_command_pattern(command: &str) -> String {
    let cmd = command.trim();
    // Handle sudo prefix
    let cmd = if cmd.starts_with("sudo ") {
        &cmd[5..]
    } else {
        cmd
    };

    // Get first word as pattern
    cmd.split_whitespace().next().unwrap_or("unknown").to_string()
}

/// Classify command risk level
pub fn classify_risk(command: &str) -> CommandRisk {
    let cmd = command.to_lowercase();

    // Critical commands - only exact root paths
    if (cmd.contains("rm -rf /") && !cmd.contains("rm -rf /tmp") && !cmd.contains("rm -rf /var") && !cmd.contains("rm -rf /home"))
        || cmd.contains("mkfs")
        || cmd.contains("dd if=")
        || cmd.contains("> /dev/")
    {
        return CommandRisk::Critical;
    }

    // High risk
    if cmd.starts_with("rm ")
        || cmd.contains("chmod")
        || cmd.contains("chown")
        || cmd.contains("systemctl stop")
        || cmd.contains("systemctl disable")
        || cmd.contains("kill ")
        || cmd.contains("pkill")
    {
        return CommandRisk::HighRisk;
    }

    // Medium risk
    if cmd.contains("pacman -S")
        || cmd.contains("apt install")
        || cmd.contains("dnf install")
        || cmd.contains("systemctl start")
        || cmd.contains("systemctl restart")
        || cmd.contains("pip install")
        || cmd.contains("npm install")
    {
        return CommandRisk::MediumRisk;
    }

    // Low risk
    if cmd.contains("echo ")
        || cmd.contains("printf")
        || cmd.contains("touch")
        || cmd.contains("mkdir")
    {
        return CommandRisk::LowRisk;
    }

    // Read-only
    if cmd.starts_with("cat ")
        || cmd.starts_with("ls")
        || cmd.starts_with("ps")
        || cmd.starts_with("df")
        || cmd.starts_with("du")
        || cmd.starts_with("free")
        || cmd.starts_with("top")
        || cmd.starts_with("htop")
        || cmd.starts_with("systemctl status")
        || cmd.starts_with("journalctl")
        || cmd.starts_with("uname")
        || cmd.starts_with("hostname")
        || cmd.starts_with("whoami")
        || cmd.starts_with("which")
        || cmd.starts_with("whereis")
        || cmd.starts_with("file ")
        || cmd.starts_with("head")
        || cmd.starts_with("tail")
        || cmd.starts_with("grep")
        || cmd.starts_with("find")
        || cmd.starts_with("locate")
    {
        return CommandRisk::ReadOnly;
    }

    CommandRisk::LowRisk
}

/// Format execution log for display
pub fn format_execution_log(log: &ExecutionLog) -> String {
    let mut lines = vec!["=== Command Execution Log ===".to_string()];
    lines.push(String::new());

    if log.records.is_empty() {
        lines.push("No commands executed yet.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total executions: {}", log.total_count()));
    lines.push(format!("Success rate: {:.1}%", log.success_rate()));
    lines.push(format!("Avg duration: {}ms", log.average_duration_ms()));
    lines.push(format!("Elevated (sudo): {}", log.elevated_count));

    // Most used
    let most_used = log.most_used(5);
    if !most_used.is_empty() {
        lines.push(String::new());
        lines.push("Most used commands:".to_string());
        for (cmd, count) in most_used {
            lines.push(format!("  {} ({} times)", cmd, count));
        }
    }

    // Recent executions
    let recent = log.recent(5);
    if !recent.is_empty() {
        lines.push(String::new());
        lines.push("Recent executions:".to_string());
        for exec in recent {
            let status = exec.status.symbol();
            let elevated = if exec.elevated { "[sudo]" } else { "" };
            lines.push(format!(
                "  [{}] {} {} ({}ms)",
                status, exec.command, elevated, exec.duration_ms
            ));
        }
    }

    // Failed commands
    let failed = log.most_failed(3);
    if !failed.is_empty() {
        lines.push(String::new());
        lines.push("Commands with failures:".to_string());
        for (cmd, count) in failed {
            lines.push(format!("  {} ({} failures)", cmd, count));
        }
    }

    lines.join("\n")
}

/// Format execution log compact
pub fn format_execution_log_compact(log: &ExecutionLog) -> String {
    format!(
        "Commands: {} ({:.1}% success) | Avg: {}ms | Sudo: {}",
        log.total_count(),
        log.success_rate(),
        log.average_duration_ms(),
        log.elevated_count
    )
}

/// Format execution log one-line
pub fn format_execution_log_oneline(log: &ExecutionLog) -> String {
    format!(
        "{} commands ({:.0}% ok)",
        log.total_count(),
        log.success_rate()
    )
}

/// Check if query is about execution log
pub fn is_execution_log_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "execution log",
        "command log",
        "commands executed",
        "commands run",
        "executed commands",
        "command history",
        "what commands",
        "ran commands",
        "execution history",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about executions
pub fn execution_fun_fact(log: &ExecutionLog) -> String {
    if log.records.is_empty() {
        return "No commands executed yet!".to_string();
    }

    let facts = [
        format!(
            "Anna has executed {} commands with a {:.1}% success rate.",
            log.total_count(),
            log.success_rate()
        ),
        format!(
            "{} commands were run with elevated privileges.",
            log.elevated_count
        ),
        {
            if let Some((cmd, count)) = log.most_used(1).first() {
                format!("The most frequently used command is '{}' ({} times).", cmd, count)
            } else {
                "No command patterns detected yet.".to_string()
            }
        },
        format!(
            "Average command execution time: {}ms.",
            log.average_duration_ms()
        ),
    ];

    facts[log.total_count() % facts.len()].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(command: &str, status: ExecStatus, elevated: bool) -> ExecutionRecord {
        ExecutionRecord {
            id: format!("exec-{}", command.len()),
            command: command.to_string(),
            working_dir: Some("/home/user".to_string()),
            user: "user".to_string(),
            elevated,
            status,
            exit_code: Some(if status == ExecStatus::Success { 0 } else { 1 }),
            duration_ms: 100,
            started_at: 1234567890,
            ticket_id: None,
            risk: classify_risk(command),
            output_excerpt: None,
            error_message: None,
            user_confirmed: true,
        }
    }

    #[test]
    fn test_exec_status() {
        assert_eq!(ExecStatus::Success.symbol(), "+");
        assert_eq!(ExecStatus::Failed.symbol(), "x");
    }

    #[test]
    fn test_command_risk() {
        assert_eq!(CommandRisk::ReadOnly.level(), 0);
        assert_eq!(CommandRisk::Critical.level(), 4);
    }

    #[test]
    fn test_classify_risk_readonly() {
        assert_eq!(classify_risk("ls -la"), CommandRisk::ReadOnly);
        assert_eq!(classify_risk("cat /etc/passwd"), CommandRisk::ReadOnly);
        assert_eq!(classify_risk("systemctl status nginx"), CommandRisk::ReadOnly);
    }

    #[test]
    fn test_classify_risk_critical() {
        assert_eq!(classify_risk("rm -rf /"), CommandRisk::Critical);
        assert_eq!(classify_risk("dd if=/dev/zero of=/dev/sda"), CommandRisk::Critical);
    }

    #[test]
    fn test_classify_risk_high() {
        assert_eq!(classify_risk("rm -rf /tmp/foo"), CommandRisk::HighRisk);
        assert_eq!(classify_risk("chmod 777 /var/www"), CommandRisk::HighRisk);
    }

    #[test]
    fn test_execution_log_record() {
        let mut log = ExecutionLog::new();
        log.record(make_record("ls -la", ExecStatus::Success, false));
        log.record(make_record("cat /etc/passwd", ExecStatus::Success, false));

        assert_eq!(log.total_count(), 2);
        assert_eq!(log.success_rate(), 100.0);
    }

    #[test]
    fn test_execution_log_success_rate() {
        let mut log = ExecutionLog::new();
        log.record(make_record("ls", ExecStatus::Success, false));
        log.record(make_record("cat foo", ExecStatus::Failed, false));

        assert_eq!(log.total_count(), 2);
        assert_eq!(log.success_rate(), 50.0);
    }

    #[test]
    fn test_execution_log_elevated() {
        let mut log = ExecutionLog::new();
        log.record(make_record("pacman -S vim", ExecStatus::Success, true));
        log.record(make_record("ls", ExecStatus::Success, false));

        assert_eq!(log.elevated_count, 1);
        assert_eq!(log.elevated().len(), 1);
    }

    #[test]
    fn test_most_used() {
        let mut log = ExecutionLog::new();
        log.record(make_record("ls -la", ExecStatus::Success, false));
        log.record(make_record("ls -l", ExecStatus::Success, false));
        log.record(make_record("cat /etc/passwd", ExecStatus::Success, false));

        let most_used = log.most_used(2);
        assert_eq!(most_used.len(), 2);
        assert_eq!(most_used[0].0, "ls");
        assert_eq!(most_used[0].1, 2);
    }

    #[test]
    fn test_extract_command_pattern() {
        assert_eq!(extract_command_pattern("ls -la /tmp"), "ls");
        assert_eq!(extract_command_pattern("sudo pacman -S vim"), "pacman");
        assert_eq!(extract_command_pattern("  cat /etc/passwd  "), "cat");
    }

    #[test]
    fn test_format_execution_log() {
        let mut log = ExecutionLog::new();
        log.record(make_record("ls -la", ExecStatus::Success, false));

        let output = format_execution_log(&log);
        assert!(output.contains("Command Execution Log"));
        assert!(output.contains("Total executions: 1"));
    }

    #[test]
    fn test_is_execution_log_query() {
        assert!(is_execution_log_query("show execution log"));
        assert!(is_execution_log_query("what commands have been executed?"));
        assert!(is_execution_log_query("command history"));
        assert!(!is_execution_log_query("what is my disk space?"));
    }

    #[test]
    fn test_execution_fun_fact() {
        let mut log = ExecutionLog::new();
        log.record(make_record("ls", ExecStatus::Success, false));

        let fact = execution_fun_fact(&log);
        assert!(!fact.is_empty());
    }

    #[test]
    fn test_format_compact_oneline() {
        let mut log = ExecutionLog::new();
        log.record(make_record("ls", ExecStatus::Success, false));

        let compact = format_execution_log_compact(&log);
        assert!(compact.contains("Commands: 1"));

        let oneline = format_execution_log_oneline(&log);
        assert!(oneline.contains("1 commands"));
    }
}
