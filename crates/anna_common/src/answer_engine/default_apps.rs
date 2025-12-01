//! Default Apps & MIME Type Module - v0.24.0
//!
//! Discovers default applications and MIME type associations.
//! All mappings are probe-derived from XDG and system configs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// ============================================================================
// MIME Type System
// ============================================================================

/// MIME type categories (for grouping)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MimeCategory {
    Text,
    Image,
    Audio,
    Video,
    Application,
    Font,
    Message,
    Model,
    Multipart,
    Chemical,
    XContent,
    Unknown,
}

impl MimeCategory {
    /// Parse category from MIME type string
    pub fn from_mime(mime: &str) -> Self {
        let category = mime.split('/').next().unwrap_or("");
        match category {
            "text" => Self::Text,
            "image" => Self::Image,
            "audio" => Self::Audio,
            "video" => Self::Video,
            "application" => Self::Application,
            "font" => Self::Font,
            "message" => Self::Message,
            "model" => Self::Model,
            "multipart" => Self::Multipart,
            "chemical" => Self::Chemical,
            "x-content" => Self::XContent,
            _ => Self::Unknown,
        }
    }
}

/// A MIME type with its details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MimeType {
    /// Full MIME type string (e.g., "text/plain")
    pub mime: String,
    /// Category
    pub category: MimeCategory,
    /// File extensions that map to this MIME type
    pub extensions: Vec<String>,
    /// Human-readable description (from shared-mime-info)
    pub description: Option<String>,
    /// Icon name (from freedesktop)
    pub icon_name: Option<String>,
}

impl MimeType {
    /// Create a new MIME type
    pub fn new(mime: &str) -> Self {
        Self {
            mime: mime.to_string(),
            category: MimeCategory::from_mime(mime),
            extensions: Vec::new(),
            description: None,
            icon_name: None,
        }
    }

    /// Add an extension
    pub fn with_extension(mut self, ext: &str) -> Self {
        self.extensions.push(ext.to_string());
        self
    }
}

// ============================================================================
// Default Application Discovery
// ============================================================================

/// A default application mapping (probe-derived)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultApp {
    /// The MIME type this handles
    pub mime_type: String,
    /// Desktop file ID (e.g., "firefox.desktop")
    pub desktop_id: String,
    /// Full path to desktop file
    pub desktop_path: Option<PathBuf>,
    /// Application name (from desktop file)
    pub app_name: Option<String>,
    /// Exec command (from desktop file)
    pub exec_command: Option<String>,
    /// Source of this association
    pub source: DefaultAppSource,
    /// Is this a fallback (not explicitly set)?
    pub is_fallback: bool,
}

/// Where the default app association came from
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DefaultAppSource {
    /// User's mimeapps.list
    UserMimeapps,
    /// System mimeapps.list
    SystemMimeapps,
    /// Desktop environment specific
    DesktopEnvironment(String),
    /// Discovered from desktop file associations
    DesktopFile,
    /// XDG default (xdg-mime query)
    XdgDefault,
    /// Fallback from MIME type database
    MimeDbFallback,
}

/// Common application roles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AppRole {
    WebBrowser,
    EmailClient,
    FileManager,
    TextEditor,
    Terminal,
    ImageViewer,
    VideoPlayer,
    AudioPlayer,
    PdfViewer,
    ArchiveManager,
    Calculator,
    Calendar,
    Screenshot,
    ScreenRecorder,
}

impl AppRole {
    /// Get the MIME types associated with this role
    pub fn associated_mimes(&self) -> Vec<&'static str> {
        match self {
            Self::WebBrowser => vec![
                "x-scheme-handler/http",
                "x-scheme-handler/https",
                "text/html",
            ],
            Self::EmailClient => vec![
                "x-scheme-handler/mailto",
                "message/rfc822",
            ],
            Self::FileManager => vec![
                "inode/directory",
                "x-scheme-handler/file",
            ],
            Self::TextEditor => vec![
                "text/plain",
                "text/x-csrc",
                "text/x-python",
            ],
            Self::Terminal => vec![
                "x-scheme-handler/terminal",
            ],
            Self::ImageViewer => vec![
                "image/png",
                "image/jpeg",
                "image/gif",
                "image/webp",
            ],
            Self::VideoPlayer => vec![
                "video/mp4",
                "video/x-matroska",
                "video/webm",
            ],
            Self::AudioPlayer => vec![
                "audio/mpeg",
                "audio/ogg",
                "audio/flac",
            ],
            Self::PdfViewer => vec![
                "application/pdf",
            ],
            Self::ArchiveManager => vec![
                "application/zip",
                "application/x-tar",
                "application/gzip",
            ],
            Self::Calculator | Self::Calendar | Self::Screenshot | Self::ScreenRecorder => vec![],
        }
    }
}

