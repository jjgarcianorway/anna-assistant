//! Clarification questions with verification probes (v0.0.32).
//!
//! When information is missing to answer a query, we ask concrete clarification
//! questions with associated probes to verify the answer.
//!
//! v0.0.39: Uses InventoryCache for installed tool detection.
//! v0.0.42: Added menu-based prompts with numeric keys (0=cancel, 9=other).
//!          Escape options always present. Verification before fact storage.

use crate::facts::{FactKey, FactsStore};
use crate::inventory::{load_or_create_inventory, InventoryCache};
use serde::{Deserialize, Serialize};

// === v0.0.42: Menu-based clarification system ===

/// Reserved numeric keys for escape options (v0.0.42)
pub const KEY_CANCEL: u8 = 0;
pub const KEY_OTHER: u8 = 9;

/// Menu-based clarification prompt (v0.0.42)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyPrompt {
    /// Unique identifier
    pub id: String,
    /// Title/header
    pub title: String,
    /// The question
    pub question: String,
    /// Available options (sorted by key)
    pub options: Vec<MenuOption>,
    /// Default selection key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_key: Option<u8>,
    /// Reason for clarification
    #[serde(default)]
    pub reason: String,
}

/// A menu option with numeric key (v0.0.42)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MenuOption {
    /// Numeric key (1-8 for options, 0=cancel, 9=other)
    pub key: u8,
    /// Display label
    pub label: String,
    /// Fact key to set if selected
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fact_key: Option<String>,
    /// Fact value to set if selected
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fact_value: Option<String>,
    /// Verification command template
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_cmd: Option<String>,
}

impl MenuOption {
    /// Create option with numeric key
    pub fn new(key: u8, label: impl Into<String>) -> Self {
        Self {
            key,
            label: label.into(),
            fact_key: None,
            fact_value: None,
            verify_cmd: None,
        }
    }

    /// Set fact to store
    pub fn with_fact(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fact_key = Some(key.into());
        self.fact_value = Some(value.into());
        self
    }

    /// Set verification command
    pub fn with_verify(mut self, cmd: impl Into<String>) -> Self {
        self.verify_cmd = Some(cmd.into());
        self
    }

    /// Create cancel option (key 0)
    pub fn cancel() -> Self {
        Self::new(KEY_CANCEL, "Cancel")
    }

    /// Create "other" option (key 9)
    pub fn other() -> Self {
        Self::new(KEY_OTHER, "Other (specify)")
    }
}

impl ClarifyPrompt {
    /// Create new prompt with escape options
    pub fn new(id: impl Into<String>, title: impl Into<String>, question: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            question: question.into(),
            options: vec![MenuOption::cancel(), MenuOption::other()],
            default_key: None,
            reason: String::new(),
        }
    }

    /// Add option (re-sorts and ensures escape options)
    pub fn add_option(mut self, opt: MenuOption) -> Self {
        self.options.retain(|o| o.key != KEY_CANCEL && o.key != KEY_OTHER);
        self.options.push(opt);
        self.options.sort_by_key(|o| o.key);
        self.options.push(MenuOption::cancel());
        self.options.push(MenuOption::other());
        self
    }

    /// Set multiple options
    pub fn with_options(mut self, opts: Vec<MenuOption>) -> Self {
        self.options = opts;
        self.ensure_escape_options();
        self
    }

    /// Set default key
    pub fn with_default(mut self, key: u8) -> Self {
        self.default_key = Some(key);
        self
    }

    /// Set reason
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = reason.into();
        self
    }

    /// Ensure escape options present
    fn ensure_escape_options(&mut self) {
        if !self.options.iter().any(|o| o.key == KEY_CANCEL) {
            self.options.push(MenuOption::cancel());
        }
        if !self.options.iter().any(|o| o.key == KEY_OTHER) {
            self.options.push(MenuOption::other());
        }
        self.options.sort_by_key(|o| match o.key {
            KEY_CANCEL => 100, // Put cancel at end
            KEY_OTHER => 101,  // Put other last
            k => k as u8,
        });
    }

    /// Get option by key
    pub fn get_option(&self, key: u8) -> Option<&MenuOption> {
        self.options.iter().find(|o| o.key == key)
    }

    /// Format as menu string
    pub fn format_menu(&self) -> String {
        let mut lines = vec![
            format!("╭─ {} ─╮", self.title),
            self.question.clone(),
            String::new(),
        ];
        for opt in &self.options {
            let marker = if self.default_key == Some(opt.key) { " ←" } else { "" };
            lines.push(format!("  [{}] {}{}", opt.key, opt.label, marker));
        }
        if !self.reason.is_empty() {
            lines.push(String::new());
            lines.push(format!("  ({})", self.reason));
        }
        lines.join("\n")
    }
}

