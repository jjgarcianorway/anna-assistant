//! Helpers unit tests (v0.0.221).

#[cfg(test)]
mod tests {
    use crate::helpers::{known_helpers, HelperPackage, HelpersRegistry, InstallSource};

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

        registry.register(HelperPackage::new("test", "Test Updated").with_available(true));
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
                .required(),
        );
        registry.register(
            HelperPackage::new("user-installed", "User Installed")
                .with_source(InstallSource::User)
                .with_available(true),
        );
        registry.register(
            HelperPackage::new("missing", "Missing")
                .with_source(InstallSource::Unknown)
                .required(),
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
                .with_source(InstallSource::Anna),
        );

        let json = serde_json::to_string(&registry).unwrap();
        let parsed: HelpersRegistry = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed.get("test").unwrap().version, Some("1.0".to_string()));
    }
}
