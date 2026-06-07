//! Agent configuration

use clap::{CommandFactory, Parser};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

static CONFIG_FILE_PATH: OnceLock<Option<String>> = OnceLock::new();

/// Proxy authentication for SOCKS5
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Socks5Auth {
    pub username: String,
    pub password: String,
}

/// Proxy authentication for HTTP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpAuth {
    pub username: String,
    pub password: String,
}

/// Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ProxyConfig {
    /// No proxy - direct connection
    #[serde(rename = "none")]
    None,

    /// SOCKS5 proxy
    #[serde(rename = "socks5")]
    Socks5 {
        host: String,
        port: u16,
        auth: Option<Socks5Auth>,
    },

    /// HTTP CONNECT proxy
    #[serde(rename = "http")]
    Http {
        host: String,
        port: u16,
        auth: Option<HttpAuth>,
    },
}

impl Default for ProxyConfig {
    fn default() -> Self {
        ProxyConfig::None
    }
}

/// Reconnection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconnectionConfig {
    /// Initial reconnection delay in milliseconds
    pub base_delay_ms: u64,
    /// Maximum reconnection delay in milliseconds
    pub max_delay_ms: u64,
    /// Maximum retry count, 0 means infinite
    pub max_retries: u32,
    /// Jitter ratio (0.0-1.0)
    pub jitter: f64,
}

impl Default for ReconnectionConfig {
    fn default() -> Self {
        Self {
            base_delay_ms: 1000,
            max_delay_ms: 300_000,
            max_retries: 5,
            jitter: 0.1,
        }
    }
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Hub WebSocket address (host:port)
    pub hub: String,
    /// Agent name
    pub name: String,
    /// Agent key for authentication
    pub key: String,
    /// Agent version
    pub version: String,
    /// Hostname
    pub hostname: Option<String>,
    /// Proxy configuration
    #[serde(default)]
    pub proxy: ProxyConfig,
    /// Reconnection configuration
    #[serde(default)]
    pub reconnection: ReconnectionConfig,
    /// Reverse tunnel listen port (0 = disabled)
    #[serde(default)]
    pub tunnel_port: u16,
    /// P2P listen port (0 = disabled)
    #[serde(default)]
    pub p2p_port: u16,
}

