//! Progress tracker with transcript building for request handling.
//!
//! v0.0.241: Added shared events for streaming token support.
//! v0.0.247: Streaming events shared with daemon state for live polling.
//! v0.0.248: Push internal comms and stage events to streaming for real-time visibility.
//! v0.0.825: Use tokio::sync::Mutex for async-safe streaming events.

use anna_shared::progress::{ProgressEvent, RequestStage};
use anna_shared::transcript::{Actor, StageOutcome, Transcript, TranscriptEvent};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, RwLock};
use tracing::info;

/// Progress tracker for request handling with transcript building
pub struct ProgressTracker {
    events: Vec<ProgressEvent>,
    /// Shared events from streaming (can be pushed to from callbacks)
    /// v0.0.247: This Arc is shared with daemon state for live polling
    /// v0.0.825: Use tokio::sync::Mutex for async-safe access
    streaming_events: Arc<Mutex<Vec<ProgressEvent>>>,
    transcript: Transcript,
    start_time: Instant,
    current_stage: Option<RequestStage>,
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressTracker {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            streaming_events: Arc::new(Mutex::new(Vec::new())),
            transcript: Transcript::new(),
            start_time: Instant::now(),
            current_stage: None,
        }
    }

    /// v0.0.247: Create with shared streaming events from daemon state
    /// This allows RPC handler to poll streaming events in real-time
    /// v0.0.825: Updated for tokio::sync::Mutex
    pub fn with_streaming_events(streaming_events: Arc<Mutex<Vec<ProgressEvent>>>) -> Self {
        // Note: We can't clear here synchronously with tokio Mutex
        // The clearing will happen in start_stage or via explicit reset
        Self {
            events: Vec::new(),
            streaming_events,
            transcript: Transcript::new(),
            start_time: Instant::now(),
            current_stage: None,
        }
    }

    /// v0.0.825: Clear streaming events (call this at start of new request)
    pub async fn clear_streaming_events(&self) {
        let mut events = self.streaming_events.lock().await;
        events.clear();
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }

    pub fn add(&mut self, event: ProgressEvent) {
        info!("{}", event.format_debug());
        self.events.push(event);
    }

    /// v0.0.825: Async version of start_stage that properly uses tokio Mutex
    pub async fn start_stage_async(&mut self, stage: RequestStage, timeout_secs: u64) {
        self.current_stage = Some(stage);
        let event = ProgressEvent::starting(stage, timeout_secs, self.elapsed_ms());
        self.add(event.clone());
        // Push to streaming events for real-time client visibility
        {
            let mut streaming = self.streaming_events.lock().await;
            streaming.push(event);
        }
        let stage_name = format!("{:?}", stage).to_lowercase();
        self.transcript
            .push(TranscriptEvent::stage_start(self.elapsed_ms(), stage_name));
    }

    /// Sync version that spawns the async work (for non-async contexts)
    pub fn start_stage(&mut self, stage: RequestStage, timeout_secs: u64) {
        self.current_stage = Some(stage);
        let event = ProgressEvent::starting(stage, timeout_secs, self.elapsed_ms());
        self.add(event.clone());
        // Try to push to streaming events (best effort in sync context)
        let streaming = self.streaming_events.clone();
        let event_clone = event;
        tokio::spawn(async move {
            let mut events = streaming.lock().await;
            events.push(event_clone);
        });
        let stage_name = format!("{:?}", stage).to_lowercase();
        self.transcript
            .push(TranscriptEvent::stage_start(self.elapsed_ms(), stage_name));
    }

    /// v0.0.825: Async version of complete_stage
    pub async fn complete_stage_async(&mut self, stage: RequestStage) {
        let event = ProgressEvent::complete(stage, self.elapsed_ms());
        self.add(event.clone());
        {
            let mut streaming = self.streaming_events.lock().await;
            streaming.push(event);
        }
        let stage_name = format!("{:?}", stage).to_lowercase();
        self.transcript.push(TranscriptEvent::stage_end(
            self.elapsed_ms(),
            stage_name,
            StageOutcome::Ok,
        ));
        self.current_stage = None;
    }

    pub fn complete_stage(&mut self, stage: RequestStage) {
        let event = ProgressEvent::complete(stage, self.elapsed_ms());
        self.add(event.clone());
        // Try to push to streaming events (best effort in sync context)
        let streaming = self.streaming_events.clone();
        let event_clone = event;
        tokio::spawn(async move {
            let mut events = streaming.lock().await;
            events.push(event_clone);
        });
        let stage_name = format!("{:?}", stage).to_lowercase();
        self.transcript.push(TranscriptEvent::stage_end(
            self.elapsed_ms(),
            stage_name,
            StageOutcome::Ok,
        ));
        self.current_stage = None;
    }

    pub fn timeout_stage(&mut self, stage: RequestStage) {
        self.add(ProgressEvent::timeout(stage, self.elapsed_ms()));
        let stage_name = format!("{:?}", stage).to_lowercase();
        self.transcript.push(TranscriptEvent::stage_end(
            self.elapsed_ms(),
            stage_name,
            StageOutcome::Timeout,
        ));
        self.current_stage = None;
    }

    pub fn error_stage(&mut self, stage: RequestStage, error: &str) {
        let stage_name = format!("{:?}", stage).to_lowercase();
        self.transcript.push(TranscriptEvent::stage_end(
            self.elapsed_ms(),
            stage_name,
            StageOutcome::Error,
        ));
        self.transcript
            .push(TranscriptEvent::note(self.elapsed_ms(), error));
        self.current_stage = None;
    }

    /// Mark stage as skipped because deterministic router answered
    pub fn skip_stage_deterministic(&mut self, stage: RequestStage) {
        let stage_name = format!("{:?}", stage).to_lowercase();
        self.transcript.push(TranscriptEvent::stage_end(
            self.elapsed_ms(),
            stage_name,
            StageOutcome::Deterministic,
        ));
        self.current_stage = None;
    }

    pub fn add_user_message(&mut self, text: &str) {
        self.transcript.push(TranscriptEvent::message(
            self.elapsed_ms(),
            Actor::You,
            Actor::Anna,
            text,
        ));
    }

    pub fn add_translator_message(&mut self, text: &str) {
        self.transcript.push(TranscriptEvent::message(
            self.elapsed_ms(),
            Actor::Translator,
            Actor::Dispatcher,
            text,
        ));
    }

    pub fn add_probe_start(&mut self, probe_id: &str, command: &str) {
        self.transcript.push(TranscriptEvent::probe_start(
            self.elapsed_ms(),
            probe_id,
            command,
        ));
    }

    pub fn add_probe_end(
        &mut self,
        probe_id: &str,
        exit_code: i32,
        timing_ms: u64,
        preview: Option<String>,
    ) {
        self.transcript.push(TranscriptEvent::probe_end(
            self.elapsed_ms(),
            probe_id,
            exit_code,
            timing_ms,
            preview,
        ));
    }

    pub fn add_specialist_message(&mut self, text: &str) {
        self.transcript.push(TranscriptEvent::message(
            self.elapsed_ms(),
            Actor::Specialist,
            Actor::Supervisor,
            text,
        ));
    }

    /// v0.0.143: Add streaming generation progress note
    pub fn add_generation_progress(&mut self, tokens: usize) {
        self.transcript.push(TranscriptEvent::note(
            self.elapsed_ms(),
            format!("Generated {} tokens...", tokens),
        ));
    }

    /// v0.0.145: Add LLM generation progress event (for client polling)
    pub fn add_generation_event(&mut self, stage: RequestStage, tokens: usize) {
        self.add(ProgressEvent::generation(stage, tokens, self.elapsed_ms()));
    }

    /// v0.0.145: Add internal comms message (IT staff chatter)
    /// v0.0.248: Also push to streaming events for real-time visibility
    /// v0.0.825: Use tokio spawn for async Mutex access
    pub fn add_internal_comms(&mut self, stage: RequestStage, from: &str, message: &str) {
        let event = ProgressEvent::internal_comms(stage, from, message, self.elapsed_ms());
        self.add(event.clone());
        // Push to streaming events (best effort in sync context)
        let streaming = self.streaming_events.clone();
        let event_clone = event;
        tokio::spawn(async move {
            let mut events = streaming.lock().await;
            events.push(event_clone);
        });
    }

    /// v0.0.238: Add streaming token for real-time output
    pub fn add_streaming_token(&mut self, stage: RequestStage, token: &str, is_final: bool) {
        // Don't log individual tokens to avoid noise
        self.events.push(ProgressEvent::streaming_token(
            stage,
            token,
            is_final,
            self.elapsed_ms(),
        ));
    }

    /// v0.0.241: Get streaming sink for use in callbacks
    /// Returns a clone of the shared events vector for thread-safe push access
    pub fn streaming_sink(&self) -> StreamingSink {
        StreamingSink {
            events: Arc::clone(&self.streaming_events),
            start_time: self.start_time,
        }
    }

    /// Record Anna's final answer (THE authoritative response to the user)
    /// Uses FinalAnswer kind, not Message, to ensure proper answer source detection.
    pub fn add_final_answer(&mut self, text: &str) {
        self.transcript
            .push(TranscriptEvent::final_answer(self.elapsed_ms(), text));
    }

    /// v0.0.302: Record LLM call details (debug mode only)
    pub fn add_llm_call(
        &mut self,
        stage: &str,
        model: &str,
        prompt: &str,
        response: &str,
        duration_ms: u64,
        tokens: Option<u32>,
    ) {
        self.transcript.push(TranscriptEvent::llm_call(
            self.elapsed_ms(),
            stage,
            model,
            prompt,
            response,
            duration_ms,
            tokens,
        ));
    }

    /// v0.0.241: Get all events (including streaming events)
    /// Merges main events with streaming events
    /// v0.0.825: Async version for tokio Mutex
    pub async fn events_async(&self) -> Vec<ProgressEvent> {
        let mut all = self.events.clone();
        let streaming = self.streaming_events.lock().await;
        all.extend(streaming.iter().cloned());
        // Sort by elapsed_ms to maintain temporal order
        all.sort_by_key(|e| e.elapsed_ms);
        all
    }

    /// Sync version - returns only local events (streaming events may be missed)
    pub fn events(&self) -> Vec<ProgressEvent> {
        self.events.clone()
    }

    pub fn take_transcript(self) -> Transcript {
        self.transcript
    }

    pub fn transcript_clone(&self) -> Transcript {
        self.transcript.clone()
    }

    /// Get mutable reference to transcript (v0.0.39)
    pub fn transcript_mut(&mut self) -> &mut Transcript {
        &mut self.transcript
    }
}

