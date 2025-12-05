//! Parser for `df -h` output.
//!
//! Parses disk usage information into typed structs with exact byte values.

use super::atoms::{parse_percent, parse_size, ParseError, ParseErrorReason};
use serde::{Deserialize, Serialize};

/// Disk usage information for a single mount point.
/// All size values are in bytes (u64).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiskUsage {
    /// Filesystem device (e.g., "/dev/sda1")
    pub filesystem: String,
    /// Mount point (e.g., "/", "/home")
    pub mount: String,
    /// Total size in bytes
    pub size_bytes: u64,
    /// Used space in bytes
    pub used_bytes: u64,
    /// Available space in bytes
    pub available_bytes: u64,
    /// Percent used (0-100), taken directly from df's Use% column
    pub percent_used: u8,
}

/// Mount aliases for claim key mapping.
/// Maps common names to canonical mount paths.
const MOUNT_ALIASES: &[(&str, &str)] = &[
    ("root", "/"),
    ("home", "/home"),
    ("var", "/var"),
    ("tmp", "/tmp"),
    ("boot", "/boot"),
    ("usr", "/usr"),
    ("opt", "/opt"),
];

/// Resolve a mount alias to its canonical path.
/// Returns None if not a known alias.
pub fn resolve_mount_alias(alias: &str) -> Option<&'static str> {
    let alias_lower = alias.to_lowercase();
    MOUNT_ALIASES
        .iter()
        .find(|(a, _)| *a == alias_lower)
        .map(|(_, path)| *path)
}

/// Parse `df -h` output into a list of DiskUsage entries.
///
/// Expected format (standard df -h output):
/// ```text
/// Filesystem      Size  Used Avail Use% Mounted on
/// /dev/sda1        50G   35G   12G  75% /
/// /dev/sda2       200G  150G   40G  79% /home
/// tmpfs           7.8G  1.2M  7.8G   1% /dev/shm
/// ```
pub fn parse_df(probe_id: &str, output: &str) -> Result<Vec<DiskUsage>, ParseError> {
    let mut entries = Vec::new();
    let mut header_seen = false;

    for (line_idx, line) in output.lines().enumerate() {
        let line_num = line_idx + 1;
        let line = line.trim();

        // Skip empty lines
        if line.is_empty() {
            continue;
        }

        // Skip header line
        if line.starts_with("Filesystem") {
            header_seen = true;
            continue;
        }

        // Only parse after header
        if !header_seen {
            continue;
        }

        // Parse data row
        match parse_df_row(probe_id, line, line_num) {
            Ok(entry) => entries.push(entry),
            Err(e) => {
                // For df, we continue parsing other rows on error
                // but we should at least log or track the error
                // For now, propagate the first error
                return Err(e);
            }
        }
    }

    if entries.is_empty() && !header_seen {
        return Err(ParseError::new(
            probe_id,
            ParseErrorReason::MissingSection("Filesystem header".to_string()),
            output,
        ));
    }

    Ok(entries)
}

/// Parse a single df output row.
fn parse_df_row(probe_id: &str, line: &str, line_num: usize) -> Result<DiskUsage, ParseError> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    // Expected: Filesystem Size Used Avail Use% Mounted_on
    // But "Mounted on" can contain spaces, so we need at least 6 parts
    if parts.len() < 6 {
        return Err(
            ParseError::new(probe_id, ParseErrorReason::MalformedRow, line).with_line(line_num)
        );
    }

    let filesystem = parts[0].to_string();
    let size_bytes = parse_size(parts[1]).map_err(|reason| ParseError {
        probe_id: probe_id.to_string(),
        line_num: Some(line_num),
        raw: parts[1].to_string(),
        reason,
    })?;
    let used_bytes = parse_size(parts[2]).map_err(|reason| ParseError {
        probe_id: probe_id.to_string(),
        line_num: Some(line_num),
        raw: parts[2].to_string(),
        reason,
    })?;
    let available_bytes = parse_size(parts[3]).map_err(|reason| ParseError {
        probe_id: probe_id.to_string(),
        line_num: Some(line_num),
        raw: parts[3].to_string(),
        reason,
    })?;
    let percent_used = parse_percent(parts[4]).map_err(|reason| ParseError {
        probe_id: probe_id.to_string(),
        line_num: Some(line_num),
        raw: parts[4].to_string(),
        reason,
    })?;

    // Mount point is the rest of the line (may contain spaces)
    // But typically it's just parts[5] for simple mounts
    let mount = if parts.len() == 6 {
        parts[5].to_string()
    } else {
        // Reconstruct mount path from remaining parts
        parts[5..].join(" ")
    };

    Ok(DiskUsage {
        filesystem,
        mount,
        size_bytes,
        used_bytes,
        available_bytes,
        percent_used,
    })
}

