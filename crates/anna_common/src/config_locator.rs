//! Config Locator v7.30.0 - Evidence-Based Configuration Discovery
//!
//! Non-negotiables:
//! - No LLM, no hardcoded per-app paths
//! - No guessing - every "exists" claim backed by filesystem checks
//! - No truncation - wrap everything
//! - No irrelevant config paths for other apps
//! - If data unavailable, omit the section
//!
//! Two buckets:
//! 1. "Detected on this system" (facts) - paths that exist right now
//! 2. "Recommended locations" (references) - from documentation only

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use serde::{Deserialize, Serialize};

/// Detected config entry - a path that EXISTS on this system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedConfig {
    pub path: String,
    pub kind: ConfigKind,
    pub scope: ConfigScope,
    pub owner_uid: u32,
    pub owner_gid: u32,
    pub last_modified_epoch: u64,
    pub evidence: String,  // exact probe used (stat, find, etc.)
}

/// Recommended config entry - from documentation, may not exist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedConfig {
    pub path_pattern: String,  // may include $XDG_CONFIG_HOME, $HOME, etc.
    pub scope: ConfigScope,
    pub priority: u32,  // lower is more canonical
    pub source: DocSource,
    pub source_ref: String,  // page name and section header when possible
    pub note: String,  // short explanation, no HTML
}

/// Kind of config entry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigKind {
    File,
    Dir,
    Symlink,
}

impl ConfigKind {
    pub fn from_path(path: &Path) -> Option<Self> {
        let metadata = fs::symlink_metadata(path).ok()?;
        if metadata.is_symlink() {
            Some(ConfigKind::Symlink)
        } else if metadata.is_dir() {
            Some(ConfigKind::Dir)
        } else if metadata.is_file() {
            Some(ConfigKind::File)
        } else {
            None
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ConfigKind::File => "file",
            ConfigKind::Dir => "dir",
            ConfigKind::Symlink => "symlink",
        }
    }
}

/// Scope of config
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigScope {
    User,
    System,
}

impl ConfigScope {
    pub fn from_path(path: &str) -> Self {
        let expanded = expand_path_vars(path);
        if expanded.starts_with("/etc")
            || expanded.starts_with("/usr")
            || expanded.starts_with("/var")
        {
            ConfigScope::System
        } else {
            ConfigScope::User
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ConfigScope::User => "user",
            ConfigScope::System => "system",
        }
    }
}

/// Documentation source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DocSource {
    ArchWikiDocs,
    Man,
    UsrShareDoc,
    HelpFlag,
}

impl DocSource {
    pub fn label(&self) -> &'static str {
        match self {
            DocSource::ArchWikiDocs => "arch-wiki-docs",
            DocSource::Man => "man",
            DocSource::UsrShareDoc => "/usr/share/doc",
            DocSource::HelpFlag => "--help",
        }
    }
}

/// Documented precedence rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrecedenceRule {
    pub paths: Vec<String>,  // ordered list of paths
    pub source: DocSource,
    pub source_ref: String,
    pub verbatim_quote: String,  // exact text from docs stating precedence
}

/// Complete config discovery result for a software identity
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigDiscovery {
    pub identity: String,
    pub detected: Vec<DetectedConfig>,
    pub recommended: Vec<RecommendedConfig>,
    pub precedence: Option<PrecedenceRule>,  // None if not explicitly documented
    pub doc_excerpts_path: Option<String>,  // path to cleaned excerpt file
}

/// Full config index for all discovered software
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigIndex {
    pub version: String,
    pub generated_epoch: u64,
    pub entries: std::collections::HashMap<String, ConfigDiscovery>,
}

// ============================================================================
// Main API
// ============================================================================

