//! Episode Storage - Persistent storage for action episodes
//!
//! v6.49.0: SQLite-based episode storage for rollback capability

use crate::action_episodes::{ActionEpisode, ActionKind, ActionRecord, EpisodeTags, RollbackCapability};
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;

const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS action_episodes (
    episode_id      INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at      TEXT NOT NULL,
    user_question   TEXT NOT NULL,
    final_answer_summary TEXT NOT NULL,
    tags_json       TEXT NOT NULL,
    rollback_capability TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS action_actions (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    episode_id      INTEGER NOT NULL,
    action_id       INTEGER NOT NULL,
    kind            TEXT NOT NULL,
    command         TEXT NOT NULL,
    cwd             TEXT,
    files_touched   TEXT NOT NULL,
    backup_paths    TEXT NOT NULL,
    notes           TEXT,
    FOREIGN KEY (episode_id) REFERENCES action_episodes(episode_id)
);

CREATE INDEX IF NOT EXISTS idx_episodes_created ON action_episodes(created_at);
CREATE INDEX IF NOT EXISTS idx_actions_episode ON action_actions(episode_id);
"#;

/// Episode storage manager
pub struct EpisodeStorage {
    conn: Connection,
}

impl EpisodeStorage {
    /// Create new episode storage
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)
            .context("Failed to open episode storage database")?;

        conn.execute_batch(SCHEMA_SQL)
            .context("Failed to initialize episode storage schema")?;

        Ok(Self { conn })
    }

    /// Store action episode
    pub fn store_action_episode(&self, episode: &ActionEpisode) -> Result<i64> {
        let tags_json = serde_json::to_string(&episode.tags)?;
        let rollback_capability = format!("{:?}", episode.rollback_capability);

        self.conn.execute(
            "INSERT INTO action_episodes (created_at, user_question, final_answer_summary, tags_json, rollback_capability)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                episode.created_at.to_rfc3339(),
                &episode.user_question,
                &episode.final_answer_summary,
                &tags_json,
                &rollback_capability,
            ],
        )?;

        let episode_id = self.conn.last_insert_rowid();

        // Store actions
        for action in &episode.actions {
            let files_touched = serde_json::to_string(&action.files_touched)?;
            let backup_paths = serde_json::to_string(&action.backup_paths)?;
            let kind_str = format!("{:?}", action.kind);

            self.conn.execute(
                "INSERT INTO action_actions (episode_id, action_id, kind, command, cwd, files_touched, backup_paths, notes)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    episode_id,
                    action.id,
                    &kind_str,
                    &action.command,
                    &action.cwd,
                    &files_touched,
                    &backup_paths,
                    &action.notes,
                ],
            )?;
        }

        Ok(episode_id)
    }

    /// List recent action episodes
    pub fn list_action_episodes_recent(&self, limit: usize) -> Result<Vec<ActionEpisode>> {
        let mut stmt = self.conn.prepare(
            "SELECT episode_id, created_at, user_question, final_answer_summary, tags_json, rollback_capability
             FROM action_episodes
             ORDER BY created_at DESC
             LIMIT ?1"
        )?;

        let episodes = stmt.query_map(params![limit], |row| {
            let episode_id: i64 = row.get(0)?;
            let created_at_str: String = row.get(1)?;
            let user_question: String = row.get(2)?;
            let final_answer_summary: String = row.get(3)?;
            let tags_json: String = row.get(4)?;
            let rollback_capability_str: String = row.get(5)?;

            Ok((episode_id, created_at_str, user_question, final_answer_summary, tags_json, rollback_capability_str))
        })?;

        let mut result = Vec::new();
        for episode_row in episodes {
            let (episode_id, created_at_str, user_question, final_answer_summary, tags_json, rollback_capability_str) = episode_row?;

            if let Some(episode) = self.load_episode_by_id(episode_id, &created_at_str, &user_question, &final_answer_summary, &tags_json, &rollback_capability_str)? {
                result.push(episode);
            }
        }

        Ok(result)
    }

    /// List action episodes by topic
    pub fn list_action_episodes_by_topic(&self, topic: &str, limit: usize) -> Result<Vec<ActionEpisode>> {
        let mut stmt = self.conn.prepare(
            "SELECT episode_id, created_at, user_question, final_answer_summary, tags_json, rollback_capability
             FROM action_episodes
             WHERE tags_json LIKE ?1
             ORDER BY created_at DESC
             LIMIT ?2"
        )?;

        let topic_pattern = format!("%\"{}%", topic);
        let episodes = stmt.query_map(params![&topic_pattern, limit], |row| {
            let episode_id: i64 = row.get(0)?;
            let created_at_str: String = row.get(1)?;
            let user_question: String = row.get(2)?;
            let final_answer_summary: String = row.get(3)?;
            let tags_json: String = row.get(4)?;
            let rollback_capability_str: String = row.get(5)?;

            Ok((episode_id, created_at_str, user_question, final_answer_summary, tags_json, rollback_capability_str))
        })?;

        let mut result = Vec::new();
        for episode_row in episodes {
            let (episode_id, created_at_str, user_question, final_answer_summary, tags_json, rollback_capability_str) = episode_row?;

            if let Some(episode) = self.load_episode_by_id(episode_id, &created_at_str, &user_question, &final_answer_summary, &tags_json, &rollback_capability_str)? {
                result.push(episode);
            }
        }

        Ok(result)
    }

    /// Load action episode by ID
    pub fn load_action_episode(&self, episode_id: i64) -> Result<Option<ActionEpisode>> {
        let mut stmt = self.conn.prepare(
            "SELECT created_at, user_question, final_answer_summary, tags_json, rollback_capability
             FROM action_episodes
             WHERE episode_id = ?1"
        )?;

        let mut rows = stmt.query(params![episode_id])?;

        if let Some(row) = rows.next()? {
            let created_at_str: String = row.get(0)?;
            let user_question: String = row.get(1)?;
            let final_answer_summary: String = row.get(2)?;
            let tags_json: String = row.get(3)?;
            let rollback_capability_str: String = row.get(4)?;

            self.load_episode_by_id(episode_id, &created_at_str, &user_question, &final_answer_summary, &tags_json, &rollback_capability_str)
        } else {
            Ok(None)
        }
    }

    /// Load episode by ID (helper)
    fn load_episode_by_id(
        &self,
        episode_id: i64,
        created_at_str: &str,
        user_question: &str,
        final_answer_summary: &str,
        tags_json: &str,
        rollback_capability_str: &str,
    ) -> Result<Option<ActionEpisode>> {
        let created_at = chrono::DateTime::parse_from_rfc3339(created_at_str)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

        let tags: EpisodeTags = serde_json::from_str(tags_json).unwrap_or_else(|_| EpisodeTags {
            topics: vec![],
            domain: None,
        });

        let rollback_capability = match rollback_capability_str {
            "Full" => RollbackCapability::Full,
            "Partial" => RollbackCapability::Partial,
            _ => RollbackCapability::None,
        };

        // Load actions
        let mut stmt = self.conn.prepare(
            "SELECT action_id, kind, command, cwd, files_touched, backup_paths, notes
             FROM action_actions
             WHERE episode_id = ?1
             ORDER BY action_id"
        )?;

        let actions_iter = stmt.query_map(params![episode_id], |row| {
            let action_id: i64 = row.get(0)?;
            let kind_str: String = row.get(1)?;
            let command: String = row.get(2)?;
            let cwd: Option<String> = row.get(3)?;
            let files_touched_json: String = row.get(4)?;
            let backup_paths_json: String = row.get(5)?;
            let notes: Option<String> = row.get(6)?;

            Ok((action_id, kind_str, command, cwd, files_touched_json, backup_paths_json, notes))
        })?;

        let mut actions = Vec::new();
        for action_row in actions_iter {
            let (action_id, kind_str, command, cwd, files_touched_json, backup_paths_json, notes) = action_row?;

            let kind = match kind_str.as_str() {
                "EditFile" => ActionKind::EditFile,
                "CreateFile" => ActionKind::CreateFile,
                "DeleteFile" => ActionKind::DeleteFile,
                "MoveFile" => ActionKind::MoveFile,
                "InstallPackages" => ActionKind::InstallPackages,
                "RemovePackages" => ActionKind::RemovePackages,
                "EnableServices" => ActionKind::EnableServices,
                "DisableServices" => ActionKind::DisableServices,
                "StartServices" => ActionKind::StartServices,
                "StopServices" => ActionKind::StopServices,
                _ => ActionKind::RunCommand,
            };

            let files_touched: Vec<String> = serde_json::from_str(&files_touched_json).unwrap_or_default();
            let backup_paths: Vec<String> = serde_json::from_str(&backup_paths_json).unwrap_or_default();

            actions.push(ActionRecord {
                id: action_id,
                kind,
                command,
                cwd,
                files_touched,
                backup_paths,
                notes,
            });
        }

        Ok(Some(ActionEpisode {
            episode_id,
            created_at,
            user_question: user_question.to_string(),
            final_answer_summary: final_answer_summary.to_string(),
            tags,
            actions,
            rollback_capability,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action_episodes::EpisodeBuilder;
    use tempfile::tempdir;

    #[test]
    fn test_store_and_load_episode() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let storage = EpisodeStorage::new(&db_path).unwrap();

        let mut builder = EpisodeBuilder::new("make my vim use 4 spaces");
        builder.add_action(ActionRecord {
            id: 0,
            kind: ActionKind::EditFile,
            command: "edit ~/.vimrc".to_string(),
            cwd: Some("/home/user".to_string()),
            files_touched: vec!["/home/user/.vimrc".to_string()],
            backup_paths: vec!["/home/user/.vimrc.bak".to_string()],
            notes: Some("vim config".to_string()),
        });

        let episode = builder
            .with_final_answer_summary("Updated vim")
            .with_tags(EpisodeTags {
                topics: vec!["vim".to_string()],
                domain: Some("editor".to_string()),
            })
            .build();

        let episode_id = storage.store_action_episode(&episode).unwrap();
        assert!(episode_id > 0);

        let loaded = storage.load_action_episode(episode_id).unwrap();
        assert!(loaded.is_some());

        let loaded = loaded.unwrap();
        assert_eq!(loaded.user_question, "make my vim use 4 spaces");
        assert_eq!(loaded.actions.len(), 1);
        assert_eq!(loaded.actions[0].kind, ActionKind::EditFile);
    }

    #[test]
    fn test_list_recent_episodes() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let storage = EpisodeStorage::new(&db_path).unwrap();

        // Store 3 episodes
        for i in 1..=3 {
            let episode = EpisodeBuilder::new(&format!("test {}", i))
                .with_final_answer_summary("done")
                .build();
            storage.store_action_episode(&episode).unwrap();
        }

        let episodes = storage.list_action_episodes_recent(2).unwrap();
        assert_eq!(episodes.len(), 2);
        // Most recent first
        assert_eq!(episodes[0].user_question, "test 3");
        assert_eq!(episodes[1].user_question, "test 2");
    }

    #[test]
    fn test_list_by_topic() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let storage = EpisodeStorage::new(&db_path).unwrap();

        let vim_episode = EpisodeBuilder::new("vim test")
            .with_tags(EpisodeTags {
                topics: vec!["vim".to_string()],
                domain: Some("editor".to_string()),
            })
            .build();

        let ssh_episode = EpisodeBuilder::new("ssh test")
            .with_tags(EpisodeTags {
                topics: vec!["ssh".to_string()],
                domain: Some("network".to_string()),
            })
            .build();

        storage.store_action_episode(&vim_episode).unwrap();
        storage.store_action_episode(&ssh_episode).unwrap();

        let vim_episodes = storage.list_action_episodes_by_topic("vim", 10).unwrap();
        assert_eq!(vim_episodes.len(), 1);
        assert_eq!(vim_episodes[0].user_question, "vim test");
    }

    #[test]
    fn test_rollback_capability_persistence() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let storage = EpisodeStorage::new(&db_path).unwrap();

        let mut builder = EpisodeBuilder::new("test");
        builder.add_action(ActionRecord {
            id: 0,
            kind: ActionKind::DeleteFile,
            command: "rm file".to_string(),
            cwd: None,
            files_touched: vec!["/test.txt".to_string()],
            backup_paths: vec![], // No backup -> None capability
            notes: None,
        });

        let episode = builder.build();
        assert_eq!(episode.rollback_capability, RollbackCapability::None);

        let episode_id = storage.store_action_episode(&episode).unwrap();
        let loaded = storage.load_action_episode(episode_id).unwrap().unwrap();

        assert_eq!(loaded.rollback_capability, RollbackCapability::None);
    }
}
