// v0.0.647: Renderer Statistics (Phase 223)
// Statistics tracking for settings rendering

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::{RenderTarget, RenderTheme};

/// Renderer stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RendererStats {
    /// Total renders
    pub total_renders: usize,
    /// By target
    pub by_target: HashMap<String, usize>,
    /// By theme
    pub by_theme: HashMap<String, usize>,
    /// Total lines rendered
    pub total_lines: usize,
}

impl RendererStats {
    /// Record render
    pub fn record(&mut self, target: RenderTarget, theme: RenderTheme, line_count: usize) {
        self.total_renders += 1;
        *self.by_target.entry(target.to_string()).or_insert(0) += 1;
        *self.by_theme.entry(theme.to_string()).or_insert(0) += 1;
        self.total_lines += line_count;
    }

    /// Average lines per render
    pub fn average_lines(&self) -> f64 {
        if self.total_renders == 0 {
            0.0
        } else {
            self.total_lines as f64 / self.total_renders as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_record() {
        let mut s = RendererStats::default();
        s.record(RenderTarget::Terminal, RenderTheme::Default, 10);
        s.record(RenderTarget::Html, RenderTheme::Dark, 20);
        assert_eq!(s.total_renders, 2);
        assert_eq!(s.total_lines, 30);
    }
}