/// Discover config for a single software identity
/// Returns evidence-based config discovery with strict scoping
pub fn discover_config(identity: &str) -> ConfigDiscovery {
    let mut discovery = ConfigDiscovery {
        identity: identity.to_string(),
        ..Default::default()
    };

    // Step 1: Gather documentation sources (priority order)
    let doc_result = gather_documentation(identity);

    // Step 2: Extract recommended paths from documentation
    discovery.recommended = extract_recommended_from_docs(identity, &doc_result);

    // Step 3: Extract precedence if explicitly documented
    discovery.precedence = extract_precedence_from_docs(identity, &doc_result);

    // Step 4: Detect actually existing config paths
    discovery.detected = detect_existing_configs(identity, &discovery.recommended);

    // Step 5: Save doc excerpts if we found any
    if !doc_result.excerpts.is_empty() {
        discovery.doc_excerpts_path = save_doc_excerpts(identity, &doc_result.excerpts);
    }

    discovery
}

/// Load config discovery from cache
pub fn load_cached_config(identity: &str) -> Option<ConfigDiscovery> {
    let index_path = Path::new("/var/lib/anna/kdb/config_index.json");
    if !index_path.exists() {
        return None;
    }

    let content = fs::read_to_string(index_path).ok()?;
    let index: ConfigIndex = serde_json::from_str(&content).ok()?;
    index.entries.get(identity).cloned()
}

/// Check if config index needs refresh (TTL 6 hours or pacman changes)
pub fn config_index_stale() -> bool {
    let index_path = Path::new("/var/lib/anna/kdb/config_index.json");
    if !index_path.exists() {
        return true;
    }

    // Check modification time
    if let Ok(metadata) = fs::metadata(index_path) {
        if let Ok(mtime) = metadata.modified() {
            if let Ok(duration) = mtime.duration_since(std::time::UNIX_EPOCH) {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let age_secs = now.saturating_sub(duration.as_secs());

                // Stale if older than 6 hours
                if age_secs > 6 * 3600 {
                    return true;
                }
            }
        }
    }

    // Check if pacman log is newer than index
    let pacman_log = Path::new("/var/log/pacman.log");
    if let (Ok(index_meta), Ok(pacman_meta)) = (fs::metadata(index_path), fs::metadata(pacman_log)) {
        if let (Ok(index_mtime), Ok(pacman_mtime)) = (index_meta.modified(), pacman_meta.modified()) {
            if pacman_mtime > index_mtime {
                return true;
            }
        }
    }

    false
}

// ============================================================================
// Documentation Gathering
// ============================================================================

#[derive(Default)]
struct DocResult {
    man_content: Option<String>,
    wiki_content: Option<String>,
    wiki_page_name: Option<String>,
    usr_share_doc_content: Option<String>,
    help_content: Option<String>,
    excerpts: Vec<String>,
}

fn gather_documentation(identity: &str) -> DocResult {
    let mut result = DocResult::default();

    // 1. Man page (most authoritative)
    if let Some(man_content) = get_man_page_content(identity) {
        let cleaned = extract_config_relevant_lines(&man_content, identity);
        if !cleaned.is_empty() {
            result.man_content = Some(cleaned.clone());
            result.excerpts.push(format!("# Source: man {}\n\n{}", identity, cleaned));
        }
    }

    // 2. Arch Wiki (local docs only)
    if let Some((wiki_content, page_name)) = get_arch_wiki_content(identity) {
        let cleaned = extract_config_relevant_lines(&wiki_content, identity);
        if !cleaned.is_empty() {
            result.wiki_content = Some(cleaned.clone());
            result.wiki_page_name = Some(page_name.clone());
            result.excerpts.push(format!("# Source: Arch Wiki: {}\n\n{}", page_name, cleaned));
        }
    }

    // 3. /usr/share/doc for relevant packages
    if let Some(doc_content) = get_usr_share_doc_content(identity) {
        let cleaned = extract_config_relevant_lines(&doc_content, identity);
        if !cleaned.is_empty() {
            result.usr_share_doc_content = Some(cleaned.clone());
            result.excerpts.push(format!("# Source: /usr/share/doc/{}\n\n{}", identity, cleaned));
        }
    }

    // 4. --help output (only if it contains explicit config file flags)
    if let Some(help_content) = get_help_flag_content(identity) {
        let cleaned = extract_config_relevant_lines(&help_content, identity);
        if !cleaned.is_empty() && cleaned.contains("config") {
            result.help_content = Some(cleaned.clone());
            result.excerpts.push(format!("# Source: {} --help\n\n{}", identity, cleaned));
        }
    }

    result
}

