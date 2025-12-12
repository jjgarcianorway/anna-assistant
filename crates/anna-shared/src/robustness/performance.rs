//! Performance tracking and streaming updates (v0.0.433).
//!
//! Provides incremental feedback and timing breakdowns.

use super::timeouts::{TimeoutStage, TimingSummary};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Streaming update event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamingUpdate {
    /// Starting to check system data.
    ProbesStarted,
    /// A probe completed.
    ProbeCompleted {
        name: String,
        status: ProbeStatus,
        duration_ms: u64,
    },
    /// A specialist is reviewing.
    SpecialistStarted { name: String, department: String },
    /// Specialist finished.
    SpecialistFinished { name: String, duration_ms: u64 },
    /// Processing timed out.
    Timeout { stage: String, elapsed_ms: u64 },
    /// Retrying due to parse failure.
    Retrying { attempt: usize },
    /// Final answer ready.
    Complete { success: bool },
}

/// Probe status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProbeStatus {
    Ok,
    Failed,
    Timeout,
    Skipped,
}

impl ProbeStatus {
    /// Format as display string.
    pub fn display(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Failed => "failed",
            Self::Timeout => "timeout",
            Self::Skipped => "skipped",
        }
    }
}

impl StreamingUpdate {
    /// Format for REPL display.
    pub fn format(&self) -> String {
        match self {
            Self::ProbesStarted => "Checking system data...".to_string(),
            Self::ProbeCompleted {
                name,
                status,
                duration_ms,
            } => {
                format!(
                    "[probe] {} → {} ({}ms)",
                    name,
                    status.display(),
                    duration_ms
                )
            }
            Self::SpecialistStarted { name, department } => {
                format!("{} ({}) is reviewing the data...", name, department)
            }
            Self::SpecialistFinished { name, duration_ms } => {
                format!("{} finished ({}ms)", name, duration_ms)
            }
            Self::Timeout { stage, elapsed_ms } => {
                format!("Timeout at {} after {}ms", stage, elapsed_ms)
            }
            Self::Retrying { attempt } => {
                format!("Retrying (attempt {})", attempt)
            }
            Self::Complete { success } => {
                if *success {
                    "Analysis complete.".to_string()
                } else {
                    "Analysis incomplete.".to_string()
                }
            }
        }
    }
}

/// Timing breakdown for debugging.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimingBreakdown {
    /// Translator classification time.
    pub translator_ms: u64,
    /// Junior LLM call time.
    pub junior_llm_ms: u64,
    /// Senior LLM call time.
    pub senior_llm_ms: u64,
    /// JSON parsing time.
    pub parse_ms: u64,
    /// Total probe execution time.
    pub probes_ms: u64,
    /// Knowledge lookup time.
    pub knowledge_ms: u64,
    /// Network latency (if applicable).
    pub network_ms: u64,
    /// Total wall-clock time.
    pub total_ms: u64,
    /// Stages that timed out.
    pub timed_out_stages: Vec<String>,
    /// Retry count.
    pub retries: usize,
}

impl TimingBreakdown {
    /// Create from timing summary.
    pub fn from_summary(summary: &TimingSummary) -> Self {
        Self {
            translator_ms: summary.translator_ms,
            junior_llm_ms: summary.junior_llm_ms,
            senior_llm_ms: summary.senior_llm_ms,
            parse_ms: summary.parse_ms,
            probes_ms: summary.probes_ms,
            knowledge_ms: summary.knowledge_ms,
            total_ms: summary.total_ms,
            timed_out_stages: summary
                .timeouts
                .iter()
                .map(|t| t.name().to_string())
                .collect(),
            ..Default::default()
        }
    }

    /// Format for debug display.
    pub fn format_debug(&self) -> String {
        let mut parts = Vec::new();

        if self.translator_ms > 0 {
            parts.push(format!("translator: {}ms", self.translator_ms));
        }
        if self.junior_llm_ms > 0 {
            parts.push(format!("junior_llm: {}ms", self.junior_llm_ms));
        }
        if self.senior_llm_ms > 0 {
            parts.push(format!("senior_llm: {}ms", self.senior_llm_ms));
        }
        if self.parse_ms > 0 {
            parts.push(format!("parse: {}ms", self.parse_ms));
        }
        if self.probes_ms > 0 {
            parts.push(format!("probes: {}ms", self.probes_ms));
        }
        if self.knowledge_ms > 0 {
            parts.push(format!("knowledge: {}ms", self.knowledge_ms));
        }

        parts.push(format!("total: {}ms", self.total_ms));

        if !self.timed_out_stages.is_empty() {
            parts.push(format!("TIMEOUTS: {}", self.timed_out_stages.join(", ")));
        }

        if self.retries > 0 {
            parts.push(format!("retries: {}", self.retries));
        }

        parts.join(" | ")
    }

    /// Check if any timeouts occurred.
    pub fn had_timeouts(&self) -> bool {
        !self.timed_out_stages.is_empty()
    }
}

/// Performance tracker for a single ticket.
pub struct PerformanceTracker {
    /// Start time.
    start: Instant,
    /// Individual stage timings.
    stages: Vec<(TimeoutStage, u64)>,
    /// Probe timings.
    probes: Vec<(String, ProbeStatus, u64)>,
    /// Updates emitted.
    updates: Vec<StreamingUpdate>,
    /// Current stage start.
    current_stage_start: Option<(TimeoutStage, Instant)>,
    /// Retry count.
    retries: usize,
}

