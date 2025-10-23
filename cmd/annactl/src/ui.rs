use serde::Deserialize;
use std::env;
use std::fs;
use std::io::IsTerminal;
use std::path::Path;
use time::format_description::well_known::Rfc3339;
use time::{format_description, OffsetDateTime, UtcOffset};

const CONFIG_PATH: &str = "/etc/anna/config.toml";
const DEFAULT_FORMAT: &str = "[year]-[month]-[day] [hour]:[minute]:[second]";

#[derive(Debug, Clone)]
pub struct UiCfg {
    pub fancy: bool,
    pub datetime_format: Option<String>,
}

impl Default for UiCfg {
    fn default() -> Self {
        Self {
            fancy: true,
            datetime_format: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Style {
    pub color: bool,
    pub emoji: bool,
    pub bold: bool,
}

#[derive(Deserialize, Default)]
struct RawUiCfg {
    #[serde(default)]
    fancy: Option<bool>,
    #[serde(default)]
    datetime_format: Option<String>,
}

#[derive(Deserialize, Default)]
struct RawConfig {
    #[serde(default)]
    ui: RawUiCfg,
}

pub fn load_ui_cfg() -> UiCfg {
    let path = Path::new(CONFIG_PATH);
    if let Ok(data) = fs::read_to_string(path) {
        if let Ok(raw) = toml::from_str::<RawConfig>(&data) {
            let mut cfg = UiCfg::default();
            if let Some(fancy) = raw.ui.fancy {
                cfg.fancy = fancy;
            }
            cfg.datetime_format = raw.ui.datetime_format.filter(|s| !s.trim().is_empty());
            return cfg;
        }
    }
    UiCfg::default()
}

pub fn detect_style(cfg: &UiCfg) -> Style {
    if env::var_os("NO_COLOR").is_some() {
        return Style {
            color: false,
            emoji: cfg.fancy && supports_emoji(),
            bold: false,
        };
    }
    let stdout_is_tty = std::io::stdout().is_terminal();
    if !stdout_is_tty || !cfg.fancy {
        return Style {
            color: false,
            emoji: cfg.fancy && supports_emoji(),
            bold: false,
        };
    }
    Style {
        color: true,
        emoji: supports_emoji(),
        bold: true,
    }
}

fn supports_emoji() -> bool {
    env::var("LC_ALL")
        .or_else(|_| env::var("LC_CTYPE"))
        .or_else(|_| env::var("LANG"))
        .map(|val| val.to_uppercase().contains("UTF-8"))
        .unwrap_or(false)
}

pub fn fmt_local(iso: &str, cfg: &UiCfg) -> String {
    let parsed = match OffsetDateTime::parse(iso, &Rfc3339) {
        Ok(dt) => dt,
        Err(_) => return iso.to_string(),
    };
    let offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
    let local = parsed.to_offset(offset);
    let pattern = cfg.datetime_format.as_deref().unwrap_or(DEFAULT_FORMAT);
    let owned_pattern;
    let pattern_ref: &str = if let Some(converted) = convert_strftime(pattern) {
        owned_pattern = converted;
        &owned_pattern
    } else {
        pattern
    };
    let fmt = match format_description::parse(pattern_ref) {
        Ok(desc) => desc,
        Err(_) => match format_description::parse(DEFAULT_FORMAT) {
            Ok(desc) => desc,
            Err(_) => return iso.to_string(),
        },
    };
    match local.format(&fmt) {
        Ok(rendered) => rendered,
        Err(_) => iso.to_string(),
    }
}

fn convert_strftime(pattern: &str) -> Option<String> {
    if !pattern.contains('%') {
        return None;
    }

    let mut result = String::new();
    let mut chars = pattern.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '%' {
            result.push(ch);
            continue;
        }
        match chars.next() {
            Some('%') => result.push('%'),
            Some('Y') => result.push_str("[year]"),
            Some('m') => result.push_str("[month]"),
            Some('d') => result.push_str("[day]"),
            Some('H') => result.push_str("[hour]"),
            Some('M') => result.push_str("[minute]"),
            Some('S') => result.push_str("[second]"),
            Some('F') => result.push_str("[year]-[month]-[day]"),
            Some('T') => result.push_str("[hour]:[minute]:[second]"),
            Some(_other) => {
                // Unsupported token; abort conversion to avoid incorrect formats.
                return None;
            }
            None => return None,
        }
    }
    Some(result)
}

pub fn head(style: &Style, title: &str) -> String {
    decorate(style, title, "\x1b[36m", Some('⚙'))
}

pub fn ok(style: &Style, text: &str) -> String {
    decorate(style, text, "\x1b[32m", Some('✅'))
}

pub fn warn(style: &Style, text: &str) -> String {
    decorate(style, text, "\x1b[33m", Some('⚠'))
}

pub fn err(style: &Style, text: &str) -> String {
    decorate(style, text, "\x1b[31m", Some('✖'))
}

pub fn bullet(style: &Style, text: &str) -> String {
    if style.emoji {
        format!(" • {text}")
    } else {
        format!(" - {text}")
    }
}

pub fn kv(style: &Style, key: &str, value: &str) -> String {
    if style.color {
        format!("{}{}{}: {}", bold_prefix(style), key, reset(style), value)
    } else {
        format!("{}: {}", key, value)
    }
}

fn decorate(style: &Style, text: &str, color: &str, emoji: Option<char>) -> String {
    let emoji_prefix = if style.emoji {
        emoji.map(|c| format!("{c} ")).unwrap_or_default()
    } else {
        String::new()
    };
    if style.color {
        format!(
            "{}{}{}{}{}",
            emoji_prefix,
            color,
            bold_prefix(style),
            text,
            reset(style)
        )
    } else {
        format!("{}{}", emoji_prefix, text)
    }
}

fn bold_prefix(style: &Style) -> &'static str {
    if style.bold {
        "\x1b[1m"
    } else {
        ""
    }
}

fn reset(style: &Style) -> &'static str {
    if style.color || style.bold {
        "\x1b[0m"
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_local_uses_format() {
        let iso = "2025-01-02T03:04:05Z";
        let cfg = UiCfg {
            fancy: true,
            datetime_format: Some("%Y/%m/%d %H:%M".into()),
        };
        let expected = {
            let parsed = OffsetDateTime::parse(iso, &Rfc3339).unwrap();
            let offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
            let local = parsed.to_offset(offset);
            let pattern = convert_strftime("%Y/%m/%d %H:%M").unwrap();
            let fmt = format_description::parse(&pattern).unwrap();
            local.format(&fmt).unwrap()
        };
        assert_eq!(fmt_local(iso, &cfg), expected);
    }
}
