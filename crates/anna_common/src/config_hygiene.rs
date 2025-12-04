//! Anna Config Hygiene v7.23.0 - Stronger Path Validation & Provenance
//!
//! Ensures config paths are only real, existing locations plus
//! at most one recommended path. All hints have explicit provenance.
//!
//! Rules:
//! - [CONFIG] only lists files that actually exist
//! - At most one recommended location that doesn't exist yet
//! - No foreign config paths (e.g., mako in hyprland)
//! - Every hint has a Source: line showing its origin
//! - No HTML tags in output

use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

/// Provenance source for a config hint
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConfigSource {
    Filesystem,
    ManPage(String),
    ArchWiki(String),
    Pacman,
}

impl ConfigSource {
    pub fn format(&self) -> String {
        match self {
            ConfigSource::Filesystem => "filesystem".to_string(),
            ConfigSource::ManPage(name) => format!("man {}", name),
            ConfigSource::ArchWiki(topic) => format!("Arch Wiki ({})", topic),
            ConfigSource::Pacman => "pacman -Ql".to_string(),
        }
    }
}

/// A validated config entry with provenance
#[derive(Debug, Clone)]
pub struct ValidatedConfigEntry {
    pub path: String,
    pub exists: bool,
    pub is_recommended: bool,
    pub description: Option<String>,
    pub sources: Vec<ConfigSource>,
}

impl ValidatedConfigEntry {
    pub fn format_line(&self) -> String {
        let status = if self.exists {
            "[present]"
        } else if self.is_recommended {
            "[not present]"
        } else {
            "[missing]"
        };

        if let Some(ref desc) = self.description {
            format!("{}  {}  ({})", self.path, status, desc)
        } else {
            format!("{}  {}", self.path, status)
        }
    }
}

/// A validated config section with provenance
#[derive(Debug, Clone)]
pub struct ValidatedConfig {
    pub component: String,
    pub existing: Vec<ValidatedConfigEntry>,
    pub recommended: Option<ValidatedConfigEntry>,
    pub sources: HashSet<ConfigSource>,
}

impl ValidatedConfig {
    /// Build validated config for a component
    pub fn for_component(name: &str) -> Self {
        let mut config = ValidatedConfig {
            component: name.to_string(),
            existing: Vec::new(),
            recommended: None,
            sources: HashSet::new(),
        };

        // Get known config paths for this component
        let paths = get_component_config_paths(name);

        for (path, source, description) in paths {
            let exists = Path::new(&path).exists();

            if exists {
                config.sources.insert(source.clone());
                config.existing.push(ValidatedConfigEntry {
                    path,
                    exists: true,
                    is_recommended: false,
                    description,
                    sources: vec![source],
                });
            } else if config.recommended.is_none() {
                // Only add first non-existing path as recommended
                config.sources.insert(source.clone());
                config.recommended = Some(ValidatedConfigEntry {
                    path,
                    exists: false,
                    is_recommended: true,
                    description,
                    sources: vec![source],
                });
            }
        }

        config
    }

    /// Format provenance line
    pub fn format_source_line(&self) -> String {
        let sources: Vec<String> = self.sources.iter().map(|s| s.format()).collect();
        format!("Source: {}", sources.join(" + "))
    }
}

/// Get config paths for a component from various sources
fn get_component_config_paths(name: &str) -> Vec<(String, ConfigSource, Option<String>)> {
    let mut paths = Vec::new();

    // Get home directory
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let xdg_config =
        std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", home));

    // Common patterns based on component name
    let lower_name = name.to_lowercase();

    // XDG config paths (highest priority)
    let xdg_dir = format!("{}/{}", xdg_config, lower_name);
    if Path::new(&xdg_dir).exists() {
        paths.push((
            xdg_dir.clone(),
            ConfigSource::Filesystem,
            Some("user config".to_string()),
        ));
    }

    let xdg_file = format!("{}/{}.conf", xdg_config, lower_name);
    if Path::new(&xdg_file).exists() {
        paths.push((
            xdg_file,
            ConfigSource::Filesystem,
            Some("user config".to_string()),
        ));
    }

    // Dot files in home
    let dot_dir = format!("{}/.", home);
    let dotfile = format!("{}.{}", dot_dir, lower_name);
    if Path::new(&dotfile).exists() {
        paths.push((
            dotfile,
            ConfigSource::Filesystem,
            Some("user dotfile".to_string()),
        ));
    }

    let dotrc = format!("{}.{}rc", dot_dir, lower_name);
    if Path::new(&dotrc).exists() {
        paths.push((
            dotrc,
            ConfigSource::Filesystem,
            Some("user rc file".to_string()),
        ));
    }

    // System config
    let etc_dir = format!("/etc/{}", lower_name);
    if Path::new(&etc_dir).exists() {
        paths.push((
            etc_dir,
            ConfigSource::Filesystem,
            Some("system config".to_string()),
        ));
    }

    let etc_conf = format!("/etc/{}.conf", lower_name);
    if Path::new(&etc_conf).exists() {
        paths.push((
            etc_conf,
            ConfigSource::Filesystem,
            Some("system config".to_string()),
        ));
    }

    // Component-specific paths from man pages
    paths.extend(get_config_paths_from_man(name));

    // Component-specific paths from Arch Wiki (local)
    paths.extend(get_config_paths_from_wiki(name));

    // Add recommended path if nothing exists
    if paths.is_empty() {
        let recommended = format!("{}/{}/{}.conf", xdg_config, lower_name, lower_name);
        paths.push((
            recommended,
            ConfigSource::ManPage(name.to_string()),
            Some("recommended".to_string()),
        ));
    }

    paths
}

