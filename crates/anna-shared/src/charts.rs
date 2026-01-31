//! ASCII Charts - Visual reports for terminal display.
//!
//! Provides beautiful ASCII visualizations for system metrics.
//! Works in any terminal without external dependencies.

/// Bar chart for comparing values.
pub struct BarChart {
    title: String,
    pub bars: Vec<(String, f64, Option<String>)>, // label, value, optional color hint
    max_label_width: usize,
    bar_width: usize,
}

impl BarChart {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            bars: Vec::new(),
            max_label_width: 15,
            bar_width: 40,
        }
    }

    pub fn add(&mut self, label: &str, value: f64) -> &mut Self {
        self.bars.push((label.to_string(), value, None));
        self
    }

    pub fn add_colored(&mut self, label: &str, value: f64, color: &str) -> &mut Self {
        self.bars.push((label.to_string(), value, Some(color.to_string())));
        self
    }

    pub fn render(&self) -> String {
        if self.bars.is_empty() {
            return format!("{}\n(no data)", self.title);
        }

        let max_val = self.bars.iter().map(|(_, v, _)| *v).fold(0.0_f64, f64::max);
        let mut lines = vec![self.title.clone(), "─".repeat(self.title.len())];

        for (label, value, color) in &self.bars {
            let label_display: String = if label.len() > self.max_label_width {
                format!("{}…", &label[..self.max_label_width - 1])
            } else {
                format!("{:>width$}", label, width = self.max_label_width)
            };

            let bar_len = if max_val > 0.0 {
                ((value / max_val) * self.bar_width as f64) as usize
            } else {
                0
            };

            let bar_char = match color.as_deref() {
                Some("red") | Some("critical") => '▓',
                Some("yellow") | Some("warning") => '▒',
                Some("green") | Some("good") => '░',
                _ => '█',
            };

            let bar: String = std::iter::repeat(bar_char).take(bar_len).collect();
            let empty: String = std::iter::repeat('·').take(self.bar_width - bar_len).collect();

            lines.push(format!("{} │{}{} {:>6.1}", label_display, bar, empty, value));
        }

        lines.join("\n")
    }
}

/// Gauge for showing percentage values.
pub struct Gauge {
    label: String,
    value: f64,
    max: f64,
    width: usize,
}

impl Gauge {
    pub fn new(label: &str, value: f64, max: f64) -> Self {
        Self {
            label: label.to_string(),
            value,
            max,
            width: 30,
        }
    }

    pub fn percentage(label: &str, percent: f64) -> Self {
        Self::new(label, percent, 100.0)
    }

    pub fn render(&self) -> String {
        let pct = if self.max > 0.0 { (self.value / self.max) * 100.0 } else { 0.0 };
        let filled = ((pct / 100.0) * self.width as f64) as usize;
        let empty = self.width - filled;

        let fill_char = if pct >= 90.0 {
            '▓' // Critical
        } else if pct >= 75.0 {
            '▒' // Warning
        } else {
            '░' // Normal
        };

        let bar: String = std::iter::repeat(fill_char).take(filled).collect();
        let empty_bar: String = std::iter::repeat('·').take(empty).collect();

        format!("{}: [{}{}] {:.1}%", self.label, bar, empty_bar, pct)
    }
}

/// Sparkline for showing trends in minimal space.
pub struct Sparkline {
    values: Vec<f64>,
}

impl Sparkline {
    const CHARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    pub fn new(values: &[f64]) -> Self {
        Self { values: values.to_vec() }
    }

    pub fn render(&self) -> String {
        if self.values.is_empty() {
            return String::new();
        }

        let min = self.values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = self.values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = max - min;

        self.values.iter().map(|v| {
            if range == 0.0 {
                Self::CHARS[4]
            } else {
                let normalized = (v - min) / range;
                let idx = ((normalized * 7.0) as usize).min(7);
                Self::CHARS[idx]
            }
        }).collect()
    }

    pub fn render_with_label(&self, label: &str) -> String {
        format!("{}: {}", label, self.render())
    }
}

/// Box for system status display.
pub struct StatusBox {
    title: String,
    items: Vec<(String, String, Status)>,
}

