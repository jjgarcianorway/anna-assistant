//! Skill Store Implementation v0.40.0
//!
//! JSON file-based persistent storage for skills.
//! Location: /var/lib/anna/knowledge/skills/

use super::schema::{AnnaLevel, Skill, SkillQuery, SkillStats, SystemStats, SKILL_SCHEMA_VERSION};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// Default skills directory
pub const SKILLS_DIR: &str = "/var/lib/anna/knowledge/skills";

/// Skill store backed by JSON files
pub struct SkillStore {
    /// Directory containing skill files
    skills_dir: PathBuf,
    /// In-memory cache of skills
    cache: Arc<RwLock<HashMap<String, Skill>>>,
    /// System-level statistics
    system_stats: Arc<RwLock<SystemStats>>,
}

impl SkillStore {
    /// Open or create the skill store at the default location
    pub fn open_default() -> Result<Self> {
        let skills_dir = Self::default_path();
        Self::open(&skills_dir)
    }

    /// Open or create the skill store at a specific path
    pub fn open(path: &PathBuf) -> Result<Self> {
        // Ensure directory exists
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create skills directory: {:?}", path))?;

        let store = Self {
            skills_dir: path.clone(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            system_stats: Arc::new(RwLock::new(SystemStats::default())),
        };

        // Load existing skills into cache
        store.load_all()?;

        // Load system stats
        store.load_system_stats()?;

        Ok(store)
    }

    /// Get the default skills directory path
    pub fn default_path() -> PathBuf {
        // Try system path first, fall back to user path
        let system_path = PathBuf::from(SKILLS_DIR);
        if system_path.parent().map(|p| p.exists()).unwrap_or(false) {
            return system_path;
        }

        // User path
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("~/.local/share"))
            .join("anna")
            .join("knowledge")
            .join("skills")
    }