/// Shared progress state for polling (reserved for future watchdog use)
#[allow(dead_code)]
pub type SharedProgress = Arc<RwLock<ProgressTracker>>;

#[allow(dead_code)]
pub fn create_progress_tracker() -> SharedProgress {
    Arc::new(RwLock::new(ProgressTracker::new()))
}

/// v0.0.241: Thread-safe sink for streaming tokens
/// Can be cloned and passed to callbacks without holding a mutable borrow
/// v0.0.825: Uses tokio::sync::Mutex for async-safe access
#[derive(Clone)]
pub struct StreamingSink {
    events: Arc<Mutex<Vec<ProgressEvent>>>,
    start_time: Instant,
}

impl StreamingSink {
    /// Push a streaming token event (async version)
    pub async fn push_token_async(&self, stage: RequestStage, token: &str, is_final: bool) {
        let elapsed = self.start_time.elapsed().as_millis() as u64;
        let mut events = self.events.lock().await;
        events.push(ProgressEvent::streaming_token(
            stage, token, is_final, elapsed,
        ));
        // v0.0.248: Debug logging for streaming verification
        if events.len() % 10 == 0 || is_final {
            tracing::debug!(
                "Streaming: {} tokens pushed, final={}",
                events.len(),
                is_final
            );
        }
    }

    /// Push a streaming token event (spawns async task)
    pub fn push_token(&self, stage: RequestStage, token: &str, is_final: bool) {
        let elapsed = self.start_time.elapsed().as_millis() as u64;
        let events = self.events.clone();
        let token = token.to_string();
        let log_needed = is_final;
        tokio::spawn(async move {
            let mut guard = events.lock().await;
            guard.push(ProgressEvent::streaming_token(
                stage, &token, is_final, elapsed,
            ));
            if guard.len() % 10 == 0 || log_needed {
                tracing::debug!(
                    "Streaming: {} tokens pushed, final={}",
                    guard.len(),
                    is_final
                );
            }
        });
    }
}
