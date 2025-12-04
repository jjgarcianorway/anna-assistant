//! Config Atlas v7.21.0 - Clean Per-Component Configuration Discovery
//!
//! Provides:
//! - Clean config file lists without cross-contamination
//! - Explicit precedence order with [present]/[missing] markers
//! - Strict identity scoping for documentation parsing
//! - Source attribution for all discovered paths
//!
//! Rules:
//! - Only list paths that actually exist OR are documented defaults
//! - Group into "system" and "user" based on directories
//! - Derive precedence from documentation, not guesses
//! - Strip all HTML/wiki markup, keep plain text and paths only

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;

/// A configuration file entry in the atlas
#[derive(Debug, Clone)]
pub struct ConfigEntry {
    pub path: String,
    pub category: ConfigCategory,
    pub status: ConfigStatus,
    pub source: String, // Where we learned about this path
}

/// Category of configuration file
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfigCategory {
    System, // /etc, /usr/share
    User,   // $HOME, $XDG_CONFIG_HOME
}

impl ConfigCategory {
    pub fn label(&self) -> &'static str {
        match self {
            ConfigCategory::System => "system",
            ConfigCategory::User => "user",
        }
    }
}

/// Status of configuration file
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfigStatus {
    Present, // File exists and is readable
    Missing, // Documented but not present
}

impl ConfigStatus {
    pub fn label(&self) -> &'static str {
        match self {
            ConfigStatus::Present => "present",
            ConfigStatus::Missing => "missing",
        }
    }
}

/// Precedence entry showing config load order
#[derive(Debug, Clone)]
pub struct PrecedenceEntry {
    pub path: String,
    pub status: ConfigStatus,
    pub order: u32, // 1 = first match wins
}

/// Complete config atlas for a component
#[derive(Debug, Clone, Default)]
pub struct ConfigAtlas {
    pub component: String,
    /// Existing config files grouped by category
    pub existing_configs: Vec<ConfigEntry>,
    /// Recommended default locations from documentation
    pub recommended_defaults: Vec<String>,
    /// Precedence order (first match wins)
    pub precedence: Vec<PrecedenceEntry>,
    /// Sources used for discovery
    pub sources: Vec<String>,
    /// Whether we found any documentation
    pub has_documentation: bool,
    /// Config file modification times for history
    pub config_mtimes: Vec<(String, u64)>,
}

impl ConfigAtlas {
    /// Get system configs only
    pub fn system_configs(&self) -> Vec<&ConfigEntry> {
        self.existing_configs
            .iter()
            .filter(|c| c.category == ConfigCategory::System && c.status == ConfigStatus::Present)
            .collect()
    }

    /// Get user configs only
    pub fn user_configs(&self) -> Vec<&ConfigEntry> {
        self.existing_configs
            .iter()
            .filter(|c| c.category == ConfigCategory::User && c.status == ConfigStatus::Present)
            .collect()
    }
}