impl PerformanceTracker {
    /// Create a new tracker.
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            stages: Vec::new(),
            probes: Vec::new(),
            updates: Vec::new(),
            current_stage_start: None,
            retries: 0,
        }
    }

    /// Start tracking a stage.
    pub fn start_stage(&mut self, stage: TimeoutStage) {
        self.end_current_stage();
        self.current_stage_start = Some((stage, Instant::now()));
    }

    /// End current stage.
    pub fn end_current_stage(&mut self) {
        if let Some((stage, start)) = self.current_stage_start.take() {
            let duration = start.elapsed().as_millis() as u64;
            self.stages.push((stage, duration));
        }
    }

    /// Record a probe completion.
    pub fn record_probe(&mut self, name: &str, status: ProbeStatus, duration_ms: u64) {
        self.probes.push((name.to_string(), status, duration_ms));
        self.updates.push(StreamingUpdate::ProbeCompleted {
            name: name.to_string(),
            status,
            duration_ms,
        });
    }

    /// Record specialist start.
    pub fn record_specialist_start(&mut self, name: &str, department: &str) {
        self.updates.push(StreamingUpdate::SpecialistStarted {
            name: name.to_string(),
            department: department.to_string(),
        });
    }

    /// Record specialist finish.
    pub fn record_specialist_finish(&mut self, name: &str, duration_ms: u64) {
        self.updates.push(StreamingUpdate::SpecialistFinished {
            name: name.to_string(),
            duration_ms,
        });
    }

    /// Record a timeout.
    pub fn record_timeout(&mut self, stage: TimeoutStage) {
        let elapsed = self.start.elapsed().as_millis() as u64;
        self.updates.push(StreamingUpdate::Timeout {
            stage: stage.name().to_string(),
            elapsed_ms: elapsed,
        });
    }

    /// Record a retry.
    pub fn record_retry(&mut self) {
        self.retries += 1;
        self.updates.push(StreamingUpdate::Retrying {
            attempt: self.retries,
        });
    }

    /// Record completion.
    pub fn record_complete(&mut self, success: bool) {
        self.end_current_stage();
        self.updates.push(StreamingUpdate::Complete { success });
    }

    /// Get elapsed time in ms.
    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    /// Get all updates.
    pub fn updates(&self) -> &[StreamingUpdate] {
        &self.updates
    }

    /// Get timing breakdown.
    pub fn breakdown(&self) -> TimingBreakdown {
        let mut breakdown = TimingBreakdown {
            total_ms: self.elapsed_ms(),
            retries: self.retries,
            ..Default::default()
        };

        for (stage, duration) in &self.stages {
            match stage {
                TimeoutStage::Translator => breakdown.translator_ms += duration,
                TimeoutStage::JuniorRouting | TimeoutStage::JuniorLlm => {
                    breakdown.junior_llm_ms += duration
                }
                TimeoutStage::SeniorRouting | TimeoutStage::SeniorLlm => {
                    breakdown.senior_llm_ms += duration
                }
                TimeoutStage::Parse => breakdown.parse_ms += duration,
                TimeoutStage::Probes => breakdown.probes_ms += duration,
                TimeoutStage::Knowledge => breakdown.knowledge_ms += duration,
                TimeoutStage::Global => {}
            }
        }

        // Add probe times
        let probe_total: u64 = self.probes.iter().map(|(_, _, d)| d).sum();
        breakdown.probes_ms = breakdown.probes_ms.max(probe_total);

        // Check for timeouts
        for update in &self.updates {
            if let StreamingUpdate::Timeout { stage, .. } = update {
                breakdown.timed_out_stages.push(stage.clone());
            }
        }

        breakdown
    }

    /// Format updates for display.
    pub fn format_updates(&self) -> Vec<String> {
        self.updates.iter().map(|u| u.format()).collect()
    }
}

impl Default for PerformanceTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_update_format() {
        let update = StreamingUpdate::ProbeCompleted {
            name: "memory".to_string(),
            status: ProbeStatus::Ok,
            duration_ms: 50,
        };

        let formatted = update.format();
        assert!(formatted.contains("memory"));
        assert!(formatted.contains("ok"));
        assert!(formatted.contains("50ms"));
    }

    #[test]
    fn test_performance_tracker() {
        let mut tracker = PerformanceTracker::new();

        tracker.start_stage(TimeoutStage::Probes);
        tracker.record_probe("memory", ProbeStatus::Ok, 30);
        tracker.record_probe("disk", ProbeStatus::Ok, 45);
        tracker.end_current_stage();

        tracker.start_stage(TimeoutStage::JuniorLlm);
        tracker.record_specialist_start("Sofia", "Desktop");
        std::thread::sleep(std::time::Duration::from_millis(10));
        tracker.record_specialist_finish("Sofia", 100);
        tracker.end_current_stage();

        tracker.record_complete(true);

        let breakdown = tracker.breakdown();
        assert!(breakdown.total_ms > 0);
        assert_eq!(tracker.updates().len(), 5); // 2 probes + start + finish + complete
    }

    #[test]
    fn test_timing_breakdown_format() {
        let breakdown = TimingBreakdown {
            translator_ms: 50,
            junior_llm_ms: 200,
            probes_ms: 100,
            total_ms: 350,
            ..Default::default()
        };

        let formatted = breakdown.format_debug();
        assert!(formatted.contains("translator: 50ms"));
        assert!(formatted.contains("junior_llm: 200ms"));
        assert!(formatted.contains("total: 350ms"));
    }
}
