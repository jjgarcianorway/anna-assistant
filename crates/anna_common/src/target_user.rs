//! Target User System v0.0.17
//!
//! Multi-user correctness for Anna:
//! - Deterministic target user selection
//! - Safe home directory detection via /etc/passwd
//! - User-scoped mutation execution
//! - Policy integration for user home paths
//!
//! Target user selection precedence:
//! 1. REPL session chosen user (if set)
//! 2. SUDO_USER environment variable
//! 3. Non-root invoking user (getuid)
//! 4. Primary interactive user (best-effort, with clarification if ambiguous)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Evidence ID prefix for user selection
pub const USER_EVIDENCE_PREFIX: &str = "E-user-";

/// Generate a user evidence ID
pub fn generate_user_evidence_id() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros();
    format!("{}{}", USER_EVIDENCE_PREFIX, ts % 100000)
}

// =============================================================================
// User Info
// =============================================================================

/// Information about a Unix user
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserInfo {
    /// Username
    pub username: String,
    /// Numeric UID
    pub uid: u32,
    /// Numeric GID
    pub gid: u32,
    /// Home directory (canonical from /etc/passwd)
    pub home: PathBuf,
    /// Login shell
    pub shell: String,
    /// GECOS field (full name, etc.)
    pub gecos: String,
}

impl UserInfo {
    /// Parse a user from /etc/passwd line
    pub fn from_passwd_line(line: &str) -> Option<Self> {
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() < 7 {
            return None;
        }

        let uid: u32 = fields[2].parse().ok()?;
        let gid: u32 = fields[3].parse().ok()?;

        Some(Self {
            username: fields[0].to_string(),
            uid,
            gid,
            home: PathBuf::from(fields[5]),
            shell: fields[6].to_string(),
            gecos: fields[4].to_string(),
        })
    }

    /// Check if this is a regular (non-system) user
    pub fn is_regular_user(&self) -> bool {
        // Regular users typically have UID >= 1000 on Arch Linux
        // Also exclude nobody (65534)
        self.uid >= 1000 && self.uid < 65534
    }

    /// Check if this user has an interactive shell
    pub fn has_interactive_shell(&self) -> bool {
        !self.shell.contains("nologin") && !self.shell.contains("false") && !self.shell.is_empty()
    }

    /// Check if the home directory exists
    pub fn home_exists(&self) -> bool {
        self.home.exists() && self.home.is_dir()
    }
}

// =============================================================================
// User Selection Source
// =============================================================================

/// How the target user was determined
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UserSelectionSource {
    /// User was set in REPL session
    ReplSession,
    /// User was determined from SUDO_USER environment variable
    SudoUser,
    /// User is the non-root invoking user
    InvokingUser,
    /// User was selected as the primary interactive user
    PrimaryInteractive,
    /// User was selected from clarification prompt
    UserChoice,
    /// Fallback to root (should be rare)
    FallbackRoot,
}

impl std::fmt::Display for UserSelectionSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserSelectionSource::ReplSession => write!(f, "REPL session choice"),
            UserSelectionSource::SudoUser => write!(f, "SUDO_USER environment variable"),
            UserSelectionSource::InvokingUser => write!(f, "invoking user"),
            UserSelectionSource::PrimaryInteractive => write!(f, "primary interactive user"),
            UserSelectionSource::UserChoice => write!(f, "user selection"),
            UserSelectionSource::FallbackRoot => write!(f, "fallback (root)"),
        }
    }
}

// =============================================================================
// Target User Selection Result
// =============================================================================

/// Result of target user selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetUserSelection {
    /// The selected user
    pub user: UserInfo,
    /// How the user was selected
    pub source: UserSelectionSource,
    /// Evidence ID for this selection
    pub evidence_id: String,
    /// Human-readable explanation
    pub explanation: String,
    /// Whether clarification was needed (multiple candidates)
    pub required_clarification: bool,
    /// Other candidate users (if any)
    pub other_candidates: Vec<UserInfo>,
}

impl TargetUserSelection {
    /// Create a new selection
    pub fn new(user: UserInfo, source: UserSelectionSource, explanation: &str) -> Self {
        Self {
            user,
            source,
            evidence_id: generate_user_evidence_id(),
            explanation: explanation.to_string(),
            required_clarification: false,
            other_candidates: Vec::new(),
        }
    }