fn get_man_page_content(identity: &str) -> Option<String> {
    let output = Command::new("man")
        .args(["-P", "cat", identity])
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        None
    }
}

fn get_arch_wiki_content(identity: &str) -> Option<(String, String)> {
    let wiki_html_dir = Path::new("/usr/share/doc/arch-wiki/html/en");
    if !wiki_html_dir.exists() {
        return None;
    }

    // Try different capitalizations
    let page_patterns = vec![
        capitalize_first(identity),
        identity.to_string(),
        identity.to_uppercase(),
    ];

    for page_name in page_patterns {
        let path = wiki_html_dir.join(format!("{}.html", page_name));
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                let cleaned = strip_html_completely(&content);
                return Some((cleaned, page_name));
            }
        }
    }

    None
}

fn get_usr_share_doc_content(identity: &str) -> Option<String> {
    let doc_dir = Path::new("/usr/share/doc").join(identity);
    if !doc_dir.exists() {
        return None;
    }

    let mut content = String::new();

    // Look for README, CONFIGURATION, etc.
    let interesting_files = ["README", "README.md", "CONFIGURATION", "CONFIG", "config.md"];

    if let Ok(entries) = fs::read_dir(&doc_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let name_upper = name.to_uppercase();

            if interesting_files.iter().any(|f| name_upper.contains(f)) {
                if let Ok(file_content) = fs::read_to_string(entry.path()) {
                    content.push_str(&format!("# File: {}\n\n", name));
                    content.push_str(&file_content);
                    content.push_str("\n\n");
                }
            }
        }
    }

    if content.is_empty() {
        None
    } else {
        Some(content)
    }
}

fn get_help_flag_content(identity: &str) -> Option<String> {
    // Only run --help for commands in PATH
    let which_output = Command::new("which")
        .arg(identity)
        .output()
        .ok()?;

    if !which_output.status.success() {
        return None;
    }

    // Try --help
    let output = Command::new(identity)
        .arg("--help")
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let combined = format!("{}\n{}", stdout, stderr);
    if combined.trim().is_empty() {
        None
    } else {
        Some(combined)
    }
}

// ============================================================================
// HTML Stripping and Content Cleaning
// ============================================================================

