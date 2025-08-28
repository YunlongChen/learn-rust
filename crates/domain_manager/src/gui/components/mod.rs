//! UI组件模块
//! 
//! 提供可重用的UI组件，将复杂的UI逻辑分解为更小、更专门的组件。
//! 这些组件独立于具体的业务逻辑，可以在不同的页面中重复使用。

// 现有组件
pub mod background;
pub mod button;
pub mod console;
pub mod credential_form;
pub mod footer;
pub mod header;
pub mod modal;
pub mod tab;
pub mod toast;
pub mod types;

// 重构后的新组件
pub mod domain_list;
pub mod dns_records;
pub mod provider_selector;
pub mod search_bar;
pub mod status_indicator;
pub mod toast_message;
pub mod navigation;
pub mod settings_panel;
pub mod help_dialog;
pub mod console_panel;

use iced::{Element, Theme};
use crate::gui::Message;

/// 组件特征
/// 
/// 定义所有UI组件的通用接口
pub trait Component<State> {
    /// 组件名称
    fn name(&self) -> &'static str;
    
    /// 渲染组件
    fn view(&self, state: &State) -> Element<Message>;
    
    /// 更新组件状态（可选）
    fn update(&mut self, _state: &mut State, _message: Message) -> bool {
        false
    }
    
    /// 检查组件是否可见
    fn is_visible(&self, _state: &State) -> bool {
        true
    }
    
    /// 检查组件是否启用
    fn is_enabled(&self, _state: &State) -> bool {
        true
    }
}

/// 可主题化组件特征
pub trait ThemeableComponent<State>: Component<State> {
    /// 使用指定主题渲染组件
    fn view_with_theme(&self, state: &State, theme: &Theme) -> Element<Message>;
}

/// 可配置组件特征
pub trait ConfigurableComponent<State, Config>: Component<State> {
    /// 应用配置
    fn apply_config(&mut self, config: Config);
    
    /// 获取当前配置
    fn get_config(&self) -> Config;
}

/// 组件样式
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentStyle {
    /// 默认样式
    Default,
    /// 主要样式
    Primary,
    /// 次要样式
    Secondary,
    /// 成功样式
    Success,
    /// 警告样式
    Warning,
    /// 错误样式
    Error,
    /// 信息样式
    Info,
    /// 自定义样式
    Custom(String),
}

/// 组件大小
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComponentSize {
    /// 小尺寸
    Small,
    /// 中等尺寸
    Medium,
    /// 大尺寸
    Large,
    /// 自定义尺寸
    Custom(f32, f32),
}

/// 组件配置
#[derive(Debug, Clone)]
pub struct ComponentConfig {
    /// 样式
    pub style: ComponentStyle,
    /// 大小
    pub size: ComponentSize,
    /// 是否可见
    pub visible: bool,
    /// 是否启用
    pub enabled: bool,
    /// 透明度
    pub opacity: f32,
    /// 边距
    pub margin: (f32, f32, f32, f32), // top, right, bottom, left
    /// 内边距
    pub padding: (f32, f32, f32, f32), // top, right, bottom, left
}

impl Default for ComponentConfig {
    fn default() -> Self {
        Self {
            style: ComponentStyle::Default,
            size: ComponentSize::Medium,
            visible: true,
            enabled: true,
            opacity: 1.0,
            margin: (0.0, 0.0, 0.0, 0.0),
            padding: (8.0, 8.0, 8.0, 8.0),
        }
    }
}
