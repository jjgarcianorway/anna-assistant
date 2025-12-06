//! Probe execution with caching, timeouts, and redaction.
//!
//! v0.45.6: Fixed silent no-op bug where unknown probe IDs were skipped without logging.
//! Now supports both translator probe IDs AND direct shell commands (from probe_spine).

use anna_shared::progress::ProgressEvent;
use anna_shared::rpc::{ProbeResult, TranslatorTicket};
use std::time::Instant;
use tokio::time::{timeout, Duration};
use tracing::{info, warn};

use crate::config::LlmConfig;
use crate::probes;
use crate::progress_tracker::ProgressTracker;
use crate::redact;
use crate::state::SharedState;
use crate::translator;

/// Resolve a probe specifier to an executable command.
/// v0.45.6: Supports three formats:
///   1. Translator probe IDs: "free", "cpu_info" → mapped via translator::probe_id_to_command
///   2. Direct shell commands: "lscpu", "free -b" → executed as-is if they look like commands
///   3. Unknown: returns None and logs warning
fn resolve_probe_command(probe_spec: &str) -> Option<String> {
    // First: try translator's probe ID mapping
    if let Some(cmd) = translator::probe_id_to_command(probe_spec) {
        return Some(cmd.to_string());
    }

    // Second: check if it looks like a direct shell command (from probe_spine)
    // Commands typically contain spaces, or are known executable names
    let known_commands = [
        "lscpu", "sensors", "free", "df", "lsblk", "lspci", "pactl", "ip", "ps",
        "systemctl", "journalctl", "pacman", "sh", "uname", "systemd-analyze",
    ];

    let first_word = probe_spec.split_whitespace().next().unwrap_or("");

    // If it starts with a known command or contains shell operators, treat as direct command
    if known_commands.iter().any(|&cmd| first_word == cmd || first_word.ends_with(cmd))
        || probe_spec.contains(' ')
        || probe_spec.contains('|')
        || probe_spec.contains('>')
    {
        return Some(probe_spec.to_string());
    }

    // Third: Unknown probe - log and return None
    warn!("v0.45.6: Unknown probe specifier '{}' - cannot resolve to command", probe_spec);
    None
}

/// Run probes with individual timeouts, caching, and redaction.
/// v0.45.6: Fixed silent no-op bug - now logs unknown probes.
pub async fn run_probes(
    state: &SharedState,
    ticket: &TranslatorTicket,
    config: &LlmConfig,
    progress: &mut ProgressTracker,
) -> Vec<ProbeResult> {
    let mut results = Vec::new();
    let mut unknown_probes: Vec<String> = Vec::new();

    info!("v0.45.6: Running {} planned probes", ticket.needs_probes.len());

    for probe_spec in &ticket.needs_probes {
        match resolve_probe_command(probe_spec) {
            Some(cmd) => {
                progress.add(ProgressEvent::probe_running(probe_spec, progress.elapsed_ms()));
                progress.add_probe_start(probe_spec, &cmd);

                let cached = { state.read().await.get_cached_probe(&cmd) };

                if let Some(mut cached_result) = cached {
                    info!("Using cached probe: {}", cmd);
                    // Redact cached output
                    let (stdout, stderr) =
                        redact::redact_probe_output(&cached_result.stdout, &cached_result.stderr);
                    cached_result.stdout = stdout;
                    cached_result.stderr = stderr;
                    let preview = cached_result.stdout.lines().next().map(|s| s.to_string());
                    progress.add_probe_end(
                        probe_spec,
                        cached_result.exit_code,
                        cached_result.timing_ms,
                        preview,
                    );
                    progress.add(ProgressEvent::probe_complete(
                        probe_spec,
                        cached_result.exit_code,
                        cached_result.timing_ms,
                        progress.elapsed_ms(),
                    ));
                    results.push(cached_result);
                } else {
                    let result = run_single_probe(&cmd, config.probe_timeout_secs).await;
                    let result = redact_probe_result(result);

                    let preview = result.stdout.lines().next().map(|s| s.to_string());
                    progress.add_probe_end(probe_spec, result.exit_code, result.timing_ms, preview);
                    progress.add(ProgressEvent::probe_complete(
                        probe_spec,
                        result.exit_code,
                        result.timing_ms,
                        progress.elapsed_ms(),
                    ));

                    if result.exit_code == 0 {
                        state.write().await.cache_probe(result.clone());
                    }
                    results.push(result);
                }
            }
            None => {
                // v0.45.6: Track unknown probes for reporting
                unknown_probes.push(probe_spec.clone());
                // Add a failed result for the unknown probe
                let result = ProbeResult {
                    command: probe_spec.clone(),
                    exit_code: -2, // Special code for "unknown probe"
                    stdout: String::new(),
                    stderr: format!("Unknown probe: {}", probe_spec),
                    timing_ms: 0,
                };
                progress.add(ProgressEvent::probe_complete(
                    probe_spec,
                    result.exit_code,
                    result.timing_ms,
                    progress.elapsed_ms(),
                ));
                results.push(result);
            }
        }
    }

    // v0.45.6: Log summary of unknown probes
    if !unknown_probes.is_empty() {
        warn!(
            "v0.45.6: {} probe(s) could not be resolved: {:?}",
            unknown_probes.len(),
            unknown_probes
        );
    }

    let executed = results.iter().filter(|r| r.exit_code != -2).count();
    let succeeded = results.iter().filter(|r| r.exit_code == 0).count();
    info!(
        "v0.45.6: Probes complete - planned={}, executed={}, succeeded={}, unknown={}",
        ticket.needs_probes.len(),
        executed,
        succeeded,
        unknown_probes.len()
    );

    {
        state.write().await.clean_probe_cache();
    }
    results
}

