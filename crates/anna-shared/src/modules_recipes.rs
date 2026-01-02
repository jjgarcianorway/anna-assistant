//! Recipe-related module declarations
//! All recipe modules for self-learning and execution

// Core recipe modules
#[path = "recipe/mod.rs"]
pub mod recipe;
#[path = "recipe_store/mod.rs"]
pub mod recipe_store;
#[path = "recipe_feedback.rs"]
pub mod recipe_feedback;

// Recipe engine modules (v0.0.406-418)
#[path = "recipe_candidate.rs"]
pub mod recipe_candidate;
#[path = "recipe_converter/mod.rs"]
pub mod recipe_converter;
#[path = "recipe_eligibility.rs"]
pub mod recipe_eligibility;
#[path = "recipe_engine/mod.rs"]
pub mod recipe_engine;
#[path = "recipe_exec_helpers.rs"]
pub mod recipe_exec_helpers;
#[path = "recipe_executor/mod.rs"]
pub mod recipe_executor;
#[path = "recipe_extractor/mod.rs"]
pub mod recipe_extractor;
#[path = "recipe_fast_path.rs"]
pub mod recipe_fast_path;
#[path = "recipe_file/mod.rs"]
pub mod recipe_file;
#[path = "recipe_index.rs"]
pub mod recipe_index;
#[path = "recipe_learner/mod.rs"]
pub mod recipe_learner;
#[path = "recipe_learning.rs"]
pub mod recipe_learning;
#[path = "recipe_matcher/mod.rs"]
pub mod recipe_matcher;
#[path = "recipe_matcher_v2.rs"]
pub mod recipe_matcher_v2;
#[path = "recipe_runtime/mod.rs"]
pub mod recipe_runtime;
#[path = "recipe_schema/mod.rs"]
pub mod recipe_schema;
#[path = "recipe_stats.rs"]
pub mod recipe_stats;
#[path = "recipe_storage.rs"]
pub mod recipe_storage;
#[path = "recipe_store_v2/mod.rs"]
pub mod recipe_store_v2;
#[path = "recipe_telemetry/mod.rs"]
pub mod recipe_telemetry;
#[path = "recipe_templates.rs"]
pub mod recipe_templates;

// Recipe versions
#[path = "recipe_v2/mod.rs"]
pub mod recipe_v2; // v0.0.420: Clean learning engine
#[path = "recipe_v3/mod.rs"]
pub mod recipe_v3; // v0.0.423: Safe learning/execution engine

// Specialized recipe modules
#[path = "seed_recipes/mod.rs"]
pub mod seed_recipes; // v0.0.418: Initial seed recipes
#[path = "learned_recipes/mod.rs"]
pub mod learned_recipes; // v0.0.416: Self-learning recipe schema
#[path = "recipe_similarity.rs"]
pub mod recipe_similarity; // v0.0.282: LLM-based recipe similarity scoring

// Domain-specific recipe modules
#[path = "config_seed_recipes.rs"]
pub mod config_seed_recipes; // v0.0.264: Seed recipes for editor configs
#[path = "cron_recipes/mod.rs"]
pub mod cron_recipes; // v0.0.234
#[path = "database_recipes/mod.rs"]
pub mod database_recipes; // v0.0.461: Database backup/restore recipes
#[path = "kubernetes_recipes/mod.rs"]
pub mod kubernetes_recipes; // v0.0.459: Kubernetes pod/deployment recipes
#[path = "docker_recipes/mod.rs"]
pub mod docker_recipes; // v0.0.235
#[path = "editor_recipe_data.rs"]
pub mod editor_recipe_data;
#[path = "editor_recipes.rs"]
pub mod editor_recipes;
#[path = "git_recipes/mod.rs"]
pub mod git_recipes;
#[path = "network_recipes/mod.rs"]
pub mod network_recipes; // v0.0.462: Network troubleshooting recipes
#[path = "package_recipes/mod.rs"]
pub mod package_recipes;
#[path = "service_recipes/mod.rs"]
pub mod service_recipes;
#[path = "shell_recipes/mod.rs"]
pub mod shell_recipes;
#[path = "ssh_recipes/mod.rs"]
pub mod ssh_recipes;
#[path = "systemd_recipes/mod.rs"]
pub mod systemd_recipes; // v0.0.233
#[path = "webserver_recipes/mod.rs"]
pub mod webserver_recipes; // v0.0.460: Nginx/Apache configuration recipes
#[path = "desktop_recipes/mod.rs"]
pub mod desktop_recipes; // v0.0.257: Desktop configuration recipes

// Recipe stats and display
#[path = "recipe_stats_display/mod.rs"]
pub mod recipe_stats_display; // v0.0.490: Recipe statistics display