/// Build a clean config atlas for a software component
pub fn build_config_atlas(name: &str) -> ConfigAtlas {
    let mut atlas = ConfigAtlas {
        component: name.to_string(),
        ..Default::default()
    };

    let mut seen_paths: HashSet<String> = HashSet::new();
    let mut doc_paths: Vec<(String, String)> = Vec::new(); // (path, source)

    // 1. Discover from man pages (most authoritative for precedence)
    let man_result = discover_from_man_scoped(name);
    if !man_result.paths.is_empty() {
        atlas.sources.push(format!("man {}", name));
        atlas.has_documentation = true;
        for (path, _) in &man_result.paths {
            if !seen_paths.contains(path) {
                seen_paths.insert(path.clone());
                doc_paths.push((path.clone(), format!("man {}", name)));
            }
        }
        // Add precedence rules from man page
        for rule in &man_result.precedence {
            atlas.precedence.push(rule.clone());
        }
    }

    // 2. Discover from Arch Wiki (scoped to this component only)
    let wiki_result = discover_from_arch_wiki_scoped(name);
    if !wiki_result.paths.is_empty() {
        if let Some(ref page) = wiki_result.page_name {
            atlas.sources.push(format!("Arch Wiki: {}", page));
        }
        atlas.has_documentation = true;
        for (path, _) in &wiki_result.paths {
            if !seen_paths.contains(path) {
                seen_paths.insert(path.clone());
                let source = wiki_result
                    .page_name
                    .as_ref()
                    .map(|p| format!("Arch Wiki: {}", p))
                    .unwrap_or_else(|| "Arch Wiki".to_string());
                doc_paths.push((path.clone(), source));
            }
        }
        // Add any precedence rules from wiki
        for rule in &wiki_result.precedence {
            if atlas.precedence.iter().all(|p| p.path != rule.path) {
                atlas.precedence.push(rule.clone());
            }
        }
    }

    // 3. Discover from pacman package files
    let pacman_paths = discover_from_pacman_scoped(name);
    for path in &pacman_paths {
        if !seen_paths.contains(path) {
            seen_paths.insert(path.clone());
            doc_paths.push((path.clone(), "pacman -Ql".to_string()));
        }
    }
    if !pacman_paths.is_empty() && !atlas.sources.iter().any(|s| s.contains("pacman")) {
        atlas.sources.push("pacman -Ql".to_string());
    }

    // 4. Discover from filesystem (conventional locations)
    let fs_paths = discover_from_filesystem_scoped(name);
    for path in &fs_paths {
        if !seen_paths.contains(path) {
            seen_paths.insert(path.clone());
            // Only add to doc_paths if we found it on filesystem but not in docs
            doc_paths.push((path.clone(), "filesystem".to_string()));
        }
    }

    // 5. Build existing configs list
    for (path, source) in &doc_paths {
        let expanded = expand_path(path);
        let exists = Path::new(&expanded).exists();
        let category = categorize_path(&expanded);

        if exists {
            atlas.existing_configs.push(ConfigEntry {
                path: path.clone(),
                category,
                status: ConfigStatus::Present,
                source: source.clone(),
            });
        }
    }

    // 6. Build recommended defaults (documented paths, whether present or not)
    for (path, _) in &doc_paths {
        if !atlas.recommended_defaults.contains(path) {
            atlas.recommended_defaults.push(path.clone());
        }
    }

    // 7. Build precedence order if not already set from man/wiki
    if atlas.precedence.is_empty() {
        build_default_precedence(&mut atlas, &doc_paths);
    } else {
        // Update status in existing precedence entries
        for entry in &mut atlas.precedence {
            let expanded = expand_path(&entry.path);
            entry.status = if Path::new(&expanded).exists() {
                ConfigStatus::Present
            } else {
                ConfigStatus::Missing
            };
        }
    }

    // 8. Collect config file modification times
    for entry in &atlas.existing_configs {
        if entry.status == ConfigStatus::Present {
            let expanded = expand_path(&entry.path);
            if let Ok(metadata) = fs::metadata(&expanded) {
                if let Ok(mtime) = metadata.modified() {
                    if let Ok(duration) = mtime.duration_since(std::time::UNIX_EPOCH) {
                        atlas
                            .config_mtimes
                            .push((entry.path.clone(), duration.as_secs()));
                    }
                }
            }
        }
    }

    // 9. If no documentation found, mark sources appropriately
    if !atlas.has_documentation && !atlas.existing_configs.is_empty() {
        atlas.sources = vec!["filesystem only, no documentation match found".to_string()];
    }

    atlas
}

