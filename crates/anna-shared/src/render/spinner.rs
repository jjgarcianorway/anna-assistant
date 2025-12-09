//! Render spinner and progress (v0.0.203).

use std::io::{self, Write};

use crate::ui::colors;

use super::types::RenderPolicy;

/// Spinner animation state
pub struct Spinner {
    frames: &'static [&'static str],
    current: usize,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            frames: &["-", "\\", "|", "/"],
            current: 0,
        }
    }

    pub fn tick(&mut self) {
        print!("\r{} ", self.frames[self.current]);
        io::stdout().flush().ok();
        self.current = (self.current + 1) % self.frames.len();
    }

    pub fn clear(&self) {
        print!("\r  \r");
        io::stdout().flush().ok();
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress renderer for streaming updates
pub struct ProgressRenderer {
    policy: RenderPolicy,
    spinner: Spinner,
    stage: Option<String>,
}

impl ProgressRenderer {
    pub fn new(policy: RenderPolicy) -> Self {
        Self {
            policy,
            spinner: Spinner::new(),
            stage: None,
        }
    }

    pub fn show_stage(&mut self, stage: &str) {
        if self.policy == RenderPolicy::Narrative {
            self.spinner.clear();
            println!("{}...{}{}", colors::DIM, stage, colors::RESET);
        }
        self.stage = Some(stage.to_string());
    }

    pub fn tick(&mut self) {
        if self.policy == RenderPolicy::Narrative {
            self.spinner.tick();
        }
    }

    pub fn complete(&mut self) {
        if self.policy == RenderPolicy::Narrative {
            self.spinner.clear();
        }
    }
}
