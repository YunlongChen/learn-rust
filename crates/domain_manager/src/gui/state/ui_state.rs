#![allow(dead_code)]
//! UI状态管理
//!
//! 管理所有与用户界面相关的状态，包括当前页面、主题、语言设置、
//! 消息显示、同步状态等。

use crate::configs::gui_config::WindowState;
use crate::gui::components::console::ConsoleTab;
use crate::gui::model::form::AddDomainField;
use crate::gui::pages::names::Page;
use crate::gui::state::data_state::Filter;
use crate::storage::DomainModal;
use crate::translations::types::locale::Locale;
use iced::window::Id;
use iced::Theme;

/// UI状态结构体
///
/// 包含所有与用户界面显示和交互相关的状态信息
#[derive(Debug, Clone)]
pub struct UiState {
    /// 当前显示的页面
    pub current_page: Page,

    /// 上一个页面（用于返回导航）
    pub last_page: Option<Page>,

    /// 当前主题
    pub theme: Theme,

    /// 当前语言设置
    pub locale: Locale,

    /// 状态栏消息
    pub message: String,

    /// 是否正在同步
    pub is_syncing: bool,

    /// Toast消息是否可见
    pub toast_visible: bool,

    /// Toast消息内容
    pub toast_message: Option<String>,

    /// 是否显示帮助页面
    pub show_help: bool,

    /// 控制台是否可见
    pub console_visible: bool,

    /// 当前控制台标签页
    pub console_tab: ConsoleTab,

    /// 是否为悬浮窗模式
    pub floating_mode: bool,

    /// 背景透明度 (0.0 - 1.0)
    pub background_opacity: f32,

    /// 窗口是否最小化
    pub window_minimize: bool,

    /// 窗口是否最大化
    pub window_maximized: bool,

    /// 加载中
    pub is_loading: bool,

    /// 添加域名表单字段
    pub add_domain_field: AddDomainField,

    /// 过滤器状态
    pub filter: Filter,

    /// 是否正在查询
    pub in_query: bool,

    /// 当前选中的域名
    pub selected_domain: Option<DomainModal>,
    pub floating_window_enabled: bool,

    /// 最小化模式
    pub thumbnail_mode: bool,

    /// 窗口配置
    pub window_state: WindowState,

    // 窗口标识
    pub id: Option<Id>,

    /// 错误消息
    pub error_message: Option<String>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            current_page: Page::Dashboard,
            last_page: None,
            theme: Theme::default(),
            locale: Locale::Chinese,
            message: String::new(),
            // ... (其他字段)
            is_syncing: false,
            toast_visible: false,
            toast_message: None,
            show_help: false,
            console_visible: false,
            console_tab: ConsoleTab::Log,
            floating_mode: false,
            background_opacity: 1.0,
            window_maximized: true,
            window_minimize: false,
            is_loading: false,
            add_domain_field: AddDomainField::default(),
            filter: Filter::default(),
            in_query: false,
            selected_domain: None,
            floating_window_enabled: false,
            thumbnail_mode: false,
            window_state: WindowState::default(),
            id: None,
            error_message: None,
        }
    }
}

impl UiState {
    /// 创建新的UI状态
    pub fn new() -> Self {
        Self::default()
    }

    /// 切换到指定页面
    pub fn navigate_to(&mut self, page: Page) {
        self.last_page = Some(self.current_page.clone());
        self.current_page = page;
    }

    /// 返回上一页
    pub fn navigate_back(&mut self) {
        if let Some(last_page) = self.last_page.take() {
            self.current_page = last_page;
        }
    }

    /// 设置状态消息
    pub fn set_message(&mut self, message: String) {
        self.message = message;
    }

    /// 显示Toast消息
    pub fn show_toast(&mut self, message: String) {
        self.toast_message = Some(message);
        self.toast_visible = true;
    }

    /// 隐藏Toast消息
    pub fn hide_toast(&mut self) {
        self.toast_visible = false;
        self.toast_message = None;
    }

    /// 切换主题
    pub fn toggle_theme(&mut self) {
        self.theme = match self.theme {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
            _ => Theme::Light,
        };
    }

    /// 设置同步状态
    pub fn set_syncing(&mut self, syncing: bool) {
        self.is_syncing = syncing;
        if syncing {
            self.set_message("正在同步...".to_string());
        } else {
            self.set_message("同步完成".to_string());
        }
    }

    /// 切换控制台可见性
    pub fn toggle_console(&mut self) {
        self.console_visible = !self.console_visible;
    }

    /// 切换控制台标签页
    pub fn switch_console_tab(&mut self, tab: ConsoleTab) {
        self.console_tab = tab;
    }

    /// 切换悬浮窗模式
    pub fn toggle_floating_mode(&mut self) {
        self.floating_mode = !self.floating_mode;
    }

    /// 设置背景透明度
    pub fn set_background_opacity(&mut self, opacity: f32) {
        self.background_opacity = opacity.clamp(0.0, 1.0);
    }

    /// 切换窗口最大化状态
    pub fn toggle_window_maximized(&mut self) {
        self.window_maximized = !self.window_maximized;
    }

    /// 切换窗口最小化状态
    pub(crate) fn toggle_window_minimize(&mut self) {
        self.window_minimize = !self.window_minimize;
    }

    /// 设置加载状态
    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
        if loading {
            self.set_message("正在加载...".to_string());
        }
    }

    /// 更新窗口状态
    ///
    pub fn update_window_state(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.window_state.x = x;
        self.window_state.y = y;
        self.window_state.width = width;
        self.window_state.height = height;
    }
}
