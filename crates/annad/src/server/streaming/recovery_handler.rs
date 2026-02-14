//! Auto-recovery handler: when infra fails mid-query, waits for recovery
//! and re-executes the question automatically — no "try again" needed.

use anna_shared::outcome_ledger::{append_outcome, Outcome, OutcomeRecord, RequestMode};
use anna_shared::rpc::{StreamingResponse};
use anna_shared::intent_class::IntentClass;
use tokio::io::AsyncWriteExt;
use tracing::{info, warn};

use crate::ralph;
use crate::state::SharedState;

const MAX_RECOVERY_SECS: u64 = 600; // 10 min max wait

/// Handle an infra failure (ollama down, model gone) by:
/// 1. Immediately triggering recovery (clear model → init loop re-runs)
/// 2. Streaming live init_status tokens to the user on the same connection
/// 3. Once model is back, automatically re-executing the original question
pub async fn handle_infra_failure_and_recover(
    error: &anyhow::Error,
    question: &str,
    session_id: &str,
    state: SharedState,
    writer: &mut tokio::net::unix::OwnedWriteHalf,
    request_id: &str,
    intent: IntentClass,
    start_time: std::time::Instant,
) -> anyhow::Result<()> {
    warn!("Infra failure during query, triggering self-healing and waiting for recovery: {}", error);

    // Immediately clear model so the init loop re-triggers recovery
    {
        let mut guard = state.write().await;
        guard.model = None;
        guard.state = anna_shared::status::DaemonState::Starting;
        guard.init_status = "Ollama went offline — restarting automatically...".to_string();
    }

    // Stream live recovery progress to the user (same channel, no reconnect needed)
    let mut last_status = String::new();
    let recovery_start = std::time::Instant::now();

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        if recovery_start.elapsed().as_secs() >= MAX_RECOVERY_SECS {
            let msg = "Recovery is taking longer than expected. Anna is still working on it — please re-send your question.";
            let token = StreamingResponse::Token { token: msg.to_string() };
            if let Ok(json) = serde_json::to_string(&token) {
                let _ = writer.write_all(format!("{}\n", json).as_bytes()).await;
            }
            let result = anna_shared::rpc::AskResult {
                answer: msg.to_string(),
                success: false,
                iterations: 0,
                commands_executed: vec![],
                dialogue: vec![],
                needs_clarification: false,
                clarification_question: None,
                cached: false,
                citations: vec![],
                abstained: false,
                final_confidence: None,
            };
            let done = StreamingResponse::Done { result };
            if let Ok(json) = serde_json::to_string(&done) {
                let _ = writer.write_all(format!("{}\n", json).as_bytes()).await;
            }
            record_outcome(request_id, intent, Outcome::Failed, start_time);
            let _ = writer.flush().await;
            return Ok(());
        }

        let (model_ready, current_status) = {
            let s = state.read().await;
            (s.model.clone(), s.init_status.clone())
        };

        // Stream status update if it changed
        if current_status != last_status && !current_status.is_empty() {
            last_status = current_status.clone();
            let token = StreamingResponse::Token { token: format!("{} ", current_status) };
            if let Ok(json) = serde_json::to_string(&token) {
                let _ = writer.write_all(format!("{}\n", json).as_bytes()).await;
                let _ = writer.flush().await;
            }
        }

        if let Some(recovered_model) = model_ready {
            // Recovery complete — re-run the question automatically
            info!("Recovery complete ({}), re-executing question", recovered_model);
            let retry_token = StreamingResponse::Token {
                token: "\nRecovered. Answering your question now...\n".to_string(),
            };
            if let Ok(json) = serde_json::to_string(&retry_token) {
                let _ = writer.write_all(format!("{}\n", json).as_bytes()).await;
            }

            let retry_result = ralph::ralph_loop_streaming(
                &recovered_model, question, session_id, writer,
            ).await;

            if let Ok(ref ask_result) = retry_result {
                let mut state_guard = state.write().await;
                if let Some(session) = state_guard.sessions.sessions.get_mut(session_id) {
                    session.add_turn(question, &ask_result.answer, ask_result.commands_executed.clone());
                }
                if ask_result.success && !ask_result.needs_clarification && !ask_result.answer.is_empty() {
                    state_guard.cache_answer(question, &ask_result.answer);
                }
            }

            let outcome = if retry_result.as_ref().map(|r| r.success).unwrap_or(false) {
                Outcome::Resolved
            } else {
                Outcome::Failed
            };
            record_outcome(request_id, intent, outcome, start_time);
            let _ = writer.flush().await;
            return Ok(());
        }
    }
}

fn record_outcome(
    request_id: &str,
    intent: IntentClass,
    outcome: Outcome,
    start_time: std::time::Instant,
) {
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let record = OutcomeRecord::new(request_id, RequestMode::Dialogue, intent, outcome, false, duration_ms);
    let _ = append_outcome(&record);
}
