//! Telegram Chart Sender - Send charts as images.
//!
//! v0.3.125: Rich visual output for Telegram.

use teloxide::prelude::*;
use teloxide::types::InputFile;
use anyhow::Result;
use std::path::PathBuf;

/// Send a chart as an image to Telegram.
pub async fn send_chart_image(bot: &Bot, chat_id: ChatId, chart_text: &str, caption: &str) -> Result<()> {
    use anna_shared::chart_renderer::{render_chart_auto, ensure_imagemagick};

    // Ensure ImageMagick is installed
    if let Err(e) = ensure_imagemagick().await {
        tracing::warn!("Could not install ImageMagick: {}", e);
        // Fall back to text
        bot.send_message(chat_id, format!("```\n{}\n```", chart_text))
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
        return Ok(());
    }

    // Render chart to image
    match render_chart_auto(chart_text) {
        Ok(rendered) => {
            let file = InputFile::file(&rendered.path);
            bot.send_photo(chat_id, file)
                .caption(caption)
                .await?;

            // Clean up temporary file
            let _ = std::fs::remove_file(&rendered.path);
        }
        Err(e) => {
            tracing::warn!("Failed to render chart: {}", e);
            // Fall back to text
            bot.send_message(chat_id, format!("```\n{}\n```", chart_text))
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
        }
    }

    Ok(())
}

/// Send a dashboard as an image.
pub async fn send_dashboard(bot: &Bot, chat_id: ChatId) -> Result<()> {
    use anna_shared::dashboard::dashboard_summary;

    let dashboard = dashboard_summary();
    send_chart_image(bot, chat_id, &dashboard, "System Dashboard").await
}

/// Send package history chart.
pub async fn send_package_history(bot: &Bot, chat_id: ChatId, days: i64) -> Result<()> {
    use anna_shared::package_history::PackageHistory;

    let history = PackageHistory::load();
    let chart = history.chart_installations(days);

    let caption = format!("Package installations (last {} days)", days);
    send_chart_image(bot, chat_id, &chart, &caption).await
}

/// Send health report as image.
pub async fn send_health_report(bot: &Bot, chat_id: ChatId) -> Result<()> {
    use anna_shared::health_report::health_summary;

    let report = health_summary();
    send_chart_image(bot, chat_id, &report, "Health Report").await
}

/// Detect if message is requesting a chart/visualization.
pub fn wants_visualization(message: &str) -> Option<VisualizationType> {
    let msg = message.to_lowercase();

    if msg.contains("show") || msg.contains("chart") || msg.contains("graph") || msg.contains("visualize") {
        if msg.contains("package") && (msg.contains("install") || msg.contains("history")) {
            return Some(VisualizationType::PackageHistory);
        }
        if msg.contains("dashboard") || msg.contains("status") {
            return Some(VisualizationType::Dashboard);
        }
        if msg.contains("health") {
            return Some(VisualizationType::Health);
        }
    }

    None
}

#[derive(Debug, Clone)]
pub enum VisualizationType {
    PackageHistory,
    Dashboard,
    Health,
}
