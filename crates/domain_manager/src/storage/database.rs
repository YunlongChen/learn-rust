use crate::configs::database::DatabaseConfig;
use anyhow::Context;
use directories::ProjectDirs;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use sea_orm::entity::prelude::*;
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, Schema};
use std::cmp::min;
use std::path::PathBuf;
use std::time::Duration;

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

pub async fn init_database(database_config: &DatabaseConfig) -> anyhow::Result<DatabaseConnection> {
    dbg!("建立数据库连接");

    let encoded_password = utf8_percent_encode(database_config.password(), NON_ALPHANUMERIC);

    // postgres
    let string = format!(
        "postgres://{}:{}@{}:{}/{}",
        database_config.user(),
        encoded_password,
        database_config.host(),
        database_config.port(),
        database_config.schema()
    );

    // 使用sqlite数据库
    // sqlite://path/to/db.sqlite?mode=rwc

    dbg!("建立数据库连接，地址：「{}」", &string);

    let mut options = ConnectOptions::new(string);

    let cpus = num_cpus::get() as u32;
    dbg!("当前系统cpu数量:{}", cpus);

    options
        .max_connections(10)
        .sqlx_logging(true)
        .min_connections(min(2, 10))
        .connect_timeout(Duration::from_secs(3))
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(60))
        .max_lifetime(Duration::from_secs(300))
        .sqlx_logging_level(log::LevelFilter::Trace);

    let result = Database::connect(options)
        .await
        .with_context(|| anyhow::anyhow!("初始化数据库连接失败!"));
    dbg!("连接成功");
    result
}
//
// /// 初始化数据库连接
// pub fn init_database() -> Result<Connection, Box<dyn Error>> {
//     let db_path = get_database_path();
//     let db_dir = db_path.parent().expect("数据库路径无效");
//
//     let user_json = serde_json::to_string(&db_dir).unwrap();
//     info!("初始化数据路径：{}", user_json);
//
//     // 创建目录（如果不存在）
//     if !db_dir.exists() {
//         info!("初始化路径不存在：创建路径");
//         std::fs::create_dir_all(db_dir)?;
//     }
//
//     info!("数据库初始化成功");
//     // 打开数据库连接
//     let mut conn = Connection::open(&db_path)?;
//
//     // 应用加密（如果启用）
//     #[cfg(feature = "database-encryption")]
//     {
//         let key = DatabaseKeyManager::get_database_key();
//         conn.pragma_update(None, "key", &key)?;
//     }
//
//     // 设置数据库参数
//     conn.pragma_update(None, "journal_mode", "WAL")?;
//     conn.pragma_update(None, "synchronous", "NORMAL")?;
//     conn.pragma_update(None, "foreign_keys", "ON")?;
//
//     // 检查数据库版本
//     let current_version: u32 = conn
//         .query_row("PRAGMA user_version", [], |row| row.get(0))
//         .unwrap_or(0);
//
//     info!("当前数据库版：「{}」", current_version);
//     info!("当前数据库版本：「{}」", current_version);
//
//     // 执行迁移
//     if current_version < CURRENT_DB_VERSION {
//         info!("本地数据库小于当前版本，开始进行版本迁移");
//         let migrations = crate::storage::migrations::get_migrations();
//         migrations.to_version(&mut conn, CURRENT_DB_VERSION as usize)?;
//     }
//     info!("初始化数据库成功");
//     Ok(conn)
// }

/// 创建内存数据库（用于测试）
#[cfg(test)]
pub async fn init_memory_database() -> anyhow::Result<DatabaseConnection> {
    let result = Database::connect("sqlite::memory:")
        .await
        .with_context(|| anyhow::anyhow!("初始化数据库连接失败!"));
    dbg!("连接成功");

    // 获取数据库后端类型
    match result {
        Ok(connection) => {
            dbg!("获取数据库连接成功");
            let builder = connection.get_database_backend();
            let schema = Schema::new(builder);

            let model = super::domain::ActiveModel {
                id: Default::default(),
                name: Default::default(),
                provider_id: Default::default(),
                status: Default::default(),
                created_at: Default::default(),
                updated_at: Default::default(),
            };

            //
            // // 创建所有实体对应的表
            // for entity in get_all_entities() {
            //     let stmt = builder.build(&schema.create_table_from_entity(entity));
            //     connection.execute(stmt).await?;
            // }
            Ok(connection)
        }
        Err(err) => {
            dbg!("获取数据库连接失败！");
            dbg!("{?}", &err);
            Err(err)
        }
    }

    // // 创建所有实体对应的表
    // for entity in get_all_entities() {
    //     let stmt = builder.build(&schema.create_table_from_entity(entity));
    //     db.execute(stmt).await?;
    // }
    //
    //
    // result
    // // 执行迁移
    // let migrations = crate::storage::migrations::get_migrations();
    // migrations.to_latest(&mut conn)?;

    // Ok(conn)
}

// async fn setup_schema(db: &DbConn) {
//
//     // Setup Schema helper
//     let schema = Schema::new(DbBackend::Sqlite);
//
//     // Derive from Entity
//     let stmt: TableCreateStatement = schema.create_table_from_entity(DomainEntity);
//
//     // Or setup manually
//     assert_eq!(
//         stmt.build(SqliteQueryBuilder),
//         Table::create()
//             .table(DomainEntity)
//             .col(
//                 ColumnDef::new(DomainEntity ::Column::Id)
//                     .integer()
//                     .not_null()
//             )
//             .build(SqliteQueryBuilder)
//     );
//
//     // Execute create table statement
//     let result = db
//         .execute(db.get_database_backend().build(&stmt))
//         .await;
// }
