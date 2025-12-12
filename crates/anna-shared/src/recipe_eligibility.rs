//! Recipe eligibility checker for Anna's learning system.
//! v0.0.418: Determines if a ticket can produce or update a recipe.
//!
//! A ticket is eligible to produce a recipe if:
//! - ticket.status == "ok"
//! - specialist.confidence >= 0.9
//! - There is at least one concrete command or edit in actions
//! - The intent is learnable (config change, repeatable diagnostic, simple fix)

use serde::{Deserialize, Serialize};

/// Result of eligibility check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EligibilityResult {
    /// Whether this ticket can produce a recipe
    pub eligible: bool,
    /// Why it's eligible or not
    pub reason: String,
    /// Eligibility score (0.0-1.0) for prioritization
    pub score: f32,
    /// What kind of recipe this would produce
    pub recipe_type: Option<RecipeType>,
    /// Detected intent for recipe creation
    pub detected_intent: Option<String>,
}

/// Type of recipe that can be created.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecipeType {
    /// Config file modification (append, replace, ensure)
    ConfigChange,
    /// Repeatable diagnostic (check status, get info)
    RepeatableDiagnostic,
    /// Simple fix (restart service, clear cache)
    SimpleFix,
    /// Service management (enable, disable, restart)
    ServiceAction,
    /// Package query or action
    PackageAction,
}

impl RecipeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RecipeType::ConfigChange => "config_change",
            RecipeType::RepeatableDiagnostic => "repeatable_diagnostic",
            RecipeType::SimpleFix => "simple_fix",
            RecipeType::ServiceAction => "service_action",
            RecipeType::PackageAction => "package_action",
        }
    }
}

/// Ticket data needed for eligibility check.
#[derive(Debug, Clone)]
pub struct TicketForEligibility {
    pub status: String,
    pub confidence: u8,
    pub intent: Option<String>,
    pub domain: Option<String>,
    pub user_query: String,
    pub specialist_summary: Option<String>,
    pub actions: Vec<String>,
    pub commands_executed: Vec<String>,
    pub files_modified: Vec<String>,
    pub has_citations: bool,
}

/// Intents that are learnable (can produce recipes).
const LEARNABLE_INTENTS: &[&str] = &[
    // Config changes
    "configure_editor_feature",
    "configure_desktop",
    "configure_shell",
    "configure_service",
    "enable_feature",
    "disable_feature",
    // Diagnostics
    "check_disk_usage",
    "check_free_ram",
    "check_swap",
    "check_services",
    "check_boot_time",
    "check_uptime",
    "check_package",
    "check_config",
    // Service actions
    "enable_service",
    "disable_service",
    "restart_service",
    "start_service",
    "stop_service",
    // Package actions
    "install_package",
    "remove_package",
    "update_packages",
    // Simple fixes
    "fix_permissions",
    "clear_cache",
    "reload_config",
];

/// Check if a ticket is eligible to produce a recipe.
pub fn check_eligibility(ticket: &TicketForEligibility) -> EligibilityResult {
    // Rule 1: Status must be "ok"
    if ticket.status != "ok" {
        return EligibilityResult {
            eligible: false,
            reason: format!("Ticket status is '{}', not 'ok'", ticket.status),
            score: 0.0,
            recipe_type: None,
            detected_intent: None,
        };
    }

    // Rule 2: Confidence must be high (>= 90)
    if ticket.confidence < 90 {
        return EligibilityResult {
            eligible: false,
            reason: format!("Confidence {} is below threshold 90", ticket.confidence),
            score: 0.0,
            recipe_type: None,
            detected_intent: None,
        };
    }

    // Rule 3: Must have concrete actions OR commands OR file modifications
    let has_concrete_action = !ticket.actions.is_empty()
        || !ticket.commands_executed.is_empty()
        || !ticket.files_modified.is_empty();

    if !has_concrete_action {
        return EligibilityResult {
            eligible: false,
            reason: "No concrete actions, commands, or file modifications".into(),
            score: 0.0,
            recipe_type: None,
            detected_intent: None,
        };
    }

    // Rule 4: Intent must be learnable
    let intent = ticket.intent.as_deref().unwrap_or("");
    let is_learnable_intent = LEARNABLE_INTENTS
        .iter()
        .any(|&i| intent.contains(i) || i.contains(intent));

    // Detect recipe type from context
    let recipe_type = detect_recipe_type(ticket);

    if !is_learnable_intent && recipe_type.is_none() {
        return EligibilityResult {
            eligible: false,
            reason: format!("Intent '{}' is not learnable", intent),
            score: 0.0,
            recipe_type: None,
            detected_intent: Some(intent.to_string()),
        };
    }

    // Calculate eligibility score
    let mut score = 0.5; // Base score for meeting requirements

    // Boost for high confidence
    score += (ticket.confidence - 90) as f32 * 0.05; // +0.05 per point above 90

    // Boost for having citations (grounded in docs)
    if ticket.has_citations {
        score += 0.1;
    }

    // Boost for having file modifications (concrete change)
    if !ticket.files_modified.is_empty() {
        score += 0.1;
    }

    // Boost for having commands (actionable)
    if !ticket.commands_executed.is_empty() {
        score += 0.05;
    }

    score = score.min(1.0);

    EligibilityResult {
        eligible: true,
        reason: "Ticket meets all eligibility criteria".into(),
        score,
        recipe_type,
        detected_intent: Some(intent.to_string()),
    }
}

