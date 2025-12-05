//! Clarification questions with verification probes (v0.0.32).
//!
//! When information is missing to answer a query, we ask concrete clarification
//! questions with associated probes to verify the answer.
//!
//! v0.0.39: Uses InventoryCache for installed tool detection.
//! v0.0.42: Added menu-based prompts with numeric keys (0=cancel, 9=other).
//! v0.0.44: Moved v2 types to clarify_v2.rs module.

use crate::facts::{FactKey, FactsStore};
use crate::inventory::{load_or_create_inventory, InventoryCache};
use serde::{Deserialize, Serialize};

// Re-export v0.0.44 types from clarify_v2
pub use crate::clarify_v2::{
    ClarifyOption as ClarifyOptionV2, ClarifyRequest, ClarifyResponse, ClarifyResult,
    VerifyFailureTracker, editor_request, find_installed_alternatives, invalidate_on_uninstall,
    process_response, should_skip, store_fact, KEY_CANCEL, KEY_OTHER,
};

/// Menu-based clarification prompt (v0.0.42)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyPrompt {
    pub id: String,
    pub title: String,
    pub question: String,
    pub options: Vec<MenuOption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_key: Option<u8>,
    #[serde(default)]
    pub reason: String,
}

/// A menu option with numeric key (v0.0.42)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MenuOption {
    pub key: u8,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fact_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fact_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_cmd: Option<String>,
}

impl MenuOption {
    pub fn new(key: u8, label: impl Into<String>) -> Self {
        Self { key, label: label.into(), fact_key: None, fact_value: None, verify_cmd: None }
    }

    pub fn with_fact(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fact_key = Some(key.into());
        self.fact_value = Some(value.into());
        self
    }

    pub fn with_verify(mut self, cmd: impl Into<String>) -> Self {
        self.verify_cmd = Some(cmd.into());
        self
    }

    pub fn cancel() -> Self { Self::new(KEY_CANCEL, "Cancel") }
    pub fn other() -> Self { Self::new(KEY_OTHER, "Other (specify)") }
}

impl ClarifyPrompt {
    pub fn new(id: impl Into<String>, title: impl Into<String>, question: impl Into<String>) -> Self {
        Self {
            id: id.into(), title: title.into(), question: question.into(),
            options: vec![MenuOption::cancel(), MenuOption::other()],
            default_key: None, reason: String::new(),
        }
    }

    pub fn add_option(mut self, opt: MenuOption) -> Self {
        self.options.retain(|o| o.key != KEY_CANCEL && o.key != KEY_OTHER);
        self.options.push(opt);
        self.options.sort_by_key(|o| o.key);
        self.options.push(MenuOption::cancel());
        self.options.push(MenuOption::other());
        self
    }

    pub fn with_options(mut self, opts: Vec<MenuOption>) -> Self {
        self.options = opts;
        self.ensure_escape_options();
        self
    }

    pub fn with_default(mut self, key: u8) -> Self { self.default_key = Some(key); self }
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self { self.reason = reason.into(); self }

    fn ensure_escape_options(&mut self) {
        if !self.options.iter().any(|o| o.key == KEY_CANCEL) { self.options.push(MenuOption::cancel()); }
        if !self.options.iter().any(|o| o.key == KEY_OTHER) { self.options.push(MenuOption::other()); }
        self.options.sort_by_key(|o| match o.key {
            KEY_CANCEL => 100, KEY_OTHER => 101, k => k,
        });
    }

    pub fn get_option(&self, key: u8) -> Option<&MenuOption> {
        self.options.iter().find(|o| o.key == key)
    }

    pub fn format_menu(&self) -> String {
        let mut lines = vec![format!("╭─ {} ─╮", self.title), self.question.clone(), String::new()];
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
    Answered { key: u8, label: String, prompt_id: String },
    Cancelled,
    Other { text: String },
    VerificationFailed { selected: String, reason: String, alternative: Option<String> },
}

impl ClarifyOutcome {
    pub fn is_success(&self) -> bool { matches!(self, Self::Answered { .. } | Self::Other { .. }) }
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
            opts.push(MenuOption::new(key, *label).with_fact("preferred_editor", *cmd)
                .with_verify(format!("command -v {}", cmd)));
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
        ("vim", &["nvim", "vi", "nano"]), ("nvim", &["vim", "vi", "nano"]),
        ("emacs", &["vim", "nano", "code"]), ("code", &["vim", "nano", "nvim"]),
        ("nano", &["vim", "micro", "vi"]),
    ];

    for (t, alternatives) in alts {
        if *t == tool {
            for alt in *alternatives {
                if cache.is_installed(alt).unwrap_or(false) { return Some(alt.to_string()); }
            }
        }
    }
    None
}

