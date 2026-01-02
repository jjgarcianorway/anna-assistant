//! Core types for deterministic probe rules.

/// A deterministic probe rule.
#[derive(Debug, Clone)]
pub struct ProbeRule {
    /// Intent ID for matching.
    pub intent_id: &'static str,
    /// Keywords that trigger this rule (all must match).
    pub keywords: &'static [&'static str],
    /// Negative keywords (if any match, rule doesn't apply).
    pub negative_keywords: &'static [&'static str],
    /// Exact probes to run (in order).
    pub probes: &'static [&'static str],
    /// Description for debugging.
    pub description: &'static str,
}
