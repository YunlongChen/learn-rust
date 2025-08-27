use crate::gui::model::domain::Domain;
use crate::gui::styles::types::gradient_type::GradientType;
use crate::translations::types::language::Language;
use crate::{StyleType, DOMAIN_MANAGER_LOWERCASE, VERSION};
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::Display;
use std::fs::read_to_string;
use tracing::{error, info};

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
/// 窗口状态配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WindowState {
    /// 窗口X坐标位置
    pub x: f32,
    /// 窗口Y坐标位置
    pub y: f32,
    /// 窗口宽度
    pub width: f32,
    /// 窗口高度
    pub height: f32,
}

impl Default for WindowState {
    fn default() -> Self {
        WindowState {
            x: 100.0,
            y: 100.0,
            width: 1200.0,
            height: 800.0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub domain_names: Vec<Domain>,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub license: LICENCE,
    pub locale: String,
    pub style_type: StyleType,
    pub language: Language,
    pub color_gradient: GradientType,
    pub ali_access_key_id: Option<String>,
    pub ali_access_key_secret: Option<String>,
    /// 窗口状态配置
    #[serde(default)]
    pub window_state: WindowState,
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
            ali_access_key_id: None,
            ali_access_key_secret: None,
            window_state: WindowState::default(),
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
            version: VERSION.to_string(),
            author: String::new(),
            license: LICENCE::MulanPSL2,
            locale: String::from("en"),
            style_type: StyleType::Day,
            language: Language::ZH,
            color_gradient: GradientType::None,
            ali_access_key_id: None,
            ali_access_key_secret: None,
            window_state: WindowState::default(),
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
        info!("从文件读取配置信息");
        let file = Self::load_file(&file_name);
        if let Some(ref file_content) = file {
            info!("从文件加载配置文件：内容: {}", &file_content);
            Self::new_from_string(&file_content).into()
        } else {
            error!("Loading config file failed!");
            Self::default()
        }
    }

    /// 保存配置到文件
    ///
    pub fn save_to_file(&self, file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("保存配置到文件: {}", file_name);
        let default_path = format!("{}\\config\\", env!("CARGO_MANIFEST_DIR"));
        let public_path = env::var("PUBLIC_PATH").unwrap_or(default_path);
        let file_path = format!("{}{}", public_path, file_name);
        
        // 确保目录存在
        if let Some(parent) = std::path::Path::new(&file_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let json_content = serde_json::to_string_pretty(self)?;
        std::fs::write(&file_path, json_content)?;
        info!("配置已保存到: {}", file_path);
        Ok(())
    }
    
    /// 更新窗口状态
    ///
    pub fn update_window_state(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.window_state.x = x;
        self.window_state.y = y;
        self.window_state.width = width;
        self.window_state.height = height;
    }

    ///
    /// Load a file from the static directory
    ///
    fn load_file(file_name: &str) -> Option<String> {
        info!("load_file: {}", file_name);
        let default_path = format!("{}\\config\\", env!("CARGO_MANIFEST_DIR"));
        info!("default_path: {}", default_path);
        let public_path = env::var("PUBLIC_PATH").unwrap_or(default_path);
        let file_path = format!("{}{}", public_path, file_name);
        info!("Loading file: {}", file_path);
        read_to_string(file_path).ok()
    }
}
