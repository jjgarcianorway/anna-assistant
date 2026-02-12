//! Meta-Learning - Anna learns how to learn and improves her own strategies.
//!
//! Philosophy: Don't just solve tasks, get better at solving tasks.
//!
//! Tracks:
//! - Which strategies work for which task types
//! - Common failure patterns and how to avoid them
//! - Self-reflection insights for continuous improvement
//! - Optimal approaches discovered through experience

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::Utc;
use tracing::{debug, info};

const STRATEGY_DB: &str = "/var/lib/anna/meta_learning/strategies.json";
const REFLECTION_LOG: &str = "/var/lib/anna/meta_learning/reflections.json";

/// A learned strategy for a type of task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    /// Type of task (e.g., "package management", "network config")
    pub task_type: String,
    /// Description of the strategy
    pub description: String,
    /// Steps that typically work
    pub steps: Vec<String>,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f32,
    /// Times this strategy was used
    pub use_count: u32,
    /// Last updated
    pub updated_at: String,
    /// Key insights
    pub insights: Vec<String>,
}

/// Self-reflection after completing a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reflection {
    /// When this reflection occurred
    pub timestamp: String,
    /// What task was completed
    pub task: String,
    /// What worked well
    pub successes: Vec<String>,
    /// What could be improved
    pub improvements: Vec<String>,
    /// New insights gained
    pub insights: Vec<String>,
    /// Confidence in these insights (0.0 to 1.0)
    pub confidence: f32,
}

/// Strategy database
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StrategyDatabase {
    pub strategies: HashMap<String, Strategy>,
    pub total_tasks: u32,
}

