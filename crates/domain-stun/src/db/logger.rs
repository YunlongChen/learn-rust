use chrono::Local;
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement, Value};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: i64,
    pub timestamp: String,
    pub level: String,
    pub category: String,
    pub agent_id: Option<String>,
    pub message: String,
    pub details: Option<String>,
}

pub async fn log_to_db(
    db: &DatabaseConnection,
    level: &str,
    category: &str,
    agent_id: Option<&str>,
    message: &str,
    details: Option<&str>,
) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let formatted = format!("[{}] [{}] [{}] {}", timestamp, level, category, message);
    match level {
        "ERROR" => error!("{}", formatted),
        "WARN" => tracing::warn!("{}", formatted),
        "DEBUG" => tracing::debug!("{}", formatted),
        _ => info!("{}", formatted),
    }

    let result = db
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "INSERT INTO logs (timestamp, level, category, agent_id, message, details) VALUES ($1, $2, $3, $4, $5, $6)",
            vec![
                timestamp.into(),
                level.to_string().into(),
                category.to_string().into(),
                agent_id.map(|s| s.to_string()).into(),
                message.to_string().into(),
                details.map(|s| s.to_string()).into(),
            ],
        ))
        .await;

    if let Err(e) = result {
        error!("Failed to write log to database: {}", e);
    }
}

pub async fn query_logs(
    db: &DatabaseConnection,
    level: Option<&str>,
    category: Option<&str>,
    page: u32,
    page_size: u32,
) -> (Vec<LogEntry>, u64) {
    let offset = (page.saturating_sub(1) * page_size) as i64;

    let mut conditions = Vec::new();
    let mut values: Vec<Value> = Vec::new();

    if let Some(lvl) = level {
        if !lvl.is_empty() {
            conditions.push(format!("level = ${}", values.len() + 1));
            values.push(lvl.to_string().into());
        }
    }
    if let Some(cat) = category {
        if !cat.is_empty() {
            conditions.push(format!("category = ${}", values.len() + 1));
            values.push(cat.to_string().into());
        }
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let count_sql = format!("SELECT COUNT(*) as cnt FROM logs {}", where_clause);
    let total = match db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            &count_sql,
            values.clone(),
        ))
        .await
    {
        Ok(Some(row)) => {
            use sea_orm::TryGetable;
            row.try_get_by_index::<i64>(0).unwrap_or(0) as u64
        }
        _ => 0,
    };

    let param_offset = values.len() + 1;
    let mut query_values = values;
    query_values.push(page_size.into());
    query_values.push(offset.into());

    let query_sql = format!(
        "SELECT id, timestamp, level, category, agent_id, message, details FROM logs {} ORDER BY id DESC LIMIT ${} OFFSET ${}",
        where_clause, param_offset, param_offset + 1
    );

    let mut entries = Vec::new();
    if let Ok(rows) = db
        .query_all(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            &query_sql,
            query_values,
        ))
        .await
    {
        use sea_orm::TryGetable;
        for row in rows {
            entries.push(LogEntry {
                id: row.try_get_by_index::<i64>(0).unwrap_or(0),
                timestamp: row.try_get_by_index::<String>(1).unwrap_or_default(),
                level: row.try_get_by_index::<String>(2).unwrap_or_default(),
                category: row.try_get_by_index::<String>(3).unwrap_or_default(),
                agent_id: row.try_get_by_index::<Option<String>>(4).unwrap_or(None),
                message: row.try_get_by_index::<String>(5).unwrap_or_default(),
                details: row.try_get_by_index::<Option<String>>(6).unwrap_or(None),
            });
        }
    }

    (entries, total)
}
