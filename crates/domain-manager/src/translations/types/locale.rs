use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Locale {
    #[serde(rename = "zh_CN")]
    Chinese,

    #[serde(rename = "en")]
    English,
}

impl Locale {
    pub fn next(self) -> Self {
        match self {
            Self::Chinese => {
                info!("Switching to English");
                Self::English
            }
            Self::English => {
                info!("Switching to Chinese");
                Self::Chinese
            }
        }
    }
    pub fn code(&self) -> &str {
        match self {
            Self::Chinese => "zh_CN",
            Self::English => "en",
        }
    }
}

impl From<String> for Locale {
    fn from(value: String) -> Self {
        match value.as_str() {
            "zh_CN" => Self::Chinese,
            "en" => Self::English,
            &_ => Self::Chinese,
        }
    }
}
