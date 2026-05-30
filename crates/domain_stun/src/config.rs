//! Configuration for Domain STUN server

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StunConfig {
    pub bind_host: String,
    pub bind_port: u16,
    pub realm: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TurnConfig {
    pub bind_host: String,
    pub bind_port: u16,
    pub max_allocations: usize,
    pub default_lifetime: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub stun: StunConfig,
    pub turn: TurnConfig,
    pub database: DatabaseConfig,
}

impl Config {
    pub fn load() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3479,
            },
            stun: StunConfig {
                bind_host: "0.0.0.0".to_string(),
                bind_port: 3478,
                realm: "domain-stun".to_string(),
            },
            turn: TurnConfig {
                bind_host: "0.0.0.0".to_string(),
                bind_port: 3478,
                max_allocations: 1000,
                default_lifetime: 600,
            },
            database: DatabaseConfig {
                url: "sqlite:domain_stun.db".to_string(),
            },
        }
    }
}
