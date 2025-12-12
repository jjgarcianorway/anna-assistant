//! Acceptance Tests (Part 4) - v0.0.442.
//!
//! Tests that MUST pass after implementing ticket integrity fixes.
//!
//! 1) Clarification and editor - ask which editor, no probes first
//! 2) Swap - check /proc/swaps, not package
//! 3) Packages - clean output, no "2 is installed" garbage
//! 4) Wallpapers - one clean clarification, correct domain
//! 5) Stats honesty - parse errors reflected in stats

use super::clarification::*;
use super::outcome::*;
use super::package_system::*;

/// Test: Editor syntax highlighting must ask which editor FIRST.
#[test]
fn test_editor_syntax_asks_which_editor() {
    // Given: No known facts about editor
    let facts = KnownFacts::new();

    // When: User asks about syntax highlighting
    let decision = check_clarification_needed("editor.syntax_status", &facts);

    // Then: Must ask for clarification, NOT proceed to probes
    match decision {
        ClarificationDecision::NeedClarification { question, missing_facts } => {
            assert!(
                question.question.to_lowercase().contains("editor"),
                "Question must ask about editor"
            );
            assert!(
                missing_facts.contains(&"editor.name".to_string()),
                "editor.name must be in missing facts"
            );
        }
        _ => panic!("Expected NeedClarification, got {:?}", decision),
    }
}

/// Test: Swap question must NOT return "swap package not installed".
#[test]
fn test_swap_question_is_system_not_package() {
    // Given: User asks "do I have swap?"
    let question = "do I have swap?";

    // When: We classify the question
    let classification = classify_question(question, None);

    // Then: Must be System intent, NOT Package intent
    match classification {
        QuestionClassification::System(SystemIntent::SwapConfigured) => {
            // Correct!
        }
        QuestionClassification::Package(_) => {
            panic!("WRONG: 'do I have swap?' should NOT be classified as Package intent")
        }
        _ => panic!("Expected System(SwapConfigured), got {:?}", classification),
    }
}

/// Test: Swap with no swap configured.
#[test]
fn test_swap_status_none() {
    // Given: Empty /proc/swaps
    let output = "Filename\t\t\t\tType\t\tSize\t\tUsed\t\tPriority\n";

    // When: We parse swap status
    let status = SwapStatus::from_proc_swaps(output);

    // Then: Must report NO swap
    assert!(!status.has_swap, "Should have no swap");
    assert_eq!(status.kind, SwapKind::None);

    let display = status.display();
    assert!(
        display.contains("NO swap"),
        "Display must say 'NO swap', got: {}",
        display
    );
}

/// Test: Swap with swap file configured.
#[test]
fn test_swap_status_with_swapfile() {
    // Given: /proc/swaps with swap file
    let output = "Filename\t\t\t\tType\t\tSize\t\tUsed\t\tPriority\n/swapfile\tfile\t\t8388604\t\t0\t\t-2\n";

    // When: We parse swap status
    let status = SwapStatus::from_proc_swaps(output);

    // Then: Must report swap correctly
    assert!(status.has_swap, "Should have swap");
    assert_eq!(status.kind, SwapKind::File);
    assert!(status.total_swap_gib > 7.0, "Should be ~8 GiB");

    let display = status.display();
    assert!(
        display.contains("GiB") && display.contains("swapfile"),
        "Display must show size and type, got: {}",
        display
    );
}

/// Test: "is nano installed?" must return clean package status.
#[test]
fn test_package_nano_not_installed() {
    // Given: pacman says nano not installed
    let output = "NOT_INSTALLED";

    // When: We parse package status
    let status = PackageStatus::from_pacman_output("nano", output);

    // Then: Clean output
    assert!(!status.installed);

    let display = status.display();
    assert!(
        display.contains("nano") && display.contains("NOT installed"),
        "Display must be clean, got: {}",
        display
    );
    // MUST NOT contain garbage like "2 is installed"
    assert!(
        !display.contains("2 is installed"),
        "Must not have '2 is installed' garbage"
    );
}

/// Test: "is nano installed?" when it IS installed.
#[test]
fn test_package_nano_installed() {
    // Given: pacman says nano is installed
    let output = "nano 7.2-1";

    // When: We parse package status
    let status = PackageStatus::from_pacman_output("nano", output);

    // Then: Clean output with version
    assert!(status.installed);
    assert_eq!(status.version, Some("7.2-1".to_string()));

    let display = status.display();
    assert!(
        display.contains("nano") && display.contains("7.2-1") && display.contains("installed"),
        "Display must show package and version, got: {}",
        display
    );
}

/// Test: Parse error must result in ParseError outcome, NOT Answered.
#[test]
fn test_parse_error_outcome() {
    // Given: Conditions where JSON was invalid
    let conditions = AnsweredConditions {
        specialist_responded: true,
        json_valid: false, // <-- Parse error
        schema_valid: false,
        answer_rendered: false,
        no_internal_errors: true,
    };

    // When: We determine outcome
    let outcome = conditions.determine_outcome();

    // Then: Must be ParseError, NOT Answered
    assert_eq!(outcome, TicketOutcome::ParseError);
    assert!(!outcome.is_success(), "ParseError must not count as success");
}

