// #![cfg_attr(windows, windows_subsystem = "windows")]

mod api;
mod configs;
mod countries;
mod gui;
mod mmdb;
mod model;
mod notifications;
mod translations;
mod utils;

use crate::configs::config::Config;
use crate::gui::manager::DomainManager;
pub use crate::gui::styles::types::style_type::StyleType;
use gui::types::message::Message;
use rust_i18n::i18n;
use std::{panic, process};

const TITLE_SIZE: u16 = 36;
const TITLE_PADDING: u16 = 20;
const CONTENT_SIZE: u16 = 20;

const DOMAIN_MANAGER_LOWERCASE: &str = "domain_manager";

/// Update period (milliseconds)
pub const PERIOD_TICK: u64 = 1000;

pub const FONT_FAMILY_NAME: &str = "Sarasa Mono SC for Sniffnet";
pub const ICON_FONT_FAMILY_NAME: &str = "Icons for Sniffnet";

i18n!("locales", fallback = "en");

pub fn main() -> iced::Result {
    // 读取配置文件
    let config = Config::new_from_file("config.json");
    dbg!("配置文件信息：应用名称：{:?}", &config.name);

    // kill the main thread as soon as a secondary thread panics
    let orig_hook = panic::take_hook();

    panic::set_hook(Box::new(move |panic_info| {
        // invoke the default handler and exit the process
        dbg!("程序崩溃了，退出程序！");
        orig_hook(panic_info);
        process::exit(1);
    }));

    let domain_manager = DomainManager::new(config);
    dbg!("启动程序");
    domain_manager.start()
}
