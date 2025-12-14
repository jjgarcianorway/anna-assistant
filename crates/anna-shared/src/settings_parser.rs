// v0.0.646: Settings Parser (Phase 222)
// Parser for parsing settings from various formats

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Parse source type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ParseSource {
    /// JSON source
    #[default]
    Json,
    /// TOML source
    Toml,
    /// YAML source
    Yaml,
    /// INI source
    Ini,
    /// Environment source
    Env,
}

impl std::fmt::Display for ParseSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Toml => write!(f, "toml"),
            Self::Yaml => write!(f, "yaml"),
            Self::Ini => write!(f, "ini"),
            Self::Env => write!(f, "env"),
        }
    }
}

/// Parse mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ParseMode {
    /// Strict mode
    #[default]
    Strict,
    /// Lenient mode
    Lenient,
    /// Permissive mode
    Permissive,
    /// Validate only
    ValidateOnly,
}

impl std::fmt::Display for ParseMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Strict => write!(f, "strict"),
            Self::Lenient => write!(f, "lenient"),
            Self::Permissive => write!(f, "permissive"),
            Self::ValidateOnly => write!(f, "validate_only"),
        }
    }
}

/// Parser config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserConfig {
    /// Parse source
    pub source: ParseSource,
    /// Parse mode
    pub mode: ParseMode,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Allow unknown keys
    pub allow_unknown: bool,
    /// Collect errors
    pub collect_errors: bool,
}

impl ParserConfig {
    /// Create new config
    pub fn new(source: ParseSource) -> Self {
        Self {
            source,
            mode: ParseMode::Strict,
            category: None,
            allow_unknown: false,
            collect_errors: true,
        }
    }

    /// Set mode
    pub fn mode(mut self, mode: ParseMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set allow unknown
    pub fn allow_unknown(mut self, allow: bool) -> Self {
        self.allow_unknown = allow;
        self
    }

    /// Set collect errors
    pub fn collect_errors(mut self, collect: bool) -> Self {
        self.collect_errors = collect;
        self
    }
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self::new(ParseSource::Json)
    }
}

/// Parse error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseError {
    /// Error message
    pub message: String,
    /// Line number
    pub line: Option<usize>,
    /// Column number
    pub column: Option<usize>,
    /// Key path
    pub path: Option<String>,
}

impl ParseError {
    /// Create new error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            line: None,
            column: None,
            path: None,
        }
    }

    /// Set location
    pub fn at(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    /// Set path
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
}

/// Parse result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    /// Was successful
    pub success: bool,
    /// Parsed values
    pub values: HashMap<String, String>,
    /// Errors
    pub errors: Vec<ParseError>,
    /// Source type
    pub source: ParseSource,
}

impl ParseResult {
    /// Create success result
    pub fn success(values: HashMap<String, String>, source: ParseSource) -> Self {
        Self {
            success: true,
            values,
            errors: Vec::new(),
            source,
        }
    }

    /// Create failure result
    pub fn failure(errors: Vec<ParseError>, source: ParseSource) -> Self {
        Self {
            success: false,
            values: HashMap::new(),
            errors,
            source,
        }
    }

    /// Value count
    pub fn value_count(&self) -> usize {
        self.values.len()
    }

    /// Error count
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }
}

/// Parser stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParserStats {
    /// Total parses
    pub total_parses: usize,
    /// Successful parses
    pub successful: usize,
    /// Failed parses
    pub failed: usize,
    /// By source
    pub by_source: HashMap<String, usize>,
    /// Total values parsed
    pub total_values: usize,
}

impl ParserStats {
    /// Record parse
    pub fn record(&mut self, source: ParseSource, success: bool, value_count: usize) {
        self.total_parses += 1;
        if success {
            self.successful += 1;
            self.total_values += value_count;
        } else {
            self.failed += 1;
        }
        *self.by_source.entry(source.to_string()).or_insert(0) += 1;
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_parses == 0 {
            0.0
        } else {
            self.successful as f64 / self.total_parses as f64
        }
    }
}

/// Settings parser
#[derive(Debug, Clone, Default)]
pub struct SettingsParser {
    /// Config
    config: ParserConfig,
    /// Results
    results: Vec<ParseResult>,
    /// Stats
    stats: ParserStats,
}