/// Discover config paths from man pages, scoped to the identity
fn discover_from_man_scoped(name: &str) -> ManDiscoveryResult {
    let mut result = ManDiscoveryResult::default();

    // Try to get man page content
    let output = Command::new("man").args(["-P", "cat", name]).output();

    let content = match output {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).to_string(),
        _ => return result,
    };

    // Extract from FILES section
    if let Some(files_section) = extract_man_section(&content, "FILES") {
        let paths = extract_paths_scoped(&files_section, name);
        for path in paths {
            result.paths.push((path, "FILES section".to_string()));
        }
    }

    // Extract from CONFIGURATION section
    if let Some(config_section) = extract_man_section(&content, "CONFIGURATION") {
        let paths = extract_paths_scoped(&config_section, name);
        for path in paths {
            if !result.paths.iter().any(|(p, _)| p == &path) {
                result
                    .paths
                    .push((path, "CONFIGURATION section".to_string()));
            }
        }
    }

    // Try to extract precedence order from content
    result.precedence = extract_precedence_from_man(&content, name);

    result
}

#[derive(Default)]
struct ManDiscoveryResult {
    paths: Vec<(String, String)>,
    precedence: Vec<PrecedenceEntry>,
}

/// Extract a section from man page content
fn extract_man_section(content: &str, section_name: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_section = false;
    let mut section_content = String::new();

    for line in &lines {
        let trimmed = line.trim();

        // Section headers are typically at the start of a line, all caps
        if trimmed == section_name || trimmed.starts_with(&format!("{} ", section_name)) {
            in_section = true;
            continue;
        }

        // End of section: another all-caps header
        if in_section
            && !trimmed.is_empty()
            && trimmed
                .chars()
                .all(|c| c.is_uppercase() || c.is_whitespace())
        {
            if trimmed.len() > 2 && !trimmed.contains('/') {
                break;
            }
        }

        if in_section {
            section_content.push_str(line);
            section_content.push('\n');
        }
    }

    if section_content.is_empty() {
        None
    } else {
        Some(section_content)
    }
}

