//! Recipe Planner - Planner/Critic LLM loop for command recipe generation
//!
//! This module implements the controlled two-step dialogue:
//! 1. Planner LLM: Generates command recipes based on question + docs + telemetry
//! 2. Critic LLM: Validates recipes against docs and safety rules
//!
//! The loop runs with a hard limit (max 3 iterations) to prevent infinite spinning.
//! If validation fails after limit, Anna provides manual explanation instead.

use crate::command_recipe::{CriticResult, Recipe, SafetyPolicy};
use crate::llm::{LlmClient, LlmConfig, LlmPrompt};
use crate::template_library::TemplateLibrary;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Maximum planner/critic iterations before giving up
const MAX_PLANNING_ITERATIONS: usize = 3;

/// Planner/Critic orchestrator
pub struct RecipePlanner {
    /// Template library for common operations
    template_library: TemplateLibrary,

    /// Safety policy for command validation
    safety_policy: SafetyPolicy,

    /// Doc retrieval system (placeholder for future RAG)
    doc_retriever: DocRetriever,

    /// LLM configuration for planner and critic
    llm_config: LlmConfig,
}

impl RecipePlanner {
    pub fn new(llm_config: LlmConfig) -> Self {
        Self {
            template_library: TemplateLibrary::new(),
            safety_policy: SafetyPolicy::default(),
            doc_retriever: DocRetriever::new(),
            llm_config,
        }
    }

    /// Plan and validate a recipe for the given question
    ///
    /// This runs the planner/critic loop up to MAX_PLANNING_ITERATIONS times.
    /// Returns: Ok(Recipe) if approved, Err if validation fails
    pub async fn plan_recipe(
        &self,
        question: &str,
        telemetry_summary: &str,
    ) -> Result<PlanningResult> {
        let mut iteration = 0;
        let mut planner_context = PlannerContext {
            question: question.to_string(),
            telemetry: telemetry_summary.to_string(),
            retrieved_docs: vec![],
            available_templates: self.template_library.list_templates(),
            previous_rejection: None,
        };

        // Retrieve relevant docs (Phase 2: implement real RAG)
        planner_context.retrieved_docs = self.doc_retriever.retrieve_docs(question).await?;

        loop {
            iteration += 1;

            if iteration > MAX_PLANNING_ITERATIONS {
                return Ok(PlanningResult::Failed {
                    reason: format!(
                        "Could not create safe recipe after {} iterations",
                        MAX_PLANNING_ITERATIONS
                    ),
                    explanation: self.generate_manual_explanation(&planner_context).await?,
                });
            }

            // Step 1: Planner generates recipe
            let recipe = self.call_planner_llm(&planner_context).await?;

            // Static validation before critic
            if let Err(e) = self.validate_recipe_safety(&recipe) {
                planner_context.previous_rejection = Some(CriticResult {
                    approved: false,
                    reasoning: "Static safety validation failed".to_string(),
                    issues: vec![e],
                    corrections: vec!["Review safety policy and choose safer commands".to_string()],
                });
                continue;
            }

            // Step 2: Critic validates recipe
            let critic_result = self
                .call_critic_llm(&recipe, &planner_context)
                .await?;

            if critic_result.approved {
                // Success! Return approved recipe
                let mut final_recipe = recipe;
                final_recipe.critic_approval = Some(critic_result);
                return Ok(PlanningResult::Success(final_recipe));
            } else {
                // Rejected - loop with feedback
                planner_context.previous_rejection = Some(critic_result);
            }
        }
    }

    /// Validate recipe against safety policy
    fn validate_recipe_safety(&self, recipe: &Recipe) -> Result<(), String> {
        for step in &recipe.steps {
            step.validate_against_policy(&self.safety_policy)?;
        }
        Ok(())
    }

    /// Call planner LLM to generate recipe
    async fn call_planner_llm(&self, context: &PlannerContext) -> Result<Recipe> {
        // Build planner prompt
        let prompt_text = self.build_planner_prompt(context);

        // Create LLM client
        let client = LlmClient::from_config(&self.llm_config)
            .context("Failed to create LLM client for planner")?;

        // Create LLM prompt
        let prompt = LlmPrompt {
            system: "You are a command recipe planner for Arch Linux. Your role is to generate safe, validated command sequences based on documentation.".to_string(),
            user: prompt_text,
            conversation_history: None,
        };

        // Call LLM and get response
        let response = client
            .chat(&prompt)
            .map_err(|e| anyhow::anyhow!("Planner LLM call failed: {}", e))?;

        // Parse JSON response into Recipe
        let recipe: Recipe = serde_json::from_str(&response.text)
            .context("Failed to parse planner LLM response as Recipe JSON")?;

        Ok(recipe)
    }

