//! User detection and preferences for reports.

/// User preferences for reports
#[derive(Clone, Default)]
pub struct ReportPreferences {
    pub user_name: Option<String>,
}

impl ReportPreferences {
    pub fn load() -> Self {
        Self {
            user_name: detect_logged_in_user(),
        }
    }
}

/// Detect the actual logged-in user (not root when running as daemon).
pub fn detect_logged_in_user() -> Option<String> {
    // 1. Check SUDO_USER (if run via sudo)
    if let Ok(user) = std::env::var("SUDO_USER") {
        if !user.is_empty() && user != "root" {
            return Some(user);
        }
    }

    // 2. Check /run/user/* directories for logged-in user UIDs
    if let Ok(entries) = std::fs::read_dir("/run/user") {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                // Skip root (uid 0)
                if name == "0" {
                    continue;
                }
                // Convert UID to username
                if let Ok(uid) = name.parse::<u32>() {
                    if let Some(user) = uid_to_username(uid) {
                        return Some(user);
                    }
                }
            }
        }
    }

    // 3. Try `who` command to get logged-in users
    if let Ok(output) = std::process::Command::new("who").output() {
        let who_output = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = who_output.lines().next() {
            if let Some(user) = line.split_whitespace().next() {
                if !user.is_empty() && user != "root" {
                    return Some(user.to_string());
                }
            }
        }
    }

    // 4. Try `logname` command
    if let Ok(output) = std::process::Command::new("logname").output() {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !name.is_empty() && name != "root" {
            return Some(name);
        }
    }

    // 5. Fallback to USER env var
    std::env::var("USER").ok().filter(|u| !u.is_empty() && u != "root")
}

/// Convert UID to username using /etc/passwd
fn uid_to_username(uid: u32) -> Option<String> {
    if let Ok(passwd) = std::fs::read_to_string("/etc/passwd") {
        for line in passwd.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 {
                if let Ok(line_uid) = parts[2].parse::<u32>() {
                    if line_uid == uid {
                        return Some(parts[0].to_string());
                    }
                }
            }
        }
    }
    None
}