/// Find disk usage for a specific mount point.
pub fn find_by_mount<'a>(entries: &'a [DiskUsage], mount: &str) -> Option<&'a DiskUsage> {
    // First try exact match
    if let Some(entry) = entries.iter().find(|e| e.mount == mount) {
        return Some(entry);
    }

    // Then try alias resolution
    if let Some(canonical) = resolve_mount_alias(mount) {
        return entries.iter().find(|e| e.mount == canonical);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const DF_OUTPUT_STANDARD: &str = r#"Filesystem      Size  Used Avail Use% Mounted on
/dev/sda1        50G   35G   12G  75% /
/dev/sda2       200G  150G   40G  79% /home
tmpfs           7.8G  1.2M  7.8G   1% /dev/shm
"#;

    const DF_OUTPUT_TMPFS: &str = r#"Filesystem      Size  Used Avail Use% Mounted on
tmpfs           1.6G  2.1M  1.6G   1% /run
tmpfs           7.8G     0  7.8G   0% /sys/fs/cgroup
"#;

    #[test]
    fn golden_parse_df_standard() {
        let entries = parse_df("df", DF_OUTPUT_STANDARD).unwrap();
        assert_eq!(entries.len(), 3);

        // Root partition
        let root = &entries[0];
        assert_eq!(root.filesystem, "/dev/sda1");
        assert_eq!(root.mount, "/");
        assert_eq!(root.size_bytes, 53_687_091_200); // 50G
        assert_eq!(root.used_bytes, 37_580_963_840); // 35G
        assert_eq!(root.available_bytes, 12_884_901_888); // 12G
        assert_eq!(root.percent_used, 75);

        // Home partition
        let home = &entries[1];
        assert_eq!(home.mount, "/home");
        assert_eq!(home.size_bytes, 214_748_364_800); // 200G
        assert_eq!(home.percent_used, 79);

        // tmpfs
        let tmpfs = &entries[2];
        assert_eq!(tmpfs.mount, "/dev/shm");
        assert_eq!(tmpfs.percent_used, 1);
    }

    #[test]
    fn golden_parse_df_zero_percent() {
        let entries = parse_df("df", DF_OUTPUT_TMPFS).unwrap();
        let cgroup = &entries[1];
        assert_eq!(cgroup.percent_used, 0);
        assert_eq!(cgroup.used_bytes, 0);
    }

    #[test]
    fn golden_find_by_mount() {
        let entries = parse_df("df", DF_OUTPUT_STANDARD).unwrap();

        // Exact match
        assert_eq!(find_by_mount(&entries, "/").unwrap().mount, "/");
        assert_eq!(find_by_mount(&entries, "/home").unwrap().mount, "/home");

        // Alias match
        assert_eq!(find_by_mount(&entries, "root").unwrap().mount, "/");
        assert_eq!(find_by_mount(&entries, "home").unwrap().mount, "/home");

        // Not found
        assert!(find_by_mount(&entries, "/nonexistent").is_none());
        assert!(find_by_mount(&entries, "disk").is_none()); // vague, non-auditable
    }

    #[test]
    fn golden_parse_df_missing_header() {
        let output = "/dev/sda1 50G 35G 12G 75% /";
        let result = parse_df("df", output);
        assert!(result.is_err());
    }

    #[test]
    fn golden_parse_df_malformed_row() {
        let output = r#"Filesystem      Size  Used Avail Use% Mounted on
/dev/sda1        50G
"#;
        let result = parse_df("df", output);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err.reason, ParseErrorReason::MalformedRow));
        assert_eq!(err.line_num, Some(2));
    }

    #[test]
    fn golden_mount_alias_resolution() {
        assert_eq!(resolve_mount_alias("root"), Some("/"));
        assert_eq!(resolve_mount_alias("ROOT"), Some("/")); // case insensitive
        assert_eq!(resolve_mount_alias("home"), Some("/home"));
        assert_eq!(resolve_mount_alias("disk"), None); // not an alias
        assert_eq!(resolve_mount_alias(""), None);
    }
}
