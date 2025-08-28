//! 重构后的域名管理器
//! 
//! 采用模块化架构，分离UI渲染、业务逻辑、数据管理和事件处理

use crate::gui::state::{AppState, StateUpdate};
use crate::gui::handlers::{
    MessageHandler, DomainHandler, DnsHandler, SyncHandler, WindowHandler,
    EventHandler, AsyncEventHandler, HandlerResult
};
use crate::gui::services::ServiceManager;
use crate::gui::components::{
    Component, DomainListComponent, DnsRecordsComponent
};
use crate::gui::{Message, Page};
use crate::models::{Domain, DnsRecord};
use crate::database::Database;

use iced::widget::{column, container, row};
use iced::{Element, Length, Task};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// 重构后的域名管理器
/// 
/// 采用模块化架构设计，职责清晰分离：
/// - 状态管理：AppState 统一管理所有应用状态
/// - 事件处理：各种专门的Handler处理不同类型的事件
/// - 业务服务：ServiceManager管理所有业务服务
/// - UI组件：可重用的组件负责UI渲染
#[derive(Debug)]
pub struct DomainManagerV2 {
    /// 应用状态
    state: AppState,
    
    /// 消息处理器
    message_handler: MessageHandler,
    
    /// 域名处理器
    domain_handler: DomainHandler,
    
    /// DNS处理器
    dns_handler: DnsHandler,
    
    /// 同步处理器
    sync_handler: SyncHandler,
    
    /// 窗口处理器
    window_handler: WindowHandler,
    
    /// 服务管理器
    service_manager: ServiceManager,
    
    /// UI组件
    domain_list_component: DomainListComponent,
    dns_records_component: DnsRecordsComponent,
    
    /// 数据库连接
    database: Option<Arc<RwLock<Database>>>,
    
    /// 初始化状态
    initialized: bool,
}

impl DomainManagerV2 {
    /// 创建新的域名管理器实例
    pub fn new() -> Self {
        info!("创建新的域名管理器实例 (V2)");
        
        Self {
            state: AppState::new(),
            message_handler: MessageHandler::new(),
            domain_handler: DomainHandler::new(),
            dns_handler: DnsHandler::new(),
            sync_handler: SyncHandler::new(),
            window_handler: WindowHandler::new(),
            service_manager: ServiceManager::new(),
            domain_list_component: DomainListComponent::new(),
            dns_records_component: DnsRecordsComponent::new(),
            database: None,
            initialized: false,
        }
    }
    
    /// 初始化管理器
    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.initialized {
            warn!("管理器已经初始化，跳过重复初始化");
            return Ok(());
        }
        
        info!("开始初始化域名管理器");
        
        // 初始化服务管理器
        self.service_manager.initialize().await?;
        
        // 初始化数据库连接
        if let Some(db) = self.service_manager.get_database_service().await {
            self.database = Some(db);
            info!("数据库连接已建立");
        }
        
        // 加载初始数据
        self.load_initial_data().await?;
        
        // 标记为已初始化
        self.initialized = true;
        
