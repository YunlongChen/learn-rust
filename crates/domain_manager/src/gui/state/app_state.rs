//! 应用状态管理
//!
//! 整合UI状态和数据状态，提供统一的状态管理接口。
//! 这是应用程序状态管理的核心模块。

use super::{DataState, UiState};
use crate::configs::gui_config::Config;
// TODO: 实现Config模块
// use crate::config::Config;

use crate::gui::pages::names::Page;
// TODO: 定义Language类型
use crate::gui::styles::types::style_type;
use crate::storage::entities::domain::Model as DomainModel;
use crate::translations::types::locale::Locale;
use iced::{Point, Size, Theme};
use sea_orm::DatabaseConnection;
use style_type::StyleType;
use tracing::info;

/// 应用程序状态结构体
///
/// 包含应用程序的所有状态信息，是状态管理的顶层结构
#[derive(Debug, Clone)]
pub struct AppState {
    /// UI相关状态
    pub ui: UiState,

    /// 数据相关状态
    pub data: DataState,

    /// 应用配置
    pub config: Config,

    /// 应用是否已初始化
    pub initialized: bool,

    /// 应用版本
    pub version: String,

    /// 浮动窗口是否启用
    pub floating_window_enabled: bool,

    /// 数据库连接
    pub database: Option<DatabaseConnection>,
}

/// 状态更新枚举
///
/// 定义不同类型的状态更新操作
#[derive(Debug, Clone)]
pub enum StateUpdate {
    /// UI状态更新
    Ui(UiUpdate),

    /// 数据状态更新
    Data(DataUpdate),

    /// 配置更新
    Config(ConfigUpdate),

    /// 导航更新
    Navigation(Page),
}

/// UI状态更新枚举
#[derive(Debug, Clone)]
pub enum UiUpdate {
    /// 导航到指定页面
    NavigateTo(Page),

    /// 返回上一页
    NavigateBack,

    /// 设置当前页面
    SetCurrentPage(Page),

    /// 设置上一页
    SetLastPage(Page),

    /// 设置消息
    SetMessage(String),

    /// 显示Toast
    ShowToast(String),

    /// 隐藏Toast
    HideToast,

    /// 切换主题
    ToggleTheme,

    /// 设置主题
    SetTheme(iced::Theme),

    /// 设置选中的域名
    SetSelectedDomain(Option<crate::storage::entities::domain::Model>),

    /// 设置选中的提供商
    SetSelectedProvider(Option<crate::gui::model::domain::DnsProvider>),

    /// 设置Toast消息
    SetToastMessage(Option<String>),

    /// 设置Toast可见性
    SetToastVisible(bool),

    /// 设置浮动窗口启用状态
    SetFloatingWindowEnabled(bool),

    /// 设置缩略图模式
    SetThumbnailMode(bool),

    /// 设置控制台可见性
    SetConsoleVisible(bool),

    /// 清除控制台日志
    ClearConsoleLogs,

    /// 设置搜索查询
    SetSearchQuery(String),

    /// 设置同步状态
    SetSyncing(bool),

    /// 切换控制台
    ToggleConsole,

    /// 切换悬浮窗模式
    ToggleFloatingMode,

    /// 切换浮动窗口
    ToggleFloatingWindow,

    /// 切换缩略图模式
    ToggleThumbnail,

    /// 切换语言
    ToggleLocale,

    /// 设置背景透明度
    SetBackgroundOpacity(f32),
    SetLoading(bool),
    SetError(Option<String>),
}

/// 数据状态更新枚举
#[derive(Debug, Clone)]
pub enum DataUpdate {
    /// 设置域名列表
    SetDomains(Vec<DomainModel>),

    /// 添加域名
    AddDomain(DomainModel),

    /// 删除域名
    RemoveDomain(usize),

    /// 选择域名
    SelectDomain(DomainModel),

    /// 设置搜索内容
    SetSearchContent(String),

    /// 清除所有数据
    Clear,
}

/// 配置更新枚举
#[derive(Debug, Clone)]
pub enum ConfigUpdate {
    /// 更新语言设置
    SetLocale(Locale),

    /// 更新主题设置
    SetTheme(Theme),

    /// 保存配置
    Save,

