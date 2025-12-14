// v0.0.604: Settings Compiler (Phase 180)
// Compilation and optimization of settings configurations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Compile stage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompileStage {
    /// Parse input
    Parse,
    /// Validate structure
    Validate,
    /// Optimize
    Optimize,
    /// Generate output
    Generate,
    /// Finalize
    Finalize,
}

impl std::fmt::Display for CompileStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse => write!(f, "parse"),
            Self::Validate => write!(f, "validate"),
            Self::Optimize => write!(f, "optimize"),
            Self::Generate => write!(f, "generate"),
            Self::Finalize => write!(f, "finalize"),
        }
    }
}

/// Compile result status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompileStatus {
    /// Success
    Success,
    /// Warning
    Warning,
    /// Error
    Error,
    /// Skipped
    Skipped,
}

impl std::fmt::Display for CompileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
            Self::Skipped => write!(f, "skipped"),
        }
    }
}

/// Compile diagnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileDiag {
    /// Stage
    pub stage: CompileStage,
    /// Status
    pub status: CompileStatus,
    /// Message
    pub message: String,
    /// Location (key path)
    pub location: Option<String>,
}

impl CompileDiag {
    /// Create new diagnostic
    pub fn new(stage: CompileStage, status: CompileStatus, message: impl Into<String>) -> Self {
        Self {
            stage,
            status,
            message: message.into(),
            location: None,
        }
    }

    /// Set location
    pub fn at(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Is error
    pub fn is_error(&self) -> bool {
        self.status == CompileStatus::Error
    }
}

/// Compile options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileOptions {
    /// Enable optimizations
    pub optimize: bool,
    /// Strict mode
    pub strict: bool,
    /// Include debug info
    pub debug_info: bool,
    /// Target categories
    pub categories: Vec<SettingsCategory>,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            optimize: true,
            strict: false,
            debug_info: false,
            categories: Vec::new(),
        }
    }
}

impl CompileOptions {
    /// Create new options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set optimize
    pub fn optimize(mut self, opt: bool) -> Self {
        self.optimize = opt;
        self
    }

    /// Set strict
    pub fn strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    /// Set debug info
    pub fn debug_info(mut self, debug: bool) -> Self {
        self.debug_info = debug;
        self
    }

    /// Add category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.categories.push(category);
        self
    }
}

/// Compile output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileOutput {
    /// Status
    pub status: CompileStatus,
    /// Diagnostics
    pub diagnostics: Vec<CompileDiag>,
    /// Compiled data
    pub data: HashMap<String, String>,
    /// Duration ms
    pub duration_ms: u64,
}

impl CompileOutput {
    /// Create success output
    pub fn success() -> Self {
        Self {
            status: CompileStatus::Success,
            diagnostics: Vec::new(),
            data: HashMap::new(),
            duration_ms: 0,
        }
    }

    /// Create error output
    pub fn error(diag: CompileDiag) -> Self {
        Self {
            status: CompileStatus::Error,
            diagnostics: vec![diag],
            data: HashMap::new(),
            duration_ms: 0,
        }
    }

    /// Add diagnostic
    pub fn add_diag(&mut self, diag: CompileDiag) {
        if diag.is_error() {
            self.status = CompileStatus::Error;
        } else if diag.status == CompileStatus::Warning && self.status == CompileStatus::Success {
            self.status = CompileStatus::Warning;
        }
        self.diagnostics.push(diag);
    }

    /// Add data
    pub fn add_data(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.data.insert(key.into(), value.into());
    }

    /// Has errors
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.is_error())
    }

    /// Error count
    pub fn error_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.is_error()).count()
    }
}

/// Settings compiler
#[derive(Debug, Clone, Default)]
pub struct SettingsCompiler {
    /// Options
    options: CompileOptions,
    /// Compile history
    history: Vec<CompileOutput>,
    /// Max history
    max_history: usize,
}

