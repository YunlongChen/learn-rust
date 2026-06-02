pub mod logger;
pub mod models;

use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, SqlxSqliteConnector, Statement};
use tracing::info;

pub async fn init_db(database_url: &str) -> DatabaseConnection {
    let url = if let Ok(env_url) = std::env::var("DOMAIN_STUN_DB_URL") {
        info!("Using database URL from DOMAIN_STUN_DB_URL: {}", env_url);
        env_url
    } else {
        database_url.to_string()
    };

    info!("Connecting to database: {}", url);

    let db = if url.starts_with("sqlite:") {
        let path = url.strip_prefix("sqlite:").unwrap_or("domain-stun.db");
        let abs_path = if path == ":memory:" {
            path.to_string()
        } else {
            let p = std::path::Path::new(path);
            if p.is_relative() {
                let cwd = std::env::current_dir().expect("Failed to get current dir");
                let abs = cwd.join(path);
                if let Some(parent) = abs.parent() {
                    std::fs::create_dir_all(parent)
                        .expect(&format!("Failed to create directory: {:?}", parent));
                }
                abs.to_string_lossy().to_string()
            } else {
                path.to_string()
            }
        };
        info!("Opening SQLite at: {}", abs_path);
        let opts = if abs_path == ":memory:" {
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&abs_path)
        } else {
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&abs_path)
                .create_if_missing(true)
        };
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await
            .expect("Failed to connect to SQLite database");
        SqlxSqliteConnector::from_sqlx_sqlite_pool(pool)
    } else {
        Database::connect(&url)
            .await
            .expect("Failed to connect to database")
    };

    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        r#"
        CREATE TABLE IF NOT EXISTS agents (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            public_addr TEXT,
            nat_type TEXT NOT NULL DEFAULT '',
            connected_at TEXT NOT NULL,
            last_seen TEXT NOT NULL
        )
        "#
        .to_string(),
    ))
    .await
    .expect("Failed to create agents table");

    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        r#"
        CREATE TABLE IF NOT EXISTS logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            level TEXT NOT NULL,
            category TEXT NOT NULL,
            agent_id TEXT,
            message TEXT NOT NULL,
            details TEXT
        )
        "#
        .to_string(),
    ))
    .await
    .expect("Failed to create logs table");

    info!("Database initialized successfully");
    db
}
