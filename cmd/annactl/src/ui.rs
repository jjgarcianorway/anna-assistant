use serde::Deserialize;
use std::env;
use std::fs;
use std::io::IsTerminal;
#[cfg(target_os = "linux")]
use std::sync::Once;
use time::format_description::well_known::Rfc3339;
use time::{format_description, OffsetDateTime, UtcOffset};

#[cfg(target_os = "linux")]
use std::ffi::{CStr, CString};

use crate::paths::AnnaPaths;

const DEFAULT_FORMAT: &str = "[year]-[month]-[day] [hour]:[minute]:[second]";

#[derive(Debug, Clone)]
pub struct UiCfg {
    pub fancy: bool,
    pub datetime_format: Option<String>,
    pub colors: bool,
    pub emojis: bool,
    pub theme: Theme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}

impl Default for UiCfg {
    fn default() -> Self {
        Self {
            fancy: true,
            datetime_format: None,
            colors: true,
            emojis: true,
            theme: Theme::Dark,
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
    #[serde(default)]
    colors: Option<bool>,
    #[serde(default)]
    emojis: Option<bool>,
    #[serde(default)]
    theme: Option<String>,
}

#[derive(Deserialize, Default)]
struct RawConfig {
    #[serde(default)]
    ui: RawUiCfg,
}

pub fn load_ui_cfg() -> UiCfg {
    load_ui_cfg_from_paths(&AnnaPaths::detect())
}

pub fn load_ui_cfg_from_paths(paths: &AnnaPaths) -> UiCfg {
    let config_path = paths.config_dir.join("config.toml");

    if let Ok(data) = fs::read_to_string(&config_path) {
        if let Ok(raw) = toml::from_str::<RawConfig>(&data) {
            let mut cfg = UiCfg::default();
            if let Some(fancy) = raw.ui.fancy {
                cfg.fancy = fancy;
            }
            if let Some(colors) = raw.ui.colors {
                cfg.colors = colors;
            }
            if let Some(emojis) = raw.ui.emojis {
                cfg.emojis = emojis;
            }
            if let Some(theme_str) = raw.ui.theme.as_deref() {
                cfg.theme = match theme_str.to_lowercase().as_str() {
                    "light" => Theme::Light,
                    _ => Theme::Dark,
                };
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
            emoji: cfg.emojis && supports_emoji(),
            bold: false,
        };
    }
    let stdout_is_tty = std::io::stdout().is_terminal();
    if !stdout_is_tty {
        return Style {
            color: false,
            emoji: cfg.emojis && supports_emoji(),
            bold: false,
        };
    }
    Style {
        color: cfg.colors,
        emoji: cfg.emojis && supports_emoji(),
        bold: cfg.colors,
    }
}

pub fn supports_emoji() -> bool {
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
    if let Some(pattern) = cfg.datetime_format.as_deref() {
        if let Some(rendered) = format_with_pattern(local, pattern) {
            return rendered;
        }
    } else if let Some(rendered) = format_with_locale(local) {
        return rendered;
    }

    format_with_pattern(local, DEFAULT_FORMAT).unwrap_or_else(|| iso.to_string())
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

fn format_with_pattern(dt: OffsetDateTime, pattern: &str) -> Option<String> {
    let owned;
    let pattern_ref = if let Some(converted) = convert_strftime(pattern) {
        owned = converted;
        &owned
    } else {
        pattern
    };
    let desc = format_description::parse(pattern_ref).ok()?;
    dt.format(&desc).ok()
}

fn format_with_locale(dt: OffsetDateTime) -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        format_with_locale_linux(dt)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = dt;
        None
    }
}

#[cfg(target_os = "linux")]
fn format_with_locale_linux(dt: OffsetDateTime) -> Option<String> {
    use std::mem;

    static INIT: Once = Once::new();
    INIT.call_once(|| unsafe {
        let locale = CString::new("").expect("empty locale");
        libc::setlocale(libc::LC_TIME, locale.as_ptr());
    });

    let mut tm: libc::tm = unsafe { mem::zeroed() };
    tm.tm_sec = dt.second() as i32;
    tm.tm_min = dt.minute() as i32;
    tm.tm_hour = dt.hour() as i32;
    tm.tm_mday = dt.day() as i32;
    tm.tm_mon = u8::from(dt.month()) as i32 - 1;
    tm.tm_year = dt.year() - 1900;
    tm.tm_wday = dt.weekday().number_days_from_sunday() as i32;
    tm.tm_yday = dt.ordinal() as i32 - 1;
    tm.tm_isdst = -1;
    #[cfg(any(target_os = "linux", target_os = "android"))]
    {
        tm.tm_gmtoff = dt.offset().whole_seconds() as libc::c_long;
        tm.tm_zone = std::ptr::null();
    }

    let fmt = CString::new("%c").ok()?;
    let mut buf = vec![0u8; 128];
    loop {
        let len = unsafe {
            libc::strftime(
                buf.as_mut_ptr() as *mut libc::c_char,
                buf.len(),
                fmt.as_ptr(),
                &tm,
            )
        };
        if len == 0 {
            if buf.len() >= 4096 {
                return None;
            }
            buf.resize(buf.len() * 2, 0);
            continue;
        }
        let s = unsafe { CStr::from_ptr(buf.as_ptr() as *const libc::c_char) };
        return Some(s.to_string_lossy().into_owned());
    }
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

pub fn info(style: &Style, text: &str) -> String {
    decorate(style, text, "\x1b[36m", Some('ℹ'))
}

pub fn step(style: &Style, text: &str) -> String {
    decorate(style, text, "\x1b[36m", Some('›'))
}

pub fn note(style: &Style, text: &str) -> String {
    decorate(style, text, "\x1b[33m", Some('•'))
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

pub fn cyan(style: &Style, text: &str) -> String {
    paint(style, text, &["\x1b[36m"])
}

pub fn yellow(style: &Style, text: &str) -> String {
    paint(style, text, &["\x1b[33m"])
}

pub fn dim_gray(style: &Style, text: &str) -> String {
    paint(style, text, &["\x1b[2m", "\x1b[37m"])
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

fn paint(style: &Style, text: &str, codes: &[&str]) -> String {
    if style.color {
        let mut out = String::new();
        for code in codes {
            out.push_str(code);
        }
        out.push_str(text);
        out.push_str(reset(style));
        out
    } else {
        text.to_string()
    }
}

/// Prints the unified banner: 🧩 Anna v{version} • Mode: {user|system} • {context}
pub fn banner(style: &Style, context: &str) {
    let paths = crate::paths::AnnaPaths::detect();
    let version = env!("CARGO_PKG_VERSION");
    let mode = paths.mode.as_str();

    let puzzle = if style.emoji { "🧩 " } else { "" };
    let dots = if style.emoji { " • " } else { " | " };

    let text = format!(
        "{}Anna v{}{}Mode: {}{}{}",
        puzzle, version, dots, mode, dots, context
    );

    if style.color {
        println!("\x1b[36m{}\x1b[0m\n", text);
    } else {
        println!("{}\n", text);
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
            colors: true,
            emojis: true,
            theme: Theme::Dark,
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
