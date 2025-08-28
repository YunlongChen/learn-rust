//! 应用状态管理
//! 
//! 整合UI状态和数据状态，提供统一的状态管理接口。
//! 这是应用程序状态管理的核心模块。

use super::{UiState, DataState};
use crate::config::Config;
use crate::gui::pages::Page;
use crate::models::Domain;
use crate::dns::DnsProvider;

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
}

/// UI状态更新枚举
#[derive(Debug, Clone)]
pub enum UiUpdate {
    /// 导航到指定页面
    NavigateTo(Page),
    
    /// 返回上一页
    NavigateBack,
    
    /// 设置消息
    SetMessage(String),
    
    /// 显示Toast
    ShowToast(String),
    
    /// 隐藏Toast
    HideToast,
    
    /// 切换主题
    ToggleTheme,
    
    /// 设置同步状态
    SetSyncing(bool),
    
    /// 切换控制台
    ToggleConsole,
    
    /// 切换悬浮窗模式
    ToggleFloatingMode,
    
    /// 设置背景透明度
    SetBackgroundOpacity(f32),
}

/// 数据状态更新枚举
#[derive(Debug, Clone)]
pub enum DataUpdate {
    /// 设置域名列表
    SetDomains(Vec<Domain>),
    
    /// 添加域名
    AddDomain(Domain),
    
    /// 删除域名
    RemoveDomain(String),
    
    /// 选择域名
    SelectDomain(Domain),
    
    /// 设置搜索内容
    SetSearchContent(String),
    
    /// 清除所有数据
    Clear,
}

/// 配置更新枚举
#[derive(Debug, Clone)]
pub enum ConfigUpdate {
    /// 更新语言设置
    SetLocale(crate::config::Locale),
    
    /// 更新主题设置
    SetTheme(crate::gui::theme::Theme),
    
    /// 保存配置
    Save,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            ui: UiState::default(),
            data: DataState::default(),
            config: Config::default(),
            initialized: false,
            version: env!("CARGO_PKG_VERSION").to_string(),
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
        state.ui.theme = state.config.theme.clone();
        state.ui.locale = state.config.locale.clone();
        
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
        }
    }
    
    /// 更新UI状态
    pub fn update_ui(&mut self, update: UiUpdate) {
        match update {
            UiUpdate::NavigateTo(page) => {
                self.ui.navigate_to(page);
            },
            UiUpdate::NavigateBack => {
                self.ui.navigate_back();
            },
            UiUpdate::SetMessage(message) => {
                self.ui.set_message(message);
            },
            UiUpdate::ShowToast(message) => {
                self.ui.show_toast(message);
            },
            UiUpdate::HideToast => {
                self.ui.hide_toast();
            },
            UiUpdate::ToggleTheme => {
                self.ui.toggle_theme();
                // 同步更新配置
                self.config.theme = self.ui.theme.clone();
            },
            UiUpdate::SetSyncing(syncing) => {
                self.ui.set_syncing(syncing);
            },
            UiUpdate::ToggleConsole => {
                self.ui.toggle_console();
            },
            UiUpdate::ToggleFloatingMode => {
                self.ui.toggle_floating_mode();
            },
            UiUpdate::SetBackgroundOpacity(opacity) => {
                self.ui.set_background_opacity(opacity);
            },
        }
    }
    
    /// 更新数据状态
    pub fn update_data(&mut self, update: DataUpdate) {
        match update {
            DataUpdate::SetDomains(domains) => {
                self.data.set_domains(domains);
                self.ui.set_message(format!("已加载 {} 个域名", self.data.domain_list.len()));
            },
            DataUpdate::AddDomain(domain) => {
                let domain_name = domain.name.clone();
                self.data.add_domain(domain);
                self.ui.set_message(format!("已添加域名: {}", domain_name));
            },
            DataUpdate::RemoveDomain(domain_name) => {
                self.data.remove_domain(&domain_name);
                self.ui.set_message(format!("已删除域名: {}", domain_name));
            },
            DataUpdate::SelectDomain(domain) => {
                let domain_name = domain.name.clone();
                self.data.select_domain(domain);
                self.ui.set_message(format!("已选择域名: {}", domain_name));
            },
            DataUpdate::SetSearchContent(content) => {
                self.data.set_search_content(content);
            },
            DataUpdate::Clear => {
                self.data.clear();
                self.ui.set_message("数据已清除".to_string());
            },
        }
    }
    
    /// 更新配置
    pub fn update_config(&mut self, update: ConfigUpdate) {
        match update {
            ConfigUpdate::SetLocale(locale) => {
                self.config.locale = locale.clone();
                self.ui.locale = locale;
            },
            ConfigUpdate::SetTheme(theme) => {
                self.config.theme = theme.clone();
                self.ui.theme = theme;
            },
            ConfigUpdate::Save => {
                // 这里可以添加保存配置到文件的逻辑
                self.ui.set_message("配置已保存".to_string());
            },
        }
    }
    
    /// 获取当前选中的域名
    pub fn get_selected_domain(&self) -> Option<&Domain> {
        self.data.selected_domain.as_ref()
    }
    
    /// 获取过滤后的域名列表
    pub fn get_filtered_domains(&self) -> Vec<Domain> {
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
            if self.ui.is_syncing { "进行中" } else { "空闲" }
        )
    }
    
    /// 检查应用是否准备就绪
    pub fn is_ready(&self) -> bool {
        self.initialized && self.data.connection.is_some()
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