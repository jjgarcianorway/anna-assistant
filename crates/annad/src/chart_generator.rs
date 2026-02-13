//! Visual chart generation for morning briefings.
//! Uses plotters library to create professional PNG charts.

use anyhow::{anyhow, Result};
use plotters::prelude::*;
use plotters_bitmap::BitMapBackend;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

// Use explicit font paths to work in systemd environments without fontconfig
const FONT_REGULAR: &str = "/usr/share/fonts/TTF/DejaVuSans.ttf";

fn font(size: f64) -> FontDesc<'static> {
    // Fall back through available fonts
    for path in &[FONT_REGULAR, "/usr/share/fonts/noto/NotoSans-Regular.ttf",
                  "/usr/share/fonts/liberation/LiberationSans-Regular.ttf"] {
        if std::path::Path::new(path).exists() {
            return ((*path), size).into_font();
        }
    }
    ("sans-serif", size).into_font()
}

/// Chart generator for system metrics visualization.
pub struct ChartGenerator {
    width: u32,
    height: u32,
    output_dir: PathBuf,
}

impl ChartGenerator {
    /// Create a new chart generator.
    pub fn new<P: AsRef<Path>>(output_dir: P) -> Self {
        Self {
            width: 1200,
            height: 600,
            output_dir: output_dir.as_ref().to_path_buf(),
        }
    }

    /// Generate 7-day trends chart (disk, memory, boot time).
    pub fn generate_trends_chart(
        &self,
        history: &anna_shared::monitor::LongTermHistory,
    ) -> Result<PathBuf> {
        info!("Generating 7-day trends chart");

        // Get last 7 days of data
        let snapshots: Vec<_> = history
            .daily_snapshots
            .iter()
            .rev()
            .take(7)
            .rev()
            .collect();

        if snapshots.is_empty() {
            return Err(anyhow!("No historical data available for chart"));
        }

        let output_path = self.output_dir.join("trends_7day.png");
        let output_path_clone = output_path.clone();
        std::fs::create_dir_all(&self.output_dir)?;

        let root = BitMapBackend::new(&output_path, (self.width, self.height))
            .into_drawing_area();

        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption("7-Day System Trends", font(40.0))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0usize..snapshots.len() - 1, 0f32..100f32)?;

        chart
            .configure_mesh()
            .x_labels(7)
            .x_label_formatter(&|x| {
                snapshots
                    .get(*x)
                    .map(|s| s.date.split('-').last().unwrap_or("").to_string())
                    .unwrap_or_default()
            })
            .y_desc("Percentage (%)")
            .draw()?;

        // Plot memory usage (%)
        chart
            .draw_series(LineSeries::new(
                snapshots
                    .iter()
                    .enumerate()
                    .map(|(i, s)| (i, s.avg_memory_pct)),
                &BLUE,
            ))?
            .label("Memory %")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

        // Plot disk usage (convert GB to approximate %)
        let max_disk = snapshots
            .iter()
            .map(|s| s.disk_used_gb)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(100.0);

        chart
            .draw_series(LineSeries::new(
                snapshots
                    .iter()
                    .enumerate()
                    .map(|(i, s)| (i, (s.disk_used_gb / max_disk) * 100.0)),
                &RED,
            ))?
            .label("Disk (normalized)")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;

        root.present()?;

