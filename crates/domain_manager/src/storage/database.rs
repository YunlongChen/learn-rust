use crate::configs::database::DatabaseConfig;
use crate::models::domain::{DomainStatus, NewDomain};
use crate::storage::migration::migration::Migrator;
use crate::storage::{add_domain, add_domain_many, count_all_domains, list_domains};
use anyhow::Context;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use sea_orm::entity::prelude::*;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, Schema};
use sea_orm_migration::MigratorTrait;
use std::cmp::min;
use std::time::Duration;
use tracing::log::LevelFilter::Info;
use tracing::{debug, info};

// 数据库配置
pub(crate) const DB_FILE_NAME: &str = "domain_manager.db";
const CURRENT_DB_VERSION: u32 = 1;

pub async fn init_database(database_config: &DatabaseConfig) -> anyhow::Result<DatabaseConnection> {
    debug!("建立数据库连接");
    // postgres
    let url: String = database_config.into();
    debug!("建立数据库连接，地址：「{}」", &url);

    let mut options = ConnectOptions::new(url);

    let cpus = num_cpus::get() as u32;
    debug!("当前系统cpu数量:{}", cpus);

    options
        .max_connections(10)
        .sqlx_logging(true)
        .min_connections(min(2, 10))
        .connect_timeout(Duration::from_secs(3))
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(60))
        .max_lifetime(Duration::from_secs(300))
        .sqlx_logging_level(Info);

    let result = Database::connect(options)
        .await
        .with_context(|| anyhow::anyhow!("初始化数据库连接失败!"));
    info!("连接成功");

    match result {
        Ok(connection) => {
            info!("连接创建成功");
            Migrator::up(&connection, None)
                .await
                .expect("迁移数据库发生了异常！");
            Ok(connection)
        }
        Err(err) => Err(err),
    }
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
    debug!("连接成功");

    // 获取数据库后端类型
    match result {
        Ok(connection) => {
            debug!("获取数据库连接成功");
            let builder = connection.get_database_backend();
            let _schema = Schema::new(builder);

            Migrator::up(&connection, None)
                .await
                .expect("移植数据库发生了异常！");

            debug!("连接创建成功");
            Migrator::up(&connection, None)
                .await
                .expect("迁移数据库发生了异常！");
            connection.ping().await.expect("数据库连通性检查失败");
            Ok(connection)
        }
        Err(err) => {
            debug!("获取数据库连接失败！{}", err);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::model::domain::DnsProvider;
    use crate::gui::types::credential::{Credential, UsernamePasswordCredential};
    use crate::models::account::NewAccount;
    use crate::storage::{create_account, init_memory_database};
use crate::tests::test_utils::init_test_env;
use chrono::Utc;
use secrecy::{ExposeSecret, SecretString};

    #[tokio::test]
    pub async fn it_works() {
        init_test_env();
        let connection = init_memory_database().await.unwrap();

        let _vault = String::from("stanic");

        let password: SecretString = SecretString::from("12123");

        let account = create_account(
            connection.clone(),
            NewAccount {
                provider: DnsProvider::Aliyun,
                username: "stanic".to_string(),
                email: "example@qq.com".to_string(),
                credential: Credential::UsernamePassword(UsernamePasswordCredential {
                    username: _vault.clone(),
                    password: password.expose_secret().to_string(),
                }),
            },
        )
        .await
        .expect("查询数据库发生了异常".into());

        assert_eq!(account.username, "stanic", "变量名错误");

        let new_account = account.clone();

        let domain = add_domain(
            &connection,
            NewDomain {
                domain_name: String::from("chenyunlong.cn"),
                registration_date: Some(Utc::now().to_string()),
                expiration_date: None,
                registrar: None,
                status: DomainStatus::Active,
                account_id: new_account.id,
            },
        )
        .await
        .unwrap();

        assert_eq!(domain.domain_name, "chenyunlong.cn", "保存用户名异常");
        assert_eq!(domain.account_id, account.id, "保存用户名异常");
        assert_eq!(domain.updated_at, None);

        let domain_list = crate::storage::get_account_domains(&connection, Some(account.id))
            .await
            .expect("查询账号失败");
        assert_eq!(domain_list.len(), 1, "获取账号失败");

        let domain_list = crate::storage::get_account_domains(&connection, None)
            .await
            .expect("查询账号失败");
        assert_eq!(domain_list.len(), 1, "获取账号失败");

        let domain_list = list_domains(&connection).await.expect("查询账号失败");
        assert_eq!(domain_list.len(), 1, "获取账号失败");

        add_domain_many(
            &connection,
            vec![NewDomain {
                domain_name: String::from("stanic.xyz"),
                registration_date: Some(Utc::now().to_string()),
                expiration_date: None,
                registrar: None,
                status: DomainStatus::Active,
                account_id: new_account.id,
            }],
        )
        .await
        .unwrap();

        let domain_list = crate::storage::get_account_domains(&connection, Some(account.id))
            .await
            .expect("查询域名失败");
        assert_eq!(domain_list.len(), 2, "获取账号失败");

        let result = domain_list
            .iter()
            .find(|domain| domain.domain_name == "stanic.xyz")
            .unwrap();
        assert_eq!(result.status, DomainStatus::Active);
        assert_eq!(result.account_id, account.id);

        let domains_count = count_all_domains(&connection).await.unwrap();
        assert_eq!(domains_count, 2, "查询域名数量异常，数量对不上！");
    }
}
