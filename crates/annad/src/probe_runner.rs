//! Probe execution with caching, timeouts, and redaction.

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

/// Run probes with individual timeouts, caching, and redaction
pub async fn run_probes(
    state: &SharedState,
    ticket: &TranslatorTicket,
    config: &LlmConfig,
    progress: &mut ProgressTracker,
) -> Vec<ProbeResult> {
    let mut results = Vec::new();

    for probe_id in &ticket.needs_probes {
        if let Some(cmd) = translator::probe_id_to_command(probe_id) {
            progress.add(ProgressEvent::probe_running(probe_id, progress.elapsed_ms()));
            progress.add_probe_start(probe_id, cmd);

            let cached = { state.read().await.get_cached_probe(cmd) };

            if let Some(mut cached_result) = cached {
                info!("Using cached probe: {}", cmd);
                // Redact cached output
                let (stdout, stderr) =
                    redact::redact_probe_output(&cached_result.stdout, &cached_result.stderr);
                cached_result.stdout = stdout;
                cached_result.stderr = stderr;
                let preview = cached_result.stdout.lines().next().map(|s| s.to_string());
                progress.add_probe_end(
                    probe_id,
                    cached_result.exit_code,
                    cached_result.timing_ms,
                    preview,
                );
                progress.add(ProgressEvent::probe_complete(
                    probe_id,
                    cached_result.exit_code,
                    cached_result.timing_ms,
                    progress.elapsed_ms(),
                ));
                results.push(cached_result);
            } else {
                let result = run_single_probe(cmd, config.probe_timeout_secs).await;
                let result = redact_probe_result(result);

                let preview = result.stdout.lines().next().map(|s| s.to_string());
                progress.add_probe_end(probe_id, result.exit_code, result.timing_ms, preview);
                progress.add(ProgressEvent::probe_complete(
                    probe_id,
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
    }

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