// ============================================================================
// XDG Paths
// ============================================================================

/// XDG base directories for app discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XdgPaths {
    /// User config dir (~/.config)
    pub config_home: PathBuf,
    /// User data dir (~/.local/share)
    pub data_home: PathBuf,
    /// System data dirs (/usr/share, /usr/local/share)
    pub data_dirs: Vec<PathBuf>,
    /// User's mimeapps.list path
    pub user_mimeapps: PathBuf,
    /// User's applications dir
    pub user_applications: PathBuf,
}

impl XdgPaths {
    /// Construct XDG paths from environment (probe-derived)
    pub fn from_env(env_vars: &HashMap<String, String>) -> Self {
        let home = env_vars
            .get("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/home/user"));

        let config_home = env_vars
            .get("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".config"));

        let data_home = env_vars
            .get("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local/share"));

        let data_dirs = env_vars
            .get("XDG_DATA_DIRS")
            .map(|s| s.split(':').map(PathBuf::from).collect())
            .unwrap_or_else(|| {
                vec![
                    PathBuf::from("/usr/local/share"),
                    PathBuf::from("/usr/share"),
                ]
            });

        Self {
            user_mimeapps: config_home.join("mimeapps.list"),
            user_applications: data_home.join("applications"),
            config_home,
            data_home,
            data_dirs,
        }
    }
}

// ============================================================================
// Desktop File Parsing
// ============================================================================

/// Parsed desktop file entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopEntry {
    /// Desktop file ID (filename without .desktop)
    pub id: String,
    /// Full path to desktop file
    pub path: PathBuf,
    /// Application name
    pub name: String,
    /// Localized name (if available)
    pub name_localized: Option<String>,
    /// Exec command
    pub exec: String,
    /// Icon name
    pub icon: Option<String>,
    /// Categories
    pub categories: Vec<String>,
    /// MIME types this app handles
    pub mime_types: Vec<String>,
    /// Is this a terminal app?
    pub terminal: bool,
    /// Is this app hidden?
    pub no_display: bool,
    /// Generic name
    pub generic_name: Option<String>,
    /// Comment/description
    pub comment: Option<String>,
}

/// Result of parsing a desktop file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DesktopParseResult {
    Success(Box<DesktopEntry>),
    NotFound,
    ParseError(String),
    InvalidEntry(String),
}

// ============================================================================
// Default Apps Registry
// ============================================================================

/// Complete registry of default apps (all probe-derived)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DefaultAppsRegistry {
    /// MIME type to default app mapping
    pub mime_defaults: HashMap<String, DefaultApp>,
    /// App role to desktop ID mapping
    pub role_defaults: HashMap<AppRole, String>,
    /// All discovered desktop entries
    pub desktop_entries: HashMap<String, DesktopEntry>,
    /// XDG paths used for discovery
    pub xdg_paths: Option<XdgPaths>,
    /// When this registry was built
    pub built_at: i64,
    /// Source files scanned
    pub scanned_files: Vec<PathBuf>,
}

impl DefaultAppsRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a default app mapping
    pub fn set_default(&mut self, mime_type: &str, app: DefaultApp) {
        self.mime_defaults.insert(mime_type.to_string(), app);
    }

    /// Get the default app for a MIME type
    pub fn get_default(&self, mime_type: &str) -> Option<&DefaultApp> {
        self.mime_defaults.get(mime_type)
    }

    /// Get the default app for a role
    pub fn get_role_default(&self, role: &AppRole) -> Option<&DefaultApp> {
        self.role_defaults
            .get(role)
            .and_then(|id| {
                // Find the app by checking each MIME type the role handles
                role.associated_mimes()
                    .iter()
                    .filter_map(|mime| self.mime_defaults.get(*mime))
                    .find(|app| &app.desktop_id == id)
            })
    }
}

// ============================================================================
// Probe Definitions
// ============================================================================

