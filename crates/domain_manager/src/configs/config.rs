use crate::gui::model::domain::DomainName;
use crate::gui::styles::types::gradient_type::GradientType;
use crate::translations::types::language::Language;
use crate::{StyleType, DOMAIN_MANAGER_LOWERCASE};
use iced::Font;
use log::error;
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::Display;
use std::fs::read_to_string;

pub static CONFIGS: std::sync::LazyLock<Config> = std::sync::LazyLock::new(Config::default);

#[derive(Serialize, Deserialize, Debug)]
pub enum LICENCE {
    MIT,
    Apache,
    MulanPSL2,
}

///
///   "name": "Domain Manager",
//     "description": "A simple domain manager",
//     "version": "1.0.0",
//     "author": "Stanic.xyz",
//     "license": "Mulan PSL v2"
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub domain_names: Vec<DomainName>,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub license: LICENCE,
    pub locale: String,
    pub style_type: StyleType,
    pub language: Language,
    pub color_gradient: GradientType,
}

impl From<String> for Config {
    fn from(value: String) -> Self {
        Config {
            domain_names: vec![],
            name: value,
            description: String::new(),
            version: String::new(),
            author: String::new(),
            license: LICENCE::MulanPSL2,
            locale: String::from("zh-CN"),
            style_type: StyleType::Day,
            language: Language::ZH,
            color_gradient: GradientType::None,
        }
    }
}
impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            domain_names: vec![],
            name: String::from(DOMAIN_MANAGER_LOWERCASE),
            description: String::new(),
            version: String::new(),
            author: String::new(),
            license: LICENCE::MulanPSL2,
            locale: String::from("en"),
            style_type: StyleType::Day,
            language: Language::ZH,
            color_gradient: GradientType::None,
        }
    }
}

impl Config {
    /// 加载配置文件
    ///
    pub fn new_from_string(file_content: &String) -> Self {
        let config: Config = serde_json::from_str(&file_content).unwrap();
        config.into()
    }

    /// 加载配置文件
    ///
    pub fn new_from_file(file_name: &str) -> Self {
        let file = Self::load_file(&file_name);
        if let Some(ref file_content) = file {
            dbg!("Loading file content: {}", &file_content);
            Self::new_from_string(&file_content).into()
        } else {
            error!("Loading config file failed!");
            Self::default()
        }
    }

    ///
    /// Load a file from the static directory
    ///
    fn load_file(file_name: &str) -> Option<String> {
        println!("load_file: {}", file_name);
        let default_path = format!("{}\\config\\", env!("CARGO_MANIFEST_DIR"));
        println!("default_path: {}", default_path);
        let public_path = env::var("PUBLIC_PATH").unwrap_or(default_path);
        let file_path = format!("{}{}", public_path, file_name);
        println!("Loading file: {}", file_path);
        read_to_string(file_path).ok()
    }
}