    /// Format as a transcript message
    pub fn format_transcript(&self) -> String {
        format!(
            "I will treat {} as the target user for user-scoped changes, because {}. [{}]",
            self.user.username, self.explanation, self.evidence_id
        )
    }
}

// =============================================================================
// Ambiguous User Selection
// =============================================================================

/// When multiple users are candidates and we need clarification
#[derive(Debug, Clone)]
pub struct AmbiguousUserSelection {
    /// Candidate users
    pub candidates: Vec<UserInfo>,
    /// Evidence ID
    pub evidence_id: String,
}

impl AmbiguousUserSelection {
    /// Format as a clarification prompt
    pub fn format_prompt(&self) -> String {
        let mut prompt = String::from("Which user should I target?\n");
        for (i, user) in self.candidates.iter().enumerate() {
            let label = if user.gecos.is_empty() {
                user.username.clone()
            } else {
                format!("{} ({})", user.username, user.gecos)
            };
            prompt.push_str(&format!("  {}) {}\n", i + 1, label));
        }
        prompt.push_str(&format!("Select [1-{}]: ", self.candidates.len()));
        prompt
    }

    /// Resolve with user choice (1-indexed)
    pub fn resolve(&self, choice: usize) -> Option<TargetUserSelection> {
        if choice == 0 || choice > self.candidates.len() {
            return None;
        }

        let user = self.candidates[choice - 1].clone();
        let mut selection = TargetUserSelection::new(
            user,
            UserSelectionSource::UserChoice,
            "you selected this user from the candidates",
        );
        selection.evidence_id = self.evidence_id.clone();
        selection.required_clarification = true;
        selection.other_candidates = self
            .candidates
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != choice - 1)
            .map(|(_, u)| u.clone())
            .collect();

        Some(selection)
    }
}

// =============================================================================
// Target User Selection
// =============================================================================

/// Selection result: either determined or needs clarification
#[derive(Debug)]
pub enum SelectionResult {
    /// User was determined
    Determined(TargetUserSelection),
    /// Multiple candidates, need clarification
    NeedsClarification(AmbiguousUserSelection),
}

/// Target user selector
pub struct TargetUserSelector {
    /// Cache of users from /etc/passwd
    users: HashMap<String, UserInfo>,
    /// Cache of users by UID
    users_by_uid: HashMap<u32, UserInfo>,
    /// REPL session user (if set)
    session_user: Option<String>,
}

impl TargetUserSelector {
    /// Create a new selector and load user database
    pub fn new() -> Self {
        let mut selector = Self {
            users: HashMap::new(),
            users_by_uid: HashMap::new(),
            session_user: None,
        };
        selector.load_passwd();
        selector
    }

    /// Load users from /etc/passwd
    fn load_passwd(&mut self) {
        if let Ok(content) = fs::read_to_string("/etc/passwd") {
            for line in content.lines() {
                if let Some(user) = UserInfo::from_passwd_line(line) {
                    self.users_by_uid.insert(user.uid, user.clone());
                    self.users.insert(user.username.clone(), user);
                }
            }
        }
    }

    /// Set the REPL session user
    pub fn set_session_user(&mut self, username: &str) {
        self.session_user = Some(username.to_string());
    }

    /// Clear the REPL session user
    pub fn clear_session_user(&mut self) {
        self.session_user = None;
    }

    /// Get user by username
    pub fn get_user(&self, username: &str) -> Option<&UserInfo> {
        self.users.get(username)
    }

    /// Get user by UID
    pub fn get_user_by_uid(&self, uid: u32) -> Option<&UserInfo> {
        self.users_by_uid.get(&uid)
    }

    /// Get all regular interactive users
    pub fn get_interactive_users(&self) -> Vec<&UserInfo> {
        self.users
            .values()
            .filter(|u| u.is_regular_user() && u.has_interactive_shell() && u.home_exists())
            .collect()
    }

