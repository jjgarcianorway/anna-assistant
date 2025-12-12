//! Recipe matching engine (v0.0.423).
//!
//! Matches incoming queries against available recipes using:
//! - Domain matching
//! - Intent matching
//! - Keyword similarity (Jaccard)
//! - Entity matching
//! - Precondition evaluation

use std::collections::{HashMap, HashSet};

use super::{RecipeV3, MAX_RECIPES_TO_CHECK, MIN_MATCH_SCORE};

/// Result of matching a query against recipes
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// Matched recipe
    pub recipe: RecipeV3,
    /// Match score (0.0 to 1.0)
    pub score: f32,
    /// Breakdown of scoring
    pub breakdown: MatchBreakdown,
    /// Preconditions that were evaluated
    pub preconditions_met: bool,
    /// Variables extracted from query
    pub extracted_vars: HashMap<String, String>,
}

/// Breakdown of how match score was calculated
#[derive(Debug, Clone, Default)]
pub struct MatchBreakdown {
    /// Domain match contribution
    pub domain_score: f32,
    /// Intent match contribution
    pub intent_score: f32,
    /// Keyword similarity contribution
    pub keyword_score: f32,
    /// Entity match contribution
    pub entity_score: f32,
    /// Maturity bonus
    pub maturity_bonus: f32,
    /// Health penalty (for failing recipes)
    pub health_penalty: f32,
}

impl MatchBreakdown {
    /// Calculate total score
    pub fn total(&self) -> f32 {
        let base = self.domain_score * 0.15
            + self.intent_score * 0.35
            + self.keyword_score * 0.30
            + self.entity_score * 0.20;

        (base + self.maturity_bonus - self.health_penalty).clamp(0.0, 1.0)
    }
}

/// Query context for matching
#[derive(Debug, Clone, Default)]
pub struct MatchQuery {
    /// Original question
    pub question: String,
    /// Detected domain
    pub domain: Option<String>,
    /// Detected intent
    pub intent: Option<String>,
    /// Extracted keywords
    pub keywords: Vec<String>,
    /// Extracted entities (service names, package names, etc.)
    pub entities: Vec<String>,
}

impl MatchQuery {
    /// Create from raw question
    pub fn from_question(question: &str) -> Self {
        let keywords = extract_keywords(question);
        let entities = extract_entities(question);
        let domain = detect_domain(question);
        let intent = detect_intent(question);

        Self {
            question: question.to_string(),
            domain,
            intent,
            keywords,
            entities,
        }
    }

    /// Builder: set domain
    pub fn with_domain(mut self, domain: &str) -> Self {
        self.domain = Some(domain.to_string());
        self
    }

    /// Builder: set intent
    pub fn with_intent(mut self, intent: &str) -> Self {
        self.intent = Some(intent.to_string());
        self
    }

    /// Builder: add entity
    pub fn with_entity(mut self, entity: &str) -> Self {
        self.entities.push(entity.to_string());
        self
    }
}

/// Recipe matcher
pub struct RecipeMatcher {
    /// Minimum score threshold
    min_score: f32,
    /// Maximum recipes to check
    max_check: usize,
    /// Whether to evaluate preconditions
    eval_preconditions: bool,
}

impl Default for RecipeMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl RecipeMatcher {
    /// Create new matcher with defaults
    pub fn new() -> Self {
        Self {
            min_score: MIN_MATCH_SCORE,
            max_check: MAX_RECIPES_TO_CHECK,
            eval_preconditions: true,
        }
    }

    /// Set minimum score threshold
    pub fn with_min_score(mut self, score: f32) -> Self {
        self.min_score = score;
        self
    }

    /// Set whether to evaluate preconditions
    pub fn with_precondition_eval(mut self, eval: bool) -> Self {
        self.eval_preconditions = eval;
        self
    }

