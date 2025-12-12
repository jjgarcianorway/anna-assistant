//! Tests for database recipes (v0.0.461).

use super::*;

#[test]
fn test_detect_backup() {
    assert_eq!(
        detect_feature("backup database postgres"),
        Some(DatabaseFeature::BackupDatabase)
    );
    assert_eq!(
        detect_feature("database backup mysql"),
        Some(DatabaseFeature::BackupDatabase)
    );
}

#[test]
fn test_detect_restore() {
    assert_eq!(
        detect_feature("restore database from file"),
        Some(DatabaseFeature::RestoreDatabase)
    );
    assert_eq!(
        detect_feature("recover database backup"),
        Some(DatabaseFeature::RestoreDatabase)
    );
}

#[test]
fn test_detect_dump() {
    assert_eq!(
        detect_feature("dump database to sql"),
        Some(DatabaseFeature::DumpDatabase)
    );
    assert_eq!(
        detect_feature("pg_dump postgres database"),
        Some(DatabaseFeature::DumpDatabase)
    );
    assert_eq!(
        detect_feature("mysqldump mysql"),
        Some(DatabaseFeature::DumpDatabase)
    );
}

#[test]
fn test_detect_import() {
    assert_eq!(
        detect_feature("import database from sql file"),
        Some(DatabaseFeature::ImportData)
    );
    assert_eq!(
        detect_feature("import data into postgres"),
        Some(DatabaseFeature::ImportData)
    );
}

#[test]
fn test_detect_export() {
    assert_eq!(
        detect_feature("export database to csv"),
        Some(DatabaseFeature::ExportData)
    );
    assert_eq!(
        detect_feature("export data from mysql"),
        Some(DatabaseFeature::ExportData)
    );
}

#[test]
fn test_detect_status() {
    assert_eq!(
        detect_feature("check database status"),
        Some(DatabaseFeature::CheckStatus)
    );
    assert_eq!(
        detect_feature("database health check"),
        Some(DatabaseFeature::CheckStatus)
    );
}

#[test]
fn test_detect_repair() {
    assert_eq!(
        detect_feature("repair database mysql"),
        Some(DatabaseFeature::RepairDatabase)
    );
    assert_eq!(
        detect_feature("fix database corruption"),
        Some(DatabaseFeature::RepairDatabase)
    );
}

#[test]
fn test_detect_optimize() {
    assert_eq!(
        detect_feature("optimize table in database"),
        Some(DatabaseFeature::OptimizeTables)
    );
    assert_eq!(
        detect_feature("vacuum postgres database"),
        Some(DatabaseFeature::OptimizeTables)
    );
}

#[test]
fn test_detect_create_database() {
    assert_eq!(
        detect_feature("create database postgres"),
        Some(DatabaseFeature::CreateDatabase)
    );
    assert_eq!(
        detect_feature("new database mysql"),
        Some(DatabaseFeature::CreateDatabase)
    );
}

#[test]
fn test_detect_create_user() {
    assert_eq!(
        detect_feature("create user for database"),
        Some(DatabaseFeature::CreateUser)
    );
    assert_eq!(
        detect_feature("add database user mysql"),
        Some(DatabaseFeature::CreateUser)
    );
}

#[test]
fn test_detect_grant_permissions() {
    assert_eq!(
        detect_feature("grant permission on database"),
        Some(DatabaseFeature::GrantPermissions)
    );
    assert_eq!(
        detect_feature("grant privilege to user database"),
        Some(DatabaseFeature::GrantPermissions)
    );
}

#[test]
fn test_detect_show_databases() {
    assert_eq!(
        detect_feature("show database list"),
        Some(DatabaseFeature::ShowDatabases)
    );
    assert_eq!(
        detect_feature("list database in mysql"),
        Some(DatabaseFeature::ShowDatabases)
    );
}

#[test]
fn test_detect_show_tables() {
    assert_eq!(
        detect_feature("show table in database"),
        Some(DatabaseFeature::ShowTables)
    );
    assert_eq!(
        detect_feature("list table postgres"),
        Some(DatabaseFeature::ShowTables)
    );
}