        info!("Chart saved to: {}", output_path_clone.display());
        Ok(output_path_clone)
    }

    /// Generate boot time trend chart with forecast.
    pub fn generate_boot_time_chart(
        &self,
        history: &anna_shared::monitor::LongTermHistory,
        forecast_days: usize,
    ) -> Result<PathBuf> {
        info!("Generating boot time forecast chart");

        let snapshots: Vec<_> = history
            .daily_snapshots
            .iter()
            .rev()
            .take(14)
            .rev()
            .collect();

        if snapshots.len() < 3 {
            return Err(anyhow!("Insufficient data for boot time chart"));
        }

        let output_path = self.output_dir.join("boot_time_forecast.png");
        let output_path_clone = output_path.clone();

        let root = BitMapBackend::new(&output_path, (self.width, self.height))
            .into_drawing_area();

        root.fill(&WHITE)?;

        let max_boot = snapshots
            .iter()
            .map(|s| s.avg_boot_time)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(30.0);

        let total_points = snapshots.len() + forecast_days;

        let mut chart = ChartBuilder::on(&root)
            .caption("Boot Time Trend & Forecast", font(40.0))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0usize..total_points - 1, 0f32..max_boot * 1.2)?;

        chart
            .configure_mesh()
            .x_desc("Days")
            .y_desc("Boot Time (seconds)")
            .draw()?;

        // Actual boot times
        chart.draw_series(LineSeries::new(
            snapshots
                .iter()
                .enumerate()
                .map(|(i, s)| (i, s.avg_boot_time)),
            &BLUE,
        ))?;

        // Simple linear forecast
        if snapshots.len() >= 2 {
            let last_idx = snapshots.len() - 1;
            let last_val = snapshots[last_idx].avg_boot_time;
            let prev_val = snapshots[last_idx - 1].avg_boot_time;
            let slope = last_val - prev_val;

            let forecast: Vec<(usize, f32)> = (0..forecast_days)
                .map(|i| {
                    let idx = snapshots.len() + i;
                    let val = last_val + (slope * (i as f32 + 1.0));
                    (idx, val.max(0.0))
                })
                .collect();

            if !forecast.is_empty() {
                let mut forecast_with_last = vec![(last_idx, last_val)];
                forecast_with_last.extend(forecast);

                chart.draw_series(LineSeries::new(forecast_with_last, &RED.mix(0.5)))?;
            }
        }

        root.present()?;

        debug!("Boot time chart saved to: {}", output_path.display());
        Ok(output_path_clone)
    }

    /// Generate resource usage gauge chart.
    pub fn generate_resource_gauge(
        &self,
        disk_pct: f32,
        memory_pct: f32,
        load_avg: f32,
    ) -> Result<PathBuf> {
        info!("Generating resource gauge chart");

        let output_path = self.output_dir.join("resource_gauge.png");
        let output_path_clone = output_path.clone();

        let root = BitMapBackend::new(&output_path, (self.width, 400))
            .into_drawing_area();

        root.fill(&WHITE)?;

        let areas = root.split_evenly((1, 3));

        // Disk gauge
        self.draw_gauge(&areas[0], "Disk", disk_pct, 80.0, 90.0)?;

        // Memory gauge
        self.draw_gauge(&areas[1], "Memory", memory_pct, 85.0, 95.0)?;

        // Load gauge (normalize to 0-100, assuming 4 cores max)
        let load_pct = (load_avg / 4.0 * 100.0).min(100.0);
        self.draw_gauge(&areas[2], "Load", load_pct, 75.0, 90.0)?;

        root.present()?;

        debug!("Resource gauge saved to: {}", output_path.display());
        Ok(output_path_clone)
    }

    /// Draw a single gauge.
    fn draw_gauge<DB: DrawingBackend>(
        &self,
        area: &DrawingArea<DB, plotters::coord::Shift>,
        label: &str,
        value: f32,
        warning_threshold: f32,
        critical_threshold: f32,
    ) -> Result<()>
    where
        DB::ErrorType: 'static,
    {
        let color = if value >= critical_threshold {
            &RED
        } else if value >= warning_threshold {
            &YELLOW
        } else {
            &GREEN
        };

        let mut chart = ChartBuilder::on(area)
            .caption(label, font(30.0))
            .build_cartesian_2d(0f32..100f32, 0f32..1f32)?;

        // Draw gauge background
        chart.draw_series(std::iter::once(Rectangle::new(
            [(0.0, 0.3), (100.0, 0.7)],
            BLUE.mix(0.1).filled(),
        )))?;

        // Draw value bar
        chart.draw_series(std::iter::once(Rectangle::new(
            [(0.0, 0.3), (value, 0.7)],
            color.filled(),
        )))?;

        // Draw value text
        let text_style = font(25.0).color(color);
        chart.draw_series(std::iter::once(Text::new(
            format!("{:.1}%", value),
            (50.0, 0.9),
            text_style,
        )))?;

        Ok(())
    }

    /// Generate anomaly scatter chart (memory/disk over time with outliers).
    pub fn generate_anomaly_chart(
        &self,
        history: &anna_shared::monitor::LongTermHistory,
        baseline_memory: f32,
        baseline_disk: f32,
    ) -> Result<PathBuf> {
        info!("Generating anomaly detection chart");

        let snapshots: Vec<_> = history
            .daily_snapshots
            .iter()
            .rev()
            .take(30)
            .rev()
            .collect();

        if snapshots.is_empty() {
            return Err(anyhow!("No data for anomaly chart"));
        }

        let output_path = self.output_dir.join("anomalies.png");
        let output_path_clone = output_path.clone();

        let root = BitMapBackend::new(&output_path, (self.width, self.height))
            .into_drawing_area();

        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption("Anomaly Detection (30 days)", font(40.0))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0usize..snapshots.len(), 0f32..100f32)?;

        chart
            .configure_mesh()
            .x_desc("Days ago")
            .y_desc("Percentage")
            .draw()?;

        // Draw baseline zones
        chart.draw_series(std::iter::once(Rectangle::new(
            [(0, baseline_memory - 10.0), (snapshots.len(), baseline_memory + 10.0)],
            BLUE.mix(0.1).filled(),
        )))?;

        // Plot memory with anomaly highlighting
        for (i, snapshot) in snapshots.iter().enumerate() {
            let is_anomaly = (snapshot.avg_memory_pct - baseline_memory).abs() > 15.0;
            let color = if is_anomaly { &RED } else { &BLUE };

            chart.draw_series(std::iter::once(Circle::new(
                (i, snapshot.avg_memory_pct),
                3,
                color.filled(),
            )))?;
        }

        root.present()?;

        debug!("Anomaly chart saved to: {}", output_path.display());
        Ok(output_path_clone)
    }
}