/// Strip HTML completely - remove ALL markup, CSS, JS, nav, footer
fn strip_html_completely(html: &str) -> String {
    let mut result = html.to_string();

    // Remove script tags with content
    let script_re = regex::Regex::new(r"(?is)<script[^>]*>.*?</script>").unwrap();
    result = script_re.replace_all(&result, "").to_string();

    // Remove style tags with content
    let style_re = regex::Regex::new(r"(?is)<style[^>]*>.*?</style>").unwrap();
    result = style_re.replace_all(&result, "").to_string();

    // Remove nav sections
    let nav_re = regex::Regex::new(r"(?is)<nav[^>]*>.*?</nav>").unwrap();
    result = nav_re.replace_all(&result, "").to_string();

    // Remove header sections (site header, not content headers)
    let header_re = regex::Regex::new(r"(?is)<header[^>]*>.*?</header>").unwrap();
    result = header_re.replace_all(&result, "").to_string();

    // Remove footer sections
    let footer_re = regex::Regex::new(r"(?is)<footer[^>]*>.*?</footer>").unwrap();
    result = footer_re.replace_all(&result, "").to_string();

    // Remove table of contents (common wiki pattern)
    let toc_re = regex::Regex::new(r#"(?is)<div[^>]*id="toc"[^>]*>.*?</div>"#).unwrap();
    result = toc_re.replace_all(&result, "").to_string();

    // Remove sidebar
    let sidebar_re = regex::Regex::new(r#"(?is)<div[^>]*class="[^"]*sidebar[^"]*"[^>]*>.*?</div>"#).unwrap();
    result = sidebar_re.replace_all(&result, "").to_string();

    // Remove all remaining HTML tags
    let tag_re = regex::Regex::new(r"<[^>]+>").unwrap();
    result = tag_re.replace_all(&result, " ").to_string();

    // Decode common HTML entities
    result = result
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .replace("&#160;", " ")
        .replace("&ndash;", "-")
        .replace("&mdash;", "--")
        .replace("&#8211;", "-")
        .replace("&#8212;", "--");

    // Collapse multiple whitespace
    let ws_re = regex::Regex::new(r"[ \t]+").unwrap();
    result = ws_re.replace_all(&result, " ").to_string();

    // Collapse multiple newlines
    let nl_re = regex::Regex::new(r"\n\s*\n+").unwrap();
    result = nl_re.replace_all(&result, "\n\n").to_string();

    result.trim().to_string()
}

/// Extract only config-relevant lines from content
/// Keeps lines containing config-related tokens
fn extract_config_relevant_lines(content: &str, identity: &str) -> String {
    let config_tokens = [
        "config", "configuration", ".conf", "xdg_config_home", "~/.config",
        "/etc/", "rc file", "rc-file", "profile", "include", "drop-in",
        ".d/", "conf.d", "settings", "options file", identity,
    ];

    let identity_lower = identity.to_lowercase();
    let mut relevant_lines = Vec::new();
    let mut line_count = 0;

    for line in content.lines() {
        if line_count >= 40 {
            break;  // Max 40 lines of excerpts per app
        }

        let line_lower = line.to_lowercase();
        let trimmed = line.trim();

        // Skip empty lines and very short lines
        if trimmed.len() < 5 {
            continue;
        }

        // Check if line contains any config token
        let is_relevant = config_tokens.iter().any(|token| line_lower.contains(token));

        // Also include lines that look like file paths for this identity
        let has_identity_path = line_lower.contains(&format!("/{}/", identity_lower))
            || line_lower.contains(&format!("/{}", identity_lower))
            || line_lower.contains(&format!(".{}rc", identity_lower))
            || line_lower.contains(&format!("{}.conf", identity_lower));

        if is_relevant || has_identity_path {
            relevant_lines.push(trimmed.to_string());
            line_count += 1;
        }
    }

    relevant_lines.join("\n")
}

// ============================================================================
// Path Extraction
// ============================================================================

fn extract_recommended_from_docs(identity: &str, doc_result: &DocResult) -> Vec<RecommendedConfig> {
    let mut recommended = Vec::new();
    let mut seen_patterns: HashSet<String> = HashSet::new();

    // Extract from man page
    if let Some(ref content) = doc_result.man_content {
        let paths = extract_paths_strict(content, identity);
        for (i, path) in paths.into_iter().enumerate() {
            if seen_patterns.insert(path.clone()) {
                recommended.push(RecommendedConfig {
                    path_pattern: path.clone(),
                    scope: ConfigScope::from_path(&path),
                    priority: (i + 1) as u32,
                    source: DocSource::Man,
                    source_ref: format!("man {}", identity),
                    note: String::new(),
                });
            }
        }
    }

    // Extract from wiki
    if let Some(ref content) = doc_result.wiki_content {
        let page_name = doc_result.wiki_page_name.as_deref().unwrap_or(identity);
        let paths = extract_paths_strict(content, identity);
        for (i, path) in paths.into_iter().enumerate() {
            if seen_patterns.insert(path.clone()) {
                recommended.push(RecommendedConfig {
                    path_pattern: path.clone(),
                    scope: ConfigScope::from_path(&path),
                    priority: (recommended.len() + i + 1) as u32,
                    source: DocSource::ArchWikiDocs,
                    source_ref: format!("Arch Wiki: {}", page_name),
                    note: String::new(),
                });
            }
        }
    }

    // Extract from /usr/share/doc
    if let Some(ref content) = doc_result.usr_share_doc_content {
        let paths = extract_paths_strict(content, identity);
        for (i, path) in paths.into_iter().enumerate() {
            if seen_patterns.insert(path.clone()) {
                recommended.push(RecommendedConfig {
                    path_pattern: path.clone(),
                    scope: ConfigScope::from_path(&path),
                    priority: (recommended.len() + i + 1) as u32,
                    source: DocSource::UsrShareDoc,
                    source_ref: format!("/usr/share/doc/{}", identity),
                    note: String::new(),
                });
            }
        }
    }

    // Extract from --help
    if let Some(ref content) = doc_result.help_content {
        let paths = extract_paths_strict(content, identity);
        for (i, path) in paths.into_iter().enumerate() {
            if seen_patterns.insert(path.clone()) {
                recommended.push(RecommendedConfig {
                    path_pattern: path.clone(),
                    scope: ConfigScope::from_path(&path),
                    priority: (recommended.len() + i + 1) as u32,
                    source: DocSource::HelpFlag,
                    source_ref: format!("{} --help", identity),
                    note: String::new(),
                });
            }
        }
    }

    recommended
}

/// Extract paths from text with STRICT identity scoping
/// Only returns paths that actually belong to this identity
fn extract_paths_strict(text: &str, identity: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let identity_lower = identity.to_lowercase();

    // Regex for absolute paths
    let abs_path_re = regex::Regex::new(
        r#"(?:^|\s|["'`])(/(?:etc|usr/share|home/\w+|var)[^\s"'`<>|;,\)\]]+)"#
    ).unwrap();

    for cap in abs_path_re.captures_iter(text) {
        let path = cap[1].to_string().trim_end_matches(&['.', ',', ';', ':'][..]).to_string();
        if path_strictly_belongs_to(&path, &identity_lower) && looks_like_config(&path) {
            if !paths.contains(&path) {
                paths.push(path);
            }
        }
    }

    // Regex for ~/.config/X paths - more specific
    let tilde_config_re = regex::Regex::new(
        r#"~/\.config/([a-zA-Z0-9_-]+)(?:/[^\s"'`<>|;,\)\]]+)?"#
    ).unwrap();

    for cap in tilde_config_re.captures_iter(text) {
        let dir_name = &cap[1];
        if dir_name.to_lowercase() == identity_lower || dir_name.to_lowercase().starts_with(&identity_lower) {
            let full_match = cap[0].to_string();
            let path = full_match.trim_end_matches(&['.', ',', ';', ':'][..]).to_string();
            if !paths.contains(&path) {
                paths.push(path);
            }
        }
    }

    // Regex for $XDG_CONFIG_HOME/X paths
    let xdg_config_re = regex::Regex::new(
        r#"\$XDG_CONFIG_HOME/([a-zA-Z0-9_-]+)(?:/[^\s"'`<>|;,\)\]]+)?"#
    ).unwrap();

    for cap in xdg_config_re.captures_iter(text) {
        let dir_name = &cap[1];
        if dir_name.to_lowercase() == identity_lower || dir_name.to_lowercase().starts_with(&identity_lower) {
            let full_match = cap[0].to_string();
            let path = full_match.trim_end_matches(&['.', ',', ';', ':'][..]).to_string();
            if !paths.contains(&path) {
                paths.push(path);
            }
        }
    }

    // Regex for $HOME/.X paths (dotfiles)
    let home_dot_re = regex::Regex::new(
        r#"\$HOME/\.([a-zA-Z0-9_-]+)(?:rc|\.conf)?"#
    ).unwrap();

    for cap in home_dot_re.captures_iter(text) {
        let name = &cap[1];
        if name.to_lowercase() == identity_lower {
            let full_match = cap[0].to_string();
            let path = full_match.trim_end_matches(&['.', ',', ';', ':'][..]).to_string();
            if !paths.contains(&path) {
                paths.push(path);
            }
        }
    }

    paths
}

/// Check if a path STRICTLY belongs to the given identity
/// This is the key function for preventing cross-contamination
fn path_strictly_belongs_to(path: &str, identity: &str) -> bool {
    let path_lower = path.to_lowercase();

    // Must contain the identity name in the path
    // Either as a directory component or in the filename

    // Check for /identity/ or /identity (at end)
    if path_lower.contains(&format!("/{}/", identity))
        || path_lower.ends_with(&format!("/{}", identity))
    {
        return true;
    }

    // Check for identity.conf or .identityrc patterns
    if path_lower.contains(&format!("{}.conf", identity))
        || path_lower.contains(&format!("{}.d/", identity))
        || path_lower.contains(&format!(".{}rc", identity))
        || path_lower.contains(&format!("/{}rc", identity))
    {
        return true;
    }

    // Check for ~/.identity or $XDG_CONFIG_HOME/identity
    let filename_re = regex::Regex::new(
        &format!(r"[/~]\.?{}(?:$|/|\.)", regex::escape(identity))
    ).unwrap();
    if filename_re.is_match(&path_lower) {
        return true;
    }

    false
}

/// Check if path looks like a config location (not binary, lib, etc.)
fn looks_like_config(path: &str) -> bool {
    let p = path.to_lowercase();

    // Must start with known config-related prefixes
    let valid_prefixes = [
        "/etc", "/usr/share", "/home", "~", "$home", "$xdg",
    ];
    let has_valid_prefix = valid_prefixes.iter().any(|prefix| p.starts_with(prefix));
    if !has_valid_prefix {
        return false;
    }

    // Reject obvious non-config paths
    let reject_patterns = [
        "/bin/", "/lib/", "/lib64/", "/libexec/",
        "/include/", "/doc/", "/man/", "/info/",
        "/locale/", "/icons/", "/themes/", "/fonts/",
        "/pixmaps/", "/applications/", "/mime/",
        "/licenses/", "/completions/", "/bash-completion/",
        "/zsh/", "/fish/",
    ];

    if reject_patterns.iter().any(|pat| p.contains(pat)) {
        return false;
    }

    true
}

// ============================================================================
// Precedence Extraction
// ============================================================================

fn extract_precedence_from_docs(identity: &str, doc_result: &DocResult) -> Option<PrecedenceRule> {
    // Only extract precedence if EXPLICITLY documented
    // Do not invent precedence rules

    let precedence_keywords = [
        "precedence", "read first", "read before", "reads first",
        "overrides", "takes priority", "priority over", "loaded before",
        "loaded after", "first match", "fallback to",
    ];

    // Check man page first
    if let Some(ref content) = doc_result.man_content {
        if let Some(rule) = find_explicit_precedence(content, identity, DocSource::Man, &format!("man {}", identity), &precedence_keywords) {
            return Some(rule);
        }
    }

    // Check wiki
    if let Some(ref content) = doc_result.wiki_content {
        let page_name = doc_result.wiki_page_name.as_deref().unwrap_or(identity);
        if let Some(rule) = find_explicit_precedence(content, identity, DocSource::ArchWikiDocs, &format!("Arch Wiki: {}", page_name), &precedence_keywords) {
            return Some(rule);
        }
    }

    // Check /usr/share/doc
    if let Some(ref content) = doc_result.usr_share_doc_content {
        if let Some(rule) = find_explicit_precedence(content, identity, DocSource::UsrShareDoc, &format!("/usr/share/doc/{}", identity), &precedence_keywords) {
            return Some(rule);
        }
    }

    None
}

fn find_explicit_precedence(content: &str, identity: &str, source: DocSource, source_ref: &str, keywords: &[&str]) -> Option<PrecedenceRule> {
    for line in content.lines() {
        let line_lower = line.to_lowercase();

        // Check if this line contains a precedence keyword
        if keywords.iter().any(|kw| line_lower.contains(kw)) {
            // Extract paths from this line
            let paths = extract_paths_strict(line, identity);
            if paths.len() >= 2 {
                return Some(PrecedenceRule {
                    paths,
                    source,
                    source_ref: source_ref.to_string(),
                    verbatim_quote: line.trim().to_string(),
                });
            }
        }
    }

    None
}

// ============================================================================
// Detection of Existing Configs
// ============================================================================

fn detect_existing_configs(identity: &str, recommended: &[RecommendedConfig]) -> Vec<DetectedConfig> {
    let mut detected = Vec::new();
    let mut seen_paths: HashSet<String> = HashSet::new();

    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
    let xdg_config = std::env::var("XDG_CONFIG_HOME")
        .unwrap_or_else(|_| format!("{}/.config", home));

    // Check recommended paths first
    for rec in recommended {
        let expanded = expand_path_vars(&rec.path_pattern);
        if seen_paths.contains(&expanded) {
            continue;
        }

        if let Some(entry) = probe_path(&expanded, &rec.path_pattern) {
            seen_paths.insert(expanded);
            detected.push(entry);
        }
    }

    // Also check conventional locations for this identity
    let mut candidates = vec![
        // System configs
        format!("/etc/{}", identity),
        format!("/etc/{}.conf", identity),
        format!("/etc/{}/", identity),
        format!("/etc/{}.d/", identity),
        // User configs
        format!("{}/{}", xdg_config, identity),
        format!("{}/{}/{}.conf", xdg_config, identity, identity),
        format!("{}/.{}rc", home, identity),
        format!("{}/.{}", home, identity),
        // /usr/share defaults
        format!("/usr/share/{}", identity),
        format!("/usr/share/{}/{}.conf", identity, identity),
    ];

    // Add common aliases (e.g., hypr for hyprland)
    let aliases = get_identity_aliases(identity);
    for alias in aliases {
        candidates.push(format!("{}/{}", xdg_config, alias));
        candidates.push(format!("{}/{}/{}.conf", xdg_config, alias, identity));
        candidates.push(format!("/etc/{}", alias));
        candidates.push(format!("/usr/share/{}", alias));
        candidates.push(format!("/usr/share/{}/{}.conf", alias, identity));
    }

    for candidate in candidates {
        if seen_paths.contains(&candidate) {
            continue;
        }

        if let Some(entry) = probe_path(&candidate, &candidate) {
            seen_paths.insert(candidate);
            detected.push(entry);
        }
    }

    detected
}

/// Probe a path and return DetectedConfig if it exists
fn probe_path(expanded_path: &str, original_pattern: &str) -> Option<DetectedConfig> {
    let path = Path::new(expanded_path);
    if !path.exists() {
        return None;
    }

    let metadata = fs::metadata(path).ok()?;
    let kind = ConfigKind::from_path(path)?;

    #[cfg(unix)]
    let (uid, gid) = {
        use std::os::unix::fs::MetadataExt;
        (metadata.uid(), metadata.gid())
    };
    #[cfg(not(unix))]
    let (uid, gid) = (0u32, 0u32);

    let mtime = metadata.modified().ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    Some(DetectedConfig {
        path: original_pattern.to_string(),
        kind,
        scope: ConfigScope::from_path(expanded_path),
        owner_uid: uid,
        owner_gid: gid,
        last_modified_epoch: mtime,
        evidence: format!("stat {}", expanded_path),
    })
}

/// Get common config directory aliases for an identity
/// e.g., hyprland uses ~/.config/hypr, vim uses ~/.vim
fn get_identity_aliases(identity: &str) -> Vec<&'static str> {
    match identity.to_lowercase().as_str() {
        "hyprland" => vec!["hypr"],
        "vim" => vec!["vim"],
        "nvim" | "neovim" => vec!["nvim"],
        "alacritty" => vec!["alacritty"],
        "kitty" => vec!["kitty"],
        "foot" => vec!["foot"],
        "waybar" => vec!["waybar"],
        "mako" => vec!["mako"],
        "dunst" => vec!["dunst"],
        "wofi" => vec!["wofi"],
        "rofi" => vec!["rofi"],
        "firefox" => vec!["mozilla", "firefox"],
        "chromium" => vec!["chromium"],
        "mpv" => vec!["mpv"],
        "git" => vec!["git"],
        "ssh" => vec!["ssh"],
        "tmux" => vec!["tmux"],
        "bash" => vec!["bash"],
        "zsh" => vec!["zsh"],
        "fish" => vec!["fish"],
        _ => vec![],
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn expand_path_vars(path: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
    let xdg_config = std::env::var("XDG_CONFIG_HOME")
        .unwrap_or_else(|_| format!("{}/.config", home));

    path.replace("$HOME", &home)
        .replace("$XDG_CONFIG_HOME", &xdg_config)
        .replace("$XDG_CONFIG_DIRS", "/etc/xdg")
        .replace("~", &home)
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn save_doc_excerpts(identity: &str, excerpts: &[String]) -> Option<String> {
    let excerpts_dir = Path::new("/var/lib/anna/kdb/doc_excerpts");

    // Create directory if it doesn't exist (will fail silently if no permissions)
    let _ = fs::create_dir_all(excerpts_dir);

    let file_path = excerpts_dir.join(format!("{}.txt", identity));
    let content = excerpts.join("\n\n---\n\n");

    if fs::write(&file_path, &content).is_ok() {
        Some(file_path.to_string_lossy().to_string())
    } else {
        None
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_strictly_belongs_to() {
        // Hyprland paths
        assert!(path_strictly_belongs_to("/etc/hypr/hyprland.conf", "hyprland"));
        assert!(path_strictly_belongs_to("~/.config/hypr", "hypr"));
        assert!(path_strictly_belongs_to("/usr/share/hypr/hyprland.conf", "hyprland"));

        // Should NOT match other apps
        assert!(!path_strictly_belongs_to("/etc/mako/config", "hyprland"));
        assert!(!path_strictly_belongs_to("~/.config/foot/foot.ini", "hyprland"));

        // Vim paths
        assert!(path_strictly_belongs_to("~/.vimrc", "vim"));
        assert!(path_strictly_belongs_to("/etc/vim/vimrc", "vim"));
        assert!(!path_strictly_belongs_to("/etc/nvim/init.vim", "vim"));
    }

    #[test]
    fn test_looks_like_config() {
        assert!(looks_like_config("/etc/vim/vimrc"));
        assert!(looks_like_config("~/.config/hypr/hyprland.conf"));
        assert!(looks_like_config("/usr/share/hypr/hyprland.conf"));

        // Should reject
        assert!(!looks_like_config("/usr/bin/vim"));
        assert!(!looks_like_config("/usr/lib/vim"));
        assert!(!looks_like_config("/usr/share/doc/vim/README"));
        assert!(!looks_like_config("/usr/share/icons/vim.png"));
    }

    #[test]
    fn test_config_scope() {
        assert_eq!(ConfigScope::from_path("/etc/vim/vimrc"), ConfigScope::System);
        assert_eq!(ConfigScope::from_path("~/.vimrc"), ConfigScope::User);
        assert_eq!(ConfigScope::from_path("$HOME/.config/vim"), ConfigScope::User);
        assert_eq!(ConfigScope::from_path("/usr/share/hypr/hyprland.conf"), ConfigScope::System);
    }

    #[test]
    fn test_strip_html() {
        let html = "<html><head><style>css{}</style></head><body><p>Test config at /etc/test.conf</p></body></html>";
        let stripped = strip_html_completely(html);
        assert!(stripped.contains("/etc/test.conf"));
        assert!(!stripped.contains("<"));
        assert!(!stripped.contains("css"));
    }
}