        info!("域名管理器初始化完成");
        Ok(())
    }
    
    /// 加载初始数据
    async fn load_initial_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("开始加载初始数据");
        
        // 设置加载状态
        self.state.update(StateUpdate::Ui(crate::gui::state::UiUpdate::SetLoading(true)));
        
        // 加载域名列表
        match self.load_domains().await {
            Ok(domains) => {
                info!("成功加载 {} 个域名", domains.len());
                self.state.update(StateUpdate::Data(
                    crate::gui::state::DataUpdate::SetDomains(domains)
                ));
            },
            Err(e) => {
                error!("加载域名失败: {}", e);
                self.state.update(StateUpdate::Ui(
                    crate::gui::state::UiUpdate::SetError(Some(format!("加载域名失败: {}", e)))
                ));
            }
        }
        
        // 清除加载状态
        self.state.update(StateUpdate::Ui(crate::gui::state::UiUpdate::SetLoading(false)));
        
        Ok(())
    }
    
    /// 从数据库加载域名
    async fn load_domains(&self) -> Result<Vec<Domain>, Box<dyn std::error::Error>> {
        if let Some(db) = &self.database {
            let db_guard = db.read().await;
            let domains = db_guard.get_all_domains().await?;
            Ok(domains)
        } else {
            Err("数据库连接未建立".into())
        }
    }
    
    /// 处理消息
    pub fn update(&mut self, message: Message) -> Task<Message> {
        if !self.initialized {
            warn!("管理器未初始化，忽略消息: {:?}", message);
            return Task::none();
        }
        
        info!("处理消息: {:?}", message);
        
        // 首先通过消息处理器分发消息
        let handler_result = self.message_handler.handle(&mut self.state, message.clone());
        
        // 根据消息类型调用相应的专门处理器
        let specific_result = match &message {
            // 域名相关消息
            Message::DomainSelected(_) | 
            Message::DomainAdded(_) | 
            Message::DomainDeleted(_) => {
                self.domain_handler.handle(&mut self.state, message.clone())
            },
            
            // DNS相关消息
            Message::DnsRecordSelected(_) |
            Message::DnsAddRecord |
            Message::DnsEditRecord(_) |
            Message::DnsDeleteRecord(_) |
            Message::DnsFilterChanged(_) |
            Message::DnsSearchChanged(_) => {
                self.dns_handler.handle(&mut self.state, message.clone())
            },
            
            // 同步相关消息
            Message::SyncDomain(_) |
            Message::SyncAllDomains |
            Message::SyncCompleted(_) |
            Message::SyncCancelled => {
                self.sync_handler.handle(&mut self.state, message.clone())
            },
            
            // 窗口相关消息
            Message::WindowDrag |
            Message::WindowResize(_) |
            Message::WindowMinimize |
            Message::WindowMaximize |
            Message::WindowClose => {
                self.window_handler.handle(&mut self.state, message.clone())
            },
            
            // 其他消息
            _ => HandlerResult::None,
        };
        
        // 更新UI组件状态
        let domain_updated = self.domain_list_component.update(&mut self.state, message.clone());
        let dns_updated = self.dns_records_component.update(&mut self.state, message.clone());
        
        // 处理异步操作
        match specific_result {
            HandlerResult::Task(task) => task,
            HandlerResult::AsyncTask(future) => {
                Task::perform(future, |result| {
                    match result {
                        Ok(msg) => msg,
                        Err(e) => {
                            error!("异步任务执行失败: {}", e);
                            Message::Error(format!("操作失败: {}", e))
                        }
                    }
                })
            },
            HandlerResult::None => Task::none(),
        }
    }
    
    /// 渲染UI
    pub fn view(&self) -> Element<Message> {
        if !self.initialized {
            return self.render_loading_screen();
        }
        
        match self.state.ui.current_page {
            Page::Main => self.render_main_page(),
            Page::Settings => self.render_settings_page(),
            Page::Help => self.render_help_page(),
            Page::AddDomain => self.render_add_domain_page(),
            Page::EditDomain(_) => self.render_edit_domain_page(),
        }
    }
    
    /// 渲染加载屏幕
    fn render_loading_screen(&self) -> Element<Message> {
        container(
            column![
                iced::widget::text("正在初始化...")
                    .size(18)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                // 这里可以添加加载动画
            ]
            .align_items(iced::Alignment::Center)
            .spacing(10)
        )
        .center_x()
        .center_y()
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
    
    /// 渲染主页面
    fn render_main_page(&self) -> Element<Message> {
        let content = row![
            // 左侧域名列表
            container(
                column![
                    iced::widget::text("域名列表")
                        .size(16)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.3, 0.3, 0.3))),
                    self.domain_list_component.view(&self.state)
                ]
                .spacing(10)
            )
            .width(Length::FillPortion(1))
            .padding(10),
            
            // 分隔线
            container(
                iced::widget::vertical_rule(1)
            )
            .width(Length::Shrink),
            
            // 右侧DNS记录
            container(
                column![
                    iced::widget::text("DNS记录")
                        .size(16)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.3, 0.3, 0.3))),
                    self.dns_records_component.view(&self.state)
                ]
                .spacing(10)
            )
            .width(Length::FillPortion(2))
            .padding(10),
        ]
        .spacing(0)
        .height(Length::Fill);
        
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
    
    /// 渲染设置页面
    fn render_settings_page(&self) -> Element<Message> {
        container(
            iced::widget::text("设置页面")
                .size(18)
        )
        .center_x()
        .center_y()
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
    
    /// 渲染帮助页面
    fn render_help_page(&self) -> Element<Message> {
        container(
            iced::widget::text("帮助页面")
                .size(18)
        )
        .center_x()
        .center_y()
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
    
    /// 渲染添加域名页面
    fn render_add_domain_page(&self) -> Element<Message> {
        container(
            iced::widget::text("添加域名页面")
                .size(18)
        )
        .center_x()
        .center_y()
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
    
    /// 渲染编辑域名页面
    fn render_edit_domain_page(&self) -> Element<Message> {
        container(
            iced::widget::text("编辑域名页面")
                .size(18)
        )
        .center_x()
        .center_y()
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
    
    /// 获取当前状态
    pub fn get_state(&self) -> &AppState {
        &self.state
    }
    
    /// 获取可变状态引用
    pub fn get_state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }
    
    /// 检查是否已初始化
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// 关闭管理器
    pub async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("开始关闭域名管理器");
        
        // 关闭服务管理器
        self.service_manager.shutdown().await?;
        
        // 清理状态
        self.state = AppState::new();
        self.initialized = false;
        
        info!("域名管理器已关闭");
        Ok(())
    }
    
    /// 重新加载数据
    pub async fn reload(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("重新加载数据");
        
        if !self.initialized {
            return self.initialize().await;
        }
        
        self.load_initial_data().await
    }
    
    /// 获取统计信息
    pub fn get_statistics(&self) -> ManagerStatistics {
        ManagerStatistics {
            total_domains: self.state.data.domains.len(),
            total_dns_records: self.state.data.dns_records_cache
                .values()
                .map(|records| records.len())
                .sum(),
            active_domains: self.state.data.domains
                .iter()
                .filter(|domain| domain.status == "Active")
                .count(),
            last_sync_time: self.state.data.last_sync_time,
            is_syncing: self.state.ui.is_syncing,
            initialized: self.initialized,
        }
    }
}

impl Default for DomainManagerV2 {
    fn default() -> Self {
        Self::new()
    }
}

/// 管理器统计信息
#[derive(Debug, Clone)]
pub struct ManagerStatistics {
    pub total_domains: usize,
    pub total_dns_records: usize,
    pub active_domains: usize,
    pub last_sync_time: Option<chrono::DateTime<chrono::Utc>>,
    pub is_syncing: bool,
    pub initialized: bool,
}

/// 管理器错误类型
#[derive(Debug, thiserror::Error)]
pub enum ManagerError {
    #[error("管理器未初始化")]
    NotInitialized,
    
    #[error("数据库错误: {0}")]
    Database(#[from] crate::database::DatabaseError),
    
    #[error("服务错误: {0}")]
    Service(String),
    
    #[error("状态错误: {0}")]
    State(String),
    
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_manager_creation() {
        let manager = DomainManagerV2::new();
        assert!(!manager.is_initialized());
        assert_eq!(manager.get_statistics().total_domains, 0);
    }
    
    #[tokio::test]
    async fn test_manager_initialization() {
        let mut manager = DomainManagerV2::new();
        
        // 注意：这个测试可能需要模拟数据库连接
        // let result = manager.initialize().await;
        // assert!(result.is_ok());
        // assert!(manager.is_initialized());
    }
    
    #[test]
    fn test_statistics() {
        let manager = DomainManagerV2::new();
        let stats = manager.get_statistics();
        
        assert_eq!(stats.total_domains, 0);
        assert_eq!(stats.total_dns_records, 0);
        assert_eq!(stats.active_domains, 0);
        assert!(!stats.initialized);
        assert!(!stats.is_syncing);
    }
}