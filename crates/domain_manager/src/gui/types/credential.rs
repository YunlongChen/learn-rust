use crate::gui::components::credential_form::CredentialForm;
use crate::gui::model::domain::DnsProvider;
use crate::gui::types::message::Message;
use crate::StyleType;
use iced::Element;
use std::error::Error;

// 凭证类型枚举
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum Credential {
    UsernamePassword(UsernamePasswordCredential),
    Token(TokenCredential),
    ApiKey(ApiKeyCredential),
    // 添加其他凭证类型...
}

impl Credential {
    pub fn view(&self) -> Element<CredentialMessage, StyleType> {
        match self {
            Credential::UsernamePassword(credential) => credential.view(),
            Credential::Token(cre) => cre.view(),
            Credential::ApiKey(credential) => credential.view(),
        }
    }

    /// 验证凭据
    pub fn validate(dns_provider: &DnsProvider) -> Result<bool, Box<dyn Error>> {
        Ok(false)
    }
}

// 用户名密码凭证
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd)]
pub struct UsernamePasswordCredential {
    pub username: String,
    pub password: String,
}

// API令牌凭证
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd)]
pub struct TokenCredential {
    pub token: String,
}

// API密钥凭证
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd)]
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
impl From<CredentialMessage> for Message {
    fn from(credential_message: CredentialMessage) -> Self {
        match credential_message {
            CredentialMessage::UsernamePasswordChanged(username_password_message) => {
                match username_password_message {
                    UsernamePasswordMessage::UsernameChanged(credential) => {
                        Message::AddProviderFormCredentialChanged(Credential::UsernamePassword(
                            credential,
                        ))
                    }
                    UsernamePasswordMessage::PasswordChanged(credential) => {
                        Message::AddProviderFormCredentialChanged(Credential::UsernamePassword(
                            credential,
                        ))
                    }
                }
            }
            CredentialMessage::TokenChanged(token) => {
                match token {
                    TokenMessage::TokenChanged(token) => Message::AddProviderFormCredentialChanged(
                        Credential::Token(TokenCredential { token }),
                    ),
                }
            }
            CredentialMessage::ApiKeyChanged(apikey_message) => match apikey_message {
                ApiKeyMessage::ApiKeyChanged(credential) => {
                    Message::AddProviderFormCredentialChanged(Credential::ApiKey(credential))
                }
                ApiKeyMessage::ApiSecretChanged(credential) => {
                    Message::AddProviderFormCredentialChanged(Credential::ApiKey(credential))
                }
            },
        }
    }
}