/// Extract paths from text, scoped to the identity
fn extract_paths_scoped(text: &str, identity: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let path_re =
        regex::Regex::new(r#"(?:^|\s|["'])(/(?:etc|usr/share|home/\w+|~)[^\s"'<>|]+)"#).unwrap();

    for cap in path_re.captures_iter(text) {
        let path = cap[1].to_string();
        if path_belongs_to_identity(&path, identity) && looks_like_config_path(&path) {
            if !paths.contains(&path) {
                paths.push(path);
            }
        }
    }

    // Also handle $HOME and $XDG patterns
    let var_re = regex::Regex::new(r#"(?:\$HOME|\$XDG_CONFIG_HOME)[^\s"'<>|]+"#).unwrap();

    for cap in var_re.captures_iter(text) {
        let path = cap[0].to_string();
        if path_belongs_to_identity(&path, identity) {
            if !paths.contains(&path) {
                paths.push(path);
            }
        }
    }

    paths
}

/// Check if a path belongs to the given identity (strict filtering)
fn path_belongs_to_identity(path: &str, identity: &str) -> bool {
    let path_lower = path.to_lowercase();
    let id_lower = identity.to_lowercase();

    // Direct match (directory-based or filename-based)
    if path_lower.contains(&format!("/{}/", id_lower))
        || path_lower.contains(&format!("/{}", id_lower))
        || path_lower.ends_with(&format!("/{}", id_lower))
        || path_lower.contains(&format!("/{}rc", id_lower))
        || path_lower.contains(&format!("/.{}rc", id_lower))  // dotfile like .vimrc
        || path_lower.contains(&format!("/{}.conf", id_lower))
        || path_lower.contains(&format!("/{}.d/", id_lower))
    {
        return true;
    }

    // Handle common aliases
    let aliases = get_identity_aliases(identity);
    for alias in aliases {
        let alias_lower = alias.to_lowercase();
        if path_lower.contains(&format!("/{}/", alias_lower))
            || path_lower.contains(&format!("/{}", alias_lower))
            || path_lower.contains(&format!("/{}rc", alias_lower))
            || path_lower.contains(&format!("/.{}rc", alias_lower))  // dotfile
            || path_lower.contains(&format!("/{}.conf", alias_lower))
        {
            return true;
        }
    }

    false
}

/// Get known aliases for an identity
fn get_identity_aliases(identity: &str) -> Vec<&'static str> {
    match identity.to_lowercase().as_str() {
        "vim" => vec!["vim", "vimrc"],
        "nvim" | "neovim" => vec!["nvim", "neovim"],
        "hyprland" => vec!["hypr", "hyprland"],
        "waybar" => vec!["waybar"],
        "alacritty" => vec!["alacritty"],
        "kitty" => vec!["kitty"],
        "firefox" => vec!["firefox", "mozilla"],
        "chromium" | "chrome" => vec!["chromium", "chrome", "google-chrome"],
        "bash" => vec!["bash", "bashrc", "bash_profile"],
        "zsh" => vec!["zsh", "zshrc", "zprofile"],
        "fish" => vec!["fish"],
        "tmux" => vec!["tmux", "tmux.conf"],
        "git" => vec!["git", "gitconfig"],
        "ssh" => vec!["ssh", "sshd"],
        _ => vec![],
    }
}

/// Check if path looks like a config file
fn looks_like_config_path(path: &str) -> bool {
    let p = path.to_lowercase();

    // Must be in known config directories
    if !p.starts_with("/etc")
        && !p.starts_with("/usr/share")
        && !p.starts_with("/home")
        && !p.starts_with("~")
        && !p.starts_with("$home")
        && !p.starts_with("$xdg")
    {
        return false;
    }

    // Exclude obvious non-config paths
    if p.contains("/bin/")
        || p.contains("/lib/")
        || p.contains("/include/")
        || p.contains("/doc/")
        || p.contains("/man/")
        || p.contains("/locale/")
        || p.contains("/icons/")
        || p.contains("/themes/")
        || p.contains("/fonts/")
    {
        return false;
    }

    true
}

/// Extract precedence order from man page content
fn extract_precedence_from_man(content: &str, _identity: &str) -> Vec<PrecedenceEntry> {
    let mut precedence = Vec::new();

    // Look for common precedence patterns
    // "first reads X, then Y" or "X takes precedence over Y"
    let lines: Vec<&str> = content.lines().collect();
    let mut order = 1u32;

    for line in &lines {
        let lower = line.to_lowercase();

        // Look for precedence keywords
        if lower.contains("precedence")
            || lower.contains("first reads")
            || lower.contains("then reads")
            || lower.contains("overrides")
            || lower.contains("takes priority")
        {
            // Extract paths from this line
            let paths = extract_paths_scoped(line, _identity);
            for path in paths {
                if !precedence.iter().any(|p: &PrecedenceEntry| p.path == path) {
                    let expanded = expand_path(&path);
                    precedence.push(PrecedenceEntry {
                        path,
                        status: if Path::new(&expanded).exists() {
                            ConfigStatus::Present
                        } else {
                            ConfigStatus::Missing
                        },
                        order,
                    });
                    order += 1;
                }
            }
        }
    }

    precedence
}

/// Discover from Arch Wiki with strict scoping
fn discover_from_arch_wiki_scoped(name: &str) -> WikiDiscoveryResult {
    let result = WikiDiscoveryResult::default();

    // Check for arch-wiki-lite or arch-wiki-docs
    let wiki_text_dir = Path::new("/usr/share/doc/arch-wiki/text");
    let wiki_html_dir = Path::new("/usr/share/doc/arch-wiki/html");

    // Try text version first (cleaner)
    if wiki_text_dir.exists() {
        if let Some(wiki_result) = search_arch_wiki_text(name, wiki_text_dir) {
            return wiki_result;
        }
    }

    // Fall back to HTML version
    if wiki_html_dir.exists() {
        if let Some(wiki_result) = search_arch_wiki_html(name, wiki_html_dir) {
            return wiki_result;
        }
    }

    result
}

#[derive(Default)]
struct WikiDiscoveryResult {
    paths: Vec<(String, String)>,
    precedence: Vec<PrecedenceEntry>,
    page_name: Option<String>,
}

/// Search arch-wiki-lite text files
fn search_arch_wiki_text(name: &str, wiki_dir: &Path) -> Option<WikiDiscoveryResult> {
    // Try direct page name match
    let page_patterns = vec![
        format!("{}.txt.zst", capitalize_first(name)),
        format!("{}.txt.zst", name),
        format!("{}.txt.gz", capitalize_first(name)),
        format!("{}.txt.gz", name),
        format!("{}.txt", capitalize_first(name)),
        format!("{}.txt", name),
    ];

    for pattern in &page_patterns {
        let path = wiki_dir.join(pattern);
        if path.exists() {
            if let Some(content) = read_wiki_file(&path) {
                let mut result = WikiDiscoveryResult::default();
                result.page_name = Some(capitalize_first(name));

                // Extract paths scoped to this identity
                let paths = extract_wiki_config_paths(&content, name);
                for path in paths {
                    result
                        .paths
                        .push((path, "configuration section".to_string()));
                }

                // Extract precedence if documented
                result.precedence = extract_wiki_precedence(&content, name);

                if !result.paths.is_empty() {
                    return Some(result);
                }
            }
        }
    }

    None
}

/// Search arch-wiki-docs HTML files
fn search_arch_wiki_html(name: &str, wiki_dir: &Path) -> Option<WikiDiscoveryResult> {
    let page_patterns = vec![
        format!("{}.html", capitalize_first(name)),
        format!("{}.html", name),
    ];

    for pattern in &page_patterns {
        let path = wiki_dir.join("en").join(&pattern);
        if path.exists() {
            if let Some(content) = read_wiki_file(&path) {
                // Strip HTML first
                let text = strip_html(&content);

                let mut result = WikiDiscoveryResult::default();
                result.page_name = Some(capitalize_first(name));

                let paths = extract_wiki_config_paths(&text, name);
                for path in paths {
                    result
                        .paths
                        .push((path, "configuration section".to_string()));
                }

                result.precedence = extract_wiki_precedence(&text, name);

                if !result.paths.is_empty() {
                    return Some(result);
                }
            }
        }
    }

    None
}

/// Read wiki file (handles .zst, .gz, plain text)
fn read_wiki_file(path: &Path) -> Option<String> {
    let path_str = path.to_string_lossy();

    if path_str.ends_with(".zst") {
        let output = Command::new("zstdcat").arg(path).output().ok()?;
        if output.status.success() {
            return Some(String::from_utf8_lossy(&output.stdout).to_string());
        }
    } else if path_str.ends_with(".gz") {
        let output = Command::new("zcat").arg(path).output().ok()?;
        if output.status.success() {
            return Some(String::from_utf8_lossy(&output.stdout).to_string());
        }
    } else {
        return fs::read_to_string(path).ok();
    }

    None
}

/// Strip HTML tags and decode entities
fn strip_html(html: &str) -> String {
    let mut result = html.to_string();

    // Remove script and style tags with content
    let script_re = regex::Regex::new(r"(?is)<script[^>]*>.*?</script>").unwrap();
    result = script_re.replace_all(&result, "").to_string();

    let style_re = regex::Regex::new(r"(?is)<style[^>]*>.*?</style>").unwrap();
    result = style_re.replace_all(&result, "").to_string();

    // Remove all HTML tags
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
        .replace("&nbsp;", " ");

    // Collapse whitespace
    let ws_re = regex::Regex::new(r"\s+").unwrap();
    result = ws_re.replace_all(&result, " ").to_string();

    result.trim().to_string()
}

/// Extract config paths from wiki content, scoped to identity
fn extract_wiki_config_paths(content: &str, identity: &str) -> Vec<String> {
    let mut paths = Vec::new();

    // Look for Configuration section
    let sections = ["Configuration", "Config", "Files", "Setup"];
    let mut in_relevant_section = false;
    let mut section_content = String::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Check if entering a relevant section
        for section in &sections {
            if trimmed.to_lowercase().contains(&section.to_lowercase())
                && (trimmed.starts_with('#') || trimmed.starts_with('=') || trimmed.len() < 30)
            {
                in_relevant_section = true;
                section_content.clear();
                break;
            }
        }

        // Check if leaving section (new heading)
        if in_relevant_section && !trimmed.is_empty() {
            if (trimmed.starts_with('#') || trimmed.starts_with('=')) && !section_content.is_empty()
            {
                // Process collected section
                let section_paths = extract_paths_scoped(&section_content, identity);
                for p in section_paths {
                    if !paths.contains(&p) {
                        paths.push(p);
                    }
                }
                in_relevant_section = false;
            }
        }

        if in_relevant_section {
            section_content.push_str(line);
            section_content.push('\n');
        }
    }

    // Process final section if still in one
    if in_relevant_section && !section_content.is_empty() {
        let section_paths = extract_paths_scoped(&section_content, identity);
        for p in section_paths {
            if !paths.contains(&p) {
                paths.push(p);
            }
        }
    }

    // If no section found, search whole content but be strict
    if paths.is_empty() {
        paths = extract_paths_scoped(content, identity);
    }

    paths
}

