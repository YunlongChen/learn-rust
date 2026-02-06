
use serde::{Deserialize, Serialize};
use crate::gui::model::domain::DnsProvider;
use crate::gui::types::credential::Credential;

/// 账户模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub salt: String,
    pub api_keys: Vec<ApiKey>,
    pub created_at: String,
    pub last_login: Option<String>,
    pub credential_type: String,
    pub credential_data: String,
    pub provider_type: String,
}

/// 新账户创建模型
#[derive(Debug, Clone)]
pub struct NewAccount {
    pub username: String,
    pub email: String,
    pub provider: DnsProvider,
    pub credential: Credential,
}

/// API密钥模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: i32,
    pub key_name: String,
    pub key: String,
}

/// 新API密钥模型
pub struct NewApiKey {
    pub key_name: String,
    pub key: String,
}
