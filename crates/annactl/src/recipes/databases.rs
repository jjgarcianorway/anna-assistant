// Beta.175: Database Management Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct DatabasesRecipe;

#[derive(Debug, PartialEq)]
enum DatabasesOperation {
    Install,
    CheckStatus,
    ListDatabases,
}

impl DatabasesOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            DatabasesOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            DatabasesOperation::ListDatabases
        } else {
            DatabasesOperation::Install
        }
    }
}

impl DatabasesRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("mysql") || input_lower.contains("mariadb")
            || input_lower.contains("mongodb") || input_lower.contains("mongo db")
            || input_lower.contains("redis") || input_lower.contains("sqlite")
            || input_lower.contains("database") && (
                input_lower.contains("install") || input_lower.contains("setup")
                || input_lower.contains("relational") || input_lower.contains("nosql")
                || input_lower.contains("document") || input_lower.contains("key-value")
            );
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = DatabasesOperation::detect(user_input);
        match operation {
            DatabasesOperation::Install => Self::build_install_plan(telemetry),
            DatabasesOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            DatabasesOperation::ListDatabases => Self::build_list_databases_plan(telemetry),
        }
    }

    fn detect_database(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("mariadb") { "mariadb" }
        else if input_lower.contains("mysql") { "mysql" }
        else if input_lower.contains("mongodb") || input_lower.contains("mongo") { "mongodb" }
        else if input_lower.contains("redis") { "redis" }
        else if input_lower.contains("sqlite") { "sqlite" }
        else { "mariadb" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let database = Self::detect_database(user_input);
        let (db_name, package_name, description, is_aur, service_name) = match database {
            "mariadb" => ("MariaDB", "mariadb", "Open-source relational database (MySQL fork) with server and client tools", false, Some("mariadb")),
            "mysql" => ("MySQL", "mysql", "Oracle MySQL relational database server and client", true, Some("mysqld")),
            "mongodb" => ("MongoDB", "mongodb-bin", "Document-oriented NoSQL database with JSON-like documents", true, Some("mongodb")),
            "redis" => ("Redis", "redis", "In-memory key-value data store with persistence and pub/sub", false, Some("redis")),
            "sqlite" => ("SQLite", "sqlite", "Self-contained, serverless SQL database engine with CLI tools", false, None),
            _ => ("MariaDB", "mariadb", "Relational database server", false, Some("mariadb")),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("databases.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("database".to_string(), serde_json::json!(db_name));

        let install_cmd = if is_aur {
            format!("yay -S --needed --noconfirm {} || paru -S --needed --noconfirm {}", package_name, package_name)
        } else {
            format!("sudo pacman -S --needed --noconfirm {}", package_name)
        };

        let mut command_plan = vec![
            CommandStep {
                id: format!("install-{}", database),
                description: format!("Install {}", db_name),
                command: install_cmd,
                risk_level: if is_aur { RiskLevel::Medium } else { RiskLevel::Low },
                rollback_id: Some(format!("remove-{}", database)),
                requires_confirmation: is_aur,
            },
        ];

        // Add service enablement for server databases
        if let Some(service) = service_name {
            let init_notes = match database {
                "mariadb" | "mysql" => format!(" After installation, initialize the database with: sudo mysql_install_db --user=mysql --basedir=/usr --datadir=/var/lib/mysql, then enable and start the {} service.", service),
                "mongodb" => format!(" After installation, enable and start the {} service to run MongoDB server.", service),
                "redis" => format!(" After installation, enable and start the {} service. Configuration file: /etc/redis/redis.conf", service),
                _ => String::new(),
            };

            let notes = format!("{} installed. {}.{}", db_name, description, init_notes);

            return Ok(ActionPlan {
                analysis: format!("Installing {} database", db_name),
                goals: vec![format!("Install {}", db_name)],
                necessary_checks: vec![],
                command_plan,
                rollback_plan: vec![
                    RollbackStep {
                        id: format!("remove-{}", database),
                        description: format!("Remove {}", db_name),
                        command: if is_aur {
                            format!("yay -Rns --noconfirm {} || paru -Rns --noconfirm {}", package_name, package_name)
                        } else {
                            format!("sudo pacman -Rns --noconfirm {}", package_name)
                        },
                    },
                ],
                notes_for_user: notes,
                meta: anna_common::action_plan_v3::PlanMeta {
                    detection_results: anna_common::action_plan_v3::DetectionResults {
                        de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                    },
                    template_used: Some("databases_install".to_string()),
                    llm_version: "deterministic_recipe_v1".to_string(),
                },
            });
        }

        // SQLite (no service)
        let notes = format!("{} installed. {}. Use 'sqlite3' command to create and manage databases.", db_name, description);

        Ok(ActionPlan {
            analysis: format!("Installing {} database tools", db_name),
            goals: vec![format!("Install {}", db_name)],
            necessary_checks: vec![],
            command_plan,
            rollback_plan: vec![
                RollbackStep {
                    id: format!("remove-{}", database),
                    description: format!("Remove {}", db_name),
                    command: format!("sudo pacman -Rns --noconfirm {}", package_name),
                },
            ],
            notes_for_user: notes,
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("databases_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("databases.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking installed database systems".to_string(),
            goals: vec!["List installed databases".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-databases".to_string(),
                    description: "List database packages".to_string(),
                    command: "pacman -Q mariadb mysql mongodb-bin redis sqlite 2>/dev/null || echo 'No database systems installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "check-db-services".to_string(),
                    description: "Check database services".to_string(),
                    command: "systemctl status mariadb mysqld mongodb redis 2>/dev/null | grep -E '(Loaded|Active):' || echo 'No database services configured'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed database systems and service status".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("databases_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_databases_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("databases.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListDatabases"));

        Ok(ActionPlan {
            analysis: "Showing available database systems".to_string(),
            goals: vec!["List available databases for Arch Linux".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-databases".to_string(),
                    description: "Show available databases".to_string(),
                    command: r#"echo 'Database Systems for Arch Linux:

Relational Databases (SQL):
- MariaDB (official) - Open-source MySQL fork with active development and improvements
- MySQL (AUR) - Oracle MySQL database server, industry standard
- PostgreSQL (official) - Advanced object-relational database with strong standards compliance
- SQLite (official) - Self-contained, serverless, zero-configuration SQL database engine

NoSQL Document Databases:
- MongoDB (AUR) - Document-oriented database storing data in JSON-like BSON format
- CouchDB (official) - Document database with HTTP API and multi-master replication
- RavenDB (AUR) - ACID NoSQL document database with .NET focus

Key-Value Stores:
- Redis (official) - In-memory data structure store, cache, message broker, and streaming engine
- Memcached (official) - High-performance distributed memory caching system
- KeyDB (AUR) - Redis fork with multithreading and better performance

Column-Family Databases:
- Cassandra (AUR) - Distributed wide-column store for high availability and scalability
- ScyllaDB (AUR) - Cassandra-compatible database written in C++ for better performance

Graph Databases:
- Neo4j (AUR) - Native graph database for connected data with Cypher query language
- ArangoDB (AUR) - Multi-model database supporting documents, graphs, and key-value

Time-Series Databases:
- InfluxDB (AUR) - Time-series database for metrics, events, and real-time analytics
- TimescaleDB (AUR) - PostgreSQL extension for time-series data
- Prometheus (official) - Monitoring system with time-series database

Embedded Databases:
- SQLite (official) - Zero-configuration embedded database, perfect for applications
- Berkeley DB (official) - High-performance embedded database library
- LMDB (official) - Lightning Memory-Mapped Database, ultra-fast key-value store

Database Management Tools:
- DBeaver (official) - Universal database manager supporting MySQL, PostgreSQL, SQLite, etc.
- MySQL Workbench (AUR) - Visual tool for MySQL database design and administration
- pgAdmin (official) - PostgreSQL administration and development platform
- MongoDB Compass (AUR) - Official GUI for MongoDB database exploration
- RedisInsight (AUR) - Redis GUI for visualization and optimization

Command-Line Tools:
- mycli (official) - MySQL/MariaDB client with auto-completion and syntax highlighting
- pgcli (official) - PostgreSQL client with auto-completion and syntax highlighting
- litecli (official) - SQLite client with auto-completion and syntax highlighting
- redis-cli (redis package) - Official Redis command-line interface

Comparison by Use Case:

General Purpose Web Applications:
- MariaDB/MySQL: Best for traditional web apps, CMS (WordPress, Drupal), e-commerce
- PostgreSQL: Best for complex queries, data integrity, advanced features
- MongoDB: Best for flexible schemas, rapid development, JSON data

High Performance / Scale:
- Redis: Best for caching, session storage, real-time applications
- Cassandra/ScyllaDB: Best for massive scale, distributed systems, write-heavy workloads
- PostgreSQL: Best for complex queries with good performance

Development / Embedded:
- SQLite: Best for mobile apps, desktop apps, embedded systems, testing
- Redis: Best for caching layer, task queues, pub/sub messaging

Analytics / Time-Series:
- InfluxDB: Best for IoT, monitoring, metrics collection
- TimescaleDB: Best for time-series with SQL and PostgreSQL ecosystem
- ClickHouse (AUR): Best for OLAP, analytics, log analysis

Features Comparison:

MariaDB/MySQL:
- ACID transactions, replication, stored procedures, triggers
- Good performance for read-heavy workloads
- Wide ecosystem support and hosting options
- Easy migration between MariaDB and MySQL

PostgreSQL:
- Advanced SQL features (CTEs, window functions, full-text search)
- JSON/JSONB support for semi-structured data
- Foreign data wrappers for accessing external data
- Strong community and extensive extensions

MongoDB:
- Flexible schema, horizontal scaling (sharding)
- Rich query language with aggregation framework
- Change streams for real-time data
- ACID transactions across documents (4.0+)

Redis:
- Sub-millisecond latency, extremely fast
- Data structures (strings, hashes, lists, sets, sorted sets)
- Pub/Sub messaging, streams, Lua scripting
- Persistence options (RDB snapshots, AOF logs)

SQLite:
- Zero configuration, single file database
- Cross-platform, stable file format
- Full SQL support with ACID properties
- Public domain, no licensing restrictions

Installation Notes:

MariaDB/MySQL Setup:
1. Install: sudo pacman -S mariadb (or yay -S mysql)
2. Initialize: sudo mysql_install_db --user=mysql --basedir=/usr --datadir=/var/lib/mysql
3. Enable: sudo systemctl enable --now mariadb
4. Secure: sudo mysql_secure_installation

PostgreSQL Setup:
1. Install: sudo pacman -S postgresql
2. Initialize: sudo -u postgres initdb -D /var/lib/postgres/data
3. Enable: sudo systemctl enable --now postgresql
4. Create user: sudo -u postgres createuser --interactive

MongoDB Setup:
1. Install: yay -S mongodb-bin
2. Enable: sudo systemctl enable --now mongodb
3. Configuration: /etc/mongodb.conf
4. Connect: mongosh

Redis Setup:
1. Install: sudo pacman -S redis
2. Enable: sudo systemctl enable --now redis
3. Configuration: /etc/redis/redis.conf
4. Connect: redis-cli

SQLite Usage:
1. Install: sudo pacman -S sqlite
2. Create DB: sqlite3 mydb.db
3. No service needed (embedded)
4. GUI tools: sqlitebrowser (official)

Performance Tuning:

MariaDB/MySQL:
- Configure /etc/my.cnf for buffer pools, connections
- Enable slow query log for optimization
- Use EXPLAIN for query analysis

PostgreSQL:
- Configure postgresql.conf for shared_buffers, work_mem
- Use EXPLAIN ANALYZE for query optimization
- Enable pg_stat_statements extension

MongoDB:
- Create indexes for query optimization
- Use aggregation pipeline efficiently
- Configure WiredTiger cache size

Redis:
- Configure maxmemory and eviction policies
- Use pipelining for bulk operations
- Monitor with INFO command

Backup Strategies:

MariaDB/MySQL:
- mysqldump for logical backups
- Percona XtraBackup for hot backups
- Binary log replication

PostgreSQL:
- pg_dump for logical backups
- pg_basebackup for physical backups
- Write-Ahead Logging (WAL) archiving

MongoDB:
- mongodump/mongorestore for backups
- Replica sets for high availability
- Ops Manager for enterprise backups

Redis:
- RDB snapshots for point-in-time backups
- AOF (Append-Only File) for durability
- Replication for redundancy

SQLite:
- Simple file copy when database is idle
- Backup API for online backups
- WAL mode for better concurrency

Security Considerations:

- Change default passwords immediately after installation
- Use strong authentication (password policies, 2FA where available)
- Configure firewall rules to restrict database access
- Enable SSL/TLS for network connections
- Regular security updates
- Principle of least privilege for user permissions
- Audit logging for compliance requirements
- Encrypt sensitive data at rest and in transit

Replication & High Availability:

MariaDB: Master-slave, master-master, Galera Cluster
PostgreSQL: Streaming replication, logical replication, Patroni
MongoDB: Replica sets, sharding for horizontal scaling
Redis: Master-replica replication, Redis Sentinel, Redis Cluster
Cassandra: Multi-datacenter replication built-in

Monitoring Tools:

- Prometheus + Grafana for metrics visualization
- pgBadger for PostgreSQL log analysis
- MySQL Enterprise Monitor or Percona Monitoring
- MongoDB Cloud Manager
- RedisInsight for Redis monitoring
- Netdata for real-time system and database monitoring'"#.to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Comprehensive database options for Arch Linux with setup guides".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("databases_list_databases".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches() {
        assert!(DatabasesRecipe::matches_request("install mariadb"));
        assert!(DatabasesRecipe::matches_request("install mysql"));
        assert!(DatabasesRecipe::matches_request("setup mongodb"));
        assert!(DatabasesRecipe::matches_request("install redis"));
        assert!(DatabasesRecipe::matches_request("install sqlite"));
        assert!(DatabasesRecipe::matches_request("install database"));
        assert!(!DatabasesRecipe::matches_request("what is mysql"));
    }

    #[test]
    fn test_install_plan_mariadb() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install mariadb".to_string());
        let plan = DatabasesRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
        assert!(plan.notes_for_user.contains("MariaDB"));
    }

    #[test]
    fn test_install_plan_sqlite() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install sqlite".to_string());
        let plan = DatabasesRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
        assert!(plan.notes_for_user.contains("SQLite"));
    }

    #[test]
    fn test_detect_database() {
        assert_eq!(DatabasesRecipe::detect_database("install mariadb"), "mariadb");
        assert_eq!(DatabasesRecipe::detect_database("setup mongodb"), "mongodb");
        assert_eq!(DatabasesRecipe::detect_database("install redis"), "redis");
    }
}
