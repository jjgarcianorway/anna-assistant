//! Database query matching (v0.0.461).

use super::recipes::builtin_recipes;
use super::types::{DatabaseFeature, DatabaseRecipe};

/// Detect if a query is about databases
pub fn detect_feature(query: &str) -> Option<DatabaseFeature> {
    let lower = query.to_lowercase();

    // First check if it's even a database query
    if !is_database_query(&lower) {
        return None;
    }

    // Find all matching keywords and return the feature with the longest match
    let mut best_match: Option<(DatabaseFeature, usize)> = None;

    for feature in all_features() {
        for keyword in feature.keywords() {
            if lower.contains(keyword) {
                let keyword_len = keyword.len();
                if best_match.is_none() || keyword_len > best_match.unwrap().1 {
                    best_match = Some((feature, keyword_len));
                }
            }
        }
    }

    best_match.map(|(f, _)| f)
}

/// Match a query to a recipe
pub fn match_query(query: &str) -> Option<DatabaseRecipe> {
    let feature = detect_feature(query)?;

    builtin_recipes()
        .into_iter()
        .find(|r| r.feature == feature)
}

/// Check if query is about databases
fn is_database_query(query: &str) -> bool {
    let database_indicators = [
        "database",
        "db",
        "postgresql",
        "postgres",
        "psql",
        "mysql",
        "mariadb",
        "sqlite",
        "mongodb",
        "mongo",
        "redis",
        "sql",
        "dump",
        "backup",
        "restore",
        "pg_dump",
        "mysqldump",
        "table",
    ];

    database_indicators.iter().any(|k| query.contains(k))
}

/// Get all database features
fn all_features() -> Vec<DatabaseFeature> {
    vec![
        DatabaseFeature::BackupDatabase,
        DatabaseFeature::RestoreDatabase,
        DatabaseFeature::DumpDatabase,
        DatabaseFeature::ImportData,
        DatabaseFeature::ExportData,
        DatabaseFeature::CheckStatus,
        DatabaseFeature::RepairDatabase,
        DatabaseFeature::OptimizeTables,
        DatabaseFeature::CreateDatabase,
        DatabaseFeature::CreateUser,
        DatabaseFeature::GrantPermissions,
        DatabaseFeature::ShowDatabases,
        DatabaseFeature::ShowTables,
        DatabaseFeature::ExecuteQuery,
        DatabaseFeature::TestConnection,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_backup() {
        assert_eq!(
            detect_feature("backup database postgresql"),
            Some(DatabaseFeature::BackupDatabase)
        );
    }

    #[test]
    fn test_detect_restore() {
        assert_eq!(
            detect_feature("restore database from backup"),
            Some(DatabaseFeature::RestoreDatabase)
        );
    }

    #[test]
    fn test_detect_dump() {
        assert_eq!(
            detect_feature("mysqldump command"),
            Some(DatabaseFeature::DumpDatabase)
        );
        assert_eq!(
            detect_feature("pg_dump database"),
            Some(DatabaseFeature::DumpDatabase)
        );
    }

    #[test]
    fn test_not_database_query() {
        assert_eq!(detect_feature("how much disk space"), None);
        assert_eq!(detect_feature("restart nginx"), None);
    }

    #[test]
    fn test_match_query() {
        let recipe = match_query("backup database postgres");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, DatabaseFeature::BackupDatabase);
    }
}
