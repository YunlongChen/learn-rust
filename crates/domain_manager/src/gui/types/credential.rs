use crate::gui::components::credential_form::CredentialForm;
use crate::gui::handlers::message_handler::{MessageCategory, ProviderMessage};
use crate::gui::model::domain::DnsProvider;
use crate::models::account::Account;
use crate::StyleType;
use iced::Element;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tracing::debug;

// 凭证类型枚举
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum Credential {
    UsernamePassword(UsernamePasswordCredential),
    Token(TokenCredential),
    ApiKey(ApiKeyCredential),
    // 添加其他凭证类型...
}

impl Credential {
    pub fn view(&self) -> Element<'_, CredentialMessage, StyleType> {
        match self {
            Credential::UsernamePassword(credential) => credential.view(),
            Credential::Token(cre) => cre.view(),
            Credential::ApiKey(credential) => credential.view(),
        }
    }

    /// 验证凭据
    pub fn validate(dns_provider: &DnsProvider) -> Result<bool, Box<dyn Error>> {
        debug!("验证提供者信息：{:?}", dns_provider);
        Ok(false)
    }

    pub fn credential_type(&self) -> String {
        match self {
            Credential::UsernamePassword(_) => "UsernamePassword".into(),
            Credential::Token(_) => "Token".into(),
            Credential::ApiKey(_) => "ApiKey".into(),
        }
    }

    pub fn raw_data(&self) -> String {
        match self {
            Credential::UsernamePassword(credential) => serde_json::to_string(credential).unwrap(),
            Credential::Token(credential) => serde_json::to_string(credential).unwrap(),
            Credential::ApiKey(credential) => serde_json::to_string(credential).unwrap(),
        }
    }
}

impl TryFrom<Account> for Credential {
    type Error = anyhow::Error; // 或者你的自定义错误类型

    fn try_from(value: Account) -> Result<Self, Self::Error> {
        match value.credential_type.as_str() {
            "UsernamePassword" => {
                let username_password = serde_json::from_str(&value.credential_data)?;
                Ok(Credential::UsernamePassword(username_password))
            }
            "Token" => {
                let token = serde_json::from_str(&value.credential_data)?;
                Ok(Credential::Token(token))
            }
            "ApiKey" => {
                let api_key = serde_json::from_str(&value.credential_data)?;
                Ok(Credential::ApiKey(api_key))
            }
            _ => anyhow::bail!("Unknown credential type: {}", value.credential_type),
        }
    }
}

// 用户名密码凭证
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub struct UsernamePasswordCredential {
    pub username: String,
    pub password: String,
}

// API令牌凭证
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub struct TokenCredential {
    pub token: String,
}

// API密钥凭证
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub struct ApiKeyCredential {
    pub api_key: String,
    pub api_secret: String,
}

// 凭证专用消息
#[derive(Debug, Clone)]
pub enum CredentialMessage {
    UsernamePasswordChanged(UsernamePasswordMessage),
    TokenChanged(TokenMessage),
    ApiKeyChanged(ApiKeyMessage),
}

// 用户名密码凭证消息
#[derive(Debug, Clone)]
pub enum UsernamePasswordMessage {
    UsernameChanged(UsernamePasswordCredential),
    PasswordChanged(UsernamePasswordCredential),
}

// API令牌凭证消息
#[derive(Debug, Clone)]
pub enum TokenMessage {
    TokenChanged(String),
}

// API密钥凭证消息
#[derive(Debug, Clone)]
pub enum ApiKeyMessage {
    ApiKeyChanged(ApiKeyCredential),
    ApiSecretChanged(ApiKeyCredential),
}

// 实现从子消息到 CredentialMessage 的自动转换
impl From<UsernamePasswordMessage> for CredentialMessage {
    fn from(msg: UsernamePasswordMessage) -> Self {
        CredentialMessage::UsernamePasswordChanged(msg)
    }
}

impl From<TokenMessage> for CredentialMessage {
    fn from(msg: TokenMessage) -> Self {
        CredentialMessage::TokenChanged(msg)
    }
}

impl From<ApiKeyMessage> for CredentialMessage {
    fn from(msg: ApiKeyMessage) -> Self {
        CredentialMessage::ApiKeyChanged(msg)
    }
}

// 实现从 CredentialMessage 到顶层 Message 的转换
impl From<CredentialMessage> for MessageCategory {
    fn from(credential_message: CredentialMessage) -> Self {
        match credential_message {
            CredentialMessage::UsernamePasswordChanged(username_password_message) => {
                match username_password_message {
                    UsernamePasswordMessage::UsernameChanged(credential) => {
                        MessageCategory::Provider(ProviderMessage::AddFormCredentialChanged(
                            Credential::UsernamePassword(credential),
                        ))
                    }
                    UsernamePasswordMessage::PasswordChanged(credential) => {
                        MessageCategory::Provider(ProviderMessage::AddFormCredentialChanged(
                            Credential::UsernamePassword(credential),
                        ))
                    }
                }
            }
            CredentialMessage::TokenChanged(token) => match token {
                TokenMessage::TokenChanged(token) => {
                    MessageCategory::Provider(ProviderMessage::AddFormCredentialChanged(
                        Credential::Token(TokenCredential { token }),
                    ))
                }
            },
            CredentialMessage::ApiKeyChanged(apikey_message) => match apikey_message {
                ApiKeyMessage::ApiKeyChanged(credential) => {
                    MessageCategory::Provider(ProviderMessage::AddFormCredentialChanged(
                        Credential::ApiKey(ApiKeyCredential {
                            api_key: credential.api_key,
                            api_secret: credential.api_secret,
                        }),
                    ))
                }
                ApiKeyMessage::ApiSecretChanged(credential) => {
                    MessageCategory::Provider(ProviderMessage::AddFormCredentialChanged(
                        Credential::ApiKey(ApiKeyCredential {
                            api_key: credential.api_key,
                            api_secret: credential.api_secret,
                        }),
                    ))
                }
            },
        }
    }
}