/// Extract precedence rules from wiki content
fn extract_wiki_precedence(content: &str, identity: &str) -> Vec<PrecedenceEntry> {
    let mut precedence = Vec::new();

    // Look for precedence indicators
    for line in content.lines() {
        let lower = line.to_lowercase();
        if lower.contains("precedence")
            || lower.contains("read first")
            || lower.contains("read before")
            || lower.contains("overrides")
            || lower.contains("takes priority")
        {
            let paths = extract_paths_scoped(line, identity);
            for (i, path) in paths.into_iter().enumerate() {
                if !precedence.iter().any(|p: &PrecedenceEntry| p.path == path) {
                    let expanded = expand_path(&path);
                    precedence.push(PrecedenceEntry {
                        path,
                        status: if Path::new(&expanded).exists() {
                            ConfigStatus::Present
                        } else {
                            ConfigStatus::Missing
                        },
                        order: (i + 1) as u32,
                    });
                }
            }
        }
    }

    precedence
}

/// Discover config files from pacman package listing
fn discover_from_pacman_scoped(name: &str) -> Vec<String> {
    let mut paths = Vec::new();

    let output = Command::new("pacman").args(["-Ql", name]).output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if let Some(path) = line.split_whitespace().nth(1) {
                    if path.starts_with("/etc/") && looks_like_config_path(path) {
                        if !paths.contains(&path.to_string()) {
                            paths.push(path.to_string());
                        }
                    }
                }
            }
        }
    }

    paths
}