/// Probe for discovering default apps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultAppProbe {
    /// Probe ID
    pub id: String,
    /// What this probe discovers
    pub target: DefaultAppProbeTarget,
    /// Command to run
    pub command: String,
    /// Fact key prefix for results
    pub fact_key_prefix: String,
}

/// What the probe targets
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DefaultAppProbeTarget {
    /// Query XDG default for a MIME type
    XdgQuery(String),
    /// Parse mimeapps.list file
    MimeappsList,
    /// Scan desktop files
    DesktopFiles,
    /// Query environment for XDG paths
    XdgPaths,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mime_category_parsing() {
        assert_eq!(MimeCategory::from_mime("text/plain"), MimeCategory::Text);
        assert_eq!(MimeCategory::from_mime("image/png"), MimeCategory::Image);
        assert_eq!(MimeCategory::from_mime("video/mp4"), MimeCategory::Video);
        assert_eq!(MimeCategory::from_mime("audio/mpeg"), MimeCategory::Audio);
        assert_eq!(
            MimeCategory::from_mime("application/pdf"),
            MimeCategory::Application
        );
        assert_eq!(MimeCategory::from_mime("unknown"), MimeCategory::Unknown);
    }

    #[test]
    fn test_mime_type_creation() {
        let mime = MimeType::new("text/plain")
            .with_extension("txt")
            .with_extension("text");
        assert_eq!(mime.mime, "text/plain");
        assert_eq!(mime.category, MimeCategory::Text);
        assert_eq!(mime.extensions, vec!["txt", "text"]);
    }

    #[test]
    fn test_app_role_mimes() {
        let browser_mimes = AppRole::WebBrowser.associated_mimes();
        assert!(browser_mimes.contains(&"x-scheme-handler/http"));
        assert!(browser_mimes.contains(&"text/html"));

        let email_mimes = AppRole::EmailClient.associated_mimes();
        assert!(email_mimes.contains(&"x-scheme-handler/mailto"));
    }

    #[test]
    fn test_xdg_paths_from_env() {
        let mut env = HashMap::new();
        env.insert("HOME".to_string(), "/home/testuser".to_string());

        let paths = XdgPaths::from_env(&env);
        assert_eq!(paths.config_home, PathBuf::from("/home/testuser/.config"));
        assert_eq!(
            paths.data_home,
            PathBuf::from("/home/testuser/.local/share")
        );
        assert_eq!(
            paths.user_mimeapps,
            PathBuf::from("/home/testuser/.config/mimeapps.list")
        );
    }

    #[test]
    fn test_xdg_paths_custom() {
        let mut env = HashMap::new();
        env.insert("HOME".to_string(), "/home/testuser".to_string());
        env.insert("XDG_CONFIG_HOME".to_string(), "/custom/config".to_string());
        env.insert("XDG_DATA_HOME".to_string(), "/custom/data".to_string());

        let paths = XdgPaths::from_env(&env);
        assert_eq!(paths.config_home, PathBuf::from("/custom/config"));
        assert_eq!(paths.data_home, PathBuf::from("/custom/data"));
    }

    #[test]
    fn test_default_apps_registry() {
        let mut registry = DefaultAppsRegistry::new();

        let app = DefaultApp {
            mime_type: "text/html".to_string(),
            desktop_id: "firefox.desktop".to_string(),
            desktop_path: Some(PathBuf::from("/usr/share/applications/firefox.desktop")),
            app_name: Some("Firefox".to_string()),
            exec_command: Some("firefox %u".to_string()),
            source: DefaultAppSource::UserMimeapps,
            is_fallback: false,
        };

        registry.set_default("text/html", app);
        let retrieved = registry.get_default("text/html");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().desktop_id, "firefox.desktop");
    }

    #[test]
    fn test_desktop_entry_serialize() {
        let entry = DesktopEntry {
            id: "firefox".to_string(),
            path: PathBuf::from("/usr/share/applications/firefox.desktop"),
            name: "Firefox".to_string(),
            name_localized: None,
            exec: "firefox %u".to_string(),
            icon: Some("firefox".to_string()),
            categories: vec!["Network".to_string(), "WebBrowser".to_string()],
            mime_types: vec!["text/html".to_string()],
            terminal: false,
            no_display: false,
            generic_name: Some("Web Browser".to_string()),
            comment: Some("Browse the web".to_string()),
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("firefox"));

        let parsed: DesktopEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Firefox");
    }
}