/// Test: Stats must show parse errors separately.
#[test]
fn test_stats_show_parse_errors() {
    // Given: Mixed outcomes
    let mut stats = HonestTicketStats::new();
    stats.record(TicketOutcome::Answered);
    stats.record(TicketOutcome::Answered);
    stats.record(TicketOutcome::ParseError);
    stats.record(TicketOutcome::ParseError);
    stats.record(TicketOutcome::ParseError);
    stats.record(TicketOutcome::ProbeError);

    // Then: Stats must be honest
    assert_eq!(stats.total_tickets, 6);
    assert_eq!(stats.answered, 2);
    assert_eq!(stats.failed_parse, 3);
    assert_eq!(stats.probe_failures, 1);

    // Success rate must be 2/6, NOT 100%
    let rate = stats.success_rate();
    assert!(
        (rate - 0.333).abs() < 0.01,
        "Success rate should be ~33%, got {}",
        rate
    );

    // Display must show failures
    let display = stats.display();
    assert!(display.contains("failed_parse"), "Must show failed_parse");
    assert!(display.contains("3"), "Must show count of 3 parse errors");
}

/// Test: Wallpaper question classification.
#[test]
fn test_wallpaper_clarification_required() {
    // Given: No known facts
    let facts = KnownFacts::new();

    // When: User asks about wallpapers
    let is_required = is_clarification_required_intent("wallpapers_location");

    // Then: Should require clarification
    assert!(is_required, "wallpapers_location should require clarification");

    // And the decision should request clarification
    let decision = check_clarification_needed("wallpapers_location", &facts);
    assert!(
        matches!(decision, ClarificationDecision::NeedClarification { .. }),
        "Should need clarification"
    );
}

/// Test: Once facts are known, proceed to probes.
#[test]
fn test_clarification_proceeds_when_facts_known() {
    // Given: User already told us their editor
    let mut facts = KnownFacts::new();
    facts.add("editor.name", "vim", FactSource::User);
    facts.add("editor.config_path", "~/.vimrc", FactSource::User);

    // When: User asks about syntax highlighting
    let decision = check_clarification_needed("editor.syntax_status", &facts);

    // Then: Should proceed to probes, NOT ask again
    assert!(
        matches!(decision, ClarificationDecision::ProceedToProbes),
        "Should proceed to probes when facts known"
    );
}

/// Test: "is steam installed?" is a package question.
#[test]
fn test_steam_is_package_question() {
    let classification = classify_question("is steam installed?", Some("steam"));
    match classification {
        QuestionClassification::Package(PackageIntent::CheckInstalled { package }) => {
            assert_eq!(package, "steam");
        }
        _ => panic!("Expected Package(CheckInstalled), got {:?}", classification),
    }
}

/// Test: "do I have games?" is ambiguous (not a specific package).
#[test]
fn test_games_is_ambiguous() {
    let classification = classify_question("do I have games in this laptop?", None);
    // This should either be Ambiguous or require special handling
    // It should NOT return "games package is not installed"
    assert!(
        !matches!(
            classification,
            QuestionClassification::Package(PackageIntent::CheckInstalled { package }) if package == "games"
        ),
        "'games' should not be treated as a literal package name"
    );
}

/// Test: Error message detection.
#[test]
fn test_error_message_detection() {
    assert!(is_parse_error("Failed to parse specialist response. Parse error: Timeout"));
    assert!(is_parse_error("Invalid JSON in response"));
    assert!(!is_parse_error("Everything worked fine"));

    assert_eq!(
        outcome_from_error("Failed to parse specialist response"),
        TicketOutcome::ParseError
    );
    assert_eq!(
        outcome_from_error("Probe execution failed"),
        TicketOutcome::ProbeError
    );
}

/// Test: Clarification question format (no duplicates).
#[test]
fn test_clarification_display_format() {
    let question = ClarificationQuestion {
        question: "Which editor do you mean?".to_string(),
        options: vec![
            ClarificationOption::new("vim", "vim"),
            ClarificationOption::new("nano", "nano"),
            ClarificationOption::new("something else", "__other__"),
            ClarificationOption::new("cancel", "__cancel__"),
        ],
        fact_to_set: "editor.name".to_string(),
    };

    let display = question.display();

    // Check format
    assert!(display.contains("Which editor"));
    assert!(display.contains("1) vim"));
    assert!(display.contains("2) nano"));
    assert!(display.contains("9) something else")); // Other is 9
    assert!(display.contains("0) cancel")); // Cancel is 0

    // Count occurrences of "editor" - should only be in the question, not duplicated
    let editor_count = display.matches("editor").count();
    assert_eq!(editor_count, 1, "Question should not be duplicated");
}