impl SettingsCompiler {
    /// Create new compiler
    pub fn new() -> Self {
        Self {
            max_history: 50,
            ..Default::default()
        }
    }

    /// With options
    pub fn with_options(options: CompileOptions) -> Self {
        Self {
            options,
            max_history: 50,
            ..Default::default()
        }
    }

    /// Get options
    pub fn options(&self) -> &CompileOptions {
        &self.options
    }

    /// Set options
    pub fn set_options(&mut self, options: CompileOptions) {
        self.options = options;
    }

    /// Record compile
    pub fn record(&mut self, output: CompileOutput) {
        self.history.push(output);
        while self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// Get history
    pub fn history(&self) -> &[CompileOutput] {
        &self.history
    }

    /// Recent compiles
    pub fn recent(&self, count: usize) -> Vec<&CompileOutput> {
        self.history.iter().rev().take(count).collect()
    }

    /// History count
    pub fn history_count(&self) -> usize {
        self.history.len()
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.history.is_empty() {
            1.0
        } else {
            let success = self.history.iter().filter(|o| o.status == CompileStatus::Success).count();
            success as f64 / self.history.len() as f64
        }
    }
}

/// Format compiler
pub fn format_compiler(compiler: &SettingsCompiler) -> String {
    let mut output = String::new();
    output.push_str("Settings Compiler:\n");
    output.push_str(&format!("  Optimize: {}\n", compiler.options.optimize));
    output.push_str(&format!("  Strict: {}\n", compiler.options.strict));
    output.push_str(&format!("  History: {}\n", compiler.history_count()));
    output.push_str(&format!("  Success rate: {:.1}%\n", compiler.success_rate() * 100.0));
    output
}

/// Check if query is about compiler
pub fn is_compiler_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("compile")
        || lower.contains("build settings")
        || lower.contains("optimize settings")
}

/// Fun fact about compiler
pub fn compiler_fun_fact() -> &'static str {
    "Anna compiles and optimizes your settings for maximum performance!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_display() {
        assert_eq!(format!("{}", CompileStage::Parse), "parse");
        assert_eq!(format!("{}", CompileStage::Optimize), "optimize");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", CompileStatus::Success), "success");
        assert_eq!(format!("{}", CompileStatus::Error), "error");
    }

    #[test]
    fn test_diag_new() {
        let d = CompileDiag::new(CompileStage::Parse, CompileStatus::Error, "failed");
        assert!(d.is_error());
    }

    #[test]
    fn test_diag_at() {
        let d = CompileDiag::new(CompileStage::Validate, CompileStatus::Warning, "warn")
            .at("settings.key");
        assert!(d.location.is_some());
    }

    #[test]
    fn test_options_default() {
        let o = CompileOptions::new();
        assert!(o.optimize);
        assert!(!o.strict);
    }

    #[test]
    fn test_options_builder() {
        let o = CompileOptions::new()
            .optimize(false)
            .strict(true)
            .debug_info(true);
        assert!(!o.optimize);
        assert!(o.strict);
    }

    #[test]
    fn test_output_success() {
        let o = CompileOutput::success();
        assert_eq!(o.status, CompileStatus::Success);
        assert!(!o.has_errors());
    }

    #[test]
    fn test_output_error() {
        let d = CompileDiag::new(CompileStage::Parse, CompileStatus::Error, "err");
        let o = CompileOutput::error(d);
        assert!(o.has_errors());
    }

    #[test]
    fn test_compiler_new() {
        let c = SettingsCompiler::new();
        assert_eq!(c.history_count(), 0);
    }

    #[test]
    fn test_compiler_record() {
        let mut c = SettingsCompiler::new();
        c.record(CompileOutput::success());
        assert_eq!(c.history_count(), 1);
    }

    #[test]
    fn test_is_compiler_query() {
        assert!(is_compiler_query("compile settings"));
        assert!(!is_compiler_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = compiler_fun_fact();
        assert!(fact.contains("compile"));
    }
}