/// Outcome of menu interaction (v0.0.42)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClarifyOutcome {
    /// User selected an option
    Answered { key: u8, label: String, prompt_id: String },
    /// User cancelled (key 0)
    Cancelled,
    /// User chose other with free text
    Other { text: String },
    /// Verification failed
    VerificationFailed { selected: String, reason: String, alternative: Option<String> },
}

impl ClarifyOutcome {
    /// Check if successful
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Answered { .. } | Self::Other { .. })
    }

    /// Get selected text
    pub fn selected_text(&self) -> Option<&str> {
        match self {
            Self::Answered { label, .. } => Some(label),
            Self::Other { text } => Some(text),
            _ => None,
        }
    }
}

/// Generate editor menu prompt from inventory (v0.0.42)
pub fn editor_menu_prompt(cache: &InventoryCache) -> ClarifyPrompt {
    let editors = [
        ("vim", "Vim"), ("nvim", "Neovim"), ("nano", "Nano"),
        ("emacs", "Emacs"), ("code", "VS Code"), ("micro", "Micro"),
    ];

    let mut opts = Vec::new();
    let mut key: u8 = 1;

    for (cmd, label) in &editors {
        if cache.is_installed(cmd).unwrap_or(false) && key < KEY_OTHER {
            opts.push(
                MenuOption::new(key, *label)
                    .with_fact("preferred_editor", *cmd)
                    .with_verify(format!("command -v {}", cmd))
            );
            key += 1;
        }
    }

    ClarifyPrompt::new("editor_select", "Editor Selection", "Which editor do you prefer?")
        .with_options(opts)
        .with_reason("I need to know your editor to configure it")
}

/// Find installed alternative when verification fails (v0.0.42)
pub fn find_installed_alternative(tool: &str, cache: &InventoryCache) -> Option<String> {
    let alts: &[(&str, &[&str])] = &[
        ("vim", &["nvim", "vi", "nano"]),
        ("nvim", &["vim", "vi", "nano"]),
        ("emacs", &["vim", "nano", "code"]),
        ("code", &["vim", "nano", "nvim"]),
        ("nano", &["vim", "micro", "vi"]),
    ];

    for (t, alternatives) in alts {
        if *t == tool {
            for alt in *alternatives {
                if cache.is_installed(alt).unwrap_or(false) {
                    return Some(alt.to_string());
                }
            }
        }
    }
    None
}

// === Legacy types (v0.0.32-v0.0.39) preserved for backward compat ===

/// What kind of clarification is needed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClarifyKind {
    /// Which text editor they prefer
    PreferredEditor,
    /// Which service they're asking about
    ServiceName,
    /// Which mount point or disk
    MountPoint,
    /// Which network interface
    NetworkInterface,
    /// Which process/application
    ProcessName,
    /// Custom clarification
    Custom(String),
}

impl std::fmt::Display for ClarifyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PreferredEditor => write!(f, "preferred_editor"),
            Self::ServiceName => write!(f, "service_name"),
            Self::MountPoint => write!(f, "mount_point"),
            Self::NetworkInterface => write!(f, "network_interface"),
            Self::ProcessName => write!(f, "process_name"),
            Self::Custom(s) => write!(f, "custom:{}", s),
        }
    }
}

/// A clarification question with verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyQuestion {
    /// What kind of clarification this is
    pub kind: ClarifyKind,
    /// The question to ask the user
    pub question: String,
    /// Probe command to verify the answer (if applicable)
    pub verify_probe: Option<String>,
    /// Hint about what values are valid (for UI)
    pub hint: Option<String>,
    /// Default value if known from facts
    pub default: Option<String>,
}

impl ClarifyQuestion {
    /// Create a new clarification question
    pub fn new(kind: ClarifyKind, question: impl Into<String>) -> Self {
        Self {
            kind,
            question: question.into(),
            verify_probe: None,
            hint: None,
            default: None,
        }
    }

    /// Add verification probe
    pub fn with_verify(mut self, probe: impl Into<String>) -> Self {
        self.verify_probe = Some(probe.into());
        self
    }