impl SettingsParser {
    /// Create new parser
    pub fn new(config: ParserConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            stats: ParserStats::default(),
        }
    }

    /// Parse input
    pub fn parse(&mut self, input: &str) -> ParseResult {
        let result = self.do_parse(input);
        self.stats.record(
            self.config.source,
            result.success,
            result.value_count(),
        );
        self.results.push(result.clone());
        result
    }

    /// Do parse
    fn do_parse(&self, input: &str) -> ParseResult {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return ParseResult::failure(
                vec![ParseError::new("Empty input")],
                self.config.source,
            );
        }

        // Simple key=value parsing
        let mut values = HashMap::new();
        let mut errors = Vec::new();

        for (line_num, line) in trimmed.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim().to_string();
                let value = line[eq_pos + 1..].trim().to_string();
                values.insert(key, value);
            } else if self.config.mode == ParseMode::Strict {
                errors.push(
                    ParseError::new("Invalid line format")
                        .at(line_num + 1, 1)
                );
            }
        }

        if !errors.is_empty() && self.config.mode == ParseMode::Strict {
            ParseResult::failure(errors, self.config.source)
        } else {
            ParseResult::success(values, self.config.source)
        }
    }

    /// Get results
    pub fn results(&self) -> &[ParseResult] {
        &self.results
    }

    /// Get stats
    pub fn stats(&self) -> &ParserStats {
        &self.stats
    }

    /// Result count
    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    /// Clear results
    pub fn clear(&mut self) {
        self.results.clear();
    }
}

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

/// Check if query is about parser
pub fn is_parser_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("parser") || lower.contains("parse settings") || lower.contains("parse config")
}

/// Fun fact about parser
pub fn parser_fun_fact() -> &'static str {
    "Anna's settings parsers read configs from any format!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_source_display() {
        assert_eq!(format!("{}", ParseSource::Json), "json");
        assert_eq!(format!("{}", ParseSource::Toml), "toml");
    }

    #[test]
    fn test_parse_mode_display() {
        assert_eq!(format!("{}", ParseMode::Strict), "strict");
        assert_eq!(format!("{}", ParseMode::Lenient), "lenient");
    }

    #[test]
    fn test_config_new() {
        let c = ParserConfig::new(ParseSource::Json);
        assert_eq!(c.mode, ParseMode::Strict);
    }

    #[test]
    fn test_config_builder() {
        let c = ParserConfig::new(ParseSource::Toml)
            .mode(ParseMode::Lenient)
            .allow_unknown(true);
        assert_eq!(c.mode, ParseMode::Lenient);
        assert!(c.allow_unknown);
    }

    #[test]
    fn test_error_new() {
        let e = ParseError::new("test error").at(1, 5);
        assert_eq!(e.line, Some(1));
        assert_eq!(e.column, Some(5));
    }

    #[test]
    fn test_result_success() {
        let mut values = HashMap::new();
        values.insert("key".to_string(), "value".to_string());
        let r = ParseResult::success(values, ParseSource::Json);
        assert!(r.success);
        assert_eq!(r.value_count(), 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = ParserStats::default();
        s.record(ParseSource::Json, true, 5);
        s.record(ParseSource::Json, false, 0);
        assert_eq!(s.total_parses, 2);
        assert_eq!(s.successful, 1);
    }

    #[test]
    fn test_parser_new() {
        let p = SettingsParser::new(ParserConfig::new(ParseSource::Json));
        assert_eq!(p.result_count(), 0);
    }

    #[test]
    fn test_parser_parse_simple() {
        let mut p = SettingsParser::new(ParserConfig::new(ParseSource::Ini));
        let r = p.parse("key=value");
        assert!(r.success);
        assert_eq!(r.values.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_parser_parse_multiline() {
        let mut p = SettingsParser::new(ParserConfig::new(ParseSource::Ini));
        let r = p.parse("key1=value1\nkey2=value2");
        assert!(r.success);
        assert_eq!(r.value_count(), 2);
    }

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

    #[test]
    fn test_is_parser_query() {
        assert!(is_parser_query("settings parser"));
        assert!(!is_parser_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = parser_fun_fact();
        assert!(fact.contains("parser"));
    }
}
