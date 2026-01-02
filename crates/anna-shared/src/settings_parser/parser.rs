// v0.0.646: Settings Parser (Phase 222)
// Core parser implementation

use std::collections::HashMap;

use super::config::ParserConfig;
use super::result::{ParseResult, ParserStats};
use super::types::{ParseError, ParseMode};

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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::ParseSource;

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
}
