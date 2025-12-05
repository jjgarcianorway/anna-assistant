//! Helper package tracking for Anna dependencies.
//!
//! Tracks external packages that Anna depends on (e.g., ollama).
//! Distinguishes between packages installed by Anna vs. user-installed.
//!
//! v0.0.28: Initial implementation.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A helper package that Anna depends on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelperPackage {
    /// Package identifier (e.g., "ollama")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Package version if known
    pub version: Option<String>,
    /// How the package was installed
    pub install_source: InstallSource,
    /// Whether the package is currently available
    pub available: bool,
    /// Path to the package binary if known
    pub binary_path: Option<PathBuf>,
    /// Whether this is a required dependency
    pub required: bool,
}

/// How a helper package was installed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallSource {
    /// Installed by Anna (auto-install)
    Anna,
    /// Installed by user (system package manager, manual, etc.)
    User,
    /// Bundled with Anna
    Bundled,
    /// Unknown source
    Unknown,
}

impl Default for InstallSource {
    fn default() -> Self {
        Self::Unknown
    }
}

impl std::fmt::Display for InstallSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Anna => write!(f, "anna"),
            Self::User => write!(f, "user"),
            Self::Bundled => write!(f, "bundled"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

impl HelperPackage {
    /// Create a new helper package
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: None,
            install_source: InstallSource::Unknown,
            available: false,
            binary_path: None,
            required: false,
        }
    }

    /// Set the version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set the install source
    pub fn with_source(mut self, source: InstallSource) -> Self {
        self.install_source = source;
        self
    }

    /// Set availability
    pub fn with_available(mut self, available: bool) -> Self {
        self.available = available;
        self
    }

    /// Set binary path
    pub fn with_binary_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.binary_path = Some(path.into());
        self
    }

    /// Mark as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Check if this package was installed by Anna
    pub fn installed_by_anna(&self) -> bool {
        self.install_source == InstallSource::Anna
    }
}

/// Registry of helper packages.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HelpersRegistry {
    /// Tracked packages
    pub packages: Vec<HelperPackage>,
}

impl HelpersRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self { packages: Vec::new() }
    }

    /// Add or update a package
    pub fn register(&mut self, package: HelperPackage) {
        if let Some(existing) = self.packages.iter_mut().find(|p| p.id == package.id) {
            *existing = package;
        } else {
            self.packages.push(package);
        }
    }

    /// Get a package by ID
    pub fn get(&self, id: &str) -> Option<&HelperPackage> {
        self.packages.iter().find(|p| p.id == id)
    }

    /// Get a mutable package by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut HelperPackage> {
        self.packages.iter_mut().find(|p| p.id == id)
    }

    /// Remove a package
    pub fn remove(&mut self, id: &str) -> Option<HelperPackage> {
        if let Some(pos) = self.packages.iter().position(|p| p.id == id) {
            Some(self.packages.remove(pos))
        } else {
            None
        }
    }

    /// Get all packages installed by Anna
    pub fn anna_installed(&self) -> Vec<&HelperPackage> {
        self.packages.iter().filter(|p| p.installed_by_anna()).collect()
    }

    /// Get all required packages
    pub fn required_packages(&self) -> Vec<&HelperPackage> {
        self.packages.iter().filter(|p| p.required).collect()
    }

    /// Get all available packages
    pub fn available_packages(&self) -> Vec<&HelperPackage> {
        self.packages.iter().filter(|p| p.available).collect()
    }

    /// Get all unavailable required packages
    pub fn missing_required(&self) -> Vec<&HelperPackage> {
        self.packages.iter().filter(|p| p.required && !p.available).collect()
    }

    /// Check if all required packages are available
    pub fn all_required_available(&self) -> bool {
        self.packages.iter().filter(|p| p.required).all(|p| p.available)
    }

    /// Get count of packages
    pub fn len(&self) -> usize {
        self.packages.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.packages.is_empty()
    }

    /// Clear all packages (for reset)
    pub fn clear(&mut self) {
        self.packages.clear();
    }
}

/// Known helper packages for Anna.
/// Returns a registry with default package definitions (not detected yet).
pub fn known_helpers() -> HelpersRegistry {
    let mut registry = HelpersRegistry::new();

    // Ollama - LLM backend
    registry.register(
        HelperPackage::new("ollama", "Ollama")
            .required()
    );

    registry
}

/// Detect a helper package on the system.
/// Returns updated package with availability and path info.
pub fn detect_helper(id: &str) -> Option<HelperPackage> {
    match id {
        "ollama" => detect_ollama(),
        _ => None,
    }
}

