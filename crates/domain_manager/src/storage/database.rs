use directories::ProjectDirs;
use log::{info, log};
use rusqlite::{Connection, Result};
use std::error::Error;
use std::path::PathBuf;

// 数据库配置
const DB_FILE_NAME: &str = "domain_manager.db";
const CURRENT_DB_VERSION: u32 = 1;

/// 获取数据库路径
pub fn get_database_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("xyz", "stanic", "DomainManager") {
        let mut path = proj_dirs.data_dir().to_path_buf();
        path.push("database");
        path.push(DB_FILE_NAME);
        return path;
    }

    // 后备方案：当前目录
    PathBuf::from(DB_FILE_NAME)
}

/// 初始化数据库连接
pub fn init_database() -> Result<Connection, Box<dyn Error>> {
    let db_path = get_database_path();
    let db_dir = db_path.parent().expect("数据库路径无效");

    let user_json = serde_json::to_string(&db_dir).unwrap();
    info!("初始化数据路径：{}", user_json);

    // 创建目录（如果不存在）
    if !db_dir.exists() {
        info!("初始化路径不存在：创建路径");
        std::fs::create_dir_all(db_dir)?;
    }

    info!("数据库初始化成功");
    // 打开数据库连接
    let mut conn = Connection::open(&db_path)?;

    // 应用加密（如果启用）
    #[cfg(feature = "database-encryption")]
    {
        let key = DatabaseKeyManager::get_database_key();
        conn.pragma_update(None, "key", &key)?;
    }

    // 设置数据库参数
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;

    // 检查数据库版本
    let current_version: u32 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap_or(0);

    info!("当前数据库版：「{}」", current_version);
    info!("当前数据库版本：「{}」", current_version);

    // 执行迁移
    if current_version < CURRENT_DB_VERSION {
        info!("本地数据库小于当前版本，开始进行版本迁移");
        let migrations = crate::storage::migrations::get_migrations();
        migrations.to_version(&mut conn, CURRENT_DB_VERSION as usize)?;
    }
    info!("初始化数据库成功");
    Ok(conn)
}

/// 创建内存数据库（用于测试）
#[cfg(test)]
pub fn init_memory_database() -> Result<Connection, Box<dyn Error>> {
    let mut conn = Connection::open_in_memory()?;
    conn.pragma_update(None, "foreign_keys", "ON")?;

    // 执行迁移
    let migrations = crate::storage::migrations::get_migrations();
    migrations.to_latest(&mut conn)?;

    Ok(conn)
}