    /// Call critic LLM to validate recipe
    async fn call_critic_llm(
        &self,
        recipe: &Recipe,
        context: &PlannerContext,
    ) -> Result<CriticResult> {
        // Build critic prompt
        let prompt_text = self.build_critic_prompt(recipe, context);

        // Create LLM client
        let client = LlmClient::from_config(&self.llm_config)
            .context("Failed to create LLM client for critic")?;

        // Create LLM prompt
        let prompt = LlmPrompt {
            system: "You are a safety critic for Arch Linux command recipes. Your role is to validate that proposed commands are safe, well-documented, and address the user's question correctly.".to_string(),
            user: prompt_text,
            conversation_history: None,
        };

        // Call LLM and get response
        let response = client
            .chat(&prompt)
            .map_err(|e| anyhow::anyhow!("Critic LLM call failed: {}", e))?;

        // Parse JSON response into CriticResult
        let result: CriticResult = serde_json::from_str(&response.text)
            .context("Failed to parse critic LLM response as CriticResult JSON")?;

        Ok(result)
    }

    /// Build planner LLM prompt
    fn build_planner_prompt(&self, context: &PlannerContext) -> String {
        let mut prompt = String::new();

        prompt.push_str("# Role: Command Recipe Planner\n\n");
        prompt.push_str("Generate a JSON command recipe to answer the user's question.\n\n");

        prompt.push_str("## User Question\n");
        prompt.push_str(&context.question);
        prompt.push_str("\n\n");

        prompt.push_str("## System Telemetry\n");
        prompt.push_str(&context.telemetry);
        prompt.push_str("\n\n");

        if !context.retrieved_docs.is_empty() {
            prompt.push_str("## Retrieved Documentation\n");
            for doc in &context.retrieved_docs {
                prompt.push_str("- ");
                prompt.push_str(doc);
                prompt.push_str("\n");
            }
            prompt.push_str("\n");
        }

        if !context.available_templates.is_empty() {
            prompt.push_str("## Available Templates\n");
            for template_id in &context.available_templates {
                if let Some(template) = self.template_library.get(template_id) {
                    prompt.push_str(&format!("- {}: {}\n", template.id, template.description));
                }
            }
            prompt.push_str("\n");
        }

        prompt.push_str("## Instructions\n");
        prompt.push_str("1. Use templates where possible (safer and tested)\n");
        prompt.push_str("2. Cite documentation sources for each command\n");
        prompt.push_str("3. For write operations, include rollback commands\n");
        prompt.push_str("4. Add validation patterns to check command output\n");
        prompt.push_str("5. Generate JSON conforming to Recipe schema\n\n");

        prompt.push_str("## Output Format\n");
        prompt.push_str("You MUST respond ONLY with valid JSON conforming to this Recipe schema:\n");
        prompt.push_str("{\n");
        prompt.push_str("  \"question\": \"<original question>\",\n");
        prompt.push_str("  \"steps\": [],\n");
        prompt.push_str("  \"overall_safety\": \"safe\" | \"needs_confirmation\" | \"blocked\",\n");
        prompt.push_str("  \"all_read_only\": true | false,\n");
        prompt.push_str("  \"wiki_sources\": [\"<url1>\", \"<url2>\"],\n");
        prompt.push_str("  \"summary\": \"<what this recipe will do>\"\n");
        prompt.push_str("}\n\n");

        if let Some(rejection) = &context.previous_rejection {
            prompt.push_str("## Previous Attempt Rejected\n");
            prompt.push_str(&format!("Reason: {}\n", rejection.reasoning));
            prompt.push_str("Issues:\n");
            for issue in &rejection.issues {
                prompt.push_str(&format!("- {}\n", issue));
            }
            prompt.push_str("Suggested corrections:\n");
            for correction in &rejection.corrections {
                prompt.push_str(&format!("- {}\n", correction));
            }
            prompt.push_str("\n");
        }

        prompt.push_str("Generate the recipe now:\n");
        prompt
    }

