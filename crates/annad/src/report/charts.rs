//! Chart generation using plotters.

use plotters::prelude::*;
use plotters_bitmap::BitMapBackend;
use std::path::Path;

use crate::anomaly::{AnomalyStore, Sample};

/// Render a 24-hour metrics line chart to PNG.
pub fn render_metrics_chart(output_path: &Path) -> Result<(), String> {
    let store = AnomalyStore::load();

    // Get data for RAM, Load, and Disk
    let ram_samples = store.metrics.get("RAM").map(|h| &h.samples[..]).unwrap_or(&[]);
    let load_samples = store.metrics.get("Load1").map(|h| &h.samples[..]).unwrap_or(&[]);
    let disk_samples = store.metrics.get("Disk").map(|h| &h.samples[..]).unwrap_or(&[]);

    if ram_samples.is_empty() && load_samples.is_empty() && disk_samples.is_empty() {
        return Err("No metrics data available yet".to_string());
    }

    // Create chart
    let root = BitMapBackend::new(output_path, (600, 300))
        .into_drawing_area();
    root.fill(&WHITE).map_err(|e| e.to_string())?;

    // Find time range
    let now = chrono::Utc::now().timestamp();
    let start = now - 86400; // 24 hours ago

    let mut chart = ChartBuilder::on(&root)
        .caption("24-Hour System Metrics", ("sans-serif", 18))
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(start..now, 0f64..100f64)
        .map_err(|e| e.to_string())?;

    chart.configure_mesh()
        .x_labels(6)
        .y_labels(5)
        .x_label_formatter(&|x| {
            let dt = chrono::DateTime::from_timestamp(*x, 0).unwrap_or_default();
            dt.format("%H:%M").to_string()
        })
        .y_label_formatter(&|y| format!("{}%", y))
        .draw()
        .map_err(|e| e.to_string())?;

    // Draw RAM line (blue)
    if !ram_samples.is_empty() {
        let points: Vec<(i64, f64)> = ram_samples.iter()
            .map(|s| (s.timestamp, s.value))
            .collect();
        chart.draw_series(LineSeries::new(points, &BLUE))
            .map_err(|e| e.to_string())?
            .label("RAM")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 15, y)], &BLUE));
    }

    // Draw Disk line (green)
    if !disk_samples.is_empty() {
        let points: Vec<(i64, f64)> = disk_samples.iter()
            .map(|s| (s.timestamp, s.value))
            .collect();
        chart.draw_series(LineSeries::new(points, &GREEN))
            .map_err(|e| e.to_string())?
            .label("Disk")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 15, y)], &GREEN));
    }

    // Draw Load line (red, scaled to 0-100 range assuming max load of 10)
    if !load_samples.is_empty() {
        let points: Vec<(i64, f64)> = load_samples.iter()
            .map(|s| (s.timestamp, (s.value * 10.0).min(100.0)))
            .collect();
        chart.draw_series(LineSeries::new(points, &RED))
            .map_err(|e| e.to_string())?
            .label("Load (x10)")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 15, y)], &RED));
    }

    // Draw legend
    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .position(SeriesLabelPosition::UpperRight)
        .draw()
        .map_err(|e| e.to_string())?;

    root.present().map_err(|e| e.to_string())?;
    Ok(())
}

/// Render a boot time trend bar chart.
pub fn render_boot_chart(boot_times: &[f32], output_path: &Path) -> Result<(), String> {
    if boot_times.is_empty() {
        return Err("No boot time data available".to_string());
    }

    let root = BitMapBackend::new(output_path, (400, 200))
        .into_drawing_area();
    root.fill(&WHITE).map_err(|e| e.to_string())?;

    let max_time = boot_times.iter().cloned().fold(0f32, f32::max);
    let count = boot_times.len();

    let mut chart = ChartBuilder::on(&root)
        .caption("Boot Time Trend (seconds)", ("sans-serif", 16))
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(0..count, 0f32..(max_time * 1.2))
        .map_err(|e| e.to_string())?;

    chart.configure_mesh()
        .x_labels(count.min(10))
        .y_labels(5)
        .draw()
        .map_err(|e| e.to_string())?;

    // Draw bars
    chart.draw_series(
        boot_times.iter().enumerate().map(|(i, &time)| {
            let color = if i == count - 1 { BLUE.filled() } else { CYAN.filled() };
            Rectangle::new([(i, 0.0), (i + 1, time)], color)
        })
    ).map_err(|e| e.to_string())?;

    root.present().map_err(|e| e.to_string())?;
    Ok(())
}

/// Render a simple sparkline for embedding inline (very small chart).
pub fn render_sparkline(samples: &[Sample], output_path: &Path) -> Result<(), String> {
    if samples.is_empty() {
        return Err("No samples for sparkline".to_string());
    }

    let root = BitMapBackend::new(output_path, (120, 30))
        .into_drawing_area();
    root.fill(&WHITE).map_err(|e| e.to_string())?;

    let min_val = samples.iter().map(|s| s.value).fold(f64::INFINITY, f64::min);
    let max_val = samples.iter().map(|s| s.value).fold(f64::NEG_INFINITY, f64::max);
    let range = (max_val - min_val).max(1.0);

    let points: Vec<(i32, i32)> = samples.iter().enumerate()
        .map(|(i, s)| {
            let x = (i * 120 / samples.len().max(1)) as i32;
            let y = (28.0 - ((s.value - min_val) / range * 26.0)) as i32;
            (x, y)
        })
        .collect();

    if points.len() >= 2 {
        for window in points.windows(2) {
            root.draw(&PathElement::new(
                vec![window[0], window[1]],
                &BLUE,
            )).map_err(|e| e.to_string())?;
        }
    }

    root.present().map_err(|e| e.to_string())?;
    Ok(())
}
