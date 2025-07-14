use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    port: Option<u16>,
}

impl ServerConfig {
    pub fn port(&self) -> u16 {
        self.port.unwrap_or(8080)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig { port: Some(8080) }
    }
}