    /// Load all skills from disk into cache
    fn load_all(&self) -> Result<()> {
        let mut cache = self.cache.write().unwrap();

        if !self.skills_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&self.skills_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Some(filename) = path.file_stem() {
                    if filename.to_string_lossy() == "_system_stats" {
                        continue; // Skip system stats file
                    }
                }

                match fs::read_to_string(&path) {
                    Ok(content) => match serde_json::from_str::<Skill>(&content) {
                        Ok(skill) => {
                            cache.insert(skill.skill_id.clone(), skill);
                        }
                        Err(e) => {
                            eprintln!("  !  Failed to parse skill {:?}: {}", path, e);
                        }
                    },
                    Err(e) => {
                        eprintln!("  !  Failed to read skill {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Load system stats from disk
    fn load_system_stats(&self) -> Result<()> {
        let stats_path = self.skills_dir.join("_system_stats.json");
        if stats_path.exists() {
            let content = fs::read_to_string(&stats_path)?;
            if let Ok(stats) = serde_json::from_str::<SystemStats>(&content) {
                let mut system_stats = self.system_stats.write().unwrap();
                *system_stats = stats;
            }
        }
        Ok(())
    }

    /// Save system stats to disk
    fn save_system_stats(&self) -> Result<()> {
        let stats_path = self.skills_dir.join("_system_stats.json");
        let stats = self.system_stats.read().unwrap();
        let json = serde_json::to_string_pretty(&*stats)?;
        fs::write(stats_path, json)?;
        Ok(())
    }

    /// Get the file path for a skill
    fn skill_path(&self, skill_id: &str) -> PathBuf {
        // Sanitize skill_id for filesystem
        let safe_id = skill_id.replace(['/', '\\', ':'], "_");
        self.skills_dir.join(format!("{}.json", safe_id))
    }

    /// Save a skill to disk
    fn save_skill(&self, skill: &Skill) -> Result<()> {
        let path = self.skill_path(&skill.skill_id);
        let json = serde_json::to_string_pretty(skill)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Insert or update a skill
    pub fn upsert(&self, skill: &Skill) -> Result<bool> {
        let mut cache = self.cache.write().unwrap();
        let is_new = !cache.contains_key(&skill.skill_id);
        cache.insert(skill.skill_id.clone(), skill.clone());
        drop(cache); // Release lock before I/O

        self.save_skill(skill)?;
        Ok(is_new)
    }

    /// Get a skill by ID
    pub fn get(&self, skill_id: &str) -> Option<Skill> {
        let cache = self.cache.read().unwrap();
        cache.get(skill_id).cloned()
    }

    /// List all skills
    pub fn list(&self) -> Vec<Skill> {
        let cache = self.cache.read().unwrap();
        cache.values().cloned().collect()
    }

    /// Query skills with filters
    pub fn query(&self, filter: &SkillQuery) -> Vec<Skill> {
        let cache = self.cache.read().unwrap();
        let mut results: Vec<_> = cache
            .values()
            .filter(|skill| {
                // Filter by intent
                if let Some(ref intent) = filter.intent {
                    if !skill.intent.eq_ignore_ascii_case(intent) {
                        return false;
                    }
                }

                // Filter by reliability
                if let Some(min_rel) = filter.min_reliability {
                    if skill.stats.reliability_score < min_rel {
                        return false;
                    }
                }

                // Filter by builtin
                if let Some(builtin) = filter.builtin_only {
                    if skill.builtin != builtin {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // Sort by reliability (highest first)
        results.sort_by(|a, b| {
            b.stats
                .reliability_score
                .partial_cmp(&a.stats.reliability_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply limit
        if let Some(limit) = filter.limit {
            results.truncate(limit);
        }

        results
    }

    /// Find skills matching a question
    pub fn find_by_question(&self, question: &str, min_score: f64) -> Vec<(Skill, f64)> {
        let cache = self.cache.read().unwrap();
        let mut matches: Vec<_> = cache
            .values()
            .map(|skill| {
                let score = skill.match_score(question);
                (skill.clone(), score)
            })
            .filter(|(_, score)| *score >= min_score)
            .collect();

        // Sort by score (highest first)
        matches.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        matches
    }

    /// Find skills by intent
    pub fn find_by_intent(&self, intent: &str) -> Vec<Skill> {
        self.query(&SkillQuery::new().intent(intent))
    }

    /// Record a successful skill execution
    pub fn record_success(&self, skill_id: &str, latency_ms: u64) -> Result<()> {
        let mut cache = self.cache.write().unwrap();
        if let Some(skill) = cache.get_mut(skill_id) {
            skill.stats.record_success(latency_ms);
            let skill_clone = skill.clone();
            drop(cache);
            self.save_skill(&skill_clone)?;
        }

        // Update system stats
        {
            let mut stats = self.system_stats.write().unwrap();
            stats.skill_answers += 1;
            stats.total_questions += 1;
            // Update average latency
            let total = stats.total_questions;
            stats.avg_latency_ms = if total == 1 {
                latency_ms
            } else {
                (stats.avg_latency_ms * (total - 1) + latency_ms) / total
            };
        }
        self.save_system_stats()?;

        Ok(())
    }

    /// Record a failed skill execution
    pub fn record_failure(&self, skill_id: &str) -> Result<()> {
        let mut cache = self.cache.write().unwrap();
        if let Some(skill) = cache.get_mut(skill_id) {
            skill.stats.record_failure();
            let skill_clone = skill.clone();
            drop(cache);
            self.save_skill(&skill_clone)?;
        }

        // Update system stats
        {
            let mut stats = self.system_stats.write().unwrap();
            stats.replan_count += 1;
        }
        self.save_system_stats()?;

        Ok(())
    }

    /// Record a re-plan (skill not found or failed)
    pub fn record_replan(&self) -> Result<()> {
        let mut stats = self.system_stats.write().unwrap();
        stats.replan_count += 1;
        stats.total_questions += 1;
        drop(stats);
        self.save_system_stats()
    }

    /// Record a clarification request
    pub fn record_clarification(&self) -> Result<()> {
        let mut stats = self.system_stats.write().unwrap();
        stats.clarification_count += 1;
        drop(stats);
        self.save_system_stats()
    }

    /// Delete a skill
    pub fn delete(&self, skill_id: &str) -> Result<bool> {
        let mut cache = self.cache.write().unwrap();
        let existed = cache.remove(skill_id).is_some();
        drop(cache);

        if existed {
            let path = self.skill_path(skill_id);
            if path.exists() {
                fs::remove_file(path)?;
            }
        }

        Ok(existed)
    }

    /// Get the count of skills
    pub fn count(&self) -> usize {
        let cache = self.cache.read().unwrap();
        cache.len()
    }

    /// Get average reliability across all skills
    pub fn average_reliability(&self) -> f64 {
        let cache = self.cache.read().unwrap();
        if cache.is_empty() {
            return 0.0;
        }
        let sum: f64 = cache.values().map(|s| s.stats.reliability_score).sum();
        sum / cache.len() as f64
    }

    /// Get Anna's current level
    pub fn get_level(&self) -> AnnaLevel {
        let skill_count = self.count();
        let avg_reliability = self.average_reliability();
        let stats = self.system_stats.read().unwrap();
        AnnaLevel::from_stats(skill_count, avg_reliability, stats.total_questions)
    }

    /// Get system statistics
    pub fn get_system_stats(&self) -> SystemStats {
        let mut stats = self.system_stats.read().unwrap().clone();
        stats.level = Some(self.get_level());
        stats
    }

    /// Get skills directory path
    pub fn path(&self) -> &PathBuf {
        &self.skills_dir
    }

    /// Create a skill from a successful command execution
    pub fn learn_from_success(
        &self,
        intent: &str,
        description: &str,
        command_parts: &[String],
        question: &str,
        latency_ms: u64,
    ) -> Result<Skill> {
        // Generate skill ID from intent
        let skill_id = format!("{}.learned_{}", intent, &uuid::Uuid::new_v4().to_string()[..8]);

        // Join command parts back into template
        let command_template = command_parts.join(" ");

        let mut skill = Skill::new(&skill_id, intent, description, &command_template);
        skill.question_examples.push(question.to_string());
        skill.stats.record_success(latency_ms);

        self.upsert(&skill)?;

        Ok(skill)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_store() -> (SkillStore, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("skills");
        let store = SkillStore::open(&path).unwrap();
        (store, dir)
    }

    #[test]
    fn test_create_store() {
        let (store, _dir) = test_store();
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn test_upsert_and_get() {
        let (store, _dir) = test_store();

        let skill = Skill::new(
            "test.skill",
            "test",
            "Test skill",
            "echo {{message}}",
        );

        let is_new = store.upsert(&skill).unwrap();
        assert!(is_new);

        let retrieved = store.get("test.skill");
        assert!(retrieved.is_some());
        let s = retrieved.unwrap();
        assert_eq!(s.intent, "test");
    }

    #[test]
    fn test_query_by_intent() {
        let (store, _dir) = test_store();

        store.upsert(&Skill::new("logs.service", "logs_for_service", "Logs", "journalctl")).unwrap();
        store.upsert(&Skill::new("logs.window", "logs_for_service", "Logs window", "journalctl --since")).unwrap();
        store.upsert(&Skill::new("disk.free", "disk_usage", "Disk space", "df")).unwrap();

        let results = store.find_by_intent("logs_for_service");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_record_success() {
        let (store, _dir) = test_store();

        let skill = Skill::new("test.skill", "test", "Test", "echo hello");
        store.upsert(&skill).unwrap();

        store.record_success("test.skill", 100).unwrap();
        store.record_success("test.skill", 150).unwrap();

        let updated = store.get("test.skill").unwrap();
        assert_eq!(updated.stats.success_count, 2);
    }

    #[test]
    fn test_find_by_question() {
        let (store, _dir) = test_store();

        let skill = Skill::new(
            "journalctl.service",
            "logs_for_service",
            "Show systemd service logs",
            "journalctl -u {{service}}",
        )
        .with_example("show the log of annad service");

        store.upsert(&skill).unwrap();

        let matches = store.find_by_question("show the log of annad service", 0.1);
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_delete() {
        let (store, _dir) = test_store();

        let skill = Skill::new("test.skill", "test", "Test", "echo");
        store.upsert(&skill).unwrap();
        assert_eq!(store.count(), 1);

        store.delete("test.skill").unwrap();
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn test_persistence() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("skills");

        // Create and save
        {
            let store = SkillStore::open(&path).unwrap();
            let skill = Skill::new("persistent.skill", "test", "Test", "echo");
            store.upsert(&skill).unwrap();
        }

        // Reload and verify
        {
            let store = SkillStore::open(&path).unwrap();
            let skill = store.get("persistent.skill");
            assert!(skill.is_some());
        }
    }
}
