//! Helper utilities for translation.

/// Infer unit from fact name.
pub fn infer_unit(fact_name: &str) -> &'static str {
    let lower = fact_name.to_lowercase();
    if lower.contains("gib") || lower.contains("_gib") {
        " GiB"
    } else if lower.contains("mib") || lower.contains("_mib") {
        " MiB"
    } else if lower.contains("pct") || lower.contains("percent") {
        "%"
    } else if lower.contains("_s") || lower.contains("time_s") || lower.contains("seconds") {
        "s"
    } else if lower.contains("_ms") {
        "ms"
    } else if lower.contains("temp") || lower.contains("_c") {
        "°C"
    } else if lower.contains("count") {
        ""
    } else {
        ""
    }
}
