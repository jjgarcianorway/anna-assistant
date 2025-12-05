//! Progress tracker with transcript building for request handling.

use anna_shared::progress::{ProgressEvent, RequestStage};
use anna_shared::transcript::{Actor, StageOutcome, Transcript, TranscriptEvent};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::info;

/// Progress tracker for request handling with transcript building
pub struct ProgressTracker {
    events: Vec<ProgressEvent>,
    transcript: Transcript,
    start_time: Instant,
    current_stage: Option<RequestStage>,
}

impl ProgressTracker {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            transcript: Transcript::new(),
            start_time: Instant::now(),
            current_stage: None,
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }

    pub fn add(&mut self, event: ProgressEvent) {
        info!("{}", event.format_debug());
        self.events.push(event);
    }

    pub fn start_stage(&mut self, stage: RequestStage, timeout_secs: u64) {
        self.current_stage = Some(stage);
        self.add(ProgressEvent::starting(
            stage,
            timeout_secs,
            self.elapsed_ms(),
        ));
        let stage_name = format!("{:?}", stage).to_lowercase();
        self.transcript
            .push(TranscriptEvent::stage_start(self.elapsed_ms(), stage_name));
    }

    pub fn complete_stage(&mut self, stage: RequestStage) {
        self.add(ProgressEvent::complete(stage, self.elapsed_ms()));
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

    pub fn add_anna_response(&mut self, text: &str) {
        self.transcript.push(TranscriptEvent::message(
            self.elapsed_ms(),
            Actor::Anna,
            Actor::You,
            text,
        ));
    }

    pub fn events(&self) -> &[ProgressEvent] {
        &self.events
    }

    pub fn take_transcript(self) -> Transcript {
        self.transcript
    }

    pub fn transcript_clone(&self) -> Transcript {
        self.transcript.clone()
    }
}

/// Shared progress state for polling (reserved for future watchdog use)
#[allow(dead_code)]
pub type SharedProgress = Arc<RwLock<ProgressTracker>>;

#[allow(dead_code)]
pub fn create_progress_tracker() -> SharedProgress {
    Arc::new(RwLock::new(ProgressTracker::new()))
}