/// Detect ollama installation.
fn detect_ollama() -> Option<HelperPackage> {
    // Check common paths
    let paths = [
        "/usr/local/bin/ollama",
        "/usr/bin/ollama",
        "/opt/homebrew/bin/ollama", // macOS
    ];

    for path in paths {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(
                HelperPackage::new("ollama", "Ollama")
                    .required()
                    .with_available(true)
                    .with_binary_path(p)
                    .with_source(InstallSource::User) // Assume user if found
            );
        }
    }

    // Check if ollama is in PATH (will be handled by caller via `which ollama`)
    None
}

/// Path to the helpers store file.
pub fn helpers_store_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    PathBuf::from(home).join(".anna").join("helpers.json")
}

/// Load helpers registry from disk.
pub fn load_helpers() -> HelpersRegistry {
    let path = helpers_store_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(registry) = serde_json::from_str(&content) {
                return registry;
            }
        }
    }
    HelpersRegistry::new()
}

/// Save helpers registry to disk.
pub fn save_helpers(registry: &HelpersRegistry) -> std::io::Result<()> {
    let path = helpers_store_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(registry)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(&path, content)
}

/// Clear helpers store (for reset) (v0.0.28)
pub fn clear_helpers_store() -> std::io::Result<()> {
    let path = helpers_store_path();
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helper_package_new() {
        let pkg = HelperPackage::new("test", "Test Package");
        assert_eq!(pkg.id, "test");
        assert_eq!(pkg.name, "Test Package");
        assert!(!pkg.available);
        assert!(!pkg.required);
    }

    #[test]
    fn test_helper_package_builders() {
        let pkg = HelperPackage::new("ollama", "Ollama")
            .with_version("0.1.0")
            .with_source(InstallSource::Anna)
            .with_available(true)
            .with_binary_path("/usr/bin/ollama")
            .required();

        assert_eq!(pkg.version, Some("0.1.0".to_string()));
        assert_eq!(pkg.install_source, InstallSource::Anna);
        assert!(pkg.available);
        assert!(pkg.required);
        assert!(pkg.installed_by_anna());
    }

    #[test]
    fn test_install_source_display() {
        assert_eq!(InstallSource::Anna.to_string(), "anna");
        assert_eq!(InstallSource::User.to_string(), "user");
        assert_eq!(InstallSource::Bundled.to_string(), "bundled");
        assert_eq!(InstallSource::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_registry_operations() {
        let mut registry = HelpersRegistry::new();
        assert!(registry.is_empty());

        let pkg = HelperPackage::new("test", "Test").required();
        registry.register(pkg);

        assert_eq!(registry.len(), 1);
        assert!(registry.get("test").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_registry_update() {
        let mut registry = HelpersRegistry::new();

        registry.register(HelperPackage::new("test", "Test"));
        assert!(!registry.get("test").unwrap().available);

        registry.register(
            HelperPackage::new("test", "Test Updated").with_available(true)
        );
        assert!(registry.get("test").unwrap().available);
        assert_eq!(registry.len(), 1); // Still one package
    }

    #[test]
    fn test_registry_filters() {
        let mut registry = HelpersRegistry::new();

        registry.register(
            HelperPackage::new("anna-installed", "Anna Installed")
                .with_source(InstallSource::Anna)
                .with_available(true)
                .required()
        );
        registry.register(
            HelperPackage::new("user-installed", "User Installed")
                .with_source(InstallSource::User)
                .with_available(true)
        );
        registry.register(
            HelperPackage::new("missing", "Missing")
                .with_source(InstallSource::Unknown)
                .required()
        );

        assert_eq!(registry.anna_installed().len(), 1);
        assert_eq!(registry.required_packages().len(), 2);
        assert_eq!(registry.available_packages().len(), 2);
        assert_eq!(registry.missing_required().len(), 1);
        assert!(!registry.all_required_available());
    }

    #[test]
    fn test_known_helpers() {
        let registry = known_helpers();
        assert!(registry.get("ollama").is_some());
        assert!(registry.get("ollama").unwrap().required);
    }

    #[test]
    fn test_registry_clear() {
        let mut registry = known_helpers();
        assert!(!registry.is_empty());

        registry.clear();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_registry_serialization() {
        let mut registry = HelpersRegistry::new();
        registry.register(
            HelperPackage::new("test", "Test")
                .with_version("1.0")
                .with_source(InstallSource::Anna)
        );

        let json = serde_json::to_string(&registry).unwrap();
        let parsed: HelpersRegistry = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed.get("test").unwrap().version, Some("1.0".to_string()));
    }
}
