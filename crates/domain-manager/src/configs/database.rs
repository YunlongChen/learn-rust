use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    db_type: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    user: Option<String>,
    password: Option<String>,
    database: Option<String>,
    schema: Option<String>,
}

impl DatabaseConfig {
    pub fn db_type(&self) -> &str {
        match &self.db_type {
            None => "sqlite",
            Some(db) => db.as_str(),
        }
    }

    pub fn host(&self) -> &str {
        self.host.as_deref().unwrap_or("localhost")
    }

    pub fn port(&self) -> u16 {
        self.port.unwrap_or(5432)
    }

    pub fn schema(&self) -> &str {
        self.schema.as_deref().unwrap_or("public")
    }
    pub fn user(&self) -> &str {
        self.user.as_deref().unwrap_or("postgres")
    }

    pub fn password(&self) -> &str {
        self.password.as_deref().unwrap_or("postgres")
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            db_type: None,
            host: None,
            port: None,
            user: None,
            password: None,
            database: None,
            schema: None,
        }
    }
}
