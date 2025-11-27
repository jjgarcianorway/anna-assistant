//! User Question Flow v0.15.0
//!
//! Interactive question handling for LLM-A user questions.
//! Supports single choice, multi choice, and free text.

use anna_common::{QuestionOption, QuestionStyle, UserAnswer, UserAnswerValue, UserQuestion};
use chrono::Utc;
use owo_colors::OwoColorize;
use std::io::{self, BufRead, Write};

/// Display a question and collect user response
pub fn ask_user(question: &UserQuestion) -> io::Result<UserAnswer> {
    println!();

    // Show why this question is being asked
    println!(
        "{}  {}",
        "?".bright_cyan().bold(),
        "Anna needs more information".bright_white().bold()
    );
    println!("   {}", question.reason.dimmed());
    println!();

    // Display the question
    println!("   {}", question.question.bright_white());
    println!();

    let answer_value = match question.style {
        QuestionStyle::SingleChoice => {
            ask_single_choice(&question.options, question.allow_free_text)?
        }
        QuestionStyle::MultiChoice => {
            ask_multi_choice(&question.options, question.allow_free_text)?
        }
        QuestionStyle::FreeText => ask_free_text()?,
    };

    Ok(UserAnswer {
        question_ref: compute_question_ref(question),
        answer: answer_value,
        answered_at: Utc::now(),
    })
}

/// Single choice - user picks exactly one option
fn ask_single_choice(options: &[QuestionOption], allow_free: bool) -> io::Result<UserAnswerValue> {
    // Display options
    for (i, opt) in options.iter().enumerate() {
        println!("   {}  {}", format!("[{}]", i + 1).cyan(), opt.label);
    }

    if allow_free {
        println!(
            "   {}  {}",
            format!("[{}]", options.len() + 1).cyan(),
            "Other (type your answer)".dimmed()
        );
    }

    println!();

    loop {
        print!("   {}  ", "Enter number:".bright_magenta());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().lock().read_line(&mut input)?;
        let input = input.trim();

        // Check for cancel
        if input.eq_ignore_ascii_case("cancel") || input.eq_ignore_ascii_case("quit") {
            return Ok(UserAnswerValue::Text("__CANCELLED__".to_string()));
        }

        // Parse number
        if let Ok(num) = input.parse::<usize>() {
            if num >= 1 && num <= options.len() {
                let selected = &options[num - 1];
                println!(
                    "   {}  Selected: {}",
                    "+".bright_green(),
                    selected.label.bright_white()
                );
                return Ok(UserAnswerValue::Single(selected.id.clone()));
            }

            if allow_free && num == options.len() + 1 {
                // Free text option
                return ask_free_text();
            }
        }

        println!(
            "   {}  Please enter a number between 1 and {}",
            "!".yellow(),
            if allow_free {
                options.len() + 1
            } else {
                options.len()
            }
        );
    }
}

/// Multi choice - user picks one or more options
fn ask_multi_choice(options: &[QuestionOption], allow_free: bool) -> io::Result<UserAnswerValue> {
    println!(
        "   {}",
        "(Enter numbers separated by commas, e.g., 1,3,4)".dimmed()
    );
    println!();

    // Display options
    for (i, opt) in options.iter().enumerate() {
        println!("   {}  {}", format!("[{}]", i + 1).cyan(), opt.label);
    }

    if allow_free {
        println!(
            "   {}  {}",
            format!("[{}]", options.len() + 1).cyan(),
            "Other (type your answer)".dimmed()
        );
    }

    println!();

    loop {
        print!("   {}  ", "Enter numbers:".bright_magenta());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().lock().read_line(&mut input)?;
        let input = input.trim();

        // Check for cancel
        if input.eq_ignore_ascii_case("cancel") || input.eq_ignore_ascii_case("quit") {
            return Ok(UserAnswerValue::Text("__CANCELLED__".to_string()));
        }

        // Parse comma-separated numbers
        let parts: Vec<&str> = input.split(',').map(|s| s.trim()).collect();
        let mut selected_ids = Vec::new();
        let mut has_free_text = false;
        let mut valid = true;

        for part in parts {
            if let Ok(num) = part.parse::<usize>() {
                if num >= 1 && num <= options.len() {
                    selected_ids.push(options[num - 1].id.clone());
                } else if allow_free && num == options.len() + 1 {
                    has_free_text = true;
                } else {
                    valid = false;
                    break;
                }
            } else {
                valid = false;
                break;
            }
        }

        if !valid || (selected_ids.is_empty() && !has_free_text) {
            println!(
                "   {}  Please enter valid numbers between 1 and {}",
                "!".yellow(),
                if allow_free {
                    options.len() + 1
                } else {
                    options.len()
                }
            );
            continue;
        }

        if has_free_text {
            // Get free text input and add to selections
            let free = ask_free_text()?;
            if let UserAnswerValue::Text(text) = free {
                if !text.is_empty() && text != "__CANCELLED__" {
                    selected_ids.push(format!("other:{}", text));
                }
            }
        }

        // Display selected options
        let selected_labels: Vec<String> = selected_ids
            .iter()
            .filter_map(|id| {
                if let Some(text) = id.strip_prefix("other:") {
                    Some(format!("Other: {}", text))
                } else {
                    options
                        .iter()
                        .find(|o| &o.id == id)
                        .map(|o| o.label.clone())
                }
            })
            .collect();

        println!(
            "   {}  Selected: {}",
            "+".bright_green(),
            selected_labels.join(", ").bright_white()
        );

        return Ok(UserAnswerValue::Multiple(selected_ids));
    }
}