    //修改窗口配置
    UpdateWindowConfig(Size, Point),
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            ui: UiState::default(),
            data: DataState::default(),
            config: Config::default(),
            initialized: false,
            floating_window_enabled: false,
            version: env!("CARGO_PKG_VERSION").to_string(),
            database: None,
        }
    }
}

impl AppState {
    /// 创建新的应用状态
    pub fn new() -> Self {
        Self::default()
    }

    /// 使用指定配置创建应用状态
    pub fn with_config(config: Config) -> Self {
        let mut state = Self::new();
        state.config = config;

        // 根据配置初始化UI状态
        state.ui.theme = match state.config.style_type {
            StyleType::Day => Theme::TokyoNightLight,
            StyleType::Night => Theme::SolarizedDark,
            _ => Theme::TokyoNightLight,
        };
        state.ui.locale = state.config.locale.clone();
        rust_i18n::set_locale(state.ui.locale.code());
        state
    }

    /// 初始化应用状态
    pub fn initialize(&mut self) {
        if !self.initialized {
            // 执行初始化逻辑
            self.ui.set_message("应用初始化完成".to_string());
            self.initialized = true;
        }
    }

    /// 清理应用状态
    pub fn clean(&mut self) {
        self.initialized = false;
        self.ui.set_message("应用已清理".to_string());
    }

    /// 重置应用状态
    pub fn reset(&mut self) {
        self.ui = UiState::default();
        self.data.clear();
        self.initialized = false;
        self.ui.set_message("应用已重置".to_string());
    }

    /// 更新状态
    pub fn update(&mut self, update: StateUpdate) {
        match update {
            StateUpdate::Ui(ui_update) => self.update_ui(ui_update),
            StateUpdate::Data(data_update) => self.update_data(data_update),
            StateUpdate::Config(config_update) => self.update_config(config_update),
            StateUpdate::Navigation(page) => self.ui.navigate_to(page),
        }
    }

    /// 更新UI状态
    pub fn update_ui(&mut self, update: UiUpdate) {
        match update {
            UiUpdate::NavigateTo(page) => {
                self.ui.navigate_to(page);
            }
            UiUpdate::NavigateBack => {
                self.ui.navigate_back();
            }
            UiUpdate::SetCurrentPage(page) => {
                self.ui.current_page = page;
            }
            UiUpdate::SetLastPage(page) => {
                self.ui.last_page = Some(page);
            }
            UiUpdate::SetMessage(message) => {
                self.ui.set_message(message);
            }
            UiUpdate::ShowToast(message) => {
                self.ui.show_toast(message);
            }
            UiUpdate::HideToast => {
                self.ui.hide_toast();
            }
            UiUpdate::ToggleTheme => {
                self.ui.toggle_theme();
                // 同步更新配置
                // 没有持久化
                // self.config.theme = self.ui.theme.clone();
            }
            UiUpdate::SetTheme(theme) => {
                self.ui.theme = theme.clone();
            }
            UiUpdate::SetSearchQuery(query) => {
                self.data.search_content = query;
            }
            UiUpdate::SetSyncing(syncing) => {
                self.ui.set_syncing(syncing);
            }
            UiUpdate::ToggleConsole => {
                self.ui.toggle_console();
            }
            UiUpdate::ToggleFloatingMode => {
                self.ui.toggle_floating_mode();
            }
            UiUpdate::SetBackgroundOpacity(opacity) => {
                self.ui.set_background_opacity(opacity);
            }
            UiUpdate::SetLoading(loading) => {
                self.ui.is_loading = loading;
            }
            UiUpdate::SetError(error) => {
                // 这里可以根据需要处理错误状态
                if let Some(err_msg) = error {
                    self.ui.set_message(format!("错误: {}", err_msg));
                }
            }
            UiUpdate::SetSelectedDomain(model) => {
                self.ui.selected_domain = model;
            }
            UiUpdate::SetSelectedProvider(_dns_provider) => {
                // 处理选中的DNS提供商
            }
            UiUpdate::SetToastMessage(message) => {
                self.ui.toast_message = message;
            }
            UiUpdate::SetToastVisible(visible) => {
                self.ui.toast_visible = visible;
            }
            UiUpdate::SetFloatingWindowEnabled(enabled) => {
                self.ui.floating_window_enabled = enabled;
            }
            UiUpdate::SetThumbnailMode(enabled) => {
                self.ui.thumbnail_mode = enabled;
            }
            UiUpdate::SetConsoleVisible(visible) => {
                self.ui.console_visible = visible;
            }
            UiUpdate::ClearConsoleLogs => {
                // 清除控制台日志的逻辑
                info!("清除控制台日志！");
            }
            UiUpdate::ToggleFloatingWindow => {
                self.ui.floating_window_enabled = !self.ui.floating_window_enabled;
            }
            UiUpdate::ToggleThumbnail => {
                self.ui.thumbnail_mode = !self.ui.thumbnail_mode;
            }
            UiUpdate::ToggleLocale => {
                self.ui.locale = self.ui.locale.next();
                rust_i18n::set_locale(self.ui.locale.code());
            }
        }
    }

