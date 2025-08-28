//! 消息处理器
//! 
//! 负责分发和处理应用程序中的各种消息，将复杂的消息处理逻辑
//! 分解为更小、更专门的处理器。

use super::{
    DomainHandler, DnsHandler, SyncHandler, WindowHandler,
    HandlerResult, EventHandler
};
use crate::gui::state::{AppState, StateUpdate, UiUpdate, DataUpdate};
use crate::gui::{Message, pages::Page};
use iced::Task;

/// 消息分类枚举
/// 
/// 将消息按功能分类，便于分发到对应的处理器
#[derive(Debug, Clone)]
pub enum MessageCategory {
    /// 导航相关消息
    Navigation(NavigationMessage),
    /// 域名相关消息
    Domain(DomainMessage),
    /// DNS相关消息
    Dns(DnsMessage),
    /// 同步相关消息
    Sync(SyncMessage),
    /// 窗口相关消息
    Window(WindowMessage),
    /// 配置相关消息
    Config(ConfigMessage),
    /// UI相关消息
    Ui(UiMessage),
}

/// 导航消息
#[derive(Debug, Clone)]
pub enum NavigationMessage {
    PageChanged(Page),
    ToHelp,
    OpenHelp,
    CloseHelp,
    Back,
}

/// 域名消息
#[derive(Debug, Clone)]
pub enum DomainMessage {
    Selected(String),
    AddFormChanged(String),
    SubmitForm,
    Delete(String),
    Query(String),
}

/// DNS消息
#[derive(Debug, Clone)]
pub enum DnsMessage {
    QueryRecord(String),
    AddRecord {
        domain: String,
        record_type: String,
        name: String,
        value: String,
        ttl: u32,
    },
    DeleteRecord {
        domain: String,
        record_id: String,
    },
    ProviderSelected(String),
    ProviderChange(String),
}

/// 同步消息
#[derive(Debug, Clone)]
pub enum SyncMessage {
    Start,
    StartAll,
    Complete(Result<Vec<crate::models::Domain>, String>),
    AllComplete(Result<(), String>),
    Cancel,
}

/// 窗口消息
#[derive(Debug, Clone)]
pub enum WindowMessage {
    Drag,
    StartDrag,
    Moved(iced::Point),
    Resized(iced::Size),
    Minimize,
    Maximize,
    ToggleFloating,
}

/// 配置消息
#[derive(Debug, Clone)]
pub enum ConfigMessage {
    ChangeLocale(crate::config::Locale),
    ToggleTheme,
    BackgroundOpacityChanged(f32),
    Save,
}

/// UI消息
#[derive(Debug, Clone)]
pub enum UiMessage {
    SearchContentChanged(String),
    ShowToast(String),
    HideToast,
    ToggleConsole,
    ClearConsoleLog,
    ConsoleTabChanged(crate::gui::state::ui_state::ConsoleTab),
}

/// 消息处理器
/// 
/// 负责将消息分发到对应的专门处理器
pub struct MessageHandler {
    domain_handler: DomainHandler,
    dns_handler: DnsHandler,
    sync_handler: SyncHandler,
    window_handler: WindowHandler,
}

impl MessageHandler {
    /// 创建新的消息处理器
    pub fn new() -> Self {
        Self {
            domain_handler: DomainHandler::new(),
            dns_handler: DnsHandler::new(),
            sync_handler: SyncHandler::new(),
            window_handler: WindowHandler::new(),
        }
    }
    