    /// Add hint
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Add default from facts
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }
}

/// An option in a clarification question (v0.0.34)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyOption {
    /// Key to select this option (e.g., "1", "vim")
    pub key: String,
    /// Display label
    pub label: String,
    /// Evidence strings for this option (e.g., "installed: true", "recently used: 12 times")
    pub evidence: Vec<String>,
}

impl ClarifyOption {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            evidence: vec![],
        }
    }

    pub fn with_evidence(mut self, ev: impl Into<String>) -> Self {
        self.evidence.push(ev.into());
        self
    }
}

/// User's answer to a clarification question (v0.0.34)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyAnswer {
    /// ID of the question being answered
    pub question_id: String,
    /// Key of the selected option
    pub selected_key: String,
}

/// Result of a clarification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClarifyResult {
    /// Answer was verified, can learn fact
    Verified {
        kind: ClarifyKind,
        value: String,
        probe_output: Option<String>,
    },
    /// Answer could not be verified
    Unverified {
        kind: ClarifyKind,
        value: String,
        reason: String,
    },
    /// User declined to answer
    Declined,
}

/// Generate clarification question for a given kind
pub fn generate_question(kind: ClarifyKind, facts: &FactsStore) -> ClarifyQuestion {
    match &kind {
        ClarifyKind::PreferredEditor => {
            let default = facts.get_verified(&FactKey::PreferredEditor)
                .map(|s| s.to_string());
            ClarifyQuestion::new(kind, "What text editor do you prefer? (e.g., vim, nano, code)")
                .with_verify("which {}")
                .with_hint("vim, nano, emacs, code, nvim")
                .with_default(default.unwrap_or_default())
        }
        ClarifyKind::ServiceName => {
            ClarifyQuestion::new(kind, "Which service are you asking about?")
                .with_verify("systemctl is-active {}")
                .with_hint("nginx, docker, sshd")
        }
        ClarifyKind::MountPoint => {
            ClarifyQuestion::new(kind, "Which mount point or disk are you asking about?")
                .with_verify("df {}")
                .with_hint("/, /home, /var")
        }
        ClarifyKind::NetworkInterface => {
            let default = facts.get_verified(&FactKey::NetworkPrimaryInterface)
                .map(|s| s.to_string());
            ClarifyQuestion::new(kind, "Which network interface?")
                .with_verify("ip addr show {}")
                .with_hint("eth0, wlan0, enp3s0")
                .with_default(default.unwrap_or_default())
        }
        ClarifyKind::ProcessName => {
            ClarifyQuestion::new(kind, "Which process or application?")
                .with_verify("pgrep -x {}")
                .with_hint("firefox, chrome, code")
        }
        ClarifyKind::Custom(desc) => {
            ClarifyQuestion::new(kind.clone(), format!("Please specify: {}", desc))
        }
    }
}

/// Convert ClarifyKind to FactKey for storage (if applicable)
pub fn kind_to_fact_key(kind: &ClarifyKind, value: &str) -> Option<FactKey> {
    match kind {
        ClarifyKind::PreferredEditor => Some(FactKey::PreferredEditor),
        ClarifyKind::ServiceName => Some(FactKey::UnitExists(value.to_string())),
        ClarifyKind::MountPoint => Some(FactKey::MountExists(value.to_string())),
        ClarifyKind::NetworkInterface => Some(FactKey::NetworkPrimaryInterface),
        ClarifyKind::ProcessName => Some(FactKey::BinaryAvailable(value.to_string())),
        ClarifyKind::Custom(_) => None,
    }
}

/// Build verification command from template
pub fn build_verify_command(template: &str, value: &str) -> String {
    template.replace("{}", value)
}

/// Check if clarification is needed based on query and facts
pub fn needs_clarification(query: &str, facts: &FactsStore) -> Option<ClarifyKind> {
    let q = query.to_lowercase();

    // Check for editor-related queries without specified editor
    if (q.contains("edit") || q.contains("open") || q.contains("editor"))
        && !q.contains("vim")
        && !q.contains("nano")
        && !q.contains("emacs")
        && !q.contains("code")
        && !facts.has_verified(&FactKey::PreferredEditor)
    {
        return Some(ClarifyKind::PreferredEditor);
    }

    // Check for service queries without specific service
    if (q.contains("service") || q.contains("systemctl"))
        && !q.contains("--failed")
        && !extract_service_name(&q).is_some()
    {
        return Some(ClarifyKind::ServiceName);
    }

    // Check for mount/disk queries without specific path
    if (q.contains("mount") || q.contains("partition"))
        && !q.contains("/")
    {
        return Some(ClarifyKind::MountPoint);
    }

    None
}