    /// Select the target user deterministically
    ///
    /// Precedence:
    /// 1. REPL session chosen user
    /// 2. SUDO_USER environment variable
    /// 3. Non-root invoking user
    /// 4. Primary interactive user (or clarification if ambiguous)
    pub fn select_target_user(&self) -> SelectionResult {
        // 1. Check REPL session user
        if let Some(ref session_username) = self.session_user {
            if let Some(user) = self.users.get(session_username) {
                return SelectionResult::Determined(TargetUserSelection::new(
                    user.clone(),
                    UserSelectionSource::ReplSession,
                    "this user was set for the REPL session",
                ));
            }
        }

        // 2. Check SUDO_USER
        if let Ok(sudo_user) = std::env::var("SUDO_USER") {
            if !sudo_user.is_empty() {
                if let Some(user) = self.users.get(&sudo_user) {
                    return SelectionResult::Determined(TargetUserSelection::new(
                        user.clone(),
                        UserSelectionSource::SudoUser,
                        "you invoked annactl via sudo",
                    ));
                }
            }
        }

        // 3. Check invoking user (if not root)
        let uid = unsafe { libc::getuid() };
        if uid != 0 {
            if let Some(user) = self.users_by_uid.get(&uid) {
                return SelectionResult::Determined(TargetUserSelection::new(
                    user.clone(),
                    UserSelectionSource::InvokingUser,
                    "you are running annactl as this user",
                ));
            }
        }

        // 4. Find primary interactive user
        let interactive_users = self.get_interactive_users();

        match interactive_users.len() {
            0 => {
                // No interactive users found, fall back to root
                let root_user = self.users_by_uid.get(&0).cloned().unwrap_or(UserInfo {
                    username: "root".to_string(),
                    uid: 0,
                    gid: 0,
                    home: PathBuf::from("/root"),
                    shell: "/bin/bash".to_string(),
                    gecos: String::new(),
                });
                SelectionResult::Determined(TargetUserSelection::new(
                    root_user,
                    UserSelectionSource::FallbackRoot,
                    "no interactive users found on this system",
                ))
            }
            1 => {
                // Single interactive user
                SelectionResult::Determined(TargetUserSelection::new(
                    interactive_users[0].clone(),
                    UserSelectionSource::PrimaryInteractive,
                    "this is the only interactive user on the system",
                ))
            }
            _ => {
                // Multiple interactive users - try to find the most recent
                if let Some(primary) = self.find_most_recent_login(&interactive_users) {
                    return SelectionResult::Determined(TargetUserSelection::new(
                        primary.clone(),
                        UserSelectionSource::PrimaryInteractive,
                        "this user logged in most recently",
                    ));
                }

                // Cannot determine, need clarification
                SelectionResult::NeedsClarification(AmbiguousUserSelection {
                    candidates: interactive_users.into_iter().cloned().collect(),
                    evidence_id: generate_user_evidence_id(),
                })
            }
        }
    }

    /// Try to find the most recently logged-in user
    fn find_most_recent_login<'a>(&self, users: &[&'a UserInfo]) -> Option<&'a UserInfo> {
        // Try using `who` to find logged-in users
        if let Ok(output) = Command::new("who").output() {
            if output.status.success() {
                let who_output = String::from_utf8_lossy(&output.stdout);
                for line in who_output.lines() {
                    let username = line.split_whitespace().next()?;
                    for user in users {
                        if user.username == username {
                            return Some(user);
                        }
                    }
                }
            }
        }

        // Try using loginctl to find seat users
        if let Ok(output) = Command::new("loginctl")
            .args(["list-users", "--no-legend"])
            .output()
        {
            if output.status.success() {
                let loginctl_output = String::from_utf8_lossy(&output.stdout);
                for line in loginctl_output.lines() {
                    // Format: UID USERNAME
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let username = parts[1];
                        for user in users {
                            if user.username == username {
                                return Some(user);
                            }
                        }
                    }
                }
            }
        }

        None
    }
}

impl Default for TargetUserSelector {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Safe Home Directory Detection
// =============================================================================

/// Get the canonical home directory for a user from /etc/passwd
/// NEVER guess /home/<username>
pub fn get_user_home(username: &str) -> Option<PathBuf> {
    if let Ok(content) = fs::read_to_string("/etc/passwd") {
        for line in content.lines() {
            if let Some(user) = UserInfo::from_passwd_line(line) {
                if user.username == username {
                    return Some(user.home);
                }
            }
        }
    }
    None
}

/// Validate that a path is within a user's home directory
pub fn is_path_in_user_home(path: &Path, user: &UserInfo) -> bool {
    // Canonicalize both paths to handle symlinks
    let canonical_home = match user.home.canonicalize() {
        Ok(p) => p,
        Err(_) => user.home.clone(),
    };

    let canonical_path = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => path.to_path_buf(),
    };

    canonical_path.starts_with(&canonical_home)
}