/// Detect recipe type from ticket context.
fn detect_recipe_type(ticket: &TicketForEligibility) -> Option<RecipeType> {
    let query = ticket.user_query.to_lowercase();
    let summary = ticket
        .specialist_summary
        .as_deref()
        .unwrap_or("")
        .to_lowercase();
    let combined = format!("{} {}", query, summary);

    // Check for config changes
    if !ticket.files_modified.is_empty() {
        let has_config_file = ticket.files_modified.iter().any(|f| {
            f.contains(".conf")
                || f.contains(".cfg")
                || f.contains(".ini")
                || f.contains(".toml")
                || f.contains(".yaml")
                || f.contains(".yml")
                || f.contains(".json")
                || f.contains("rc") // .vimrc, .bashrc, etc.
                || f.contains("config")
        });
        if has_config_file {
            return Some(RecipeType::ConfigChange);
        }
    }

    // Check for service actions
    let service_keywords = [
        "systemctl",
        "service",
        "enable",
        "disable",
        "restart",
        "start",
        "stop",
    ];
    if service_keywords.iter().any(|k| combined.contains(k)) {
        return Some(RecipeType::ServiceAction);
    }

    // Check for package actions
    let package_keywords = ["pacman", "yay", "install", "remove", "uninstall", "package"];
    if package_keywords.iter().any(|k| combined.contains(k)) {
        return Some(RecipeType::PackageAction);
    }

    // Check for diagnostics
    let diag_keywords = [
        "check", "status", "how much", "what is", "show", "list", "disk", "ram", "memory", "cpu",
        "swap", "uptime", "boot",
    ];
    if diag_keywords.iter().any(|k| combined.contains(k)) {
        return Some(RecipeType::RepeatableDiagnostic);
    }

    // Check for simple fixes
    let fix_keywords = ["fix", "repair", "clear", "clean", "reset", "reload"];
    if fix_keywords.iter().any(|k| combined.contains(k)) {
        return Some(RecipeType::SimpleFix);
    }

    None
}

/// Check if a specific intent is learnable.
pub fn is_learnable_intent(intent: &str) -> bool {
    LEARNABLE_INTENTS
        .iter()
        .any(|&i| intent.contains(i) || i.contains(intent))
}

/// Get list of learnable intents.
pub fn learnable_intents() -> &'static [&'static str] {
    LEARNABLE_INTENTS
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ticket(status: &str, confidence: u8, intent: &str) -> TicketForEligibility {
        TicketForEligibility {
            status: status.into(),
            confidence,
            intent: Some(intent.into()),
            domain: Some("desktop".into()),
            user_query: "enable syntax highlighting in vim".into(),
            specialist_summary: Some("Added syntax enable to .vimrc".into()),
            actions: vec!["Edited ~/.vimrc".into()],
            commands_executed: vec![],
            files_modified: vec!["~/.vimrc".into()],
            has_citations: true,
        }
    }

    #[test]
    fn test_eligible_ticket() {
        let ticket = make_ticket("ok", 95, "configure_editor_feature");
        let result = check_eligibility(&ticket);
        assert!(result.eligible);
        assert!(result.score > 0.5);
        assert_eq!(result.recipe_type, Some(RecipeType::ConfigChange));
    }

    #[test]
    fn test_ineligible_partial_status() {
        let ticket = make_ticket("partial", 95, "configure_editor_feature");
        let result = check_eligibility(&ticket);
        assert!(!result.eligible);
        assert!(result.reason.contains("partial"));
    }

    #[test]
    fn test_ineligible_low_confidence() {
        let ticket = make_ticket("ok", 75, "configure_editor_feature");
        let result = check_eligibility(&ticket);
        assert!(!result.eligible);
        assert!(result.reason.contains("90")); // Threshold is 90
    }

    #[test]
    fn test_diagnostic_detection() {
        let mut ticket = make_ticket("ok", 95, "check_disk_usage");
        ticket.user_query = "check disk usage".into();
        ticket.specialist_summary = Some("Disk is 60% full".into()); // Clear the default summary
        ticket.files_modified = vec![];
        let result = check_eligibility(&ticket);
        assert!(result.eligible);
        assert_eq!(result.recipe_type, Some(RecipeType::RepeatableDiagnostic));
    }

    #[test]
    fn test_service_detection() {
        let mut ticket = make_ticket("ok", 95, "enable_service");
        ticket.user_query = "enable sshd service".into();
        ticket.commands_executed = vec!["systemctl enable sshd".into()];
        ticket.files_modified = vec![];
        let result = check_eligibility(&ticket);
        assert!(result.eligible);
        assert_eq!(result.recipe_type, Some(RecipeType::ServiceAction));
    }
}