    /// 处理消息
    pub fn handle_message(&self, state: &mut AppState, message: Message) -> Task<Message> {
        let category = self.categorize_message(message);
        
        match category {
            MessageCategory::Navigation(msg) => self.handle_navigation(state, msg),
            MessageCategory::Domain(msg) => {
                match self.domain_handler.handle(state, msg) {
                    HandlerResult::None => Task::none(),
                    HandlerResult::Task(task) => task,
                    HandlerResult::StateUpdated => Task::none(),
                    HandlerResult::StateUpdatedWithTask(task) => task,
                }
            },
            MessageCategory::Dns(msg) => {
                match self.dns_handler.handle(state, msg) {
                    HandlerResult::None => Task::none(),
                    HandlerResult::Task(task) => task,
                    HandlerResult::StateUpdated => Task::none(),
                    HandlerResult::StateUpdatedWithTask(task) => task,
                }
            },
            MessageCategory::Sync(msg) => {
                match self.sync_handler.handle(state, msg) {
                    HandlerResult::None => Task::none(),
                    HandlerResult::Task(task) => task,
                    HandlerResult::StateUpdated => Task::none(),
                    HandlerResult::StateUpdatedWithTask(task) => task,
                }
            },
            MessageCategory::Window(msg) => {
                match self.window_handler.handle(state, msg) {
                    HandlerResult::None => Task::none(),
                    HandlerResult::Task(task) => task,
                    HandlerResult::StateUpdated => Task::none(),
                    HandlerResult::StateUpdatedWithTask(task) => task,
                }
            },
            MessageCategory::Config(msg) => self.handle_config(state, msg),
            MessageCategory::Ui(msg) => self.handle_ui(state, msg),
        }
    }
    
    /// 将消息分类
    fn categorize_message(&self, message: Message) -> MessageCategory {
        match message {
            // 导航相关
            Message::PageChanged(page) => MessageCategory::Navigation(NavigationMessage::PageChanged(page)),
            Message::ToHelp => MessageCategory::Navigation(NavigationMessage::ToHelp),
            Message::OpenHelp => MessageCategory::Navigation(NavigationMessage::OpenHelp),
            Message::CloseHelp => MessageCategory::Navigation(NavigationMessage::CloseHelp),
            
            // 域名相关
            Message::DomainSelected(domain) => MessageCategory::Domain(DomainMessage::Selected(domain)),
            Message::AddDomainFormChanged(content) => MessageCategory::Domain(DomainMessage::AddFormChanged(content)),
            Message::SubmitDomainForm => MessageCategory::Domain(DomainMessage::SubmitForm),
            Message::DomainDeleted(domain) => MessageCategory::Domain(DomainMessage::Delete(domain)),
            Message::QueryDomainResult(result) => {
                // 这里需要根据实际的Message定义来处理
                MessageCategory::Domain(DomainMessage::Query("unknown".to_string()))
            },
            
            // DNS相关
            Message::QueryDomainDnsRecord(domain) => MessageCategory::Dns(DnsMessage::QueryRecord(domain)),
            Message::DnsProviderSelected(provider) => MessageCategory::Dns(DnsMessage::ProviderSelected(provider)),
            Message::DnsProviderChange(provider) => MessageCategory::Dns(DnsMessage::ProviderChange(provider)),
            
            // 同步相关
            Message::Sync => MessageCategory::Sync(SyncMessage::Start),
            Message::SyncAllDomainsComplete(result) => {
                match result {
                    Ok(_) => MessageCategory::Sync(SyncMessage::AllComplete(Ok(()))),
                    Err(e) => MessageCategory::Sync(SyncMessage::AllComplete(Err(e))),
                }
            },
            
            // 窗口相关
            Message::DragWindow => MessageCategory::Window(WindowMessage::Drag),
            Message::StartDragWindow => MessageCategory::Window(WindowMessage::StartDrag),
            Message::WindowMoved(point) => MessageCategory::Window(WindowMessage::Moved(point)),
            Message::WindowResized(size) => MessageCategory::Window(WindowMessage::Resized(size)),
            Message::WindowMinimize => MessageCategory::Window(WindowMessage::Minimize),
            Message::WindowMaximize => MessageCategory::Window(WindowMessage::Maximize),
            Message::ToggleFloatingMode => MessageCategory::Window(WindowMessage::ToggleFloating),
            
            // 配置相关
            Message::ChangeLocale(locale) => MessageCategory::Config(ConfigMessage::ChangeLocale(locale)),
            Message::ToggleTheme => MessageCategory::Config(ConfigMessage::ToggleTheme),
            Message::BackgroundOpacityChanged(opacity) => MessageCategory::Config(ConfigMessage::BackgroundOpacityChanged(opacity)),
            
            // UI相关
            Message::SearchContentChanged(content) => MessageCategory::Ui(UiMessage::SearchContentChanged(content)),
            Message::ShowToast(message) => MessageCategory::Ui(UiMessage::ShowToast(message)),
            Message::HideToast => MessageCategory::Ui(UiMessage::HideToast),
            Message::ToggleConsoleTab => MessageCategory::Ui(UiMessage::ToggleConsole),
            Message::ClearConsoleLog => MessageCategory::Ui(UiMessage::ClearConsoleLog),
            
            // 默认处理
            _ => {
                // 对于未分类的消息，暂时归类为UI消息
                MessageCategory::Ui(UiMessage::ShowToast("未知消息类型".to_string()))
            }
        }
    }
    