/// Run a single probe command with timeout
async fn run_single_probe(cmd: &str, timeout_secs: u64) -> ProbeResult {
    let probe_start = Instant::now();
    let cmd_owned = cmd.to_string();

    match timeout(
        Duration::from_secs(timeout_secs),
        tokio::task::spawn_blocking(move || probes::run_command_structured(&cmd_owned)),
    )
    .await
    {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => {
            warn!("Probe task error: {}", e);
            ProbeResult {
                command: cmd.to_string(),
                exit_code: -1,
                stdout: String::new(),
                stderr: format!("Task error: {}", e),
                timing_ms: probe_start.elapsed().as_millis() as u64,
            }
        }
        Err(_) => {
            warn!("Probe timeout: {}", cmd);
            ProbeResult {
                command: cmd.to_string(),
                exit_code: -1,
                stdout: String::new(),
                stderr: "Probe timeout".to_string(),
                timing_ms: timeout_secs * 1000,
            }
        }
    }
}

/// Redact sensitive content from probe output
fn redact_probe_result(mut result: ProbeResult) -> ProbeResult {
    let (stdout, stderr) = redact::redact_probe_output(&result.stdout, &result.stderr);
    result.stdout = stdout;
    result.stderr = stderr;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_translator_probe_id() {
        // Translator probe IDs should map to commands
        assert_eq!(resolve_probe_command("free"), Some("free -h".to_string()));
        assert_eq!(resolve_probe_command("cpu_info"), Some("lscpu".to_string()));
        assert_eq!(resolve_probe_command("top_memory"), Some("ps aux --sort=-%mem".to_string()));
    }

    #[test]
    fn test_resolve_direct_shell_command() {
        // Direct shell commands from probe_spine should be executed as-is
        assert_eq!(resolve_probe_command("lscpu"), Some("lscpu".to_string()));
        assert_eq!(resolve_probe_command("free -b"), Some("free -b".to_string()));
        assert_eq!(resolve_probe_command("df -h"), Some("df -h".to_string()));
        assert_eq!(
            resolve_probe_command("sh -lc 'command -v nano'"),
            Some("sh -lc 'command -v nano'".to_string())
        );
        assert_eq!(
            resolve_probe_command("lspci | grep -i audio"),
            Some("lspci | grep -i audio".to_string())
        );
        assert_eq!(
            resolve_probe_command("pactl list cards 2>/dev/null || true"),
            Some("pactl list cards 2>/dev/null || true".to_string())
        );
    }

    #[test]
    fn test_resolve_unknown_probe() {
        // Unknown probes should return None
        assert_eq!(resolve_probe_command("completely_unknown_probe"), None);
        assert_eq!(resolve_probe_command("invalid"), None);
    }

    #[test]
    fn test_resolve_probe_spine_commands() {
        // Commands generated by probe_to_command() in probe_spine.rs
        assert_eq!(resolve_probe_command("sensors"), Some("sensors".to_string()));
        assert_eq!(
            resolve_probe_command("journalctl -p err -b --no-pager -o json | head -50"),
            Some("journalctl -p err -b --no-pager -o json | head -50".to_string())
        );
        assert_eq!(
            resolve_probe_command("systemctl --failed --no-pager"),
            Some("systemctl --failed --no-pager".to_string())
        );
        assert_eq!(
            resolve_probe_command("pacman -Q nano 2>/dev/null"),
            Some("pacman -Q nano 2>/dev/null".to_string())
        );
    }
}
