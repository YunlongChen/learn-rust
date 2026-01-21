use crate::configs::database::DatabaseConfig;
use crate::configs::server::ServerConfig;
use ::config::{Config, FileFormat};
use anyhow::{anyhow, Context};
use directories::ProjectDirs;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;
use tracing::{error, info};

pub mod config_settings;
pub mod config_window;
pub mod database;
pub mod gui_config;
pub mod server;

static CONFIG: LazyLock<AppConfig> = LazyLock::new(|| {
    info!("加载数据库配置。。。");
    let config = AppConfig::load()
        .map_err(|err| {
            error!("读取配置文件发生异常,错误信息：{:?}", err);
        })
        .unwrap_or_else(|err| {
            error!("读取配置文件发生异常，使用默认配置:错误信息：{:?}", err);
            AppConfig::default()
        });
    info!("加载配置成功:{:?}", config);
    config
});

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        Config::builder()
            .add_source(
                config::File::with_name("application")
                    .required(true)
                    .format(FileFormat::Yaml),
            )
            .build()
            .with_context(|| anyhow!("Failed to load config"))?
            .try_deserialize()
            .with_context(|| anyhow::anyhow!("Failed to deserialize config"))
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        info!("读取应用默认配置。。。。。");
        AppConfig {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
        }
    }
}

pub fn get() -> &'static AppConfig {
    info!("获取数据库配置");
    &CONFIG
}

/// 获取数据库路径
pub fn get_database_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("xyz", "stanic", "DomainManager") {
        let mut path = proj_dirs.data_dir().to_path_buf();
        path.push("database");
        path.push(crate::storage::database::DB_FILE_NAME);
        return path;
    }

    // 后备方案：当前目录
    PathBuf::from(crate::storage::database::DB_FILE_NAME)
}

impl From<&DatabaseConfig> for String {
    fn from(value: &DatabaseConfig) -> Self {
        let encoded_password = utf8_percent_encode(value.password(), NON_ALPHANUMERIC);

        info!("当前数据库类型：「{}」", value.db_type());
        match value.db_type() {
            "postgres" => {
                info!("初始化 postgres 数据库");
                format!(
                    "postgres://{}:{}@{}:{}/{}",
                    value.user(),
                    encoded_password,
                    value.host(),
                    value.port(),
                    value.schema()
                )
            }
            _ => {
                info!("初始化sqlite数据库");
                let db_path = get_database_path();
                info!("路径地址：{:?}", db_path);

                if let Some(parent) = db_path.parent() {
                    // 如果目录不存在，则递归创建
                    if !parent.exists() {
                        fs::create_dir_all(parent).expect("创建路径发生了异常");
                    }
                }

                // 使用sqlite数据库
                let string = format!(
                    "sqlite://{}?mode=rwc",
                    get_database_path().to_str().unwrap()
                );

                string
            }
        }
    }
}
