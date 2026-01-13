//! Intent types and classification for the Translator.
//!
//! The Translator parses user natural language deterministically into
//! structured intents. No LLM reasoning is exposed to the user.

use serde::{Deserialize, Serialize};

/// Types of actions the user might want
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntentAction {
    /// Query information about the system (read-only)
    Query,
    /// Configure or change system settings
    Configure,
    /// Execute a command or task
    Execute,
    /// Install/remove packages
    Package,
    /// Troubleshoot a problem
    Troubleshoot,
    /// Get help or explanation (how-to)
    Help,
    /// Change Anna's own configuration
    AnnaConfig,
    /// Undo a previous action
    Undo,
    /// Unknown/needs clarification
    Unknown,
}

impl IntentAction {
    /// Whether this action modifies the system
    pub fn modifies_system(&self) -> bool {
        matches!(
            self,
            IntentAction::Configure
                | IntentAction::Execute
                | IntentAction::Package
                | IntentAction::Undo
        )
    }

    /// Whether this action typically needs user confirmation
    pub fn needs_confirmation(&self) -> bool {
        self.modifies_system()
    }
}

/// Subcategory for more precise intent classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntentSubject {
    // Query subjects
    DiskUsage,
    MemoryUsage,
    CpuUsage,
    NetworkStatus,
    ServiceStatus,
    PackageInfo,
    SystemInfo,
    ProcessInfo,

    // Configure subjects
    ServiceControl,
    FileEdit,
    PermissionChange,
    NetworkConfig,

    // Package subjects
    PackageInstall,
    PackageRemove,
    PackageSearch,
    PackageUpdate,

    // Troubleshoot subjects
    ErrorDiagnosis,
    PerformanceIssue,
    ConnectivityIssue,
    BootIssue,

    // Help subjects
    HowTo,
    Explanation,
    ManPage,

    // Generic
    Generic(String),
}

/// User intent extracted from natural language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIntent {
    /// Primary action type
    pub action: IntentAction,

    /// Specific subject/topic
    pub subject: IntentSubject,

    /// Raw subject string for display
    pub subject_raw: String,

    /// Extracted parameters (package names, service names, file paths, etc.)
    pub parameters: Vec<String>,

    /// Original user input (preserved for context)
    pub original_input: String,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,

    /// Classification method used
    pub classification_method: ClassificationMethod,
}

/// How the intent was classified
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClassificationMethod {
    /// Direct pattern match (deterministic, highest confidence)
    PatternMatch,
    /// Keyword-based classification
    KeywordMatch,
    /// Fuzzy matching with lower confidence
    FuzzyMatch,
    /// Could not classify
    Unknown,
}

impl Default for UserIntent {
    fn default() -> Self {
        Self {
            action: IntentAction::Unknown,
            subject: IntentSubject::Generic(String::new()),
            subject_raw: String::new(),
            parameters: vec![],
            original_input: String::new(),
            confidence: 0.0,
            classification_method: ClassificationMethod::Unknown,
        }
    }
}

impl UserIntent {
    /// Create a new intent with pattern match confidence
    pub fn from_pattern(
        action: IntentAction,
        subject: IntentSubject,
        subject_raw: &str,
        input: &str,
    ) -> Self {
        Self {
            action,
            subject,
            subject_raw: subject_raw.to_string(),
            parameters: vec![],
            original_input: input.to_string(),
            confidence: 0.95,
            classification_method: ClassificationMethod::PatternMatch,
        }
    }

    /// Create a new intent with keyword match confidence
    pub fn from_keywords(
        action: IntentAction,
        subject: IntentSubject,
        subject_raw: &str,
        input: &str,
        keyword_count: usize,
        total_keywords: usize,
    ) -> Self {
        let base_confidence = 0.7;
        let keyword_boost = if total_keywords > 0 {
            (keyword_count as f32 / total_keywords as f32) * 0.2
        } else {
            0.0
        };

        Self {
            action,
            subject,
            subject_raw: subject_raw.to_string(),
            parameters: vec![],
            original_input: input.to_string(),
            confidence: base_confidence + keyword_boost,
            classification_method: ClassificationMethod::KeywordMatch,
        }
    }

    /// Add a parameter to the intent
    pub fn with_parameter(mut self, param: String) -> Self {
        self.parameters.push(param);
        self
    }

    /// Add multiple parameters
    pub fn with_parameters(mut self, params: Vec<String>) -> Self {
        self.parameters.extend(params);
        self
    }

    /// Check if this intent needs confirmation before execution
    pub fn needs_confirmation(&self) -> bool {
        self.action.needs_confirmation()
    }

    /// Check if confidence is high enough for direct execution
    pub fn is_high_confidence(&self) -> bool {
        self.confidence >= 0.85
    }

    /// Check if confidence is too low and needs clarification
    pub fn needs_clarification(&self) -> bool {
        self.confidence < 0.6 || self.action == IntentAction::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_action_modifies_system() {
        assert!(!IntentAction::Query.modifies_system());
        assert!(!IntentAction::Help.modifies_system());
        assert!(IntentAction::Configure.modifies_system());
        assert!(IntentAction::Package.modifies_system());
        assert!(IntentAction::Execute.modifies_system());
    }

    #[test]
    fn test_intent_from_pattern() {
        let intent = UserIntent::from_pattern(
            IntentAction::Query,
            IntentSubject::DiskUsage,
            "disk",
            "how much disk space",
        );
        assert_eq!(intent.confidence, 0.95);
        assert_eq!(intent.classification_method, ClassificationMethod::PatternMatch);
    }

    #[test]
    fn test_intent_from_keywords() {
        let intent = UserIntent::from_keywords(
            IntentAction::Query,
            IntentSubject::MemoryUsage,
            "memory",
            "show ram usage",
            2,
            3,
        );
        assert!(intent.confidence > 0.7);
        assert!(intent.confidence < 0.95);
    }

    #[test]
    fn test_intent_needs_clarification() {
        let low_confidence = UserIntent {
            confidence: 0.3,
            ..Default::default()
        };
        assert!(low_confidence.needs_clarification());

        let unknown = UserIntent {
            action: IntentAction::Unknown,
            confidence: 0.9,
            ..Default::default()
        };
        assert!(unknown.needs_clarification());
    }

    #[test]
    fn test_intent_is_high_confidence() {
        let high = UserIntent {
            confidence: 0.90,
            ..Default::default()
        };
        assert!(high.is_high_confidence());

        let low = UserIntent {
            confidence: 0.70,
            ..Default::default()
        };
        assert!(!low.is_high_confidence());
    }
}
