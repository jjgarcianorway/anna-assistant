//! Atomic parsing functions for probe output.
//!
//! All parsing is deterministic: no floats, no heuristics.
//! Size parsing uses rational arithmetic with exact rounding.

use serde::{Deserialize, Serialize};

/// Parse error with context for diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseError {
    /// Which probe produced this error
    pub probe_id: String,
    /// Line number where error occurred (1-indexed), if applicable
    pub line_num: Option<usize>,
    /// Raw input that failed to parse
    pub raw: String,
    /// Why parsing failed
    pub reason: ParseErrorReason,
}

/// Specific reason for parse failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParseErrorReason {
    /// Input was negative (starts with -)
    NegativeValue,
    /// No digits found in input
    EmptyNumber,
    /// Non-numeric characters in numeric field
    InvalidNumber,
    /// Size suffix not recognized (e.g., "X" instead of "G")
    UnknownSuffix(String),
    /// Result exceeds u64 range
    Overflow,
    /// Percent value > 100
    PercentOutOfRange(u8),
    /// Expected column not found
    MissingColumn(usize),
    /// Row format doesn't match expected structure
    MalformedRow,
    /// Required section not found in output
    MissingSection(String),
}

impl ParseError {
    pub fn new(probe_id: &str, reason: ParseErrorReason, raw: &str) -> Self {
        Self {
            probe_id: probe_id.to_string(),
            line_num: None,
            raw: raw.to_string(),
            reason,
        }
    }

    pub fn with_line(mut self, line_num: usize) -> Self {
        self.line_num = Some(line_num);
        self
    }
}

/// Parse a size string like "4.2G" into exact bytes.
///
/// Policy: round to nearest byte, ties half up.
/// Implementation: rational arithmetic, no floats.
///
/// Accepts binary prefixes: K/Ki, M/Mi, G/Gi, T/Ti (all treated as base-2).
/// No suffix = bytes.
pub fn parse_size(s: &str) -> Result<u64, ParseErrorReason> {
    let s = s.trim();

    // Reject negative before any parsing
    if s.starts_with('-') {
        return Err(ParseErrorReason::NegativeValue);
    }

    // Reject leading + (not accepted per spec)
    if s.starts_with('+') {
        return Err(ParseErrorReason::InvalidNumber);
    }

    let (num_str, suffix) = split_numeric_suffix(s)?;

    // Reject empty numeric part
    if num_str.is_empty() || num_str == "." {
        return Err(ParseErrorReason::EmptyNumber);
    }

    let multiplier: u128 = match suffix.to_uppercase().as_str() {
        "K" | "KI" | "KIB" => 1024,
        "M" | "MI" | "MIB" => 1024 * 1024,
        "G" | "GI" | "GIB" => 1024 * 1024 * 1024,
        "T" | "TI" | "TIB" => 1024 * 1024 * 1024 * 1024,
        "B" | "" => 1,
        _ => return Err(ParseErrorReason::UnknownSuffix(suffix.to_string())),
    };

    // Parse as rational: "4.2" → (42, 10)
    let (numerator, denominator) = parse_decimal_rational(num_str)?;

    // Check overflow before multiply
    let scaled_num = numerator
        .checked_mul(multiplier)
        .ok_or(ParseErrorReason::Overflow)?;

    // Round half up: (scaled_num * 2 + denominator) / (2 * denominator)
    // This is exact for all denominators, including odd ones.
    let doubled_scaled = scaled_num
        .checked_mul(2)
        .ok_or(ParseErrorReason::Overflow)?;
    let doubled_denom = denominator
        .checked_mul(2)
        .ok_or(ParseErrorReason::Overflow)?;
    let bytes = doubled_scaled
        .checked_add(denominator)
        .ok_or(ParseErrorReason::Overflow)?
        / doubled_denom;

    // Final overflow check for u64
    let bytes_u64: u64 = bytes.try_into().map_err(|_| ParseErrorReason::Overflow)?;

    Ok(bytes_u64)
}

