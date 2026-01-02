//! Learning stats section (v0.0.330).

use anna_shared::probe_learning::{LearningHealth, ProbeLearningStore, TrendDirection};
use anna_shared::ui::{colors, kv, print_section_header};

/// v0.0.330: Print probe learning statistics section
pub fn print_learning_section() {
    let store = ProbeLearningStore::load();
    let stats = store.learning_stats();

    // Only show if there's something to show
    if stats.total_queries == 0 && stats.keywords_learned == 0 {
        return;
    }

    println!();
    print_section_header("learning");

    kv("queries_processed", &format!("{}", stats.total_queries));
    kv("keywords_learned", &format!("{}", stats.keywords_learned));

    if stats.successful_patterns > 0 || stats.negative_patterns > 0 {
        kv(
            "patterns",
            &format!(
                "{}{} success{} / {}{} negative{}",
                colors::OK,
                stats.successful_patterns,
                colors::RESET,
                colors::DIM,
                stats.negative_patterns,
                colors::RESET
            ),
        );
    }

    if stats.avg_quality > 0.0 {
        let quality_color = if stats.avg_quality >= 4.0 {
            colors::OK
        } else if stats.avg_quality >= 3.0 {
            colors::WARN
        } else {
            colors::DIM
        };
        kv(
            "avg_quality",
            &format!(
                "{}{:.1}/5{}",
                quality_color,
                stats.avg_quality,
                colors::RESET
            ),
        );
    }

    // Learning stage indicator
    let stage = if stats.total_queries >= 50 && stats.keywords_learned >= 20 {
        format!("{}Expert{}", colors::OK, colors::RESET)
    } else if stats.total_queries >= 10 {
        format!("{}Growing{}", colors::WARN, colors::RESET)
    } else {
        format!("{}Learning{}", colors::DIM, colors::RESET)
    };
    kv("stage", &stage);

    // v0.0.331: Quality trend
    if let Some(trend) = store.quality_trend() {
        let (trend_icon, trend_color) = match trend.trend {
            TrendDirection::Improving => ("^", colors::OK),
            TrendDirection::Declining => ("v", colors::ERR),
            TrendDirection::Stable => ("=", colors::DIM),
        };
        kv(
            "trend",
            &format!(
                "{}{}{} {} (was {:.1}, now {:.1})",
                trend_color,
                trend_icon,
                colors::RESET,
                trend.trend,
                trend.previous_avg,
                trend.current_avg
            ),
        );
    }

    // v0.0.332: Health status and confidence
    let health = store.health_status();
    let health_color = match health {
        LearningHealth::Excellent => colors::OK,
        LearningHealth::Good => colors::OK,
        LearningHealth::Developing => colors::WARN,
        LearningHealth::NeedsAttention => colors::ERR,
        LearningHealth::Insufficient => colors::DIM,
    };
    let confidence = store.confidence_factor();
    kv(
        "health",
        &format!(
            "{}{}{} ({:.0}% confidence)",
            health_color,
            health,
            colors::RESET,
            confidence * 100.0
        ),
    );
}
