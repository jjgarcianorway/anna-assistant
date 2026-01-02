//! Utility functions for wiki cache.

/// Normalize title for consistent lookup
pub fn normalize_title(title: &str) -> String {
    title.to_lowercase().replace(' ', "_").replace('/', "_")
}

/// Simple string hash for change detection
pub fn simple_hash(s: &str) -> String {
    let mut hash: u64 = 0;
    for (i, b) in s.bytes().enumerate() {
        hash = hash.wrapping_add((b as u64).wrapping_mul((i + 1) as u64));
    }
    format!("{:016x}", hash)
}

/// Get current timestamp in seconds since Unix epoch
pub fn now_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_title() {
        assert_eq!(normalize_title("Arch Wiki"), "arch_wiki");
        assert_eq!(normalize_title("Systemd/User"), "systemd_user");
        assert_eq!(normalize_title("GRUB"), "grub");
    }

    #[test]
    fn test_simple_hash() {
        let h1 = simple_hash("hello");
        let h2 = simple_hash("world");
        let h3 = simple_hash("hello");

        assert_ne!(h1, h2);
        assert_eq!(h1, h3);
    }
}