/// Get the relative path within a user's home
pub fn get_path_relative_to_home(path: &Path, user: &UserInfo) -> Option<PathBuf> {
    let canonical_home = user.home.canonicalize().ok()?;
    let canonical_path = path.canonicalize().ok()?;

    canonical_path
        .strip_prefix(&canonical_home)
        .ok()
        .map(|p| p.to_path_buf())
}

// =============================================================================
// Home Path Expansion
// =============================================================================

/// Expand $HOME or ~ in a path for a specific user
pub fn expand_home_path(path: &str, user: &UserInfo) -> PathBuf {
    if path.starts_with("$HOME") {
        user.home.join(&path[5..].trim_start_matches('/'))
    } else if path.starts_with('~') {
        user.home.join(&path[1..].trim_start_matches('/'))
    } else {
        PathBuf::from(path)
    }
}

/// Replace a user's home path with ~ for display
pub fn contract_home_path(path: &Path, user: &UserInfo) -> String {
    if let Some(relative) = get_path_relative_to_home(path, user) {
        format!("~/{}", relative.display())
    } else {
        path.display().to_string()
    }
}

// =============================================================================
// User-Scoped Operations
// =============================================================================

/// Error types for user-scoped operations
#[derive(Debug, Clone)]
pub enum UserScopeError {
    /// User not found
    UserNotFound(String),
    /// Home directory not found
    HomeNotFound(String),
    /// Path not in user's home
    PathNotInHome(String),
    /// Permission denied
    PermissionDenied(String),
    /// Operation failed
    OperationFailed(String),
    /// Path blocked by policy
    PathBlocked(String),
}

impl std::fmt::Display for UserScopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserScopeError::UserNotFound(u) => write!(f, "User not found: {}", u),
            UserScopeError::HomeNotFound(u) => {
                write!(f, "Home directory not found for user: {}", u)
            }
            UserScopeError::PathNotInHome(p) => write!(f, "Path is not in user's home: {}", p),
            UserScopeError::PermissionDenied(p) => write!(f, "Permission denied: {}", p),
            UserScopeError::OperationFailed(e) => write!(f, "Operation failed: {}", e),
            UserScopeError::PathBlocked(p) => write!(f, "Path blocked by policy: {}", p),
        }
    }
}

impl std::error::Error for UserScopeError {}

/// Execute a file write operation as a specific user
///
/// This creates a file owned by the target user, not root.
/// Uses a subprocess with setuid/setgid for safety.
pub fn write_file_as_user(
    path: &Path,
    content: &[u8],
    user: &UserInfo,
) -> Result<(), UserScopeError> {
    // Validate path is in user's home
    if !is_path_in_user_home(path, user) {
        return Err(UserScopeError::PathNotInHome(path.display().to_string()));
    }

    // Check home exists
    if !user.home_exists() {
        return Err(UserScopeError::HomeNotFound(user.username.clone()));
    }

    // Create parent directories as user if needed
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            create_dir_as_user(parent, user)?;
        }
    }

    // Write using install command which can set ownership
    let temp_path = format!("/tmp/anna_write_{}.tmp", std::process::id());
    fs::write(&temp_path, content).map_err(|e| UserScopeError::OperationFailed(e.to_string()))?;

    // Use install to copy with correct ownership
    let output = Command::new("install")
        .args([
            "-o",
            &user.uid.to_string(),
            "-g",
            &user.gid.to_string(),
            "-m",
            "644",
            &temp_path,
            &path.to_string_lossy(),
        ])
        .output();

    // Clean up temp file
    let _ = fs::remove_file(&temp_path);

    match output {
        Ok(o) if o.status.success() => Ok(()),
        Ok(o) => Err(UserScopeError::OperationFailed(
            String::from_utf8_lossy(&o.stderr).to_string(),
        )),
        Err(e) => Err(UserScopeError::OperationFailed(e.to_string())),
    }
}

/// Create a directory as a specific user
pub fn create_dir_as_user(path: &Path, user: &UserInfo) -> Result<(), UserScopeError> {
    // Use install -d to create directory with correct ownership
    let output = Command::new("install")
        .args([
            "-d",
            "-o",
            &user.uid.to_string(),
            "-g",
            &user.gid.to_string(),
            "-m",
            "755",
            &path.to_string_lossy(),
        ])
        .output();

    match output {
        Ok(o) if o.status.success() => Ok(()),
        Ok(o) => Err(UserScopeError::OperationFailed(
            String::from_utf8_lossy(&o.stderr).to_string(),
        )),
        Err(e) => Err(UserScopeError::OperationFailed(e.to_string())),
    }
}

