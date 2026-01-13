//! Recipe learning from successful answers.

use anna_shared::recipe::{Recipe, RecipeBook, RecipeCommand, RecipeContext, RecipeSource};
use anna_shared::teaching::{
    self, CitationSource, ExperimentSummary, ProbeResult, QuestionType, TeachingContext,
};
use tracing::{debug, info, warn};

use super::verification::truncate;

/// Learn a recipe from a successful answer.
/// Only learns if the answer involved actual commands and has high confidence.
pub fn learn_recipe_from_answer(question: &str, commands: &[String], confidence: f32) {
    // Only learn from high-confidence answers with actual commands
    if confidence < 0.8 || commands.is_empty() || commands.len() > 5 {
        return;
    }

    // Extract keywords from question (significant words)
    let keywords: Vec<String> = question
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2 && !is_common_word(w))
        .map(|s| s.to_string())
        .collect();

    // Need at least 2 keywords to create a recipe
    if keywords.len() < 2 {
        return;
    }

    // Load existing recipe book
    let mut book = match RecipeBook::load() {
        Ok(b) => b,
        Err(e) => {
            warn!("Failed to load recipe book: {}", e);
            return;
        }
    };

    // Check if similar recipe already exists (same keywords)
    let existing = book.recipes.iter().any(|r| {
        let matching_keywords = r.keywords.iter().filter(|k| keywords.contains(k)).count();
        matching_keywords >= 2
    });

    if existing {
        debug!("Similar recipe already exists, skipping");
        return;
    }

    // Generate unique ID (timestamp + hash of question)
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    question.hash(&mut hasher);
    let hash = hasher.finish();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let id = format!("learned_{}_{:x}", timestamp, hash);

    // Create recipe commands
    let recipe_commands: Vec<RecipeCommand> = commands
        .iter()
        .map(|cmd| RecipeCommand {
            command: cmd.clone(),
            description: "Learned from successful answer".to_string(),
            modifies_system: is_modifying_command(cmd),
            backup_file: None,
            needs_root: cmd.starts_with("sudo "),
        })
        .collect();

    // Create the recipe
    let recipe = Recipe {
        id: id.clone(),
        name: format!("Learned: {}", truncate(question, 40)),
        keywords,
        patterns: vec![question.to_lowercase()],
        context: RecipeContext::default(),
        commands: recipe_commands,
        verification: None,
        source: RecipeSource::Llm {
            model: "ollama".to_string(),
        },
        success_count: 1,
        last_used: Some(chrono::Utc::now().to_rfc3339()),
        enabled: true,
    };

    book.add_recipe(recipe);
    if let Err(e) = book.save() {
        warn!("Failed to save recipe book: {}", e);
    } else {
        info!(
            "Learned new recipe: {} (tier=candidate, confidence={:.0}%, reason=high confidence successful answer)",
            id,
            confidence * 100.0
        );
        // Record for RPG stats
        crate::department::rpg::record_recipe_learned();
    }
}

/// Check if a word is too common to be a keyword
fn is_common_word(word: &str) -> bool {
    const COMMON: &[&str] = &[
        "the", "and", "for", "that", "this", "with", "have", "are", "from", "what", "how", "why",
        "when", "where", "who", "which", "can", "could", "would", "should", "will", "does", "did",
        "has", "had", "been", "being", "was", "were", "not", "but", "all", "any", "some", "its",
        "into", "out", "your", "you", "don", "isn", "does", "doesn", "please", "help", "want",
    ];
    COMMON.contains(&word)
}

/// Check if a command modifies the system
fn is_modifying_command(cmd: &str) -> bool {
    let modifiers = [
        "rm ", "mv ", "cp ", "mkdir ", "rmdir ", "touch ", "chmod ", "chown ", "install ",
        "pacman -S", "pacman -R", "yay -S", "yay -R", "systemctl ", "echo ", "printf ", "cat >",
        "sed -i", "tee ", "ln -s",
    ];
    modifiers.iter().any(|m| cmd.contains(m))
}

/// Build teaching context from execution state.
/// Used to generate teaching explanations with proper citations.
pub fn build_teaching_context(
    question: &str,
    commands: &[String],
    outputs: &[String],
    had_experiments: bool,
    doc_citations: &[String],
) -> TeachingContext {
    // Classify question type
    let question_type = if had_experiments {
        QuestionType::RiskyAction
    } else {
        teaching::classify_question(question)
    };

    // Build probe results
    let probes: Vec<ProbeResult> = commands
        .iter()
        .zip(outputs.iter())
        .map(|(cmd, out)| ProbeResult {
            command: cmd.clone(),
            output_summary: truncate(out, 100),
            success: !out.to_lowercase().contains("error")
                && !out.to_lowercase().contains("failed"),
        })
        .collect();

    // Convert doc citations to CitationSources
    let citations: Vec<CitationSource> = doc_citations
        .iter()
        .filter_map(|c| {
            if c.contains("man ") {
                // Parse [man command(section)]
                let parts: Vec<&str> = c
                    .trim_matches(|c| c == '[' || c == ']')
                    .strip_prefix("man ")
                    .unwrap_or("")
                    .split('(')
                    .collect();
                if !parts.is_empty() {
                    let cmd = parts[0].to_string();
                    let section = parts.get(1).map(|s| s.trim_end_matches(')').to_string());
                    return Some(CitationSource::ManPage {
                        command: cmd,
                        section,
                    });
                }
            } else if c.contains("Arch Wiki:") {
                // Parse [Arch Wiki: Article - Section]
                let content = c
                    .trim_matches(|c| c == '[' || c == ']')
                    .strip_prefix("Arch Wiki: ")
                    .unwrap_or("");
                let parts: Vec<&str> = content.split(" - ").collect();
                let article = parts.first().unwrap_or(&"").to_string();
                let section = parts.get(1).map(|s| s.to_string());
                if !article.is_empty() {
                    return Some(CitationSource::ArchWiki { article, section });
                }
            } else if c.contains("--help") {
                // Parse [command --help]
                let cmd = c
                    .trim_matches(|c| c == '[' || c == ']')
                    .replace(" --help", "");
                return Some(CitationSource::HelpOutput { command: cmd });
            }
            None
        })
        .collect();

    // Determine risk reason for risky actions
    let risk_reason = if had_experiments {
        Some(get_risk_reason(commands))
    } else {
        None
    };

    TeachingContext {
        question_type,
        probes,
        doc_citations: citations,
        experiments: vec![], // Experiments are tracked separately in the loop
        is_risky: had_experiments,
        risk_reason,
    }
}

/// Get human-readable risk reason (principle, not score)
fn get_risk_reason(commands: &[String]) -> String {
    for cmd in commands {
        if cmd.contains("systemctl") {
            return "it modifies system services".to_string();
        }
        if cmd.contains("pacman -S") || cmd.contains("pacman -R") {
            return "it installs or removes packages".to_string();
        }
        if cmd.contains("rm ") {
            return "it deletes files".to_string();
        }
        if cmd.contains("chmod") || cmd.contains("chown") {
            return "it changes file permissions".to_string();
        }
        if cmd.starts_with("sudo ") {
            return "it requires elevated privileges".to_string();
        }
    }
    "it modifies system state".to_string()
}