/// File configuration structure for TOML
#[derive(Debug, Deserialize, Serialize)]
struct FileConfig {
    #[serde(default)]
    agent: Option<FileAgentConfig>,
    #[serde(default)]
    proxy: Option<ProxyConfig>,
    #[serde(default)]
    reconnection: Option<ReconnectionConfig>,
    #[serde(default)]
    tunnel: Option<FileTunnelConfig>,
    #[serde(default)]
    p2p: Option<FileP2PConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct FileAgentConfig {
    #[serde(default)]
    hub: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct FileTunnelConfig {
    #[serde(default)]
    port: Option<u16>,
}

#[derive(Debug, Deserialize, Serialize)]
struct FileP2PConfig {
    #[serde(default)]
    port: Option<u16>,
}

impl AgentConfig {
    /// Create a new configuration
    pub fn new(hub: String, name: String, key: String) -> Self {
        Self {
            hub,
            name,
            key,
            version: env!("CARGO_PKG_VERSION").to_string(),
            hostname: hostname(),
            proxy: ProxyConfig::None,
            reconnection: ReconnectionConfig::default(),
            tunnel_port: 0,
            p2p_port: 0,
        }
    }

    /// Load configuration from file (TOML format)
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file '{}': {}", path, e))?;

        let file_config: FileConfig = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        let mut config = AgentConfig::new(
            file_config.agent.as_ref().and_then(|a| a.hub.clone()).unwrap_or_else(|| "localhost:8080".to_string()),
            file_config.agent.as_ref().and_then(|a| a.name.clone()).unwrap_or_else(|| "agent".to_string()),
            file_config.agent.as_ref().and_then(|a| a.key.clone()).unwrap_or_else(|| "".to_string()),
        );

        if let Some(proxy) = file_config.proxy {
            config.proxy = proxy;
        }

        if let Some(reconn) = file_config.reconnection {
            config.reconnection = reconn;
        }

        if let Some(tunnel) = file_config.tunnel {
            config.tunnel_port = tunnel.port.unwrap_or(0);
        }

        if let Some(p2p) = file_config.p2p {
            config.p2p_port = p2p.port.unwrap_or(0);
        }

        Ok(config)
    }

    /// Parse from command line arguments with config file and env var support
    pub fn from_args() -> Self {
        let args = Args::parse();

        // Show help and exit if --help is provided
        if args.help {
            let mut prog = Args::command();
            let _ = prog.print_help();
            println!();
            std::process::exit(0);
        }

        // Try to load from config file first
        let mut config = if let Some(config_path) = &args.config {
            AgentConfig::from_file(config_path).unwrap_or_else(|e| {
                eprintln!("Warning: {}", e);
                AgentConfig::new("localhost:8080".to_string(), "agent".to_string(), "".to_string())
            })
        } else if let Some(default_path) = find_default_config() {
            AgentConfig::from_file(&default_path).unwrap_or_else(|e| {
                eprintln!("Warning: Failed to load default config '{}': {}", default_path, e);
                AgentConfig::new("localhost:8080".to_string(), "agent".to_string(), "".to_string())
            })
        } else {
            AgentConfig::new("localhost:8080".to_string(), "agent".to_string(), "".to_string())
        };

        // Override with CLI args if provided
        config.hub = args.hub.unwrap_or(config.hub);
        config.name = args.name.unwrap_or(config.name);
        config.key = args.key.unwrap_or(config.key);
        config.tunnel_port = args.tunnel_port.unwrap_or(config.tunnel_port);
        config.p2p_port = args.p2p_port.unwrap_or(config.p2p_port);

        // Override with environment variables (highest priority)
        config.hub = env::var("DOMAIN_AGENT_HUB").unwrap_or(config.hub);
        config.name = env::var("DOMAIN_AGENT_NAME").unwrap_or(config.name);
        config.key = env::var("DOMAIN_AGENT_KEY").unwrap_or(config.key);

        if let Ok(port) = env::var("DOMAIN_AGENT_TUNNEL_PORT").unwrap_or_default().parse() {
            config.tunnel_port = port;
        }
        if let Ok(port) = env::var("DOMAIN_AGENT_P2P_PORT").unwrap_or_default().parse() {
            config.p2p_port = port;
        }

        config
    }
}

/// Find default config file in common locations
fn find_default_config() -> Option<String> {
    let candidates = [
        "agent.toml",
        "config.toml",
        "/etc/domain_agent/agent.toml",
        "./config/agent.toml",
    ];

    for path in &candidates {
        if Path::new(path).exists() {
            return Some(path.to_string());
        }
    }
    None
}

/// Get the system hostname
fn hostname() -> Option<String> {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
}

#[derive(Parser, Debug)]
#[command(name = "domain-agent")]
#[command(about = "Domain Manager Agent - connects to Domain Manager Hub")]
#[command(disable_help_flag(true))]
struct Args {
    /// Show help information
    #[arg(long, help = "Show this help message")]
    help: bool,

    /// Hub address (host:port)
    #[arg(short, long)]
    hub: Option<String>,

    /// Agent name
    #[arg(short, long)]
    name: Option<String>,

    /// Agent key
    #[arg(short, long)]
    key: Option<String>,

    /// Config file path (TOML format)
    #[arg(short = 'c', long)]
    config: Option<String>,

    /// Reverse tunnel listen port (0 = disabled)
    #[arg(short = 't', long)]
    tunnel_port: Option<u16>,

    /// P2P listen port for direct agent connections (0 = disabled)
    #[arg(short = 'p', long)]
    p2p_port: Option<u16>,
}