/// Free text - user types any response
fn ask_free_text() -> io::Result<UserAnswerValue> {
    print!("   {}  ", "Your answer:".bright_magenta());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;
    let input = input.trim().to_string();

    // Check for cancel
    if input.eq_ignore_ascii_case("cancel") || input.eq_ignore_ascii_case("quit") {
        return Ok(UserAnswerValue::Text("__CANCELLED__".to_string()));
    }

    if input.is_empty() {
        println!("   {}  No answer provided", "~".yellow());
    } else {
        println!(
            "   {}  Answer recorded: {}",
            "+".bright_green(),
            if input.len() > 50 {
                format!("{}...", &input[..50])
            } else {
                input.clone()
            }
            .bright_white()
        );
    }

    Ok(UserAnswerValue::Text(input))
}

/// Compute a reference hash for the question (for tracking)
fn compute_question_ref(question: &UserQuestion) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    question.question.hash(&mut hasher);
    format!("q-{:08x}", hasher.finish() as u32)
}

/// Display a confirmation prompt before executing high-risk commands
pub fn confirm_high_risk(command: &str, risk_explanation: &str) -> io::Result<bool> {
    println!();
    println!(
        "{}  {}",
        "!".bright_red().bold(),
        "High-risk command requires confirmation"
            .bright_white()
            .bold()
    );
    println!();
    println!("   {}  {}", "Command:".cyan(), command.bright_white());
    println!("   {}  {}", "Risk:".yellow(), risk_explanation);
    println!();

    loop {
        print!("   {}  ", "Approve? [y/N]:".bright_magenta());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().lock().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        match input.as_str() {
            "y" | "yes" => {
                println!("   {}  Command approved", "+".bright_green());
                return Ok(true);
            }
            "n" | "no" | "" => {
                println!("   {}  Command denied", "-".dimmed());
                return Ok(false);
            }
            _ => {
                println!(
                    "   {}  Please enter 'y' for yes or 'n' for no",
                    "?".yellow()
                );
            }
        }
    }
}

/// Check if user wants to continue after a warning
pub fn confirm_continue(warning: &str) -> io::Result<bool> {
    println!();
    println!("{}  {}", "~".yellow().bold(), warning.bright_white());
    println!();

    loop {
        print!("   {}  ", "Continue? [Y/n]:".bright_magenta());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().lock().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        match input.as_str() {
            "y" | "yes" | "" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => {
                println!(
                    "   {}  Please enter 'y' for yes or 'n' for no",
                    "?".yellow()
                );
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_question_ref() {
        let q = UserQuestion::free_text("What editor do you use?", "reason");
        let ref1 = compute_question_ref(&q);
        let ref2 = compute_question_ref(&q);
        assert_eq!(ref1, ref2);
        assert!(ref1.starts_with("q-"));
    }

    #[test]
    fn test_different_questions_different_refs() {
        let q1 = UserQuestion::free_text("Question 1?", "reason");
        let q2 = UserQuestion::free_text("Question 2?", "reason");
        let ref1 = compute_question_ref(&q1);
        let ref2 = compute_question_ref(&q2);
        assert_ne!(ref1, ref2);
    }
}