/// Discover config files from filesystem (conventional locations)
fn discover_from_filesystem_scoped(name: &str) -> Vec<String> {
    let mut paths = Vec::new();

    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
    let xdg_config =
        std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", home));

    // Conventional config locations to check
    let candidates = vec![
        // System configs
        format!("/etc/{}", name),
        format!("/etc/{}.conf", name),
        format!("/etc/{}/", name),
        format!("/etc/{}.d/", name),
        format!("/etc/xdg/{}/", name),
        // User configs
        format!("{}/{}", xdg_config, name),
        format!("{}/{}/", xdg_config, name),
        format!("{}/.{}rc", home, name),
        format!("{}/.{}", home, name),
        format!("{}/.config/{}", home, name),
    ];

    // Also check for common aliases
    let aliases = get_identity_aliases(name);
    for alias in aliases {
        let alias_candidates = vec![
            format!("/etc/{}", alias),
            format!("/etc/{}.conf", alias),
            format!("{}/{}", xdg_config, alias),
            format!("{}/.{}rc", home, alias),
        ];
        for candidate in alias_candidates {
            if Path::new(&candidate).exists() && !paths.contains(&candidate) {
                paths.push(candidate);
            }
        }
    }

    for candidate in candidates {
        if Path::new(&candidate).exists() && !paths.contains(&candidate) {
            paths.push(candidate);
        }
    }

    paths
}

