use rusqlite_migration::{Migrations, M};

/// 获取数据库迁移配置
pub fn get_migrations() -> Migrations<'static> {
    Migrations::new(vec![
        // 初始版本
        M::up(include_str!(
            "../../resources/migrations/001_initial_schema.sql"
        )),
        // 后续版本示例
        // M::up(include_str!("../../migrations/002_add_domain_settings.sql")),
    ])
}

/// 检查数据库版本
pub fn get_database_version(conn: &rusqlite::Connection) -> rusqlite::Result<u32> {
    let version: u32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    Ok(version)
}
