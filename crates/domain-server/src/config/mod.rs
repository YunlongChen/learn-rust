use crate::config::database::DatabaseConfig;
use crate::config::server::ServerConfig;
use anyhow::{anyhow, Context};
use config::{Config, FileFormat};
use serde::Deserialize;
use std::sync::LazyLock;
use tracing::info;

pub mod database;
pub mod server;

static CONFIG: LazyLock<AppConfig> = LazyLock::new(|| {
    info!("Loading Config");
    AppConfig::load().expect("Failed to Initialized Config!")
});

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: database::DatabaseConfig,
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

pub fn get() -> &'static AppConfig {
    &CONFIG
}
