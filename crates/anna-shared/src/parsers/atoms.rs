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
    let suffix_start = s
        .find(|c: char| c.is_ascii_alphabetic())
        .unwrap_or(s.len());

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
    ".service", ".socket", ".timer", ".mount", ".target", ".path", ".slice", ".scope", ".device",
    ".automount", ".swap",
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

#[cfg(test)]
mod tests {
    use super::*;

    // === parse_size golden tests ===

    #[test]
    fn golden_parse_size_integer_no_rounding() {
        assert_eq!(parse_size("500M"), Ok(524_288_000));
        assert_eq!(parse_size("1T"), Ok(1_099_511_627_776));
        assert_eq!(parse_size("1024"), Ok(1024));
        assert_eq!(parse_size("0"), Ok(0));
    }

    #[test]
    fn golden_parse_size_decimal_rounding() {
        // 4.2G: 42/10 * 1024³, using half-up rounding
        // = (42 * 1073741824 * 2 + 10) / 20 = 4509715661
        assert_eq!(parse_size("4.2G"), Ok(4_509_715_661));
        // 1.5 * 1024⁴ = 1649267441664 (exact, no rounding needed)
        assert_eq!(parse_size("1.5T"), Ok(1_649_267_441_664));
    }

    #[test]
    fn golden_parse_size_ties_half_up() {
        // 0.5 bytes → 1 (0.00048828125 * 1024 = 0.5 exactly)
        assert_eq!(parse_size("0.00048828125K"), Ok(1));
        // 1.5 bytes → 2 (0.00146484375 * 1024 = 1.5 exactly)
        assert_eq!(parse_size("0.00146484375K"), Ok(2));
        // 0.4 bytes → 0 (rounds down)
        assert_eq!(parse_size("0.000390625K"), Ok(0));
    }

    #[test]
    fn golden_parse_size_edge_cases() {
        // Leading/trailing whitespace OK
        assert_eq!(parse_size("  4G  "), Ok(4_294_967_296));
        // Case insensitive suffix
        assert_eq!(parse_size("1g"), Ok(1_073_741_824));
        assert_eq!(parse_size("1Gi"), Ok(1_073_741_824));
        // .5G is valid (0.5G)
        assert_eq!(parse_size(".5G"), Ok(536_870_912));
        // 5.G is valid (5.0G)
        assert_eq!(parse_size("5.G"), Ok(5_368_709_120));
        // 5. alone is 5 bytes (no suffix)
        assert_eq!(parse_size("5."), Ok(5));
        // B suffix accepted
        assert_eq!(parse_size("1024B"), Ok(1024));
    }

    #[test]
    fn golden_parse_size_errors() {
        // Negative
        assert_eq!(parse_size("-5G"), Err(ParseErrorReason::NegativeValue));
        assert_eq!(parse_size("-0"), Err(ParseErrorReason::NegativeValue));
        // Leading + rejected
        assert_eq!(parse_size("+5G"), Err(ParseErrorReason::InvalidNumber));
        // Empty
        assert_eq!(parse_size(""), Err(ParseErrorReason::EmptyNumber));
        assert_eq!(parse_size("G"), Err(ParseErrorReason::EmptyNumber));
        assert_eq!(parse_size("."), Err(ParseErrorReason::EmptyNumber));
        // Non-numeric (no numeric part, so EmptyNumber)
        assert_eq!(parse_size("abc"), Err(ParseErrorReason::EmptyNumber));
        // Unknown suffix
        assert_eq!(
            parse_size("5X"),
            Err(ParseErrorReason::UnknownSuffix("X".to_string()))
        );
        assert_eq!(
            parse_size("5GB"),
            Err(ParseErrorReason::UnknownSuffix("GB".to_string()))
        );
    }

    // === parse_percent golden tests ===

    #[test]
    fn golden_parse_percent_valid() {
        assert_eq!(parse_percent("0%"), Ok(0));
        assert_eq!(parse_percent("85%"), Ok(85));
        assert_eq!(parse_percent("100%"), Ok(100));
        assert_eq!(parse_percent("85"), Ok(85)); // without %
        assert_eq!(parse_percent("  50%  "), Ok(50)); // with whitespace
    }

    #[test]
    fn golden_parse_percent_errors() {
        assert_eq!(
            parse_percent("101%"),
            Err(ParseErrorReason::PercentOutOfRange(101))
        );
        assert_eq!(parse_percent("-5%"), Err(ParseErrorReason::NegativeValue));
        assert_eq!(parse_percent(""), Err(ParseErrorReason::EmptyNumber));
        assert_eq!(parse_percent("abc"), Err(ParseErrorReason::InvalidNumber));
    }

    // === normalize_service_name golden tests ===

    #[test]
    fn golden_normalize_service_name() {
        assert_eq!(normalize_service_name("nginx"), "nginx.service");
        assert_eq!(normalize_service_name("nginx.service"), "nginx.service");
        assert_eq!(normalize_service_name("foo.socket"), "foo.socket");
        assert_eq!(normalize_service_name("sshd@paula"), "sshd@paula.service");
        assert_eq!(
            normalize_service_name("user@1000.service"),
            "user@1000.service"
        );
        assert_eq!(normalize_service_name("-.mount"), "-.mount");
        assert_eq!(normalize_service_name("  nginx  "), "nginx.service");
    }

    // === parse_display_size golden tests ===

    #[test]
    fn golden_parse_display_size_accepts_gb_mb_kb() {
        // GB/MB/KB are accepted and treated as binary (same as GiB/MiB/KiB)
        assert_eq!(parse_display_size("4GB"), Ok(4_294_967_296));
        assert_eq!(parse_display_size("4GiB"), Ok(4_294_967_296));
        assert_eq!(parse_display_size("500MB"), Ok(524_288_000));
        assert_eq!(parse_display_size("500MiB"), Ok(524_288_000));
        assert_eq!(parse_display_size("1KB"), Ok(1024));
        assert_eq!(parse_display_size("1KiB"), Ok(1024));
        assert_eq!(parse_display_size("1TB"), Ok(1_099_511_627_776));
    }

    #[test]
    fn golden_parse_display_size_decimal() {
        // 4.2GB = 4.2 * 1024³ = 4509715661 (same as 4.2G)
        assert_eq!(parse_display_size("4.2GB"), Ok(4_509_715_661));
        assert_eq!(parse_display_size("2.5MB"), Ok(2_621_440));
    }

    #[test]
    fn golden_parse_display_size_rejects_invalid() {
        assert_eq!(
            parse_display_size("5GB/s"),
            Err(ParseErrorReason::UnknownSuffix("GB/s".to_string()))
        );
        assert_eq!(parse_display_size("-5GB"), Err(ParseErrorReason::NegativeValue));
    }
}
