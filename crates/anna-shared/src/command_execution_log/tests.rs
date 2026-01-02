//! Tests for command execution logging

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