    /// 处理导航消息
    fn handle_navigation(&self, state: &mut AppState, message: NavigationMessage) -> Task<Message> {
        match message {
            NavigationMessage::PageChanged(page) => {
                state.update(StateUpdate::Ui(UiUpdate::NavigateTo(page)));
                Task::none()
            },
            NavigationMessage::ToHelp | NavigationMessage::OpenHelp => {
                state.ui.show_help = true;
                Task::none()
            },
            NavigationMessage::CloseHelp => {
                state.ui.show_help = false;
                Task::none()
            },
            NavigationMessage::Back => {
                state.update(StateUpdate::Ui(UiUpdate::NavigateBack));
                Task::none()
            },
        }
    }
    
    /// 处理配置消息
    fn handle_config(&self, state: &mut AppState, message: ConfigMessage) -> Task<Message> {
        match message {
            ConfigMessage::ChangeLocale(locale) => {
                state.update(StateUpdate::Config(crate::gui::state::app_state::ConfigUpdate::SetLocale(locale)));
                Task::none()
            },
            ConfigMessage::ToggleTheme => {
                state.update(StateUpdate::Ui(UiUpdate::ToggleTheme));
                Task::none()
            },
            ConfigMessage::BackgroundOpacityChanged(opacity) => {
                state.update(StateUpdate::Ui(UiUpdate::SetBackgroundOpacity(opacity)));
                Task::none()
            },
            ConfigMessage::Save => {
                state.update(StateUpdate::Config(crate::gui::state::app_state::ConfigUpdate::Save));
                Task::none()
            },
        }
    }
    
    /// 处理UI消息
    fn handle_ui(&self, state: &mut AppState, message: UiMessage) -> Task<Message> {
        match message {
            UiMessage::SearchContentChanged(content) => {
                state.update(StateUpdate::Data(DataUpdate::SetSearchContent(content)));
                Task::none()
            },
            UiMessage::ShowToast(message) => {
                state.update(StateUpdate::Ui(UiUpdate::ShowToast(message)));
                Task::none()
            },
            UiMessage::HideToast => {
                state.update(StateUpdate::Ui(UiUpdate::HideToast));
                Task::none()
            },
            UiMessage::ToggleConsole => {
                state.update(StateUpdate::Ui(UiUpdate::ToggleConsole));
                Task::none()
            },
            UiMessage::ClearConsoleLog => {
                // 清除控制台日志的逻辑
                state.ui.set_message("控制台日志已清除".to_string());
                Task::none()
            },
            UiMessage::ConsoleTabChanged(tab) => {
                state.ui.switch_console_tab(tab);
                Task::none()
            },
        }
    }
}

impl Default for MessageHandler {
    fn default() -> Self {
        Self::new()
    }
}