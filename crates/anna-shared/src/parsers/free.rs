//! Parser for `free -h` output.
//!
//! Parses memory and swap information into typed structs with exact byte values.

use super::atoms::{parse_size, ParseError, ParseErrorReason};
use serde::{Deserialize, Serialize};

/// Memory information parsed from `free -h`.
/// All values are in bytes (u64).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// Total physical memory in bytes
    pub total_bytes: u64,
    /// Used memory in bytes
    pub used_bytes: u64,
    /// Free memory in bytes
    pub free_bytes: u64,
    /// Shared memory in bytes (may be 0 if not reported)
    pub shared_bytes: u64,
    /// Buffer/cache in bytes
    pub buff_cache_bytes: u64,
    /// Available memory in bytes
    pub available_bytes: u64,
    /// Swap total in bytes (None if no swap)
    pub swap_total_bytes: Option<u64>,
    /// Swap used in bytes (None if no swap)
    pub swap_used_bytes: Option<u64>,
    /// Swap free in bytes (None if no swap)
    pub swap_free_bytes: Option<u64>,
}

/// Parse `free -h` output into MemoryInfo.
///
/// Expected format (standard free -h output):
/// ```text
///               total        used        free      shared  buff/cache   available
/// Mem:          15Gi        8.2Gi       1.5Gi       512Mi       5.8Gi       6.5Gi
/// Swap:         4.0Gi       256Mi       3.8Gi
/// ```
///
/// Some systems may have different column arrangements. We parse by row label.
pub fn parse_free(probe_id: &str, output: &str) -> Result<MemoryInfo, ParseError> {
    let mut mem_row: Option<Vec<u64>> = None;
    let mut swap_row: Option<Vec<u64>> = None;

    for (line_idx, line) in output.lines().enumerate() {
        let line_num = line_idx + 1;
        let line = line.trim();

        if line.starts_with("Mem:") {
            mem_row = Some(parse_memory_row(probe_id, line, line_num)?);
        } else if line.starts_with("Swap:") {
            swap_row = Some(parse_swap_row(probe_id, line, line_num)?);
        }
    }

    let mem = mem_row.ok_or_else(|| {
        ParseError::new(
            probe_id,
            ParseErrorReason::MissingSection("Mem:".to_string()),
            output,
        )
    })?;

    // Mem row should have at least 6 values: total, used, free, shared, buff/cache, available
    if mem.len() < 6 {
        return Err(ParseError::new(
            probe_id,
            ParseErrorReason::MalformedRow,
            output,
        ));
    }

    Ok(MemoryInfo {
        total_bytes: mem[0],
        used_bytes: mem[1],
        free_bytes: mem[2],
        shared_bytes: mem[3],
        buff_cache_bytes: mem[4],
        available_bytes: mem[5],
        swap_total_bytes: swap_row.as_ref().and_then(|s| s.first().copied()),
        swap_used_bytes: swap_row.as_ref().and_then(|s| s.get(1).copied()),
        swap_free_bytes: swap_row.as_ref().and_then(|s| s.get(2).copied()),
    })
}

/// Parse the Mem: row into a vector of byte values.
fn parse_memory_row(probe_id: &str, line: &str, line_num: usize) -> Result<Vec<u64>, ParseError> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    // First part is "Mem:", rest are values
    if parts.len() < 7 {
        return Err(
            ParseError::new(probe_id, ParseErrorReason::MalformedRow, line).with_line(line_num)
        );
    }

    let mut values = Vec::new();
    for part in parts.iter().skip(1) {
        let bytes = parse_size(part).map_err(|reason| {
            ParseError {
                probe_id: probe_id.to_string(),
                line_num: Some(line_num),
                raw: part.to_string(),
                reason,
            }
        })?;
        values.push(bytes);
    }

    Ok(values)
}

/// Parse the Swap: row into a vector of byte values.
fn parse_swap_row(probe_id: &str, line: &str, line_num: usize) -> Result<Vec<u64>, ParseError> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    // First part is "Swap:", rest are values (total, used, free)
    if parts.len() < 4 {
        return Err(
            ParseError::new(probe_id, ParseErrorReason::MalformedRow, line).with_line(line_num)
        );
    }

    let mut values = Vec::new();
    for part in parts.iter().skip(1) {
        let bytes = parse_size(part).map_err(|reason| ParseError {
            probe_id: probe_id.to_string(),
            line_num: Some(line_num),
            raw: part.to_string(),
            reason,
        })?;
        values.push(bytes);
    }

    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FREE_OUTPUT_STANDARD: &str = r#"              total        used        free      shared  buff/cache   available
Mem:           15Gi       8.2Gi       1.5Gi       512Mi       5.8Gi       6.5Gi
Swap:         4.0Gi       256Mi       3.8Gi
"#;

    const FREE_OUTPUT_NO_SWAP: &str = r#"              total        used        free      shared  buff/cache   available
Mem:           15Gi       8.2Gi       1.5Gi       512Mi       5.8Gi       6.5Gi
Swap:            0B          0B          0B
"#;

    #[test]
    fn golden_parse_free_standard() {
        let info = parse_free("free", FREE_OUTPUT_STANDARD).unwrap();

        // 15Gi = 15 * 1024³ = 16106127360
        assert_eq!(info.total_bytes, 16_106_127_360);
        // 8.2Gi: 82/10 * 1024³ with half-up = 8804682957
        assert_eq!(info.used_bytes, 8_804_682_957);
        // 1.5Gi = 15/10 * 1024³ = 1610612736
        assert_eq!(info.free_bytes, 1_610_612_736);
        // 512Mi = 512 * 1024² = 536870912
        assert_eq!(info.shared_bytes, 536_870_912);
        // 5.8Gi: 58/10 * 1024³ with half-up = 6227702579
        assert_eq!(info.buff_cache_bytes, 6_227_702_579);
        // 6.5Gi: 65/10 * 1024³ with half-up = 6979321856
        assert_eq!(info.available_bytes, 6_979_321_856);

        // Swap
        // 4.0Gi: 40/10 * 1024³ = 4294967296
        assert_eq!(info.swap_total_bytes, Some(4_294_967_296));
        // 256Mi = 256 * 1024² = 268435456
        assert_eq!(info.swap_used_bytes, Some(268_435_456));
        // 3.8Gi: 38/10 * 1024³ with half-up = 4080218931
        assert_eq!(info.swap_free_bytes, Some(4_080_218_931));
    }

    #[test]
    fn golden_parse_free_no_swap() {
        let info = parse_free("free", FREE_OUTPUT_NO_SWAP).unwrap();

        assert_eq!(info.swap_total_bytes, Some(0));
        assert_eq!(info.swap_used_bytes, Some(0));
        assert_eq!(info.swap_free_bytes, Some(0));
    }

    #[test]
    fn golden_parse_free_missing_mem() {
        let output = "Some garbage\nno memory info here";
        let result = parse_free("free", output);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(
            err.reason,
            ParseErrorReason::MissingSection(ref s) if s == "Mem:"
        ));
    }

    #[test]
    fn golden_parse_free_malformed_row() {
        let output = r#"              total        used
Mem:           15Gi
"#;
        let result = parse_free("free", output);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err.reason, ParseErrorReason::MalformedRow));
    }
}
