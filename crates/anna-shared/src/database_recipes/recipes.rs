//! Database builtin recipes (v0.0.461).

use super::types::{DatabaseFeature, DatabaseRecipe};

/// Get all builtin database recipes
pub fn builtin_recipes() -> Vec<DatabaseRecipe> {
    vec![
        // Backup operations
        DatabaseRecipe::new(DatabaseFeature::BackupDatabase, "Create PostgreSQL backup")
            .for_postgres()
            .with_command("pg_dump dbname > backup.sql")
            .with_command("pg_dump -Fc dbname > backup.dump")
            .with_command("pg_dumpall > all_databases.sql")
            .with_answer(
                "PostgreSQL backup: `pg_dump dbname > backup.sql` for SQL format, \
                 `pg_dump -Fc dbname > backup.dump` for custom format (faster restore). \
                 Use `pg_dumpall` for all databases including roles.",
            )
            .with_note("Custom format (-Fc) supports parallel restore and compression"),
        DatabaseRecipe::new(DatabaseFeature::BackupDatabase, "Create MySQL/MariaDB backup")
            .for_mysql()
            .with_command("mysqldump -u root -p dbname > backup.sql")
            .with_command("mysqldump -u root -p --all-databases > all_dbs.sql")
            .with_command("mysqldump -u root -p --single-transaction dbname > backup.sql")
            .with_answer(
                "MySQL backup: `mysqldump -u root -p dbname > backup.sql`. \
                 Use `--single-transaction` for InnoDB tables to avoid locking. \
                 Use `--all-databases` for complete server backup.",
            )
            .with_note("--single-transaction provides consistent backup without locks"),
        // Restore operations
        DatabaseRecipe::new(DatabaseFeature::RestoreDatabase, "Restore PostgreSQL backup")
            .for_postgres()
            .with_command("psql dbname < backup.sql")
            .with_command("pg_restore -d dbname backup.dump")
            .with_command("pg_restore -j 4 -d dbname backup.dump")
            .with_answer(
                "PostgreSQL restore: `psql dbname < backup.sql` for SQL format, \
                 `pg_restore -d dbname backup.dump` for custom format. \
                 Use `-j 4` for parallel restore (4 jobs).",
            )
            .with_note("Create database first if restoring to new database"),
        DatabaseRecipe::new(DatabaseFeature::RestoreDatabase, "Restore MySQL/MariaDB backup")
            .for_mysql()
            .with_command("mysql -u root -p dbname < backup.sql")
            .with_command("mysql -u root -p < all_dbs.sql")
            .with_answer(
                "MySQL restore: `mysql -u root -p dbname < backup.sql`. \
                 Create the database first if it doesn't exist.",
            ),
        // Dump operations
        DatabaseRecipe::new(DatabaseFeature::DumpDatabase, "Dump database to file")
            .with_command("pg_dump -Fc dbname > dump.pgdump")
            .with_command("mysqldump -u root -p dbname > dump.sql")
            .with_command("sqlite3 db.sqlite .dump > dump.sql")
            .with_command("mongodump --db dbname --out /backup/")
            .with_answer(
                "Dump commands by database: PostgreSQL: `pg_dump`, MySQL: `mysqldump`, \
                 SQLite: `sqlite3 db .dump`, MongoDB: `mongodump --db dbname`.",
            ),
        // Import/Export
        DatabaseRecipe::new(DatabaseFeature::ImportData, "Import data from file")
            .with_command("psql -d dbname -f data.sql")
            .with_command("psql -d dbname -c \"\\copy table FROM 'data.csv' CSV HEADER\"")
            .with_command("mysql -u root -p dbname < data.sql")
            .with_command("LOAD DATA INFILE '/path/data.csv' INTO TABLE tbl")
            .with_answer(
                "PostgreSQL: `psql -f data.sql` for SQL, `\\copy` for CSV. \
                 MySQL: `mysql < data.sql` for SQL, `LOAD DATA INFILE` for CSV.",
            )
            .with_note("Use \\copy in psql (client-side) vs COPY (server-side)"),
        DatabaseRecipe::new(DatabaseFeature::ExportData, "Export data to file")
            .with_command("psql -d dbname -c \"\\copy table TO 'data.csv' CSV HEADER\"")
            .with_command("mysql -u root -p -e \"SELECT * FROM tbl\" dbname > data.txt")
            .with_answer(
                "PostgreSQL: `\\copy table TO 'file.csv' CSV HEADER`. \
                 MySQL: `mysql -e 'SELECT...' > file.txt` or `INTO OUTFILE`.",
            ),
        // Status and health
        DatabaseRecipe::new(DatabaseFeature::CheckStatus, "Check database status")
            .with_command("systemctl status postgresql")
            .with_command("systemctl status mysql")
            .with_command("systemctl status mariadb")
            .with_command("pg_isready")
            .with_command("mysqladmin -u root -p status")
            .with_answer(
                "Check service: `systemctl status postgresql/mysql`. \
                 PostgreSQL: `pg_isready` for connection test. \
                 MySQL: `mysqladmin status` for server info.",
            ),
        // Repair and optimize
        DatabaseRecipe::new(DatabaseFeature::RepairDatabase, "Repair database tables")
            .for_mysql()
            .with_command("mysqlcheck -u root -p --repair dbname")
            .with_command("mysqlcheck -u root -p --repair --all-databases")
            .with_command("REPAIR TABLE tablename")
            .with_answer(
                "MySQL repair: `mysqlcheck --repair dbname` or `REPAIR TABLE`. \
                 Use `--all-databases` to check/repair all. \
                 PostgreSQL uses `REINDEX` for index repair.",
            )
            .with_note("Backup before repair operations"),
        DatabaseRecipe::new(DatabaseFeature::OptimizeTables, "Optimize database tables")
            .with_command("VACUUM ANALYZE")
            .with_command("VACUUM FULL tablename")
            .with_command("OPTIMIZE TABLE tablename")
            .with_command("mysqlcheck -u root -p --optimize dbname")
            .with_answer(
                "PostgreSQL: `VACUUM ANALYZE` (routine), `VACUUM FULL` (reclaims space). \
                 MySQL: `OPTIMIZE TABLE` or `mysqlcheck --optimize`.",
            )
            .with_note("VACUUM FULL locks the table; prefer regular VACUUM"),
        // Database/user management
        DatabaseRecipe::new(DatabaseFeature::CreateDatabase, "Create new database")
            .with_command("createdb dbname")
            .with_command("psql -c \"CREATE DATABASE dbname\"")
            .with_command("mysql -u root -p -e \"CREATE DATABASE dbname\"")
            .with_command("sqlite3 newdb.sqlite \"\"")
            .with_answer(
                "PostgreSQL: `createdb dbname` or `CREATE DATABASE dbname`. \
                 MySQL: `CREATE DATABASE dbname`. \
                 SQLite: create by connecting to new file.",
            ),
        DatabaseRecipe::new(DatabaseFeature::CreateUser, "Create database user")
            .with_command("createuser -P username")
            .with_command("psql -c \"CREATE USER username WITH PASSWORD 'pass'\"")
            .with_command("mysql -u root -p -e \"CREATE USER 'user'@'localhost' IDENTIFIED BY 'pass'\"")
            .with_answer(
                "PostgreSQL: `createuser -P username` (prompts for password). \
                 MySQL: `CREATE USER 'user'@'host' IDENTIFIED BY 'password'`.",
            )
            .with_note("Use strong passwords and limit host access"),
        DatabaseRecipe::new(DatabaseFeature::GrantPermissions, "Grant user permissions")
            .with_command("GRANT ALL PRIVILEGES ON dbname.* TO 'user'@'localhost'")
            .with_command("GRANT SELECT, INSERT ON dbname.* TO 'user'@'localhost'")
            .with_command("psql -c \"GRANT ALL ON DATABASE dbname TO username\"")
            .with_command("FLUSH PRIVILEGES")
            .with_answer(
                "MySQL: `GRANT ALL ON db.* TO 'user'@'host'`, then `FLUSH PRIVILEGES`. \
                 PostgreSQL: `GRANT ALL ON DATABASE dbname TO user`. \
                 Grant only needed permissions (principle of least privilege).",
            )
            .with_note("FLUSH PRIVILEGES required after GRANT in MySQL"),
        // Queries
        DatabaseRecipe::new(DatabaseFeature::ShowDatabases, "List databases")
            .with_command("psql -l")
            .with_command("\\l")
            .with_command("SHOW DATABASES")
            .with_command("mysql -u root -p -e \"SHOW DATABASES\"")
            .with_answer(
                "PostgreSQL: `psql -l` or `\\l` in psql. \
                 MySQL: `SHOW DATABASES`. \
                 SQLite: each file is a database.",
            ),
        DatabaseRecipe::new(DatabaseFeature::ShowTables, "List tables in database")
            .with_command("\\dt")
            .with_command("\\dt+")
            .with_command("SHOW TABLES")
            .with_command("SELECT name FROM sqlite_master WHERE type='table'")
            .with_answer(
                "PostgreSQL: `\\dt` (basic) or `\\dt+` (with sizes). \
                 MySQL: `SHOW TABLES`. \
                 SQLite: `SELECT name FROM sqlite_master WHERE type='table'`.",
            ),
        // Connection test
        DatabaseRecipe::new(DatabaseFeature::TestConnection, "Test database connection")
            .with_command("psql -h localhost -U postgres -c \"SELECT 1\"")
            .with_command("mysql -u root -p -e \"SELECT 1\"")
            .with_command("pg_isready -h localhost -p 5432")
            .with_command("redis-cli ping")
            .with_answer(
                "PostgreSQL: `pg_isready -h host` or `psql -c 'SELECT 1'`. \
                 MySQL: `mysql -e 'SELECT 1'`. \
                 Redis: `redis-cli ping` (should return PONG).",
            ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipes_exist() {
        let recipes = builtin_recipes();
        assert!(!recipes.is_empty());
    }

    #[test]
    fn test_recipes_have_commands() {
        for recipe in builtin_recipes() {
            assert!(
                !recipe.commands.is_empty(),
                "Recipe {:?} has no commands",
                recipe.feature
            );
        }
    }

    #[test]
    fn test_recipes_have_answers() {
        for recipe in builtin_recipes() {
            assert!(
                !recipe.answer_template.is_empty(),
                "Recipe {:?} has no answer",
                recipe.feature
            );
        }
    }
}