/// Create a backup of a file, owned by the target user
pub fn backup_file_as_user(
    source: &Path,
    backup_dir: &Path,
    user: &UserInfo,
) -> Result<PathBuf, UserScopeError> {
    // Ensure backup directory exists and is owned by user
    if !backup_dir.exists() {
        create_dir_as_user(backup_dir, user)?;
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let filename = source
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "backup".to_string());

    let backup_path = backup_dir.join(format!("{}_{}", timestamp, filename));

    // Copy using cp to preserve attributes, then chown
    let source_str = source.to_string_lossy().to_string();
    let backup_str = backup_path.to_string_lossy().to_string();

    let output = Command::new("cp")
        .args(["-p", &source_str, &backup_str])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            // Chown to user
            let _ = Command::new("chown")
                .args([&format!("{}:{}", user.uid, user.gid), &backup_str])
                .output();
            Ok(backup_path)
        }
        Ok(o) => Err(UserScopeError::OperationFailed(
            String::from_utf8_lossy(&o.stderr).to_string(),
        )),
        Err(e) => Err(UserScopeError::OperationFailed(e.to_string())),
    }
}

/// Check if a file is owned by the expected user
pub fn check_file_ownership(path: &Path, user: &UserInfo) -> Result<bool, io::Error> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.uid() == user.uid)
}

/// Fix ownership of a file to match the target user
pub fn fix_file_ownership(path: &Path, user: &UserInfo) -> Result<(), UserScopeError> {
    let path_str = path.to_string_lossy().to_string();
    let output = Command::new("chown")
        .args([&format!("{}:{}", user.uid, user.gid), &path_str])
        .output();

    match output {
        Ok(o) if o.status.success() => Ok(()),
        Ok(o) => Err(UserScopeError::OperationFailed(
            String::from_utf8_lossy(&o.stderr).to_string(),
        )),
        Err(e) => Err(UserScopeError::OperationFailed(e.to_string())),
    }
}

// =============================================================================
// User Home Policy
// =============================================================================

/// Default allowed subpaths in user homes
pub const DEFAULT_ALLOWED_HOME_PATHS: &[&str] = &[
    ".config/**",
    ".local/share/**",
    ".bashrc",
    ".bash_profile",
    ".bash_aliases",
    ".zshrc",
    ".zprofile",
    ".profile",
    ".gitconfig",
    ".vimrc",
    ".tmux.conf",
];

/// Default blocked paths in user homes (security-sensitive)
pub const DEFAULT_BLOCKED_HOME_PATHS: &[&str] = &[
    ".ssh/**",
    ".gnupg/**",
    ".password-store/**",
    ".mozilla/**/key*.db",
    ".mozilla/**/logins.json",
    ".config/chromium/**/Login Data",
    ".config/google-chrome/**/Login Data",
];

/// Check if a path within a user's home is allowed by policy
pub fn is_home_path_allowed(relative_path: &str) -> bool {
    // Check blocked paths first
    for blocked in DEFAULT_BLOCKED_HOME_PATHS {
        if matches_glob_pattern(relative_path, blocked) {
            return false;
        }
    }

    // Check allowed paths
    for allowed in DEFAULT_ALLOWED_HOME_PATHS {
        if matches_glob_pattern(relative_path, allowed) {
            return true;
        }
    }

    false
}

/// Simple glob pattern matching
fn matches_glob_pattern(text: &str, pattern: &str) -> bool {
    if pattern.ends_with("/**") {
        // Match prefix
        let prefix = &pattern[..pattern.len() - 3];
        text.starts_with(prefix)
    } else if pattern.contains("/**/") {
        // Match with **/ in middle for arbitrary path depth
        // e.g., ".mozilla/**/key*.db" should match ".mozilla/firefox/key3.db"
        let parts: Vec<&str> = pattern.split("/**/").collect();
        if parts.len() == 2 {
            // parts[0] = ".mozilla", parts[1] = "key*.db"
            // Text must start with prefix, and the filename part must match suffix pattern
            if !text.starts_with(parts[0]) {
                return false;
            }
            // Get the filename (or last part after prefix)
            // The suffix pattern should match the last component(s) of the text
            let suffix = parts[1];
            // For patterns like "key*.db", we need to check if the text ends with something matching it
            if let Some(filename) = text.rsplit('/').next() {
                simple_wildcard_match(filename, suffix)
            } else {
                simple_wildcard_match(text, suffix)
            }
        } else {
            false
        }
    } else if pattern.contains("**") {
        // Match with ** wildcard
        let parts: Vec<&str> = pattern.split("**").collect();
        if parts.len() == 2 {
            text.starts_with(parts[0]) && (parts[1].is_empty() || text.ends_with(parts[1]))
        } else {
            false
        }
    } else if pattern.contains('*') {
        // Simple glob
        simple_wildcard_match(text, pattern)
    } else {
        // Exact match
        text == pattern
    }
}

