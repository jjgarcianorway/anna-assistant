//! Report helper functions (v0.0.189).

/// Sanitize mount point for use in ID
pub fn sanitize_mount(mount: &str) -> String {
    if mount == "/" {
        "root".to_string()
    } else {
        mount.trim_start_matches('/').replace('/', "_")
    }
}

/// Format bytes as human-readable
pub fn format_bytes(bytes: u64) -> String {
    const GB: u64 = 1024 * 1024 * 1024;
    const MB: u64 = 1024 * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else {
        format!("{} B", bytes)
    }
}
