use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainManagerError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("API error: {0}")]
    Api(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Service error: {0}")]
    Service(String),
}

impl DomainManagerError {
    pub fn api(msg: impl Into<String>) -> Self {
        DomainManagerError::Api(msg.into())
    }

    pub fn config(msg: impl Into<String>) -> Self {
        DomainManagerError::Config(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        DomainManagerError::NotFound(msg.into())
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        DomainManagerError::Validation(msg.into())
    }

    pub fn service(msg: impl Into<String>) -> Self {
        DomainManagerError::Service(msg.into())
    }
}