/// Parse a display size string like "4.2GB" into exact bytes.
///
/// This is a superset of parse_size that also accepts common display formats:
/// GB, MB, KB (treated as binary, same as GiB/MiB/KiB).
///
/// Used for claim extraction where LLMs write human-friendly sizes.
pub fn parse_display_size(s: &str) -> Result<u64, ParseErrorReason> {
    let s = s.trim();

    if s.starts_with('-') {
        return Err(ParseErrorReason::NegativeValue);
    }

    if s.starts_with('+') {
        return Err(ParseErrorReason::InvalidNumber);
    }

    let (num_str, suffix) = split_numeric_suffix(s)?;

    if num_str.is_empty() || num_str == "." {
        return Err(ParseErrorReason::EmptyNumber);
    }

    // Accept both binary (GiB) and display (GB) suffixes, treat all as binary
    let multiplier: u128 = match suffix.to_uppercase().as_str() {
        "K" | "KI" | "KIB" | "KB" => 1024,
        "M" | "MI" | "MIB" | "MB" => 1024 * 1024,
        "G" | "GI" | "GIB" | "GB" => 1024 * 1024 * 1024,
        "T" | "TI" | "TIB" | "TB" => 1024 * 1024 * 1024 * 1024,
        "B" | "" => 1,
        _ => return Err(ParseErrorReason::UnknownSuffix(suffix.to_string())),
    };

    let (numerator, denominator) = parse_decimal_rational(num_str)?;

    let scaled_num = numerator
        .checked_mul(multiplier)
        .ok_or(ParseErrorReason::Overflow)?;

    let doubled_scaled = scaled_num
        .checked_mul(2)
        .ok_or(ParseErrorReason::Overflow)?;
    let doubled_denom = denominator
        .checked_mul(2)
        .ok_or(ParseErrorReason::Overflow)?;
    let bytes = doubled_scaled
        .checked_add(denominator)
        .ok_or(ParseErrorReason::Overflow)?
        / doubled_denom;

    let bytes_u64: u64 = bytes.try_into().map_err(|_| ParseErrorReason::Overflow)?;

    Ok(bytes_u64)
}

/// Split "4.2G" into ("4.2", "G").
/// Contract: input is already trimmed. Neither part contains whitespace.
fn split_numeric_suffix(s: &str) -> Result<(&str, &str), ParseErrorReason> {
    // Find where digits/decimal end and suffix begins
    let suffix_start = s.find(|c: char| c.is_ascii_alphabetic()).unwrap_or(s.len());

    let num_part = &s[..suffix_start];
    let suffix_part = &s[suffix_start..];

    Ok((num_part, suffix_part))
}

/// Parse decimal string to rational (numerator, denominator).
/// "4.2" → (42, 10), "500" → (500, 1), "0.125" → (125, 1000)
/// Rejects negative, empty, or malformed inputs.
fn parse_decimal_rational(s: &str) -> Result<(u128, u128), ParseErrorReason> {
    // Should not reach here with negative (caller checks), but defensive
    if s.starts_with('-') {
        return Err(ParseErrorReason::NegativeValue);
    }

    // Reject empty
    if s.is_empty() {
        return Err(ParseErrorReason::EmptyNumber);
    }

    if let Some((int_part, frac_part)) = s.split_once('.') {
        // Reject standalone "."
        if int_part.is_empty() && frac_part.is_empty() {
            return Err(ParseErrorReason::EmptyNumber);
        }

        let int_val: u128 = if int_part.is_empty() {
            0
        } else {
            int_part
                .parse()
                .map_err(|_| ParseErrorReason::InvalidNumber)?
        };

        let frac_len = frac_part.len();
        let frac_val: u128 = if frac_part.is_empty() {
            0
        } else {
            frac_part
                .parse()
                .map_err(|_| ParseErrorReason::InvalidNumber)?
        };

        let denominator: u128 = 10u128
            .checked_pow(frac_len as u32)
            .ok_or(ParseErrorReason::Overflow)?;
        let numerator = int_val
            .checked_mul(denominator)
            .ok_or(ParseErrorReason::Overflow)?
            .checked_add(frac_val)
            .ok_or(ParseErrorReason::Overflow)?;

        Ok((numerator, denominator))
    } else {
        let val: u128 = s.parse().map_err(|_| ParseErrorReason::InvalidNumber)?;
        Ok((val, 1))
    }
}

/// Parse a percent string like "85%" into u8.
/// Rejects values > 100, negative, or malformed.
pub fn parse_percent(s: &str) -> Result<u8, ParseErrorReason> {
    let s = s.trim().trim_end_matches('%');

    if s.starts_with('-') {
        return Err(ParseErrorReason::NegativeValue);
    }

    if s.is_empty() {
        return Err(ParseErrorReason::EmptyNumber);
    }

    let val: u8 = s.parse().map_err(|_| ParseErrorReason::InvalidNumber)?;

    if val > 100 {
        return Err(ParseErrorReason::PercentOutOfRange(val));
    }

    Ok(val)
}

/// Known systemd unit suffixes.
const KNOWN_UNIT_SUFFIXES: &[&str] = &[
    ".service",
    ".socket",
    ".timer",
    ".mount",
    ".target",
    ".path",
    ".slice",
    ".scope",
    ".device",
    ".automount",
    ".swap",
];

/// Normalize a service name to canonical form.
/// If no known suffix, appends ".service".
pub fn normalize_service_name(name: &str) -> String {
    let name = name.trim();

    // If already has a known suffix, return as-is
    for suffix in KNOWN_UNIT_SUFFIXES {
        if name.ends_with(suffix) {
            return name.to_string();
        }
    }

    // Handle templated instances: sshd@foo → sshd@foo.service
    format!("{}.service", name)
}
