//! Locale detection and regional formatting

use chrono::{DateTime, Local};
use std::env;

/// Detected locale information
#[derive(Debug, Clone)]
pub struct LocaleInfo {
    pub language: String,
    pub country: Option<String>,
    pub encoding: Option<String>,
}

/// Detect system locale from environment variables
pub fn detect_locale() -> LocaleInfo {
    let locale_str = env::var("LC_ALL")
        .or_else(|_| env::var("LC_TIME"))
        .or_else(|_| env::var("LANG"))
        .unwrap_or_else(|_| "en_US.UTF-8".to_string());

    parse_locale(&locale_str)
}

/// Parse locale string (e.g., "en_US.UTF-8" or "nb_NO.UTF-8")
fn parse_locale(locale: &str) -> LocaleInfo {
    let parts: Vec<&str> = locale.split('.').collect();
    let lang_country = parts[0];
    let encoding = parts.get(1).map(|s| s.to_string());

    let lang_parts: Vec<&str> = lang_country.split('_').collect();
    let language = lang_parts[0].to_string();
    let country = lang_parts.get(1).map(|s| s.to_string());

    LocaleInfo {
        language,
        country,
        encoding,
    }
}

/// Format timestamp according to locale
///
/// - en_US: Oct 30 2025 3:45 PM
/// - nb_NO: 30.10.2025 15:45
/// - Default: ISO 8601 format
pub fn format_timestamp(dt: &DateTime<Local>) -> String {
    let locale = detect_locale();

    match locale.language.as_str() {
        "en" => {
            // 12-hour format with AM/PM
            dt.format("%b %d %Y %I:%M %p").to_string()
        }
        "nb" | "no" => {
            // Norwegian: DD.MM.YYYY HH:MM
            dt.format("%d.%m.%Y %H:%M").to_string()
        }
        "de" => {
            // German: DD.MM.YYYY HH:MM
            dt.format("%d.%m.%Y %H:%M").to_string()
        }
        "fr" => {
            // French: DD/MM/YYYY HH:MM
            dt.format("%d/%m/%Y %H:%M").to_string()
        }
        "es" => {
            // Spanish: DD/MM/YYYY HH:MM
            dt.format("%d/%m/%Y %H:%M").to_string()
        }
        "ja" => {
            // Japanese: YYYY年MM月DD日 HH:MM
            dt.format("%Y年%m月%d日 %H:%M").to_string()
        }
        "zh" => {
            // Chinese: YYYY-MM-DD HH:MM
            dt.format("%Y-%m-%d %H:%M").to_string()
        }
        _ => {
            // Default: ISO 8601 with short time
            dt.format("%H:%M").to_string() // Just show time for brevity
        }
    }
}

/// Format a duration in human-readable form
pub fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        let mins = seconds / 60;
        let secs = seconds % 60;
        format!("{}m {}s", mins, secs)
    } else {
        let hours = seconds / 3600;
        let mins = (seconds % 3600) / 60;
        format!("{}h {}m", hours, mins)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_locale() {
        let locale = parse_locale("en_US.UTF-8");
        assert_eq!(locale.language, "en");
        assert_eq!(locale.country, Some("US".to_string()));
        assert_eq!(locale.encoding, Some("UTF-8".to_string()));

        let locale = parse_locale("nb_NO");
        assert_eq!(locale.language, "nb");
        assert_eq!(locale.country, Some("NO".to_string()));
        assert_eq!(locale.encoding, None);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3661), "1h 1m");
    }
}