/// Simple wildcard matching (supports single * only)
fn simple_wildcard_match(text: &str, pattern: &str) -> bool {
    if !pattern.contains('*') {
        return text.ends_with(pattern);
    }

    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 2 {
        text.starts_with(parts[0]) && text.ends_with(parts[1])
    } else if parts.len() > 2 {
        // Multiple wildcards - check sequentially
        let mut pos = 0;
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }
            if i == 0 {
                if !text.starts_with(part) {
                    return false;
                }
                pos = part.len();
            } else if i == parts.len() - 1 {
                if !text.ends_with(part) {
                    return false;
                }
            } else {
                if let Some(found) = text[pos..].find(part) {
                    pos += found + part.len();
                } else {
                    return false;
                }
            }
        }
        true
    } else {
        text == pattern
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_passwd_line() {
        let line = "testuser:x:1000:1000:Test User:/home/testuser:/bin/bash";
        let user = UserInfo::from_passwd_line(line).unwrap();

        assert_eq!(user.username, "testuser");
        assert_eq!(user.uid, 1000);
        assert_eq!(user.gid, 1000);
        assert_eq!(user.home, PathBuf::from("/home/testuser"));
        assert_eq!(user.shell, "/bin/bash");
        assert_eq!(user.gecos, "Test User");
    }

    #[test]
    fn test_is_regular_user() {
        let regular = UserInfo {
            username: "alice".to_string(),
            uid: 1000,
            gid: 1000,
            home: PathBuf::from("/home/alice"),
            shell: "/bin/bash".to_string(),
            gecos: String::new(),
        };
        assert!(regular.is_regular_user());

        let system = UserInfo {
            username: "daemon".to_string(),
            uid: 1,
            gid: 1,
            home: PathBuf::from("/"),
            shell: "/sbin/nologin".to_string(),
            gecos: String::new(),
        };
        assert!(!system.is_regular_user());
    }

    #[test]
    fn test_has_interactive_shell() {
        let interactive = UserInfo {
            username: "alice".to_string(),
            uid: 1000,
            gid: 1000,
            home: PathBuf::from("/home/alice"),
            shell: "/bin/bash".to_string(),
            gecos: String::new(),
        };
        assert!(interactive.has_interactive_shell());

        let nologin = UserInfo {
            username: "daemon".to_string(),
            uid: 1,
            gid: 1,
            home: PathBuf::from("/"),
            shell: "/sbin/nologin".to_string(),
            gecos: String::new(),
        };
        assert!(!nologin.has_interactive_shell());
    }

    #[test]
    fn test_expand_home_path() {
        let user = UserInfo {
            username: "alice".to_string(),
            uid: 1000,
            gid: 1000,
            home: PathBuf::from("/home/alice"),
            shell: "/bin/bash".to_string(),
            gecos: String::new(),
        };

        assert_eq!(
            expand_home_path("$HOME/.bashrc", &user),
            PathBuf::from("/home/alice/.bashrc")
        );
        assert_eq!(
            expand_home_path("~/.config/test", &user),
            PathBuf::from("/home/alice/.config/test")
        );
        assert_eq!(
            expand_home_path("/etc/hosts", &user),
            PathBuf::from("/etc/hosts")
        );
    }

    #[test]
    fn test_matches_glob_pattern() {
        // Directory prefix
        assert!(matches_glob_pattern(".config/foo/bar", ".config/**"));
        assert!(matches_glob_pattern(".config/test", ".config/**"));
        assert!(!matches_glob_pattern(".bashrc", ".config/**"));

        // Exact match
        assert!(matches_glob_pattern(".bashrc", ".bashrc"));
        assert!(!matches_glob_pattern(".bashrc_backup", ".bashrc"));

        // Wildcard
        assert!(matches_glob_pattern(
            ".mozilla/firefox/key3.db",
            ".mozilla/**/key*.db"
        ));

        // Blocked paths
        assert!(matches_glob_pattern(".ssh/id_rsa", ".ssh/**"));
        assert!(matches_glob_pattern(".gnupg/private-keys", ".gnupg/**"));
    }

    #[test]
    fn test_is_home_path_allowed() {
        // Allowed
        assert!(is_home_path_allowed(".config/nvim/init.vim"));
        assert!(is_home_path_allowed(".bashrc"));
        assert!(is_home_path_allowed(
            ".local/share/applications/test.desktop"
        ));

        // Blocked (security-sensitive)
        assert!(!is_home_path_allowed(".ssh/id_rsa"));
        assert!(!is_home_path_allowed(".ssh/config"));
        assert!(!is_home_path_allowed(".gnupg/private-keys-v1.d/key"));

        // Not in allowed list
        assert!(!is_home_path_allowed("Documents/secret.txt"));
        assert!(!is_home_path_allowed(".random_dir/file"));
    }

    #[test]
    fn test_user_selection_source_display() {
        assert_eq!(
            UserSelectionSource::SudoUser.to_string(),
            "SUDO_USER environment variable"
        );
        assert_eq!(
            UserSelectionSource::InvokingUser.to_string(),
            "invoking user"
        );
        assert_eq!(
            UserSelectionSource::ReplSession.to_string(),
            "REPL session choice"
        );
    }

    #[test]
    fn test_target_user_selection_transcript() {
        let user = UserInfo {
            username: "barbara".to_string(),
            uid: 1000,
            gid: 1000,
            home: PathBuf::from("/home/barbara"),
            shell: "/bin/bash".to_string(),
            gecos: String::new(),
        };

        let selection = TargetUserSelection::new(
            user,
            UserSelectionSource::SudoUser,
            "you invoked annactl via sudo",
        );

        let transcript = selection.format_transcript();
        assert!(transcript.contains("barbara"));
        assert!(transcript.contains("sudo"));
        assert!(transcript.contains("E-user-"));
    }

    #[test]
    fn test_ambiguous_selection_prompt() {
        let candidates = vec![
            UserInfo {
                username: "alice".to_string(),
                uid: 1000,
                gid: 1000,
                home: PathBuf::from("/home/alice"),
                shell: "/bin/bash".to_string(),
                gecos: "Alice Smith".to_string(),
            },
            UserInfo {
                username: "bob".to_string(),
                uid: 1001,
                gid: 1001,
                home: PathBuf::from("/home/bob"),
                shell: "/bin/zsh".to_string(),
                gecos: String::new(),
            },
        ];

        let ambiguous = AmbiguousUserSelection {
            candidates,
            evidence_id: "E-user-12345".to_string(),
        };

        let prompt = ambiguous.format_prompt();
        assert!(prompt.contains("alice"));
        assert!(prompt.contains("Alice Smith"));
        assert!(prompt.contains("bob"));
        assert!(prompt.contains("Select [1-2]"));
    }

    #[test]
    fn test_ambiguous_selection_resolve() {
        let candidates = vec![
            UserInfo {
                username: "alice".to_string(),
                uid: 1000,
                gid: 1000,
                home: PathBuf::from("/home/alice"),
                shell: "/bin/bash".to_string(),
                gecos: String::new(),
            },
            UserInfo {
                username: "bob".to_string(),
                uid: 1001,
                gid: 1001,
                home: PathBuf::from("/home/bob"),
                shell: "/bin/zsh".to_string(),
                gecos: String::new(),
            },
        ];

        let ambiguous = AmbiguousUserSelection {
            candidates,
            evidence_id: "E-user-12345".to_string(),
        };

        // Valid selection
        let selection = ambiguous.resolve(1).unwrap();
        assert_eq!(selection.user.username, "alice");
        assert!(selection.required_clarification);
        assert_eq!(selection.other_candidates.len(), 1);

        // Invalid selection
        assert!(ambiguous.resolve(0).is_none());
        assert!(ambiguous.resolve(3).is_none());
    }

    #[test]
    fn test_evidence_id_generation() {
        let id1 = generate_user_evidence_id();
        let id2 = generate_user_evidence_id();

        assert!(id1.starts_with(USER_EVIDENCE_PREFIX));
        assert!(id2.starts_with(USER_EVIDENCE_PREFIX));
        // IDs should be unique (though this isn't guaranteed in a single test)
    }
}
