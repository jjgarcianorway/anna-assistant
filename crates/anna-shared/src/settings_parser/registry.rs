// v0.0.646: Settings Parser Registry (Phase 222)
// Registry for managing multiple parsers

use std::collections::HashMap;

use super::parser::SettingsParser;

/// Settings parser registry
#[derive(Debug, Clone, Default)]
pub struct SettingsParserRegistry {
    /// Parsers by ID
    parsers: HashMap<String, SettingsParser>,
}

impl SettingsParserRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register parser
    pub fn register(&mut self, id: impl Into<String>, parser: SettingsParser) {
        self.parsers.insert(id.into(), parser);
    }

    /// Unregister parser
    pub fn unregister(&mut self, id: &str) -> bool {
        self.parsers.remove(id).is_some()
    }

    /// Get parser
    pub fn get(&self, id: &str) -> Option<&SettingsParser> {
        self.parsers.get(id)
    }

    /// Get parser mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsParser> {
        self.parsers.get_mut(id)
    }

    /// Parser count
    pub fn count(&self) -> usize {
        self.parsers.len()
    }
}

/// Format parser registry
pub fn format_parser_registry(registry: &SettingsParserRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Parser Registry:\n");
    output.push_str(&format!("  Parsers: {}\n", registry.count()));
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::config::ParserConfig;
    use super::super::types::ParseSource;

    #[test]
    fn test_registry_new() {
        let r = SettingsParserRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsParserRegistry::new();
        r.register("parser1", SettingsParser::new(ParserConfig::new(ParseSource::Json)));
        assert_eq!(r.count(), 1);
    }
}