#[derive(Clone, Copy)]
pub enum Status {
    Good,
    Warning,
    Critical,
    Unknown,
}

impl StatusBox {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            items: Vec::new(),
        }
    }

    pub fn add(&mut self, label: &str, value: &str, status: Status) -> &mut Self {
        self.items.push((label.to_string(), value.to_string(), status));
        self
    }

    pub fn render(&self) -> String {
        let max_label = self.items.iter().map(|(l, _, _)| l.len()).max().unwrap_or(10);
        let max_value = self.items.iter().map(|(_, v, _)| v.len()).max().unwrap_or(10);
        let width = max_label + max_value + 10;

        let mut lines = Vec::new();

        // Top border
        lines.push(format!("┌{}┐", "─".repeat(width)));

        // Title
        let title_padding = (width - self.title.len()) / 2;
        lines.push(format!("│{:^width$}│", self.title, width = width));
        lines.push(format!("├{}┤", "─".repeat(width)));

        // Items
        for (label, value, status) in &self.items {
            let indicator = match status {
                Status::Good => "●",
                Status::Warning => "◐",
                Status::Critical => "○",
                Status::Unknown => "◌",
            };

            let line = format!(
                "{} {:label_width$} │ {:>value_width$}",
                indicator,
                label,
                value,
                label_width = max_label,
                value_width = max_value
            );

            let padded = format!("│ {:<width$} │", line, width = width - 2);
            lines.push(padded);
        }

        // Bottom border
        lines.push(format!("└{}┘", "─".repeat(width)));

        lines.join("\n")
    }
}

/// Multi-line trend chart.
pub struct TrendChart {
    title: String,
    series: Vec<(String, Vec<f64>)>,
    height: usize,
    width: usize,
}

impl TrendChart {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            series: Vec::new(),
            height: 10,
            width: 60,
        }
    }

    pub fn add_series(&mut self, label: &str, values: &[f64]) -> &mut Self {
        self.series.push((label.to_string(), values.to_vec()));
        self
    }

    pub fn render(&self) -> String {
        if self.series.is_empty() {
            return format!("{}\n(no data)", self.title);
        }

        // Find global min/max
        let all_values: Vec<f64> = self.series.iter()
            .flat_map(|(_, v)| v.iter().cloned())
            .collect();

        let min = all_values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = all_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = max - min;

        let mut lines = vec![self.title.clone()];

        // Render each series as a sparkline
        for (label, values) in &self.series {
            let spark = Sparkline::new(values);
            lines.push(format!("  {}: {} ({:.1}-{:.1})", label, spark.render(), min, max));
        }

        lines.join("\n")
    }
}

/// System health report combining multiple visualizations.
pub struct HealthReport {
    sections: Vec<String>,
}

impl HealthReport {
    pub fn new() -> Self {
        Self { sections: Vec::new() }
    }

    pub fn add_section(&mut self, content: String) -> &mut Self {
        self.sections.push(content);
        self
    }

    pub fn render(&self) -> String {
        self.sections.join("\n\n")
    }
}

impl Default for HealthReport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bar_chart() {
        let mut chart = BarChart::new("Disk Usage");
        chart.add("/", 75.0);
        chart.add("/home", 45.0);
        let rendered = chart.render();
        assert!(rendered.contains("Disk Usage"));
        assert!(rendered.contains("/"));
    }

    #[test]
    fn test_gauge() {
        let gauge = Gauge::percentage("CPU", 65.0);
        let rendered = gauge.render();
        assert!(rendered.contains("CPU"));
        assert!(rendered.contains("65.0%"));
    }

    #[test]
    fn test_sparkline() {
        let spark = Sparkline::new(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let rendered = spark.render();
        assert_eq!(rendered.len(), 5);
    }

    #[test]
    fn test_status_box() {
        let mut box_ = StatusBox::new("System Status");
        box_.add("CPU", "45%", Status::Good);
        box_.add("Memory", "89%", Status::Warning);
        let rendered = box_.render();
        assert!(rendered.contains("System Status"));
        assert!(rendered.contains("CPU"));
    }
}
