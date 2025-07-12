// #![cfg_attr(windows, windows_subsystem = "windows")]

mod api;
mod cli;
mod configs;
mod countries;
mod dm_logger;
mod gui;
mod mmdb;
mod model;
mod models;
mod notifications;
pub mod storage;
mod translations;
mod utils;

use crate::configs::gui_config::Config;
use crate::gui::manager::DomainManager;
use crate::gui::styles::style_constants::{
    FONT_SIZE_BODY, ICONS_BYTES, MAPLE_MONO_NF_CN_REGULAR, SARASA_MONO_BOLD_BYTES,
    SARASA_MONO_BYTES,
};
pub use crate::gui::styles::types::style_type::StyleType;
use crate::storage::init_database;
pub use crate::utils::i18_utils::get_text;
use gui::types::message::Message;
use iced::window::icon::from_rgba;
use iced::{application, window, Font, Pixels, Settings, Task};
use rust_i18n::i18n;
use sea_orm::DatabaseConnection;
use std::{panic, process};

use crate::dm_logger::init_logging;
use tracing::{error, info};

const TITLE_SIZE: u16 = 36;
const TITLE_PADDING: u16 = 20;
const CONTENT_SIZE: u16 = 20;
pub(crate) const VERSION: &str = "0.0.1";

pub(crate) const DOMAIN_MANAGER_LOWERCASE: &str = "domain_manager";

/// Update period (milliseconds)
pub const PERIOD_TICK: u64 = 1000;

pub const FONT_FAMILY_NAME: &str = "Sarasa Mono SC for DomainManager";
pub const ICON_FONT_FAMILY_NAME: &str = "Icons for Domain Manager";
pub const FONT_CN_FAMILY_NAME: &str = "Maple Mono NF CN";

i18n!("locales", fallback = "en");

#[tokio::main]
pub async fn main() -> iced::Result {
    init_logging();
    // 开始记录日志
    info!("Application Starting...");

    // 读取配置文件
    let config: Config = Config::new_from_file("config.json");
    info!("配置文件信息：应用名称：{:?}", &config.name);

    // kill the main thread as soon as a secondary thread panics
    let orig_hook = panic::take_hook();

    let database_config = &configs::get().database;

    panic::set_hook(Box::new(move |panic_info| {
        // invoke the default handler and exit the process
        error!("程序崩溃了，退出程序！");
        orig_hook(panic_info);
        process::exit(1);
    }));

    info!("读取图标！");
    let icon = match image::load_from_memory(include_bytes!("../resources/logos/raw/icon.png")) {
        Ok(buffer) => {
            let buffer = buffer.to_rgba8();
            let width = buffer.width();
            let height = buffer.height();
            let dynamic_image = image::DynamicImage::ImageRgba8(buffer);

            let result = from_rgba(dynamic_image.into_bytes(), width, height);
            match result {
                Ok(icon) => Some(icon),
                Err(err) => {
                    info!("加载图标失败：{}", err);
                    None
                }
            }
        }
        Err(err) => {
            info!("加载图标文件失败：{}", err);
            None
        }
    };

    let connection: DatabaseConnection = init_database(database_config)
        .await
        .expect("Cannot connect to database.");

    let settings = Settings {
        id: Some(String::from(DOMAIN_MANAGER_LOWERCASE)),
        fonts: vec![
            ICONS_BYTES.into(),
            MAPLE_MONO_NF_CN_REGULAR.into(),
            SARASA_MONO_BYTES.into(),
            SARASA_MONO_BOLD_BYTES.into(),
        ],
        default_font: Font::with_name(FONT_CN_FAMILY_NAME),
        default_text_size: Pixels::from(FONT_SIZE_BODY),
        ..Default::default()
    };

    let app = application(
        DOMAIN_MANAGER_LOWERCASE,
        DomainManager::update,
        DomainManager::view,
    )
    .window(window::Settings {
        // size: Size::new(1920.0, 1080.0),
        icon,
        ..Default::default()
    })
    .subscription(DomainManager::keyboard_subscription)
    .subscription(DomainManager::subscription)
    .settings(settings);
    app.run_with(move || {
        (
            DomainManager::new(config, connection),
            Task::done(Message::Started),
        )
    })
}
