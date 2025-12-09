//! Tests for atoms module (v0.0.173).

use super::atoms::*;

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
    assert_eq!(
        parse_display_size("-5GB"),
        Err(ParseErrorReason::NegativeValue)
    );
}
