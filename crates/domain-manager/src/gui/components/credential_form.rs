use crate::gui::types::credential::{
    ApiKeyCredential, ApiKeyMessage, Credential, CredentialMessage, TokenCredential, TokenMessage,
    UsernamePasswordCredential, UsernamePasswordMessage,
};
use crate::StyleType;
use iced::widget::{Column, TextInput};
use iced::Element;

// 凭证表单组件的trait
pub trait CredentialForm {
    fn view(&self) -> Element<'_, CredentialMessage, StyleType>;
    fn update(&mut self, message: CredentialMessage) -> Option<Credential>;
}

// 用户名密码凭证表单实现
impl CredentialForm for UsernamePasswordCredential {
    fn view(&self) -> Element<'_, CredentialMessage, StyleType> {
        Column::new()
            .spacing(10)
            .push(
                TextInput::new("用户名", &self.username)
                    .on_input(|username| {
                        CredentialMessage::UsernamePasswordChanged(
                            UsernamePasswordMessage::UsernameChanged(UsernamePasswordCredential {
                                username,
                                password: self.password.clone(),
                            }),
                        )
                    })
                    .padding(10),
            )
            .push(
                TextInput::new("密码", &self.password)
                    .on_input(|password| {
                        CredentialMessage::UsernamePasswordChanged(
                            UsernamePasswordMessage::PasswordChanged(UsernamePasswordCredential {
                                password,
                                username: self.username.clone(),
                            }),
                        )
                    })
                    .padding(10),
            )
            .into()
    }

    fn update(&mut self, message: CredentialMessage) -> Option<Credential> {
        match message {
            CredentialMessage::UsernamePasswordChanged(msg) => {
                match msg {
                    UsernamePasswordMessage::UsernameChanged(u) => self.username = u.username,
                    UsernamePasswordMessage::PasswordChanged(p) => self.password = p.password,
                }
                Some(Credential::UsernamePassword(self.clone()))
            }
            _ => None,
        }
    }
}

// API令牌凭证表单实现
impl CredentialForm for TokenCredential {
    fn view(&self) -> Element<'_, CredentialMessage, StyleType> {
        Column::new()
            .spacing(10)
            .push(
                TextInput::new("API令牌", &self.token)
                    .on_input(|token| {
                        CredentialMessage::TokenChanged(TokenMessage::TokenChanged(token))
                    })
                    .padding(10),
            )
            .into()
    }

    fn update(&mut self, message: CredentialMessage) -> Option<Credential> {
        match message {
            CredentialMessage::TokenChanged(msg) => {
                match msg {
                    TokenMessage::TokenChanged(t) => self.token = t,
                }
                Some(Credential::Token(self.clone()))
            }
            _ => None,
        }
    }
}

// API密钥凭证表单实现
impl CredentialForm for ApiKeyCredential {
    fn view(&self) -> Element<'_, CredentialMessage, StyleType> {
        Column::new()
            .spacing(10)
            .push(
                TextInput::new("API Key", &self.api_key)
                    .on_input(|api_key| {
                        CredentialMessage::ApiKeyChanged(ApiKeyMessage::ApiKeyChanged({
                            ApiKeyCredential {
                                api_key,
                                api_secret: self.api_secret.clone(),
                            }
                        }))
                    })
                    .padding(10),
            )
            .push(
                TextInput::new("API Secret", &self.api_secret)
                    // .password()
                    .on_input(|api_secret| {
                        CredentialMessage::ApiKeyChanged(ApiKeyMessage::ApiSecretChanged(
                            ApiKeyCredential {
                                api_secret,
                                api_key: self.api_key.clone(),
                            },
                        ))
                    })
                    .padding(10),
            )
            .into()
    }

    fn update(&mut self, message: CredentialMessage) -> Option<Credential> {
        match message {
            CredentialMessage::ApiKeyChanged(msg) => {
                match msg {
                    ApiKeyMessage::ApiKeyChanged(k) => self.api_key = k.api_key,
                    ApiKeyMessage::ApiSecretChanged(s) => self.api_secret = s.api_secret,
                }
                Some(Credential::ApiKey(self.clone()))
            }
            _ => None,
        }
    }
}
