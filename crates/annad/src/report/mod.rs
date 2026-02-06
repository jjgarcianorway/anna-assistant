//! PDF Report Generation - Professional system health reports.
//!
//! Generates personalized PDF reports with:
//! - Charts showing trends (disk, memory, CPU over time)
//! - Natural language summaries
//! - Actionable recommendations
//! - Personalized content based on user patterns

mod charts;
mod profile;
mod recommendations;
mod sections;
mod user;

use chrono::Local;
use genpdf::{elements, fonts, style, Document, Element, SimplePageDecorator};
use std::path::PathBuf;
use tracing::{info, warn};

pub use user::ReportPreferences;

/// Generate the daily PDF report with charts and personalized content.
pub fn generate_pdf_report() -> Result<PathBuf, String> {
    let prefs = ReportPreferences::load();
    let user_profile = profile::UserProfile::load();
    let now = Local::now();

    // Try to load fonts
    let font_family = fonts::from_files("/usr/share/fonts/noto", "NotoSans", None)
        .or_else(|_| fonts::from_files("/usr/share/fonts/TTF", "DejaVuSans", None))
        .or_else(|_| fonts::from_files("/usr/share/fonts/truetype/dejavu", "DejaVuSans", None))
        .map_err(|e| format!("Font error: {}", e))?;

    let mut doc = Document::new(font_family);
    doc.set_title("Anna System Report");

    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);

    // Title and greeting (use profile name if available)
    let display_name = if !user_profile.name.is_empty() {
        Some(user_profile.name.clone())
    } else {
        prefs.user_name.clone()
    };
    let greeting = sections::generate_greeting(&ReportPreferences { user_name: display_name }, &now);
    doc.push(elements::Paragraph::new(&greeting)
        .styled(style::Style::new().bold().with_font_size(18)));
    doc.push(elements::Paragraph::new(now.format("%A, %B %d, %Y at %H:%M").to_string())
        .styled(style::Style::new().with_font_size(10)));
    doc.push(elements::Break::new(1.5));

    // Executive Summary
    let summary = sections::generate_executive_summary();
    doc.push(elements::Paragraph::new("Overview")
        .styled(style::Style::new().bold().with_font_size(14)));
    doc.push(elements::Break::new(0.3));
    doc.push(elements::Paragraph::new(&summary));
    doc.push(elements::Break::new(1.0));

    // Current Status
    doc.push(elements::Paragraph::new("System Status")
        .styled(style::Style::new().bold().with_font_size(14)));
    doc.push(elements::Break::new(0.3));
    for line in sections::generate_status_section().lines() {
        if !line.is_empty() {
            doc.push(elements::Paragraph::new(line));
        }
    }
    doc.push(elements::Break::new(1.0));

    // 24-Hour Metrics Chart
    // v0.3.134: Temporarily disabled due to font rendering crash
    // TODO: Fix plotters font issue before re-enabling charts
    // let chart_path = PathBuf::from("/tmp/anna_metrics_chart.png");
    // if charts::render_metrics_chart(&chart_path).is_ok() {
    //     ...
    // }

    // Fallback to text summary (always use this for now)
    let metrics_summary = sections::generate_metrics_summary();
    if !metrics_summary.is_empty() {
        doc.push(elements::Paragraph::new("24-Hour Trends")
            .styled(style::Style::new().bold().with_font_size(14)));
        doc.push(elements::Break::new(0.3));
        doc.push(elements::Paragraph::new(&metrics_summary));
        doc.push(elements::Break::new(1.0));
    }

    // Software Updates
    let updates_section = sections::generate_updates_section();
    doc.push(elements::Paragraph::new("Software Updates")
        .styled(style::Style::new().bold().with_font_size(14)));
    doc.push(elements::Break::new(0.3));
    doc.push(elements::Paragraph::new(&updates_section));
    doc.push(elements::Break::new(1.0));

    // Your Activity (personalized section)
    let interests = user_profile.generate_interests_section();
    if !interests.contains("still being learned") {
        doc.push(elements::Paragraph::new("Your Activity")
            .styled(style::Style::new().bold().with_font_size(14)));
        doc.push(elements::Break::new(0.3));
        for line in interests.lines() {
            doc.push(elements::Paragraph::new(line));
        }
        doc.push(elements::Break::new(1.0));
    }

    // Smart Recommendations (personalized)
    let smart_recs = recommendations::generate_smart_recommendations(&user_profile);
    if !smart_recs.is_empty() {
        doc.push(elements::Paragraph::new("Recommendations For You")
            .styled(style::Style::new().bold().with_font_size(14)));
        doc.push(elements::Break::new(0.3));
        for rec in recommendations::format_recommendations_for_pdf(&smart_recs) {
            doc.push(elements::Paragraph::new(format!("* {}", rec)));
        }
        doc.push(elements::Break::new(1.0));
    } else {
        // Fallback to generic recommendations
        let generic_recs = sections::generate_recommendations();
        if !generic_recs.is_empty() {
            doc.push(elements::Paragraph::new("Recommendations")
                .styled(style::Style::new().bold().with_font_size(14)));
            doc.push(elements::Break::new(0.3));
            for rec in &generic_recs {
                doc.push(elements::Paragraph::new(format!("* {}", rec)));
            }
            doc.push(elements::Break::new(1.0));
        }
    }

    // Automated Maintenance
    doc.push(elements::Paragraph::new("Automated Maintenance")
        .styled(style::Style::new().bold().with_font_size(14)));
    doc.push(elements::Break::new(0.3));
    doc.push(elements::Paragraph::new(sections::generate_healing_section()));
    doc.push(elements::Break::new(1.0));

    // Closing
    let closing = sections::generate_closing();
    doc.push(elements::Paragraph::new(&closing)
        .styled(style::Style::new().italic()));

    // Save to file
    let filename = format!("anna_report_{}.pdf", now.format("%Y%m%d_%H%M"));
    let path = PathBuf::from("/tmp").join(&filename);

    doc.render_to_file(&path)
        .map_err(|e| format!("Failed to generate PDF: {}", e))?;

    info!("Generated PDF report: {}", path.display());
    Ok(path)
}
