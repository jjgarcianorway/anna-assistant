//! Database recipe types (v0.0.461).

use serde::{Deserialize, Serialize};

/// Database features
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseFeature {
    /// Create database backup
    BackupDatabase,
    /// Restore database from backup
    RestoreDatabase,
    /// Create database dump
    DumpDatabase,
    /// Import data
    ImportData,
    /// Export data
    ExportData,
    /// Check database status
    CheckStatus,
    /// Repair database
    RepairDatabase,
    /// Optimize tables
    OptimizeTables,
    /// Create database
    CreateDatabase,
    /// Create user
    CreateUser,
    /// Grant permissions
    GrantPermissions,
    /// Show databases
    ShowDatabases,
    /// Show tables
    ShowTables,
    /// Execute query
    ExecuteQuery,
    /// Connection test
    TestConnection,
}

/// Database type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    MariaDB,
    SQLite,
    MongoDB,
    Redis,
}

impl std::fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseType::PostgreSQL => write!(f, "PostgreSQL"),
            DatabaseType::MySQL => write!(f, "MySQL"),
            DatabaseType::MariaDB => write!(f, "MariaDB"),
            DatabaseType::SQLite => write!(f, "SQLite"),
            DatabaseType::MongoDB => write!(f, "MongoDB"),
            DatabaseType::Redis => write!(f, "Redis"),
        }
    }
}

impl DatabaseFeature {
    pub fn display_name(&self) -> &'static str {
        match self {
            DatabaseFeature::BackupDatabase => "backup database",
            DatabaseFeature::RestoreDatabase => "restore database",
            DatabaseFeature::DumpDatabase => "dump database",
            DatabaseFeature::ImportData => "import data",
            DatabaseFeature::ExportData => "export data",
            DatabaseFeature::CheckStatus => "check status",
            DatabaseFeature::RepairDatabase => "repair database",
            DatabaseFeature::OptimizeTables => "optimize tables",
            DatabaseFeature::CreateDatabase => "create database",
            DatabaseFeature::CreateUser => "create user",
            DatabaseFeature::GrantPermissions => "grant permissions",
            DatabaseFeature::ShowDatabases => "show databases",
            DatabaseFeature::ShowTables => "show tables",
            DatabaseFeature::ExecuteQuery => "execute query",
            DatabaseFeature::TestConnection => "test connection",
        }
    }

    /// Keywords that indicate this feature
    pub fn keywords(&self) -> &'static [&'static str] {
        match self {
            DatabaseFeature::BackupDatabase => {
                &["backup database", "database backup", "backup db", "db backup"]
            }
            DatabaseFeature::RestoreDatabase => {
                &["restore database", "database restore", "restore db", "recover database"]
            }
            DatabaseFeature::DumpDatabase => {
                &["dump database", "mysqldump", "pg_dump", "database dump", "dump db"]
            }
            DatabaseFeature::ImportData => {
                &["import database", "import data", "load data", "import sql"]
            }
            DatabaseFeature::ExportData => {
                &["export database", "export data", "export sql", "save database"]
            }
            DatabaseFeature::CheckStatus => {
                &["database status", "db status", "check database", "database health"]
            }
            DatabaseFeature::RepairDatabase => {
                &["repair database", "fix database", "database repair", "mysqlcheck"]
            }
            DatabaseFeature::OptimizeTables => {
                &["optimize table", "optimize database", "vacuum", "analyze table"]
            }
            DatabaseFeature::CreateDatabase => {
                &["create database", "new database", "createdb"]
            }
            DatabaseFeature::CreateUser => {
                &["create user", "database user", "new db user", "add user"]
            }
            DatabaseFeature::GrantPermissions => {
                &["grant permission", "grant privilege", "database permission", "user access"]
            }
            DatabaseFeature::ShowDatabases => {
                &["show database", "list database", "show db"]
            }
            DatabaseFeature::ShowTables => {
                &["show table", "list table", "describe table"]
            }
            DatabaseFeature::ExecuteQuery => {
                &["execute query", "run query", "sql query", "database query"]
            }
            DatabaseFeature::TestConnection => {
                &["test connection", "check connection", "connect to database", "database connect"]
            }
        }
    }
}

impl std::fmt::Display for DatabaseFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// A database recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseRecipe {
    pub feature: DatabaseFeature,
    pub database_type: Option<DatabaseType>,
    pub description: String,
    pub commands: Vec<String>,
    pub example: Option<String>,
    pub answer_template: String,
    pub notes: Vec<String>,
    /// Required tools
    pub requires: Vec<String>,
}

impl DatabaseRecipe {
    pub fn new(feature: DatabaseFeature, description: &str) -> Self {
        Self {
            feature,
            database_type: None,
            description: description.to_string(),
            commands: Vec::new(),
            example: None,
            answer_template: String::new(),
            notes: Vec::new(),
            requires: Vec::new(),
        }
    }

    pub fn for_postgres(mut self) -> Self {
        self.database_type = Some(DatabaseType::PostgreSQL);
        self.requires.push("psql".to_string());
        self
    }

    pub fn for_mysql(mut self) -> Self {
        self.database_type = Some(DatabaseType::MySQL);
        self.requires.push("mysql".to_string());
        self
    }

    pub fn for_sqlite(mut self) -> Self {
        self.database_type = Some(DatabaseType::SQLite);
        self.requires.push("sqlite3".to_string());
        self
    }

    pub fn for_mongodb(mut self) -> Self {
        self.database_type = Some(DatabaseType::MongoDB);
        self.requires.push("mongosh".to_string());
        self
    }

    pub fn with_command(mut self, cmd: &str) -> Self {
        self.commands.push(cmd.to_string());
        self
    }

    pub fn with_example(mut self, example: &str) -> Self {
        self.example = Some(example.to_string());
        self
    }

    pub fn with_answer(mut self, answer: &str) -> Self {
        self.answer_template = answer.to_string();
        self
    }

    pub fn with_note(mut self, note: &str) -> Self {
        self.notes.push(note.to_string());
        self
    }
}
