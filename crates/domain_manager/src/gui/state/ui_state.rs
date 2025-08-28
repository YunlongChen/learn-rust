//! UI状态管理
//! 
//! 管理所有与用户界面相关的状态，包括当前页面、主题、语言设置、
//! 消息显示、同步状态等。

use crate::gui::pages::Page;
use crate::gui::theme::Theme;
use crate::config::Locale;

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
    
    /// 窗口是否最大化
    pub window_maximized: bool,
}

/// 控制台标签页枚举
#[derive(Debug, Clone, PartialEq)]
pub enum ConsoleTab {
    /// 日志标签页
    Log,
    /// 错误标签页
    Error,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            current_page: Page::Domain,
            last_page: None,
            theme: Theme::default(),
            locale: Locale::default(),
            message: String::new(),
            is_syncing: false,
            toast_visible: false,
            toast_message: None,
            show_help: false,
            console_visible: false,
            console_tab: ConsoleTab::Log,
            floating_mode: false,
            background_opacity: 1.0,
            window_maximized: false,
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
}