#[test]
fn test_detect_test_connection() {
    assert_eq!(
        detect_feature("test connection to database"),
        Some(DatabaseFeature::TestConnection)
    );
    assert_eq!(
        detect_feature("check connection mysql"),
        Some(DatabaseFeature::TestConnection)
    );
}

#[test]
fn test_not_database_query() {
    assert_eq!(detect_feature("how much disk space"), None);
    assert_eq!(detect_feature("install htop"), None);
    assert_eq!(detect_feature("restart nginx"), None);
    assert_eq!(detect_feature("kubernetes pods"), None);
}

#[test]
fn test_match_query_returns_recipe() {
    let recipe = match_query("backup database postgres");
    assert!(recipe.is_some());
    let recipe = recipe.unwrap();
    assert_eq!(recipe.feature, DatabaseFeature::BackupDatabase);
    assert!(!recipe.commands.is_empty());
    assert!(!recipe.answer_template.is_empty());
}

#[test]
fn test_all_features_have_recipes() {
    let recipes = builtin_recipes();
    let features: Vec<DatabaseFeature> = recipes.iter().map(|r| r.feature).collect();

    assert!(features.contains(&DatabaseFeature::BackupDatabase));
    assert!(features.contains(&DatabaseFeature::RestoreDatabase));
    assert!(features.contains(&DatabaseFeature::DumpDatabase));
    assert!(features.contains(&DatabaseFeature::ImportData));
    assert!(features.contains(&DatabaseFeature::ExportData));
    assert!(features.contains(&DatabaseFeature::CheckStatus));
    assert!(features.contains(&DatabaseFeature::RepairDatabase));
    assert!(features.contains(&DatabaseFeature::OptimizeTables));
    assert!(features.contains(&DatabaseFeature::CreateDatabase));
    assert!(features.contains(&DatabaseFeature::CreateUser));
    assert!(features.contains(&DatabaseFeature::GrantPermissions));
    assert!(features.contains(&DatabaseFeature::ShowDatabases));
    assert!(features.contains(&DatabaseFeature::ShowTables));
    assert!(features.contains(&DatabaseFeature::TestConnection));
}

#[test]
fn test_feature_display_names() {
    assert_eq!(DatabaseFeature::BackupDatabase.display_name(), "backup database");
    assert_eq!(DatabaseFeature::RestoreDatabase.display_name(), "restore database");
    assert_eq!(DatabaseFeature::OptimizeTables.display_name(), "optimize tables");
}

#[test]
fn test_database_type_display() {
    assert_eq!(DatabaseType::PostgreSQL.to_string(), "PostgreSQL");
    assert_eq!(DatabaseType::MySQL.to_string(), "MySQL");
    assert_eq!(DatabaseType::SQLite.to_string(), "SQLite");
    assert_eq!(DatabaseType::MongoDB.to_string(), "MongoDB");
}

#[test]
fn test_recipe_builder() {
    let recipe = DatabaseRecipe::new(DatabaseFeature::BackupDatabase, "Test")
        .for_postgres()
        .with_command("pg_dump test")
        .with_example("pg_dump -Fc db > db.dump")
        .with_answer("test answer")
        .with_note("test note");

    assert_eq!(recipe.feature, DatabaseFeature::BackupDatabase);
    assert_eq!(recipe.database_type, Some(DatabaseType::PostgreSQL));
    assert!(recipe.requires.contains(&"psql".to_string()));
    assert_eq!(recipe.commands, vec!["pg_dump test"]);
    assert_eq!(recipe.example, Some("pg_dump -Fc db > db.dump".to_string()));
    assert_eq!(recipe.answer_template, "test answer");
    assert_eq!(recipe.notes, vec!["test note"]);
}

#[test]
fn test_backup_recipe_has_postgres_and_mysql() {
    let recipes = builtin_recipes();
    let backup_recipes: Vec<_> = recipes
        .iter()
        .filter(|r| r.feature == DatabaseFeature::BackupDatabase)
        .collect();

    // Should have at least one for PostgreSQL
    assert!(backup_recipes
        .iter()
        .any(|r| r.database_type == Some(DatabaseType::PostgreSQL)));
    // Should have at least one for MySQL
    assert!(backup_recipes
        .iter()
        .any(|r| r.database_type == Some(DatabaseType::MySQL)));
}
