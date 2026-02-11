//! Visual chart generation using plotters.
//! Generates PNG charts for morning briefing and on-demand visualization.

use anyhow::{Context, Result};
use plotters::prelude::*;
use plotters_bitmap::BitMapBackend;
use std::path::{Path, PathBuf};
use anna_shared::monitor::{LongTermHistory, DailySnapshot};
use anna_shared::prediction::{Forecaster, TrendDirection};

/// Chart generator for system metrics visualization.
pub struct ChartGenerator {
    width: u32,
    height: u32,
    output_dir: PathBuf,
}

impl ChartGenerator {
    /// Create new chart generator.
    pub fn new<P: AsRef<Path>>(output_dir: P) -> Self {
        let output_dir = output_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&output_dir).ok();
        Self {
            width: 1200,
            height: 800,
            output_dir,
        }
    }

    /// Generate 7-day trends chart (disk, memory, boot time).
    pub fn generate_trends_chart(&self, history: &LongTermHistory) -> Result<PathBuf> {
        if history.daily_snapshots.is_empty() {
            anyhow::bail!("No historical data available for trends chart");
        }

        let path = self.output_dir.join("trends_7day.png");
        let path_clone = path.clone();
        let root = BitMapBackend::new(&path_clone, (self.width, self.height))
            .into_drawing_area();

        root.fill(&WHITE)
            .context("Failed to fill background")?;

        // Split into 3 sub-charts
        let areas = root.split_evenly((3, 1));
        let top = &areas[0];
        let middle = &areas[1];
        let bottom = &areas[2];

        // Get last 7 days of data
        let snapshots: Vec<_> = history.daily_snapshots.iter()
            .rev()
            .take(7)
            .rev()
            .collect();

        if snapshots.is_empty() {
            anyhow::bail!("No snapshots available");
        }

        // Chart 1: Disk usage (GB)
        self.draw_disk_chart(&top, &snapshots)?;

        // Chart 2: Memory usage (%)
        self.draw_memory_chart(&middle, &snapshots)?;

        // Chart 3: Boot time (seconds)
        self.draw_boot_chart(&bottom, &snapshots)?;

        root.present()
            .context("Failed to save chart")?;

        Ok(path)
    }

    fn draw_disk_chart(
        &self,
        area: &DrawingArea<BitMapBackend, plotters::coord::Shift>,
        snapshots: &[&DailySnapshot],
    ) -> Result<()> {
        let values: Vec<f32> = snapshots.iter()
            .map(|s| s.disk_used_gb)
            .collect();

        let min_val = values.iter().cloned().fold(f32::INFINITY, f32::min).max(0.0);
        let max_val = values.iter().cloned().fold(f32::NEG_INFINITY, f32::max) * 1.1;

        let mut chart = ChartBuilder::on(area)
            .caption("Disk Usage (GB) - Last 7 Days", ("sans-serif", 30))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0..snapshots.len(), min_val..max_val)?;

        chart
            .configure_mesh()
            .x_desc("Day")
            .y_desc("GB Used")
            .draw()?;

        // Draw line
        chart.draw_series(LineSeries::new(
            values.iter().enumerate().map(|(i, &v)| (i, v)),
            &BLUE,
        ))?;

        // Draw points
        chart.draw_series(values.iter().enumerate().map(|(i, &v)| {
            Circle::new((i, v), 4, BLUE.filled())
        }))?;

        Ok(())
    }

    fn draw_memory_chart(
        &self,
        area: &DrawingArea<BitMapBackend, plotters::coord::Shift>,
        snapshots: &[&anna_shared::monitor::DailySnapshot],
    ) -> Result<()> {
        let values: Vec<f32> = snapshots.iter()
            .map(|s| s.avg_memory_pct)
            .collect();

        let min_val = 0.0f32;
        let max_val = 100.0f32;

        let mut chart = ChartBuilder::on(area)
            .caption("Memory Usage (%) - Last 7 Days", ("sans-serif", 30))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0..snapshots.len(), min_val..max_val)?;

        chart
            .configure_mesh()
            .x_desc("Day")
            .y_desc("Memory %")
            .draw()?;

        // Draw warning zone (>85%)
        chart.draw_series(std::iter::once(Rectangle::new(
            [(0, 85.0), (snapshots.len(), 100.0)],
            RED.mix(0.1).filled(),
        )))?;

        // Draw line
        chart.draw_series(LineSeries::new(
            values.iter().enumerate().map(|(i, &v)| (i, v)),
            &GREEN,
        ))?;

        // Draw points
        chart.draw_series(values.iter().enumerate().map(|(i, &v)| {
            Circle::new((i, v), 4, GREEN.filled())
        }))?;

        Ok(())
    }

    fn draw_boot_chart(
        &self,
        area: &DrawingArea<BitMapBackend, plotters::coord::Shift>,
        snapshots: &[&anna_shared::monitor::DailySnapshot],
    ) -> Result<()> {
        let values: Vec<f32> = snapshots.iter()
            .map(|s| s.avg_boot_time)
            .collect();

        let min_val = 0.0f32;
        let max_val = values.iter().cloned().fold(f32::NEG_INFINITY, f32::max) * 1.2;

        let mut chart = ChartBuilder::on(area)
            .caption("Boot Time (seconds) - Last 7 Days", ("sans-serif", 30))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0..snapshots.len(), min_val..max_val)?;

        chart
            .configure_mesh()
            .x_desc("Day")
            .y_desc("Seconds")
            .draw()?;

        // Draw line
        chart.draw_series(LineSeries::new(
            values.iter().enumerate().map(|(i, &v)| (i, v)),
            &MAGENTA,
        ))?;

        // Draw points
        chart.draw_series(values.iter().enumerate().map(|(i, &v)| {
            Circle::new((i, v), 4, MAGENTA.filled())
        }))?;

        Ok(())
    }

    /// Generate resource forecast chart with prediction line.
    pub fn generate_forecast_chart(
        &self,
        resource_name: &str,
        history_values: &[f64],
        unit: &str,
    ) -> Result<PathBuf> {
        if history_values.is_empty() {
            anyhow::bail!("No data for forecast chart");
        }

        let path = self.output_dir.join(format!("forecast_{}.png", resource_name));
        let path_clone = path.clone();
        let root = BitMapBackend::new(&path_clone, (self.width, self.height / 2))
            .into_drawing_area();

        root.fill(&WHITE)?;

        // Analyze trend and generate forecast
        let forecaster = Forecaster::default();
        let trend_analysis = anna_shared::prediction::analyze_trend(history_values, 14);

        let values_f32: Vec<f32> = history_values.iter().map(|&v| v as f32).collect();
        let min_val = values_f32.iter().cloned().fold(f32::INFINITY, f32::min).max(0.0);
        let max_val = values_f32.iter().cloned().fold(f32::NEG_INFINITY, f32::max) * 1.3;

        let mut chart = ChartBuilder::on(&root)
            .caption(
                format!("{} Forecast - Historical + 14 Day Prediction", resource_name),
                ("sans-serif", 35),
            )
            .margin(15)
            .x_label_area_size(50)
            .y_label_area_size(70)
            .build_cartesian_2d(0..(history_values.len() + 14), min_val..max_val)?;

        chart
            .configure_mesh()
            .x_desc("Days")
            .y_desc(unit)
            .draw()?;

        // Draw historical data
        chart.draw_series(LineSeries::new(
            values_f32.iter().enumerate().map(|(i, &v)| (i, v)),
            BLUE.stroke_width(2),
        ))?
        .label("Historical")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

        // Draw prediction if trend exists
        if let Some(trend) = trend_analysis {
            let forecast_points: Vec<(usize, f32)> = (history_values.len()..history_values.len() + 14)
                .map(|i| {
                    let days_ahead = i - history_values.len();
                    let predicted = trend.current + (trend.slope * days_ahead as f64);
                    (i, predicted as f32)
                })
                .collect();

            // Draw prediction line (dashed)
            chart.draw_series(LineSeries::new(
                forecast_points.iter().cloned(),
                RED.stroke_width(2),
            ))?
            .label("Predicted")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

            // Add trend direction label
            let direction_label = match trend.direction {
                TrendDirection::Increasing => "↗ Increasing",
                TrendDirection::Decreasing => "↘ Decreasing",
                TrendDirection::Stable => "→ Stable",
            };

            chart.draw_series(std::iter::once(Text::new(
                direction_label,
                (2, max_val * 0.9),
                ("sans-serif", 25).into_font().color(&BLACK),
            )))?;
        }

        chart.configure_series_labels()
            .border_style(&BLACK)
            .draw()?;

        root.present()?;

        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chart_generator_creation() {
        let gen = ChartGenerator::new("/tmp/anna_test_charts");
        assert_eq!(gen.width, 1200);
        assert_eq!(gen.height, 800);
    }
}
