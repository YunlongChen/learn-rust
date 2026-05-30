//! Agent configuration

use clap::Parser;
use serde::{Deserialize, Serialize};

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
        }
    }

    /// Parse from command line arguments
    pub fn from_args() -> Self {
        let args = Args::parse();
        Self::new(args.hub, args.name, args.key)
    }
}

/// Get the system hostname
fn hostname() -> Option<String> {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
}

#[derive(Parser, Debug)]
#[command(name = "domain_agent")]
#[command(about = "Domain Manager Agent - connects to Domain Manager Hub")]
struct Args {
    /// Hub address (host:port)
    #[arg(short, long)]
    hub: String,

    /// Agent name
    #[arg(short, long)]
    name: String,

    /// Agent key
    #[arg(short, long)]
    key: String,
}
