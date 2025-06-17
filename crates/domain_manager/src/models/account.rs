use clap::builder::Str;
use secrecy::SecretString;

/// 账户模型
#[derive(Debug)]
pub struct Account {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub encrypted_password: SecretString,
    pub salt: String,
    pub api_keys: Vec<ApiKey>,
    pub created_at: String,
    pub last_login: Option<String>,
}

/// 新账户创建模型
pub struct NewAccount {
    pub username: String,
    pub email: String,
    pub password: SecretString,
    pub master_key: SecretString,
    pub api_keys: Vec<NewApiKey>,
    pub created_at: String,
}

/// API密钥模型
#[derive(Debug)]
pub struct ApiKey {
    pub id: i64,
    pub key_name: String,
    pub key: SecretString,
}

/// 新API密钥模型
pub struct NewApiKey {
    pub key_name: String,
    pub key: SecretString,
}
