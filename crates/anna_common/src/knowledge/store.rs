//! Knowledge Store Implementation v0.11.0
//!
//! SQLite-backed persistent storage for facts.
//! Location: /var/lib/anna/knowledge.db (system) or ~/.local/share/anna/knowledge.db (user)

use super::schema::{Fact, FactHistory, FactQuery, FactStatus, SCHEMA_VERSION};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Knowledge store backed by SQLite
pub struct KnowledgeStore {
    conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl KnowledgeStore {
    /// Open or create the knowledge store at the default location
    pub fn open_default() -> Result<Self> {
        let db_path = Self::default_path();
        Self::open(&db_path)
    }

    /// Open or create the knowledge store at a specific path
    pub fn open(path: &PathBuf) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open database: {:?}", path))?;

        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: path.clone(),
        };

        store.init_schema()?;
        Ok(store)
    }

    /// Get the default database path
    pub fn default_path() -> PathBuf {
        // Try system path first, fall back to user path
        let system_path = PathBuf::from("/var/lib/anna/knowledge.db");
        if system_path.parent().map(|p| p.exists()).unwrap_or(false) {
            return system_path;
        }

        // User path
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("~/.local/share"))
            .join("anna")
            .join("knowledge.db")
    }

    /// Initialize the database schema
    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Create facts table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS facts (
                id TEXT PRIMARY KEY,
                entity TEXT NOT NULL,
                attribute TEXT NOT NULL,
                value TEXT NOT NULL,
                source TEXT NOT NULL,
                first_seen TEXT NOT NULL,
                last_seen TEXT NOT NULL,
                confidence REAL NOT NULL,
                status TEXT NOT NULL DEFAULT 'active',
                notes TEXT,
                UNIQUE(entity, attribute)
            )
            "#,
            [],
        )?;

        // Create history table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS fact_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                fact_id TEXT NOT NULL,
                old_value TEXT NOT NULL,
                new_value TEXT NOT NULL,
                old_status TEXT NOT NULL,
                new_status TEXT NOT NULL,
                reason TEXT NOT NULL,
                changed_at TEXT NOT NULL,
                FOREIGN KEY (fact_id) REFERENCES facts(id)
            )
            "#,
            [],
        )?;

        // Create schema version table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS schema_meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )
            "#,
            [],
        )?;

        // Set schema version
        conn.execute(
            "INSERT OR REPLACE INTO schema_meta (key, value) VALUES ('version', ?)",
            params![SCHEMA_VERSION.to_string()],
        )?;

        // Create indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_facts_entity ON facts(entity)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_facts_status ON facts(status)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_facts_last_seen ON facts(last_seen)",
            [],
        )?;

        Ok(())
    }

    /// Insert or update a fact
    pub fn upsert(&self, fact: &Fact) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        // Check if fact exists
        let existing: Option<(String, String, String)> = conn
            .query_row(
                "SELECT id, value, status FROM facts WHERE entity = ? AND attribute = ?",
                params![&fact.entity, &fact.attribute],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;

        if let Some((existing_id, old_value, old_status)) = existing {
            // Update existing fact
            if old_value != fact.value {
                // Record history
                conn.execute(
                    r#"
                    INSERT INTO fact_history (fact_id, old_value, new_value, old_status, new_status, reason, changed_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?)
                    "#,
                    params![
                        &existing_id,
                        &old_value,
                        &fact.value,
                        &old_status,
                        fact.status.as_str(),
                        fact.notes.as_deref().unwrap_or("updated"),
                        Utc::now().to_rfc3339()
                    ],
                )?;
            }

            conn.execute(
                r#"
                UPDATE facts SET
                    value = ?,
                    source = ?,
                    last_seen = ?,
                    confidence = ?,
                    status = ?,
                    notes = ?
                WHERE entity = ? AND attribute = ?
                "#,
                params![
                    &fact.value,
                    &fact.source,
                    fact.last_seen.to_rfc3339(),
                    fact.confidence,
                    fact.status.as_str(),
                    &fact.notes,
                    &fact.entity,
                    &fact.attribute
                ],
            )?;
            Ok(false) // Not a new fact
        } else {
            // Insert new fact
            conn.execute(
                r#"
                INSERT INTO facts (id, entity, attribute, value, source, first_seen, last_seen, confidence, status, notes)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                params![
                    &fact.id,
                    &fact.entity,
                    &fact.attribute,
                    &fact.value,
                    &fact.source,
                    fact.first_seen.to_rfc3339(),
                    fact.last_seen.to_rfc3339(),
                    fact.confidence,
                    fact.status.as_str(),
                    &fact.notes
                ],
            )?;
            Ok(true) // New fact inserted
        }
    }

    /// Get a specific fact
    pub fn get(&self, entity: &str, attribute: &str) -> Result<Option<Fact>> {
        let conn = self.conn.lock().unwrap();

        let result: Option<Fact> = conn
            .query_row(
                r#"
                SELECT id, entity, attribute, value, source, first_seen, last_seen, confidence, status, notes
                FROM facts WHERE entity = ? AND attribute = ?
                "#,
                params![entity, attribute],
                |row| {
                    Ok(Fact {
                        id: row.get(0)?,
                        entity: row.get(1)?,
                        attribute: row.get(2)?,
                        value: row.get(3)?,
                        source: row.get(4)?,
                        first_seen: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                            .unwrap_or_else(|_| Utc::now().into())
                            .with_timezone(&Utc),
                        last_seen: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                            .unwrap_or_else(|_| Utc::now().into())
                            .with_timezone(&Utc),
                        confidence: row.get(7)?,
                        status: FactStatus::from_str(&row.get::<_, String>(8)?),
                        notes: row.get(9)?,
                    })
                },
            )
            .optional()?;

        Ok(result)
    }

    /// Query facts with filters
    pub fn query(&self, filter: &FactQuery) -> Result<Vec<Fact>> {
        let conn = self.conn.lock().unwrap();

        let mut sql = String::from(
            "SELECT id, entity, attribute, value, source, first_seen, last_seen, confidence, status, notes FROM facts WHERE 1=1",
        );
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref entity) = filter.entity {
            if entity.ends_with('*') {
                let prefix = &entity[..entity.len() - 1];
                sql.push_str(" AND entity LIKE ?");
                params_vec.push(Box::new(format!("{}%", prefix)));
            } else {
                sql.push_str(" AND entity = ?");
                params_vec.push(Box::new(entity.clone()));
            }
        }

        if let Some(ref attr) = filter.attribute {
            sql.push_str(" AND attribute = ?");
            params_vec.push(Box::new(attr.clone()));
        }

        if let Some(conf) = filter.min_confidence {
            sql.push_str(" AND confidence >= ?");
            params_vec.push(Box::new(conf));
        }

        if let Some(ref statuses) = filter.status {
            let placeholders: Vec<&str> = statuses.iter().map(|_| "?").collect();
            sql.push_str(&format!(" AND status IN ({})", placeholders.join(",")));
            for s in statuses {
                params_vec.push(Box::new(s.as_str().to_string()));
            }
        }

        if let Some(ref after) = filter.seen_after {
            sql.push_str(" AND last_seen >= ?");
            params_vec.push(Box::new(after.to_rfc3339()));
        }

        sql.push_str(" ORDER BY last_seen DESC");

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(Fact {
                id: row.get(0)?,
                entity: row.get(1)?,
                attribute: row.get(2)?,
                value: row.get(3)?,
                source: row.get(4)?,
                first_seen: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .unwrap_or_else(|_| Utc::now().into())
                    .with_timezone(&Utc),
                last_seen: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .unwrap_or_else(|_| Utc::now().into())
                    .with_timezone(&Utc),
                confidence: row.get(7)?,
                status: FactStatus::from_str(&row.get::<_, String>(8)?),
                notes: row.get(9)?,
            })
        })?;

        let mut facts = Vec::new();
        for row in rows {
            facts.push(row?);
        }
        Ok(facts)
    }

    /// Mark facts as stale based on age
    pub fn mark_stale_by_age(&self, max_age_hours: i64) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let cutoff = Utc::now() - chrono::Duration::hours(max_age_hours);

        let count = conn.execute(
            "UPDATE facts SET status = 'stale' WHERE status = 'active' AND last_seen < ?",
            params![cutoff.to_rfc3339()],
        )?;

        Ok(count)
    }

    /// Get fact history
    pub fn get_history(&self, fact_id: &str) -> Result<Vec<FactHistory>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT fact_id, old_value, new_value, old_status, new_status, reason, changed_at
            FROM fact_history WHERE fact_id = ? ORDER BY changed_at DESC
            "#,
        )?;

        let rows = stmt.query_map(params![fact_id], |row| {
            Ok(FactHistory {
                fact_id: row.get(0)?,
                old_value: row.get(1)?,
                new_value: row.get(2)?,
                old_status: FactStatus::from_str(&row.get::<_, String>(3)?),
                new_status: FactStatus::from_str(&row.get::<_, String>(4)?),
                reason: row.get(5)?,
                changed_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .unwrap_or_else(|_| Utc::now().into())
                    .with_timezone(&Utc),
            })
        })?;

        let mut history = Vec::new();
        for row in rows {
            history.push(row?);
        }
        Ok(history)
    }

    /// Count facts by status
    pub fn count_by_status(&self) -> Result<std::collections::HashMap<String, usize>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare("SELECT status, COUNT(*) FROM facts GROUP BY status")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, usize>(1)?))
        })?;

        let mut counts = std::collections::HashMap::new();
        for row in rows {
            let (status, count) = row?;
            counts.insert(status, count);
        }
        Ok(counts)
    }

    /// Get total fact count
    pub fn count(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let count: usize = conn.query_row("SELECT COUNT(*) FROM facts", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Delete a fact (soft delete by marking deprecated)
    pub fn delete(&self, entity: &str, attribute: &str, reason: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        let count = conn.execute(
            "UPDATE facts SET status = 'deprecated', notes = ? WHERE entity = ? AND attribute = ?",
            params![reason, entity, attribute],
        )?;

        Ok(count > 0)
    }

    /// Get database path
    pub fn path(&self) -> &PathBuf {
        &self.db_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_store() -> (KnowledgeStore, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_knowledge.db");
        let store = KnowledgeStore::open(&path).unwrap();
        (store, dir)
    }

    #[test]
    fn test_create_store() {
        let (store, _dir) = test_store();
        assert_eq!(store.count().unwrap(), 0);
    }

    #[test]
    fn test_upsert_and_get() {
        let (store, _dir) = test_store();

        let fact = Fact::from_probe(
            "cpu:0".to_string(),
            "cores".to_string(),
            "8".to_string(),
            "cpu.info",
            0.95,
        );

        let is_new = store.upsert(&fact).unwrap();
        assert!(is_new);

        let retrieved = store.get("cpu:0", "cores").unwrap();
        assert!(retrieved.is_some());
        let f = retrieved.unwrap();
        assert_eq!(f.value, "8");
        assert_eq!(f.confidence, 0.95);
    }

    #[test]
    fn test_query_by_entity() {
        let (store, _dir) = test_store();

        store.upsert(&Fact::from_probe(
            "pkg:vim".to_string(),
            "version".to_string(),
            "9.0".to_string(),
            "pkg.info",
            0.9,
        )).unwrap();

        store.upsert(&Fact::from_probe(
            "pkg:neovim".to_string(),
            "version".to_string(),
            "0.9.0".to_string(),
            "pkg.info",
            0.9,
        )).unwrap();

        let query = FactQuery::new().entity_prefix("pkg:");
        let results = store.query(&query).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_mark_stale() {
        let (store, _dir) = test_store();

        let mut fact = Fact::from_probe(
            "test:old".to_string(),
            "value".to_string(),
            "test".to_string(),
            "test.probe",
            0.9,
        );
        // Set last_seen to 48 hours ago
        fact.last_seen = Utc::now() - chrono::Duration::hours(48);
        fact.first_seen = fact.last_seen;

        store.upsert(&fact).unwrap();

        let count = store.mark_stale_by_age(24).unwrap();
        assert_eq!(count, 1);

        let retrieved = store.get("test:old", "value").unwrap().unwrap();
        assert_eq!(retrieved.status, FactStatus::Stale);
    }
}
