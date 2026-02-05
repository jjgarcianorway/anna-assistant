//! Chart Renderer - Convert ASCII charts to images for Telegram.
//!
//! v0.3.124: Generate PNG images from charts for rich Telegram output.

use std::path::PathBuf;

/// Render configuration.
pub struct RenderConfig {
    /// Font size for text
    pub font_size: u32,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Background color (RGB)
    pub bg_color: (u8, u8, u8),
    /// Text color (RGB)
    pub text_color: (u8, u8, u8),
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            font_size: 12,
            width: 800,
            height: 600,
            bg_color: (40, 44, 52),   // Dark background
            text_color: (171, 178, 191), // Light text
        }
    }
}

/// Result of rendering a chart.
pub struct RenderedChart {
    /// Path to generated PNG
    pub path: PathBuf,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

/// Render ASCII text as an image.
///
/// Note: This is a placeholder. Full implementation would use image generation
/// libraries like `image` and `imageproc`, or call external tools like ImageMagick.
pub fn render_text_as_image(text: &str, config: &RenderConfig) -> anyhow::Result<RenderedChart> {
    // For now, write to a temporary file and use ImageMagick convert if available
    let temp_dir = PathBuf::from("/var/lib/anna/charts");
    std::fs::create_dir_all(&temp_dir)?;

    let filename = format!("chart_{}.png", uuid::Uuid::new_v4());
    let output_path = temp_dir.join(&filename);

    // Try to use ImageMagick convert
    let output = std::process::Command::new("convert")
        .arg("-size")
        .arg(format!("{}x{}", config.width, config.height))
        .arg("-background")
        .arg(format!("rgb({},{},{})", config.bg_color.0, config.bg_color.1, config.bg_color.2))
        .arg("-fill")
        .arg(format!("rgb({},{},{})", config.text_color.0, config.text_color.1, config.text_color.2))
        .arg("-font")
        .arg("DejaVu-Sans-Mono")
        .arg("-pointsize")
        .arg(config.font_size.to_string())
        .arg("-gravity")
        .arg("NorthWest")
        .arg(format!("label:{}", text))
        .arg(&output_path)
        .output();

    match output {
        Ok(result) if result.status.success() => {
            Ok(RenderedChart {
                path: output_path,
                width: config.width,
                height: config.height,
            })
        }
        _ => {
            // Fallback: save as text file with .txt extension
            let txt_path = temp_dir.join(format!("chart_{}.txt", uuid::Uuid::new_v4()));
            std::fs::write(&txt_path, text)?;
            anyhow::bail!("ImageMagick not available. Chart saved as text: {:?}", txt_path)
        }
    }
}

/// Render a chart with automatic sizing based on content.
pub fn render_chart_auto(chart_text: &str) -> anyhow::Result<RenderedChart> {
    let lines = chart_text.lines().count();
    let max_line_len = chart_text.lines().map(|l| l.len()).max().unwrap_or(80);

    let config = RenderConfig {
        font_size: 12,
        width: (max_line_len as u32 * 8).max(600),
        height: (lines as u32 * 16).max(400),
        ..Default::default()
    };

    render_text_as_image(chart_text, &config)
}

/// Check if ImageMagick is available.
pub fn is_imagemagick_available() -> bool {
    std::process::Command::new("which")
        .arg("convert")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Install ImageMagick if needed.
pub async fn ensure_imagemagick() -> anyhow::Result<()> {
    if is_imagemagick_available() {
        return Ok(());
    }

    tracing::info!("Installing ImageMagick for chart rendering...");

    let output = std::process::Command::new("sudo")
        .arg("pacman")
        .arg("-S")
        .arg("--noconfirm")
        .arg("imagemagick")
        .output()?;

    if !output.status.success() {
        anyhow::bail!("Failed to install ImageMagick");
    }

    // Mark as installed by Anna
    use crate::tool_manager::InstalledTools;
    let mut tools = InstalledTools::load();
    tools.mark_installed("imagemagick");
    tools.save()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_config_defaults() {
        let config = RenderConfig::default();
        assert_eq!(config.font_size, 12);
        assert_eq!(config.width, 800);
    }

    #[test]
    fn test_imagemagick_check() {
        // Just check that the function doesn't panic
        let _ = is_imagemagick_available();
    }
}