    /// Build critic LLM prompt
    fn build_critic_prompt(&self, recipe: &Recipe, context: &PlannerContext) -> String {
        let mut prompt = String::new();

        prompt.push_str("# Role: Recipe Critic\n\n");
        prompt.push_str("Validate the proposed command recipe against documentation and safety rules.\n\n");

        prompt.push_str("## Original Question\n");
        prompt.push_str(&context.question);
        prompt.push_str("\n\n");

        prompt.push_str("## Retrieved Documentation\n");
        for doc in &context.retrieved_docs {
            prompt.push_str("- ");
            prompt.push_str(doc);
            prompt.push_str("\n");
        }
        prompt.push_str("\n");

        prompt.push_str("## Proposed Recipe\n");
        prompt.push_str(&serde_json::to_string_pretty(recipe).unwrap_or_default());
        prompt.push_str("\n\n");

        prompt.push_str("## Validation Checklist\n");
        prompt.push_str("1. Commands match documentation in spirit\n");
        prompt.push_str("2. Respects Anna safety policies\n");
        prompt.push_str("3. Actually addresses the user question\n");
        prompt.push_str("4. Validation patterns are reasonable\n");
        prompt.push_str("5. Write operations have rollback paths\n\n");

        prompt.push_str("## Output Format\n");
        prompt.push_str("You MUST respond ONLY with valid JSON conforming to this CriticResult schema:\n");
        prompt.push_str("{\n");
        prompt.push_str("  \"approved\": true | false,\n");
        prompt.push_str("  \"reasoning\": \"<why you approved/rejected>\",\n");
        prompt.push_str("  \"issues\": [\"<issue1>\", \"<issue2>\"],\n");
        prompt.push_str("  \"corrections\": [\"<suggestion1>\", \"<suggestion2>\"]\n");
        prompt.push_str("}\n\n");
        prompt
    }

    /// Generate manual explanation when planning fails
    async fn generate_manual_explanation(&self, context: &PlannerContext) -> Result<String> {
        // Fallback: provide explanation without executable recipe
        Ok(format!(
            "I cannot create a safe, verified recipe for '{}' automatically. \
             Here is what I know from the documentation:\n\n{}",
            context.question,
            context.retrieved_docs.join("\n")
        ))
    }
}

// Removed Default impl - RecipePlanner requires LlmConfig to be specified

/// Planner context - all information for planning
#[derive(Debug, Clone)]
struct PlannerContext {
    /// User's question
    question: String,

    /// System telemetry summary
    telemetry: String,

    /// Retrieved Arch Wiki / doc snippets
    retrieved_docs: Vec<String>,

    /// Available template IDs
    available_templates: Vec<String>,

    /// Previous critic rejection (if any)
    previous_rejection: Option<CriticResult>,
}

/// Result of planning process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanningResult {
    /// Recipe approved and ready to execute
    Success(Recipe),

    /// Planning failed after max iterations
    Failed {
        reason: String,
        explanation: String,
    },
}

/// Doc retrieval system - placeholder for future RAG
pub struct DocRetriever {
    // Future: embedding index, vector DB, etc.
}

impl DocRetriever {
    pub fn new() -> Self {
        Self {}
    }

    /// Retrieve relevant docs for question (placeholder)
    ///
    /// Future implementation will:
    /// - Parse question to extract topics
    /// - Query local Arch Wiki mirror via embeddings
    /// - Return top N relevant doc chunks
    pub async fn retrieve_docs(&self, question: &str) -> Result<Vec<String>> {
        // Placeholder: hardcoded doc snippets for common questions
        let docs = if question.contains("swap") {
            vec![
                "https://wiki.archlinux.org/title/Swap - Use swapon --show to check swap status"
                    .to_string(),
                "https://wiki.archlinux.org/title/Swap - cat /proc/swaps shows swap devices"
                    .to_string(),
            ]
        } else if question.contains("GPU") || question.contains("VRAM") {
            vec![
                "https://wiki.archlinux.org/title/NVIDIA - Use nvidia-smi to check GPU memory"
                    .to_string(),
            ]
        } else if question.contains("vim") && question.contains("syntax") {
            vec![
                "https://wiki.archlinux.org/title/Vim - Add 'syntax on' to ~/.vimrc"
                    .to_string(),
            ]
        } else {
            vec!["No specific documentation found. Use general Arch Linux commands.".to_string()]
        };

        Ok(docs)
    }
}

impl Default for DocRetriever {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_doc_retrieval() {
        let retriever = DocRetriever::new();
        let docs = retriever
            .retrieve_docs("Do I have swap?")
            .await
            .unwrap();

        assert!(!docs.is_empty());
        assert!(docs[0].contains("swap"));
    }

    #[tokio::test]
    async fn test_planner_creation() {
        // Create dummy LLM config for testing
        let llm_config = LlmConfig::local("http://localhost:11434/v1", "llama3.1:8b");
        let planner = RecipePlanner::new(llm_config);
        assert!(!planner.template_library.list_templates().is_empty());
    }
}