// === Legacy types (v0.0.32-v0.0.39) ===

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClarifyKind {
    PreferredEditor, ServiceName, MountPoint, NetworkInterface, ProcessName, Custom(String),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyQuestion {
    pub kind: ClarifyKind,
    pub question: String,
    pub verify_probe: Option<String>,
    pub hint: Option<String>,
    pub default: Option<String>,
}

impl ClarifyQuestion {
    pub fn new(kind: ClarifyKind, question: impl Into<String>) -> Self {
        Self { kind, question: question.into(), verify_probe: None, hint: None, default: None }
    }
    pub fn with_verify(mut self, probe: impl Into<String>) -> Self { self.verify_probe = Some(probe.into()); self }
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self { self.hint = Some(hint.into()); self }
    pub fn with_default(mut self, default: impl Into<String>) -> Self { self.default = Some(default.into()); self }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyOption {
    pub key: String,
    pub label: String,
    pub evidence: Vec<String>,
}

impl ClarifyOption {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self { key: key.into(), label: label.into(), evidence: vec![] }
    }
    pub fn with_evidence(mut self, ev: impl Into<String>) -> Self { self.evidence.push(ev.into()); self }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyAnswer { pub question_id: String, pub selected_key: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClarifyResultLegacy {
    Verified { kind: ClarifyKind, value: String, probe_output: Option<String> },
    Unverified { kind: ClarifyKind, value: String, reason: String },
    Declined,
}

pub fn generate_question(kind: ClarifyKind, facts: &FactsStore) -> ClarifyQuestion {
    match &kind {
        ClarifyKind::PreferredEditor => {
            let default = facts.get_verified(&FactKey::PreferredEditor).map(|s| s.to_string());
            ClarifyQuestion::new(kind, "What text editor do you prefer?")
                .with_verify("which {}")
                .with_hint("vim, nano, emacs, code, nvim")
                .with_default(default.unwrap_or_default())
        }
        ClarifyKind::ServiceName => ClarifyQuestion::new(kind, "Which service?")
            .with_verify("systemctl is-active {}").with_hint("nginx, docker, sshd"),
        ClarifyKind::MountPoint => ClarifyQuestion::new(kind, "Which mount point?")
            .with_verify("df {}").with_hint("/, /home, /var"),
        ClarifyKind::NetworkInterface => {
            let default = facts.get_verified(&FactKey::NetworkPrimaryInterface).map(|s| s.to_string());
            ClarifyQuestion::new(kind, "Which network interface?")
                .with_verify("ip addr show {}").with_hint("eth0, wlan0")
                .with_default(default.unwrap_or_default())
        }
        ClarifyKind::ProcessName => ClarifyQuestion::new(kind, "Which process?")
            .with_verify("pgrep -x {}").with_hint("firefox, chrome"),
        ClarifyKind::Custom(desc) => ClarifyQuestion::new(kind.clone(), format!("Please specify: {}", desc)),
    }
}

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

pub fn build_verify_command(template: &str, value: &str) -> String { template.replace("{}", value) }

pub fn needs_clarification(query: &str, facts: &FactsStore) -> Option<ClarifyKind> {
    let q = query.to_lowercase();
    if (q.contains("edit") || q.contains("editor")) && !q.contains("vim") && !q.contains("nano")
        && !q.contains("emacs") && !q.contains("code") && !facts.has_verified(&FactKey::PreferredEditor) {
        return Some(ClarifyKind::PreferredEditor);
    }
    if (q.contains("service") || q.contains("systemctl")) && !q.contains("--failed")
        && extract_service_name(&q).is_none() {
        return Some(ClarifyKind::ServiceName);
    }
    if (q.contains("mount") || q.contains("partition")) && !q.contains("/") {
        return Some(ClarifyKind::MountPoint);
    }
    None
}

fn extract_service_name(query: &str) -> Option<String> {
    let patterns = ["nginx", "docker", "sshd", "apache", "mysql", "postgresql", "redis"];
    for p in patterns { if query.contains(p) { return Some(p.to_string()); } }
    if let Some(idx) = query.find(".service") {
        let before = &query[..idx];
        if let Some(start) = before.rfind(' ') { return Some(before[start+1..].to_string()); }
    }
    None
}

pub const KNOWN_EDITORS: &[&str] = &["vim", "vi", "nvim", "nano", "emacs", "code", "micro", "hx"];
pub const CLARIFY_CANCEL_KEY: &str = "__cancel__";
pub const CLARIFY_OTHER_KEY: &str = "__other__";

pub fn generate_editor_options_sync() -> Vec<ClarifyOption> {
    generate_editor_options_with_cache(&load_or_create_inventory())
}

pub fn generate_editor_options_with_cache(cache: &InventoryCache) -> Vec<ClarifyOption> {
    let mut options = Vec::new();
    let installed_editors = cache.installed_editors();

    for editor in KNOWN_EDITORS {
        if installed_editors.contains(editor) {
            options.push(ClarifyOption::new(*editor, *editor).with_evidence("installed: true"));
        } else if let Some(true) = cache.is_installed(editor) {
            options.push(ClarifyOption::new(*editor, *editor).with_evidence("installed: true"));
        }
    }

    if options.is_empty() {
        for editor in KNOWN_EDITORS {
            if verify_editor_installed(editor) {
                options.push(ClarifyOption::new(*editor, *editor).with_evidence("installed: true"));
                break;
            }
        }
    }

    options.push(ClarifyOption::new(CLARIFY_OTHER_KEY, "Other").with_evidence("custom input"));
    options.push(ClarifyOption::new(CLARIFY_CANCEL_KEY, "Cancel").with_evidence("skip"));
    options
}

pub fn is_cancel_selection(key: &str) -> bool { key == CLARIFY_CANCEL_KEY }
pub fn is_other_selection(key: &str) -> bool { key == CLARIFY_OTHER_KEY }

pub fn verify_editor_installed(editor: &str) -> bool {
    std::process::Command::new("which").arg(editor).output().map(|o| o.status.success()).unwrap_or(false)
}

pub fn generate_editor_clarification(facts: &FactsStore) -> (ClarifyQuestion, Vec<ClarifyOption>) {
    let default = facts.get_verified(&FactKey::PreferredEditor).map(|s| s.to_string());
    let question = ClarifyQuestion::new(ClarifyKind::PreferredEditor, "Which text editor do you prefer?")
        .with_verify("which {}").with_hint("Select from installed editors")
        .with_default(default.unwrap_or_default());
    let options = generate_editor_options_sync();
    (question, options)
}

// Tests moved to tests/clarify_tests.rs