/// Expand path variables like $HOME, ~, $XDG_CONFIG_HOME
fn expand_path(path: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
    let xdg_config =
        std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", home));

    path.replace("$HOME", &home)
        .replace("$XDG_CONFIG_HOME", &xdg_config)
        .replace("~", &home)
}

/// Categorize path as system or user
fn categorize_path(path: &str) -> ConfigCategory {
    let expanded = expand_path(path);
    if expanded.starts_with("/etc") || expanded.starts_with("/usr") || expanded.starts_with("/var")
    {
        ConfigCategory::System
    } else {
        ConfigCategory::User
    }
}

/// Build default precedence order (user configs before system)
fn build_default_precedence(atlas: &mut ConfigAtlas, doc_paths: &[(String, String)]) {
    let mut user_paths: Vec<String> = Vec::new();
    let mut system_paths: Vec<String> = Vec::new();

    for (path, _) in doc_paths {
        let expanded = expand_path(path);
        if categorize_path(&expanded) == ConfigCategory::User {
            user_paths.push(path.clone());
        } else {
            system_paths.push(path.clone());
        }
    }

    // User configs have higher precedence (first match wins)
    let mut order = 1u32;
    for path in user_paths {
        let expanded = expand_path(&path);
        atlas.precedence.push(PrecedenceEntry {
            path,
            status: if Path::new(&expanded).exists() {
                ConfigStatus::Present
            } else {
                ConfigStatus::Missing
            },
            order,
        });
        order += 1;
    }

    for path in system_paths {
        let expanded = expand_path(&path);
        atlas.precedence.push(PrecedenceEntry {
            path,
            status: if Path::new(&expanded).exists() {
                ConfigStatus::Present
            } else {
                ConfigStatus::Missing
            },
            order,
        });
        order += 1;
    }

    // Always add implicit compiled-in defaults at the end
    if !atlas.precedence.is_empty() {
        atlas.precedence.push(PrecedenceEntry {
            path: "compiled-in defaults".to_string(),
            status: ConfigStatus::Present, // Always "present" conceptually
            order,
        });
    }
}

/// Capitalize first letter of a string
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_belongs_to_identity() {
        assert!(path_belongs_to_identity("/etc/vim/vimrc", "vim"));
        assert!(path_belongs_to_identity("~/.vimrc", "vim"));
        assert!(!path_belongs_to_identity("/etc/nvim/init.vim", "vim"));
        assert!(path_belongs_to_identity("/etc/nvim/init.vim", "nvim"));
    }

    #[test]
    fn test_categorize_path() {
        assert_eq!(categorize_path("/etc/vim/vimrc"), ConfigCategory::System);
        assert_eq!(categorize_path("~/.vimrc"), ConfigCategory::User);
        assert_eq!(categorize_path("$HOME/.config/vim"), ConfigCategory::User);
    }

    #[test]
    fn test_expand_path() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/test".to_string());
        assert_eq!(expand_path("~/.vimrc"), format!("{}/.vimrc", home));
        assert_eq!(expand_path("$HOME/.config"), format!("{}/.config", home));
    }

    #[test]
    fn test_looks_like_config_path() {
        assert!(looks_like_config_path("/etc/vim/vimrc"));
        assert!(looks_like_config_path("~/.config/vim"));
        assert!(!looks_like_config_path("/usr/bin/vim"));
        assert!(!looks_like_config_path("/usr/lib/vim"));
    }
}
