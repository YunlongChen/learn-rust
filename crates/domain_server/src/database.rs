use crate::config::database::DatabaseConfig;
use anyhow::Context;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, Statement};
use std::cmp::{max, min};
use std::time::Duration;

pub async fn establish_connection(
    database_config: &DatabaseConfig,
) -> anyhow::Result<DatabaseConnection> {
    dbg!("建立数据库连接");

    let encoded_password = utf8_percent_encode(database_config.password(), NON_ALPHANUMERIC);

    let string = format!(
        "postgres://{}:{}@{}:{}/{}",
        database_config.user(),
        encoded_password,
        database_config.host(),
        database_config.port(),
        database_config.schema()
    );
    dbg!("建立数据库连接，地址：「{}」", &string);

    let mut options = ConnectOptions::new(string);

    let cpus = num_cpus::get() as u32;
    dbg!("当前系统cpu数量:{}", cpus);

    options
        .max_connections(10)
        .sqlx_logging(true)
        .connect_timeout(Duration::from_secs(3))
        .min_connections(min(2, 10))
        .max_connections(max(cpus * 8, 30))
        .connect_timeout(Duration::from_secs(3))
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(60))
        .max_lifetime(Duration::from_secs(300));

    let result = Database::connect(options)
        .await
        .with_context(|| anyhow::anyhow!("初始化数据库连接失败!"));
    dbg!("连接成功");
    result
}
