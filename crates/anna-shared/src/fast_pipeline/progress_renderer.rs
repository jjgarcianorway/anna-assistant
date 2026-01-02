//! Progress Renderer (Part G.3) - v0.0.438.
//!
//! Renders honest progress without lying or breaking parsing.

use crate::fast_pipeline::progress_types::PhaseStatus;
use crate::fast_pipeline::progress_pipeline::PipelineProgress;

/// Renders progress without lying.
pub struct ProgressRenderer {
    /// Whether to show timing.
    pub show_timing: bool,
    /// Whether to show indicators.
    pub show_indicators: bool,
    /// Compact mode (single line).
    pub compact: bool,
}

impl ProgressRenderer {
    /// Create new renderer.
    pub fn new() -> Self {
        Self {
            show_timing: true,
            show_indicators: true,
            compact: false,
        }
    }

    /// Enable compact mode.
    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    /// Disable timing.
    pub fn no_timing(mut self) -> Self {
        self.show_timing = false;
        self
    }

    /// Render progress.
    pub fn render(&self, progress: &PipelineProgress) -> String {
        if self.compact {
            self.render_compact(progress)
        } else {
            self.render_full(progress)
        }
    }

    /// Render compact (single line).
    fn render_compact(&self, progress: &PipelineProgress) -> String {
        let parts: Vec<String> = progress
            .phases
            .iter()
            .filter(|p| p.status != PhaseStatus::Pending && p.status != PhaseStatus::Skipped)
            .map(|p| {
                let indicator = if self.show_indicators {
                    p.status.indicator()
                } else {
                    ""
                };
                let time = if self.show_timing {
                    format!(" {}ms", p.elapsed_ms())
                } else {
                    String::new()
                };
                format!("{}{}{}", indicator, p.phase.label(), time)
            })
            .collect();

        let total = if self.show_timing {
            format!(" [{}ms]", progress.total_elapsed_ms())
        } else {
            String::new()
        };

        format!("{}{}", parts.join(" → "), total)
    }

    /// Render full (multi-line).
    fn render_full(&self, progress: &PipelineProgress) -> String {
        let mut lines = Vec::new();

        for phase in &progress.phases {
            if phase.status == PhaseStatus::Skipped {
                continue;
            }
            lines.push(phase.format_display());
        }

        if self.show_timing {
            lines.push(format!("Total: {}ms", progress.total_elapsed_ms()));
        }

        lines.join("\n")
    }

    /// Render current status (for streaming).
    pub fn render_current(&self, progress: &PipelineProgress) -> String {
        if let Some(phase) = progress.current_phase {
            if let Some(p) = progress.get_phase(phase) {
                let elapsed = p.elapsed_ms();
                format!("{} {}ms...", phase.label(), elapsed)
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    }
}

impl Default for ProgressRenderer {
    fn default() -> Self {
        Self::new()
    }
}
