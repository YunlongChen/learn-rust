//! Configuration module for agent-management service.

use config::{Config, ConfigError, File};
use serde::Deserialize;

/// Server configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub ws_port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            ws_port: 8081,
        }
    }
}

/// Database configuration
#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub max_connections: u32,
}

impl DatabaseConfig {
    /// Get connection URL with credentials if provided
    pub fn connection_url(&self) -> String {
        match (&self.username, &self.password) {
            (Some(user), Some(pass)) => {
                format!("postgres://{}:{}@{}", user, pass, self.url.trim_start_matches("postgres://"))
            }
            (Some(user), None) => {
                format!("postgres://{}@{}", user, self.url.trim_start_matches("postgres://"))
            }
            _ => self.url.clone(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://192.168.3.112:5432/agent_management".to_string(),
            username: Some("agent_management".into()),
            password: Some("123456".into()),
            max_connections: 10,
        }
    }
}

/// gRPC configuration
#[derive(Debug, Deserialize, Clone)]
pub struct GrpcConfig {
    pub host: String,
    pub port: u16,
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 50051,
        }
    }
}

/// REST API configuration
#[derive(Debug, Deserialize, Clone)]
pub struct RestConfig {
    pub host: String,
    pub port: u16,
}

impl Default for RestConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
        }
    }
}

/// Main application configuration
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub grpc: GrpcConfig,
    pub rest: RestConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            grpc: GrpcConfig::default(),
            rest: RestConfig::default(),
        }
    }
}

impl AppConfig {
    /// Load configuration from file and environment variables
    pub fn load() -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(File::with_name("config").required(false))
            .add_source(
                config::Environment::with_prefix("AGENT_MANAGEMENT")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        config.try_deserialize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.ws_port, 8081);
        assert_eq!(config.grpc.port, 50051);
        assert_eq!(config.rest.port, 8080);
    }
}
