//! Anna vs Claude Battle Test
//! 100 questions from trivial to insane - let's see who wins!

use std::fs;
use std::time::{Duration, Instant};
use tokio::process::Command;

#[derive(Debug, Clone)]
struct TestQuestion {
    id: usize,
    category: String,
    question: String,
}

#[derive(Debug, Clone)]
struct TestResult {
    question: TestQuestion,
    anna_response: String,
    anna_time_ms: u64,
    anna_success: bool,
    anna_error: Option<String>,
}

/// Load questions from the test file
fn load_questions() -> Vec<TestQuestion> {
    let content = fs::read_to_string("tests/anna_vs_claude_100.txt")
        .expect("Could not read test file");

    let mut questions = Vec::new();
    let mut current_category = String::new();
    let mut question_id = 0;

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and headers
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Detect category headers
        if line.starts_with("##") {
            current_category = line
                .trim_start_matches("##")
                .split('-')
                .next()
                .unwrap_or("Unknown")
                .trim()
                .to_string();
            continue;
        }

        // Parse question (format: "1. Question text")
        if let Some(dot_pos) = line.find('.') {
            if dot_pos > 0 && line[..dot_pos].chars().all(|c| c.is_numeric()) {
                question_id += 1;
                let question_text = line[dot_pos + 1..].trim().to_string();

                questions.push(TestQuestion {
                    id: question_id,
                    category: current_category.clone(),
                    question: question_text,
                });
            }
        }
    }

    println!("Loaded {} questions", questions.len());
    questions
}

/// Ask Anna a question via annactl
async fn ask_anna(question: &str) -> Result<(String, u64), String> {
    let start = Instant::now();

    let output = Command::new("./target/release/annactl")
        .arg(question)
        .output()
        .await
        .map_err(|e| format!("Failed to run annactl: {}", e))?;

    let elapsed = start.elapsed().as_millis() as u64;

    if output.status.success() {
        let response = String::from_utf8_lossy(&output.stdout).to_string();
        Ok((response, elapsed))
    } else {
        let error = String::from_utf8_lossy(&output.stderr).to_string();
        Err(error)
    }
}

/// Evaluate if Anna's response seems successful
fn evaluate_response(question: &str, response: &str) -> bool {
    let response_lower = response.to_lowercase();

    // Signs of failure
    if response_lower.contains("error:")
        || response_lower.contains("failed")
        || response_lower.contains("could not")
        || response_lower.contains("timeout")
        || response_lower.is_empty() {
        return false;
    }

    // Signs of success - contains actual data
    let has_data = response.lines().count() > 2
        || response.len() > 50
        || response.contains("✓")
        || response.contains("success");

    has_data
}

/// Run the battle test
#[tokio::main]
async fn main() {
    println!("🥊 ANNA VS CLAUDE: 100 QUESTION BATTLE TEST 🥊\n");
    println!("Loading questions...\n");

    let questions = load_questions();
    let total = questions.len();

    println!("Starting test with {} questions\n", total);
    println!("Each question will be asked to Anna");
    println!("We'll track: success rate, response time, answer quality\n");
    println!("{}", "=".repeat(80));
    println!();

    let mut results = Vec::new();
    let mut successes = 0;
    let mut failures = 0;
    let mut total_time = 0u64;

    let mut current_category = String::new();

    for question in questions {
        // Print category header
        if question.category != current_category {
            println!("\n{}", "=".repeat(80));
            println!("📁 CATEGORY: {}", question.category);
            println!("{}\n", "=".repeat(80));
            current_category = question.category.clone();
        }

        print!("Q{:3}: {} ... ", question.id, question.question);
        std::io::Write::flush(&mut std::io::stdout()).ok();

        match ask_anna(&question.question).await {
            Ok((response, time_ms)) => {
                total_time += time_ms;
                let success = evaluate_response(&question.question, &response);

                if success {
                    successes += 1;
                    println!("✓ ({} ms)", time_ms);
                } else {
                    failures += 1;
                    println!("✗ (response unclear, {} ms)", time_ms);
                }

                results.push(TestResult {
                    question: question.clone(),
                    anna_response: response,
                    anna_time_ms: time_ms,
                    anna_success: success,
                    anna_error: None,
                });
            }
            Err(error) => {
                failures += 1;
                println!("✗ ERROR");

                results.push(TestResult {
                    question: question.clone(),
                    anna_response: String::new(),
                    anna_time_ms: 0,
                    anna_success: false,
                    anna_error: Some(error),
                });
            }
        }

        // Small delay to avoid overwhelming the system
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Generate report
    println!("\n{}", "=".repeat(80));
    println!("📊 FINAL RESULTS");
    println!("{}\n", "=".repeat(80));

    let success_rate = (successes as f64 / total as f64) * 100.0;
    let avg_time = if total > 0 { total_time / total as u64 } else { 0 };

    println!("Total Questions:    {}", total);
    println!("Successful:         {} ({:.1}%)", successes, success_rate);
    println!("Failed:             {} ({:.1}%)", failures, (failures as f64 / total as f64) * 100.0);
    println!("Average Time:       {} ms", avg_time);
    println!("Total Time:         {:.2} seconds", total_time as f64 / 1000.0);
    println!();

    // Performance rating
    if success_rate >= 90.0 {
        println!("🏆 VERDICT: Anna is EXCELLENT!");
    } else if success_rate >= 75.0 {
        println!("✅ VERDICT: Anna is GOOD!");
    } else if success_rate >= 50.0 {
        println!("⚠️  VERDICT: Anna is DECENT - needs improvement");
    } else {
        println!("❌ VERDICT: Anna needs serious work");
    }

    // Save detailed results
    println!("\n📄 Saving detailed results to tests/battle_results.json...");
    let json = serde_json::to_string_pretty(&results).unwrap();
    fs::write("tests/battle_results.json", json).ok();

    println!("✨ Test complete! Check tests/battle_results.json for details.");
}
