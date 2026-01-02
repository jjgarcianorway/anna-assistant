//! File operations for recipe execution - backups, append, prepend.

/// Create backup of a file
pub fn create_backup(path: &str) -> Result<String, String> {
    let backup_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join(".anna")
        .join("backups");

    std::fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let filename = std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let backup_path = backup_dir.join(format!("{}_{}", filename, timestamp));

    if std::path::Path::new(path).exists() {
        std::fs::copy(path, &backup_path).map_err(|e| e.to_string())?;
    }

    Ok(backup_path.to_string_lossy().to_string())
}

/// Append content to file
pub fn append_to_file(path: &str, content: &str) -> Result<(), String> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    file.write_all(content.as_bytes())
        .map_err(|e| e.to_string())
}

/// Prepend content to file
pub fn prepend_to_file(path: &str, content: &str) -> Result<(), String> {
    let existing = std::fs::read_to_string(path).unwrap_or_default();
    let new_content = format!("{}{}", content, existing);
    std::fs::write(path, new_content).map_err(|e| e.to_string())
}