    /// 更新数据状态
    pub fn update_data(&mut self, update: DataUpdate) {
        match update {
            DataUpdate::SetDomains(domains) => {
                self.data.set_domains(domains);
                self.ui
                    .set_message(format!("已加载 {} 个域名", self.data.domain_list.len()));
            }
            DataUpdate::AddDomain(domain) => {
                let domain_name = domain.name.clone();
                self.data.add_domain(domain);
                self.ui.set_message(format!("已添加域名: {}", domain_name));
            }
            DataUpdate::RemoveDomain(domain_id) => {
                self.data.remove_domain(domain_id);
                self.ui.set_message(format!("已删除域名: {}", domain_id));
            }
            DataUpdate::SelectDomain(domain) => {
                let domain_name = domain.name.clone();
                self.data.select_domain(domain);
                self.ui.set_message(format!("已选择域名: {}", domain_name));
            }
            DataUpdate::SetSearchContent(content) => {
                self.data.set_search_content(content);
            }
            DataUpdate::Clear => {
                self.data.clear();
                self.ui.set_message("数据已清除".to_string());
            }
        }
    }

    /// 更新窗口状态
    ///
    pub fn update_window_state(&mut self, _x: f32, _y: f32, _width: f32, _height: f32) {
        // self.ui.update_window_state();
        self.config.save_to_file("config.json").unwrap();
    }

    /// 更新配置
    pub fn update_config(&mut self, update: ConfigUpdate) {
        match update {
            ConfigUpdate::SetLocale(locale) => {
                self.ui.locale = locale;
                rust_i18n::set_locale(self.ui.locale.code());
            }
            ConfigUpdate::SetTheme(_theme) => {
                // self.config.theme = theme.clone();
                // self.ui.theme = theme;
            }
            ConfigUpdate::Save => {
                // 这里可以添加保存配置到文件的逻辑
                self.ui.set_message("配置已保存".to_string());
            }
            ConfigUpdate::UpdateWindowConfig(_, _) => {}
        }
    }

    /// 获取当前选中的域名
    pub fn get_selected_domain(&self) -> Option<&DomainModel> {
        self.data.selected_domain.as_ref()
    }

    /// 获取过滤后的域名列表
    pub fn get_filtered_domains(&self) -> Vec<DomainModel> {
        self.data.get_filtered_domains()
    }

    /// 检查是否有未保存的更改
    pub fn has_unsaved_changes(&self) -> bool {
        self.data.has_changes
    }

    /// 获取应用状态摘要
    pub fn get_status_summary(&self) -> String {
        format!(
            "域名: {} | 记录: {} | 页面: {:?} | 同步: {}",
            self.data.stats.total_domains,
            self.data.stats.total_dns_records,
            self.ui.current_page,
            if self.ui.is_syncing {
                "进行中"
            } else {
                "空闲"
            }
        )
    }

    /// 检查应用是否准备就绪
    pub fn is_ready(&self) -> bool {
        self.initialized && self.database.is_some()
    }
}

/// 状态管理器特征
///
/// 定义状态管理的基本接口
pub trait StateManager {
    /// 获取当前状态
    fn get_state(&self) -> &AppState;

    /// 获取可变状态引用
    fn get_state_mut(&mut self) -> &mut AppState;

    /// 更新状态
    fn update_state(&mut self, update: StateUpdate);
}