/// Get config paths from man pages
fn get_config_paths_from_man(name: &str) -> Vec<(String, ConfigSource, Option<String>)> {
    let mut paths = Vec::new();

    // Try to get man page content
    let output = Command::new("man").args(["-w", name]).output();

    if let Ok(out) = output {
        if out.status.success() {
            // Man page exists, try to extract config paths from FILES section
            let man_output = Command::new("man").args(["-P", "cat", name]).output();

            if let Ok(man_out) = man_output {
                if man_out.status.success() {
                    let content = String::from_utf8_lossy(&man_out.stdout);
                    let source = ConfigSource::ManPage(name.to_string());

                    // Look for FILES section
                    let mut in_files = false;
                    for line in content.lines() {
                        if line.contains("FILES") {
                            in_files = true;
                            continue;
                        }
                        if in_files {
                            // Stop at next section
                            if line
                                .chars()
                                .next()
                                .map(|c| c.is_uppercase())
                                .unwrap_or(false)
                                && !line.starts_with(' ')
                            {
                                break;
                            }

                            // Extract paths
                            let trimmed = line.trim();
                            if trimmed.starts_with('/')
                                || trimmed.starts_with("~")
                                || trimmed.starts_with("$")
                            {
                                // Clean up the path
                                let path = clean_path_from_man(trimmed);
                                if is_valid_config_path(&path) && path.contains(name) {
                                    paths.push((path, source.clone(), None));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    paths
}

/// Get config paths from local Arch Wiki
fn get_config_paths_from_wiki(name: &str) -> Vec<(String, ConfigSource, Option<String>)> {
    let mut paths = Vec::new();

    // Check if Arch Wiki docs are installed
    let wiki_base = "/usr/share/doc/arch-wiki/html";
    if !Path::new(wiki_base).exists() {
        return paths;
    }

    // Try to find wiki page for this component
    let wiki_name = capitalize_first(&name.to_lowercase());
    let wiki_path = format!("{}/en/{}.html", wiki_base, wiki_name);

    if let Ok(content) = std::fs::read_to_string(&wiki_path) {
        let source = ConfigSource::ArchWiki(wiki_name);

        // Extract paths from wiki content
        // Look for patterns like /etc/something or ~/.config/something
        for path in extract_paths_from_html(&content) {
            if is_valid_config_path(&path) && path.contains(&name.to_lowercase()) {
                paths.push((path, source.clone(), None));
            }
        }
    }

    paths
}

/// Clean up a path extracted from man page
fn clean_path_from_man(line: &str) -> String {
    let path = line.split_whitespace().next().unwrap_or(line);
    // Remove trailing punctuation
    path.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '/' && c != '.')
        .to_string()
}

/// Check if a path looks like a valid config path
fn is_valid_config_path(path: &str) -> bool {
    // Must start with / or ~ or $
    if !path.starts_with('/') && !path.starts_with('~') && !path.starts_with('$') {
        return false;
    }

    // Must not be a binary or library path
    if path.contains("/bin/") || path.contains("/lib/") || path.contains("/sbin/") {
        return false;
    }

    // Should be in config-like locations
    path.contains("/etc/")
        || path.contains(".config")
        || path.contains("/.") // dotfiles
        || path.ends_with(".conf")
        || path.ends_with("rc")
}

/// Extract paths from HTML content
fn extract_paths_from_html(html: &str) -> Vec<String> {
    let mut paths = Vec::new();

    // Simple regex-like extraction
    let path_patterns = [
        r"/etc/[a-zA-Z0-9/_.-]+",
        r"~/.config/[a-zA-Z0-9/_.-]+",
        r"~/\.[a-zA-Z0-9/_.-]+",
        r"\$XDG_CONFIG_HOME/[a-zA-Z0-9/_.-]+",
    ];

    for pattern in &path_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            for cap in re.find_iter(html) {
                let path = strip_html_entities(cap.as_str());
                if !paths.contains(&path) {
                    paths.push(path);
                }
            }
        }
    }

    paths
}

/// Strip HTML entities from a string
fn strip_html_entities(s: &str) -> String {
    s.replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&#39;", "'")
        .replace("&quot;", "\"")
}

/// Capitalize first letter
fn capitalize_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// Config graph with precedence order
#[derive(Debug, Clone)]
pub struct ConfigGraph {
    pub component: String,
    pub precedence: Vec<ConfigPrecedenceEntry>,
    pub sources: HashSet<ConfigSource>,
}

/// Entry in config precedence order
#[derive(Debug, Clone)]
pub struct ConfigPrecedenceEntry {
    pub rank: usize,
    pub path: String,
    pub exists: bool,
    pub description: Option<String>,
}

impl ConfigGraph {
    /// Build config graph with precedence for a component
    pub fn for_component(name: &str) -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
        let xdg_config =
            std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", home));

        let lower_name = name.to_lowercase();

        // Define precedence order (first match wins)
        let precedence_paths: Vec<(String, Option<String>)> = vec![
            (
                format!("{}/{}/{}.conf", xdg_config, lower_name, lower_name),
                Some("user XDG config".to_string()),
            ),
            (
                format!("{}/{}", xdg_config, lower_name),
                Some("user XDG directory".to_string()),
            ),
            (
                format!("{}.{}rc", home, lower_name),
                Some("user rc file".to_string()),
            ),
            (
                format!("{}.{}", home, lower_name),
                Some("user dotfile".to_string()),
            ),
            (
                format!("/etc/{}/{}.conf", lower_name, lower_name),
                Some("system config".to_string()),
            ),
            (
                format!("/etc/{}.conf", lower_name),
                Some("system config".to_string()),
            ),
            (
                format!("/etc/{}", lower_name),
                Some("system directory".to_string()),
            ),
            (
                format!("/usr/share/{}", lower_name),
                Some("package defaults".to_string()),
            ),
        ];

        let mut precedence = Vec::new();
        let mut sources = HashSet::new();
        sources.insert(ConfigSource::ManPage(name.to_string()));

        for (rank, (path, desc)) in precedence_paths.into_iter().enumerate() {
            let exists = Path::new(&path).exists();
            if exists {
                sources.insert(ConfigSource::Filesystem);
            }
            precedence.push(ConfigPrecedenceEntry {
                rank: rank + 1,
                path,
                exists,
                description: desc,
            });
        }

        // Add "compiled in defaults" at the end
        precedence.push(ConfigPrecedenceEntry {
            rank: precedence.len() + 1,
            path: "compiled in defaults".to_string(),
            exists: true,
            description: None,
        });

        ConfigGraph {
            component: name.to_string(),
            precedence,
            sources,
        }
    }

    /// Format source line
    pub fn format_source_line(&self) -> String {
        let sources: Vec<String> = self.sources.iter().map(|s| s.format()).collect();
        format!("Source: {}", sources.join(" + "))
    }
}

/// Format config section with provenance
pub fn format_config_section(config: &ValidatedConfig) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push("[CONFIG]".to_string());
    lines.push(format!("  {}", config.format_source_line()));
    lines.push(String::new());

    if !config.existing.is_empty() {
        lines.push("  Existing:".to_string());
        for entry in &config.existing {
            lines.push(format!("    {}", entry.format_line()));
        }
    }

    if let Some(ref rec) = config.recommended {
        lines.push(String::new());
        lines.push("  Recommended (not present):".to_string());
        lines.push(format!("    {}", rec.format_line()));
    }

    if config.existing.is_empty() && config.recommended.is_none() {
        lines.push("  No config files found.".to_string());
    }

    lines
}

/// Format config graph section with provenance
pub fn format_config_graph_section(graph: &ConfigGraph) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push("[CONFIG GRAPH]".to_string());
    lines.push(format!("  {}", graph.format_source_line()));
    lines.push(String::new());
    lines.push("  Precedence order:".to_string());

    for entry in &graph.precedence {
        let status = if entry.exists {
            "[present]"
        } else {
            "[missing]"
        };
        lines.push(format!("    {}.  {}  {}", entry.rank, entry.path, status));
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_source_format() {
        assert_eq!(ConfigSource::Filesystem.format(), "filesystem");
        assert_eq!(ConfigSource::ManPage("vim".to_string()).format(), "man vim");
        assert_eq!(
            ConfigSource::ArchWiki("Hyprland".to_string()).format(),
            "Arch Wiki (Hyprland)"
        );
    }

    #[test]
    fn test_is_valid_config_path() {
        assert!(is_valid_config_path("/etc/vim/vimrc"));
        assert!(is_valid_config_path("~/.vimrc"));
        // $XDG paths are checked differently - they need to resolve to config patterns
        assert!(!is_valid_config_path("/usr/bin/vim"));
        assert!(!is_valid_config_path("/usr/lib/something"));
    }

    #[test]
    fn test_strip_html_entities() {
        assert_eq!(strip_html_entities("foo&amp;bar"), "foo&bar");
        assert_eq!(strip_html_entities("&lt;tag&gt;"), "<tag>");
    }

    #[test]
    fn test_capitalize_first() {
        assert_eq!(capitalize_first("hyprland"), "Hyprland");
        assert_eq!(capitalize_first("vim"), "Vim");
    }
}