    /// Find matching recipes for a query
    pub fn find_matches(&self, query: &MatchQuery, recipes: &[RecipeV3]) -> Vec<MatchResult> {
        let mut results: Vec<MatchResult> = recipes
            .iter()
            .take(self.max_check)
            .filter(|r| r.enabled)
            .filter_map(|recipe| self.score_recipe(query, recipe))
            .filter(|r| r.score >= self.min_score)
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// Find best matching recipe
    pub fn find_best(&self, query: &MatchQuery, recipes: &[RecipeV3]) -> Option<MatchResult> {
        self.find_matches(query, recipes).into_iter().next()
    }

    /// Score a single recipe against query
    fn score_recipe(&self, query: &MatchQuery, recipe: &RecipeV3) -> Option<MatchResult> {
        let mut breakdown = MatchBreakdown::default();

        // Domain matching (15%)
        if let Some(ref query_domain) = query.domain {
            let recipe_domain = format!("{:?}", recipe.matcher.domain).to_lowercase();
            if recipe_domain == query_domain.to_lowercase() || recipe_domain == "general" {
                breakdown.domain_score = 1.0;
            }
        } else {
            // No domain specified, slight bonus for general recipes
            breakdown.domain_score = 0.5;
        }

        // Intent matching (35%)
        if let Some(ref query_intent) = query.intent {
            let intent_lower = query_intent.to_lowercase();
            let matches = recipe.matcher.intents.iter().any(|i| {
                i.to_lowercase() == intent_lower || intent_lower.contains(&i.to_lowercase())
            });
            if matches {
                breakdown.intent_score = 1.0;
            } else {
                // Partial match via similarity key
                let sim_key = recipe.matcher.similarity_key.to_lowercase();
                if !sim_key.is_empty()
                    && (intent_lower.contains(&sim_key) || sim_key.contains(&intent_lower))
                {
                    breakdown.intent_score = 0.6;
                }
            }
        }

        // Keyword similarity (30%) - Jaccard index
        if !query.keywords.is_empty() && !recipe.matcher.keywords.is_empty() {
            let query_set: HashSet<_> = query.keywords.iter().map(|s| s.to_lowercase()).collect();
            let recipe_set: HashSet<_> = recipe
                .matcher
                .keywords
                .iter()
                .map(|s| s.to_lowercase())
                .collect();

            let intersection = query_set.intersection(&recipe_set).count();
            let union = query_set.union(&recipe_set).count();

            if union > 0 {
                breakdown.keyword_score = intersection as f32 / union as f32;
            }
        }

        // Entity matching (20%)
        let mut extracted_vars = HashMap::new();
        if !query.entities.is_empty() {
            // Try to match entities against patterns
            for entity in &query.entities {
                for pattern in &recipe.matcher.entity_patterns {
                    if pattern.contains("*") {
                        // Wildcard pattern
                        let prefix = pattern.trim_end_matches('*');
                        if entity.starts_with(prefix) {
                            breakdown.entity_score = 1.0;
                            // Extract the entity as a variable
                            extracted_vars.insert("entity".to_string(), entity.clone());
                            break;
                        }
                    } else if pattern == entity || pattern.to_lowercase() == entity.to_lowercase() {
                        breakdown.entity_score = 1.0;
                        extracted_vars.insert("entity".to_string(), entity.clone());
                        break;
                    }
                }
            }

            // Also try direct entity match in keywords
            if breakdown.entity_score < 1.0 {
                let recipe_kw: HashSet<_> = recipe
                    .matcher
                    .keywords
                    .iter()
                    .map(|s| s.to_lowercase())
                    .collect();
                for entity in &query.entities {
                    if recipe_kw.contains(&entity.to_lowercase()) {
                        breakdown.entity_score = 0.8;
                        extracted_vars.insert("entity".to_string(), entity.clone());
                        break;
                    }
                }
            }
        }

        // Maturity bonus (up to +0.1 for proven recipes)
        if recipe.stats.is_mature() {
            breakdown.maturity_bonus = 0.1 * recipe.stats.success_rate();
        }

        // Health penalty (up to -0.2 for failing recipes)
        if recipe.stats.is_mature() && recipe.stats.success_rate() < super::MIN_SUCCESS_RATE {
            breakdown.health_penalty = 0.2 * (1.0 - recipe.stats.success_rate());
        }

        let score = breakdown.total();

        // Skip if score is too low
        if score < self.min_score {
            return None;
        }

        // Evaluate preconditions if enabled
        let preconditions_met = if self.eval_preconditions && !recipe.preconditions.is_empty() {
            recipe
                .preconditions
                .iter()
                .all(|cond| cond.evaluate(&extracted_vars).success)
        } else {
            true
        };

        Some(MatchResult {
            recipe: recipe.clone(),
            score,
            breakdown,
            preconditions_met,
            extracted_vars,
        })
    }
}

/// Extract keywords from question
fn extract_keywords(question: &str) -> Vec<String> {
    let stop_words: HashSet<&str> = [
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
        "do", "does", "did", "will", "would", "could", "should", "may", "might", "must", "i", "my",
        "me", "you", "your", "we", "our", "they", "their", "it", "its", "this", "that", "what",
        "which", "who", "whom", "how", "why", "when", "where", "to", "of", "in", "on", "at", "by",
        "for", "with", "about", "into", "through", "can", "please", "help", "want", "need", "get",
        "just",
    ]
    .into_iter()
    .collect();

    question
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
        .filter(|w| w.len() > 2 && !stop_words.contains(w))
        .map(String::from)
        .collect()
}

/// Extract potential entities from question
fn extract_entities(question: &str) -> Vec<String> {
    let mut entities = vec![];

    // Common patterns for service names, package names, etc.
    let words: Vec<&str> = question.split_whitespace().collect();

    for word in words {
        // Check for path-like patterns first (before trimming which removes /)
        if word.starts_with('/') && word.len() > 1 {
            // Keep the path but trim trailing punctuation
            let path =
                word.trim_end_matches(|c: char| c == ',' || c == '.' || c == ':' || c == ';');
            if path.len() > 1 {
                entities.push(path.to_string());
            }
            continue;
        }

        let clean =
            word.trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_' && c != '.');

        // Service-like names (end with d or .service)
        if clean.ends_with('d') && clean.len() > 2 && clean.chars().all(|c| c.is_alphanumeric()) {
            entities.push(clean.to_string());
        }

        // .service suffix
        if clean.ends_with(".service") {
            entities.push(clean.to_string());
        }

        // Package-like names (lowercase with optional dashes)
        if clean.len() > 2
            && clean
                .chars()
                .all(|c| c.is_ascii_lowercase() || c == '-' || c.is_ascii_digit())
            && clean
                .chars()
                .next()
                .map(|c| c.is_ascii_lowercase())
                .unwrap_or(false)
        {
            entities.push(clean.to_string());
        }
    }

    entities
}

/// Detect domain from question
fn detect_domain(question: &str) -> Option<String> {
    let q = question.to_lowercase();

    let domain_patterns = [
        (
            &["service", "systemctl", "systemd", "unit", "daemon"][..],
            "systemd",
        ),
        (
            &["package", "pacman", "install", "update", "yay", "paru"],
            "package",
        ),
        (
            &["network", "wifi", "ethernet", "ip", "ping", "dns"],
            "network",
        ),
        (
            &[
                "disk",
                "mount",
                "filesystem",
                "storage",
                "drive",
                "partition",
            ],
            "disk",
        ),
        (&["memory", "ram", "swap", "oom"], "memory"),
        (&["process", "kill", "ps", "top", "htop"], "process"),
        (&["user", "account", "password", "group", "sudo"], "user"),
        (&["config", "configuration", "settings", ".conf"], "config"),
        (
            &["git", "github", "commit", "push", "pull", "branch"],
            "git",
        ),
        (&["docker", "container", "podman", "image"], "docker"),
        (
            &["vim", "nvim", "neovim", "nano", "emacs", "editor"],
            "editor",
        ),
        (&["bash", "zsh", "fish", "shell", "terminal"], "shell"),
        (&["cron", "crontab", "timer", "schedule"], "cron"),
    ];

    for (keywords, domain) in domain_patterns {
        if keywords.iter().any(|k| q.contains(k)) {
            return Some(domain.to_string());
        }
    }

    None
}

/// Detect intent from question
fn detect_intent(question: &str) -> Option<String> {
    let q = question.to_lowercase();

    let intent_patterns = [
        (&["restart", "restarting"][..], "restart"),
        (&["start", "starting", "run", "launch"], "start"),
        (&["stop", "stopping", "kill", "terminate"], "stop"),
        (&["enable", "enabling", "autostart"], "enable"),
        (&["disable", "disabling"], "disable"),
        (&["status", "check", "state", "is running"], "status"),
        (&["install", "installing", "add"], "install"),
        (&["remove", "uninstall", "delete"], "remove"),
        (&["update", "upgrade"], "update"),
        (&["list", "show all", "what are"], "list"),
        (&["how to", "how do i", "how can i"], "howto"),
        (&["why", "what is", "explain"], "explain"),
        (&["fix", "repair", "troubleshoot", "not working"], "fix"),
        (&["configure", "setup", "set up"], "configure"),
    ];

    for (keywords, intent) in intent_patterns {
        if keywords.iter().any(|k| q.contains(k)) {
            return Some(intent.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe_v3::{RecipeDomain, RecipeMatcher as RM};

    #[test]
    fn test_extract_keywords() {
        let kw = extract_keywords("How do I restart the nginx service?");
        assert!(kw.contains(&"restart".to_string()));
        assert!(kw.contains(&"nginx".to_string()));
        assert!(kw.contains(&"service".to_string()));
        assert!(!kw.contains(&"how".to_string()));
    }

    #[test]
    fn test_extract_entities() {
        let entities = extract_entities("restart nginx.service");
        assert!(entities.contains(&"nginx.service".to_string()));

        // Path extraction only works for paths starting with /
        let entities2 = extract_entities("check /etc/nginx/nginx.conf file");
        assert!(entities2.iter().any(|e| e.starts_with("/etc")));
    }

    #[test]
    fn test_detect_domain() {
        assert_eq!(
            detect_domain("restart nginx service"),
            Some("systemd".to_string())
        );
        assert_eq!(detect_domain("install vim"), Some("package".to_string()));
        assert_eq!(
            detect_domain("check network connection"),
            Some("network".to_string())
        );
    }

    #[test]
    fn test_detect_intent() {
        assert_eq!(detect_intent("restart nginx"), Some("restart".to_string()));
        assert_eq!(
            detect_intent("how do I configure vim"),
            Some("howto".to_string())
        );
        assert_eq!(
            detect_intent("nginx is not working"),
            Some("fix".to_string())
        );
    }

    #[test]
    fn test_match_query() {
        let query = MatchQuery::from_question("How do I restart nginx?");
        assert!(query.keywords.contains(&"restart".to_string()));
        assert!(query.keywords.contains(&"nginx".to_string()));
        assert_eq!(query.intent, Some("restart".to_string()));
    }

    #[test]
    fn test_recipe_matching() {
        let recipe = RecipeV3::new("restart-service", "Restart a Service").with_matcher(
            RM::new(RecipeDomain::Systemd)
                .with_intents(&["restart"])
                .with_keywords(&["restart", "service", "systemctl"])
                .with_entities(&["*"]),
        );

        let query = MatchQuery::from_question("restart nginx service");
        let matcher = super::RecipeMatcher::new();
        let results = matcher.find_matches(&query, &[recipe]);

        assert!(!results.is_empty());
        assert!(results[0].score > 0.5);
    }
}