impl StrategyDatabase {
    /// Load from disk
    pub fn load() -> Self {
        let path = PathBuf::from(STRATEGY_DB);
        if !path.exists() {
            return Self::default();
        }

        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save to disk
    pub fn save(&self) -> anyhow::Result<()> {
        let path = PathBuf::from(STRATEGY_DB);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Record a strategy
    pub fn record_strategy(&mut self, strategy: Strategy) {
        let key = strategy.task_type.clone();

        if let Some(existing) = self.strategies.get_mut(&key) {
            // Update existing strategy
            existing.use_count += 1;
            existing.success_rate =
                (existing.success_rate * (existing.use_count as f32 - 1.0) + strategy.success_rate)
                / existing.use_count as f32;
            existing.updated_at = Utc::now().to_rfc3339();

            // Merge insights
            for insight in strategy.insights {
                if !existing.insights.contains(&insight) {
                    existing.insights.push(insight);
                }
            }
        } else {
            // New strategy
            self.strategies.insert(key, strategy);
        }

        self.total_tasks += 1;
    }

    /// Get best strategy for a task type
    pub fn get_strategy(&self, task_type: &str) -> Option<&Strategy> {
        self.strategies.get(task_type)
    }

    /// Find similar strategies
    pub fn find_similar_strategies(&self, task_description: &str) -> Vec<&Strategy> {
        let desc_lower = task_description.to_lowercase();
        let words: Vec<&str> = desc_lower.split_whitespace().collect();

        let mut matches: Vec<(&Strategy, usize)> = self.strategies
            .values()
            .filter_map(|strategy| {
                let strategy_text = format!("{} {}",
                    strategy.task_type.to_lowercase(),
                    strategy.description.to_lowercase()
                );

                let match_count = words.iter()
                    .filter(|word| strategy_text.contains(*word))
                    .count();

                if match_count > words.len() / 3 {
                    Some((strategy, match_count))
                } else {
                    None
                }
            })
            .collect();

        matches.sort_by(|a, b| {
            // Sort by match count, then by success rate
            match b.1.cmp(&a.1) {
                std::cmp::Ordering::Equal => {
                    b.0.success_rate.partial_cmp(&a.0.success_rate).unwrap()
                }
                other => other,
            }
        });

        matches.into_iter()
            .take(3)
            .map(|(s, _)| s)
            .collect()
    }
}

/// Reflection log
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReflectionLog {
    pub reflections: Vec<Reflection>,
}

impl ReflectionLog {
    /// Load from disk
    pub fn load() -> Self {
        let path = PathBuf::from(REFLECTION_LOG);
        if !path.exists() {
            return Self::default();
        }

        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save to disk
    pub fn save(&self) -> anyhow::Result<()> {
        let path = PathBuf::from(REFLECTION_LOG);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Add a reflection
    pub fn add(&mut self, reflection: Reflection) {
        self.reflections.push(reflection);

        // Keep last 1000 reflections
        if self.reflections.len() > 1000 {
            self.reflections.drain(0..self.reflections.len() - 1000);
        }
    }

    /// Get recent insights
    pub fn recent_insights(&self, count: usize) -> Vec<String> {
        self.reflections
            .iter()
            .rev()
            .take(count * 3) // Take more reflections to get enough insights
            .flat_map(|r| r.insights.iter().cloned())
            .take(count)
            .collect()
    }
}

/// Perform self-reflection after completing a task
pub async fn reflect_on_task(
    model: &str,
    task: &str,
    execution_log: &str,
    success: bool,
) -> anyhow::Result<Reflection> {
    let prompt = format!(
        r#"Reflect on this completed task and identify what worked and what could improve.

TASK: "{}"

EXECUTION LOG:
{}

SUCCESS: {}

Analyze this critically:
1. What specific actions worked well?
2. What could have been done more efficiently?
3. What new insights did you gain?
4. If you had to do this again, what would you do differently?

Be specific and actionable. Focus on learnings that apply to future tasks.

Respond in JSON:
{{
    "successes": ["success 1", "success 2"],
    "improvements": ["improvement 1", "improvement 2"],
    "insights": ["insight 1", "insight 2"],
    "confidence": 0.8
}}
"#,
        task, execution_log, success
    );

    let response = crate::ollama::chat_with_timeout(model, &prompt, 60).await?;
    let json_str = extract_json(&response)?;

    let mut reflection: Reflection = serde_json::from_str(&json_str)?;
    reflection.timestamp = Utc::now().to_rfc3339();
    reflection.task = task.to_string();

    info!(
        "Self-reflection complete: {} successes, {} improvements, {} insights",
        reflection.successes.len(),
        reflection.improvements.len(),
        reflection.insights.len()
    );

    // Save reflection
    let mut log = ReflectionLog::load();
    log.add(reflection.clone());
    if let Err(e) = log.save() {
        debug!("Failed to save reflection log: {}", e);
    }

    Ok(reflection)
}

/// Learn a strategy from successful task completion
pub fn learn_strategy(
    task_type: String,
    description: String,
    steps: Vec<String>,
    success: bool,
    insights: Vec<String>,
) {
    let mut db = StrategyDatabase::load();

    let strategy = Strategy {
        task_type,
        description,
        steps,
        success_rate: if success { 1.0 } else { 0.0 },
        use_count: 1,
        updated_at: Utc::now().to_rfc3339(),
        insights,
    };

    db.record_strategy(strategy);

    if let Err(e) = db.save() {
        debug!("Failed to save strategy database: {}", e);
    } else {
        info!("Strategy learned and saved");
    }
}

/// Get guidance from past strategies for a new task
pub fn get_strategic_guidance(task_description: &str) -> String {
    let db = StrategyDatabase::load();
    let similar = db.find_similar_strategies(task_description);

    if similar.is_empty() {
        return String::new();
    }

    let mut guidance = String::from("\n## Strategic Guidance from Past Experience:\n");

    for (i, strategy) in similar.iter().enumerate() {
        guidance.push_str(&format!(
            "\n{}. {} (success rate: {:.0}%, used {} times)\n",
            i + 1,
            strategy.description,
            strategy.success_rate * 100.0,
            strategy.use_count
        ));

        if !strategy.insights.is_empty() {
            guidance.push_str("   Key insights:\n");
            for insight in &strategy.insights {
                guidance.push_str(&format!("   - {}\n", insight));
            }
        }
    }

    // Add recent cross-task insights
    let log = ReflectionLog::load();
    let recent_insights = log.recent_insights(5);
    if !recent_insights.is_empty() {
        guidance.push_str("\n   Recent learnings:\n");
        for insight in recent_insights {
            guidance.push_str(&format!("   - {}\n", insight));
        }
    }

    guidance
}

/// Extract JSON from LLM response
fn extract_json(response: &str) -> anyhow::Result<String> {
    if let Some(start) = response.find("```json") {
        if let Some(end) = response[start..].find("```") {
            let json_start = start + 7;
            let json_end = start + end;
            return Ok(response[json_start..json_end].trim().to_string());
        }
    }

    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            if end > start {
                return Ok(response[start..=end].to_string());
            }
        }
    }

    anyhow::bail!("No valid JSON found in response")
}