/// Try to extract service name from query
fn extract_service_name(query: &str) -> Option<String> {
    // Look for common service name patterns
    let patterns = ["nginx", "docker", "sshd", "apache", "mysql", "postgresql", "redis"];
    for p in patterns {
        if query.contains(p) {
            return Some(p.to_string());
        }
    }
    // Look for ".service" pattern
    if let Some(idx) = query.find(".service") {
        let before = &query[..idx];
        if let Some(start) = before.rfind(' ') {
            return Some(before[start+1..].to_string());
        }
    }
    None
}

/// Known editors to check for (v0.0.35)
pub const KNOWN_EDITORS: &[&str] = &["vim", "vi", "nvim", "nano", "emacs", "code", "micro", "hx"];

/// Special option keys for clarification (v0.0.36)
pub const CLARIFY_CANCEL_KEY: &str = "__cancel__";
pub const CLARIFY_OTHER_KEY: &str = "__other__";

/// Generate editor options based on installed editors (v0.0.35)
/// v0.0.39: Uses InventoryCache instead of running commands each time.
/// Returns ClarifyOptions with evidence for installed status.
/// v0.0.36: Always includes Cancel option; only shows installed editors.
pub fn generate_editor_options_sync() -> Vec<ClarifyOption> {
    generate_editor_options_with_cache(&load_or_create_inventory())
}

/// Generate editor options using provided cache (v0.0.39)
/// Allows tests to inject mock cache.
pub fn generate_editor_options_with_cache(cache: &InventoryCache) -> Vec<ClarifyOption> {
    let mut options = Vec::new();

    // Get installed editors from cache
    let installed_editors = cache.installed_editors();

    for editor in KNOWN_EDITORS {
        // Check cache first
        if installed_editors.contains(editor) {
            options.push(
                ClarifyOption::new(*editor, *editor).with_evidence("installed: true (cached)"),
            );
        } else if let Some(true) = cache.is_installed(editor) {
            // Fallback: check cache directly if not in installed_editors list
            options.push(
                ClarifyOption::new(*editor, *editor).with_evidence("installed: true (cached)"),
            );
        }
    }

    // If no editors found in cache, fall back to live check for at least one option
    if options.is_empty() {
        for editor in KNOWN_EDITORS {
            if verify_editor_installed(editor) {
                options.push(
                    ClarifyOption::new(*editor, *editor).with_evidence("installed: true"),
                );
                break; // At least one editor
            }
        }
    }

    // v0.0.36: Always add Other option for custom input
    options.push(
        ClarifyOption::new(CLARIFY_OTHER_KEY, "Other (type name)").with_evidence("custom input"),
    );

    // v0.0.36: Always add Cancel option at the end
    options.push(
        ClarifyOption::new(CLARIFY_CANCEL_KEY, "Cancel").with_evidence("skip question"),
    );

    options
}

/// Check if user selected cancel (v0.0.36)
pub fn is_cancel_selection(key: &str) -> bool {
    key == CLARIFY_CANCEL_KEY
}

/// Check if user selected other/custom (v0.0.36)
pub fn is_other_selection(key: &str) -> bool {
    key == CLARIFY_OTHER_KEY
}

/// Verify that an editor is installed (v0.0.35)
pub fn verify_editor_installed(editor: &str) -> bool {
    // Try `which <editor>`
    std::process::Command::new("which")
        .arg(editor)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Generate editor clarification with detected options (v0.0.35)
pub fn generate_editor_clarification(facts: &FactsStore) -> (ClarifyQuestion, Vec<ClarifyOption>) {
    let default = facts.get_verified(&FactKey::PreferredEditor)
        .map(|s| s.to_string());

    let question = ClarifyQuestion::new(
        ClarifyKind::PreferredEditor,
        "Which text editor do you prefer?"
    )
    .with_verify("which {}")
    .with_hint("Select from installed editors or specify another")
    .with_default(default.unwrap_or_default());

    let options = generate_editor_options_sync();

    (question, options)
}

// Tests moved to tests/clarify_tests.rs
