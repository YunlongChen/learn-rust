//! 消息处理器
//!
//! 负责分发和处理应用程序中的各种消息，将复杂的消息处理逻辑
//! 分解为更小、更专门的处理器。

use super::{
    DnsHandler, DomainHandler, EventHandler, ProviderHandler, SyncHandler, UiHandler, WindowHandler,
};
use crate::gui::components::console::ConsoleTab;
use crate::gui::handlers::database_handler::DataStoreHandler;
use crate::gui::model::domain::{DnsProvider, Domain};
use crate::gui::model::gui::ReloadModel;
use crate::gui::pages::domain::VerificationStatus;
use crate::gui::pages::Page;
use crate::gui::state::app_state::{StateUpdate, UiUpdate};
use crate::gui::state::AppState;
use crate::gui::types::credential::CredentialMessage;
use crate::model::dns_record_response::{Record, Type};
use crate::models::account::{Account, NewAccount};
use crate::storage::{DnsRecordModal, DomainModal, agents};
use crate::translations::types::language::Language;
use crate::translations::types::locale::Locale;
use crate::utils::types::file_info::FileInfo;
use crate::utils::types::web_page::WebPage;
use iced::{window, Point, Size, Task};
use sea_orm::DatabaseConnection;
use std::process;
use window::Id;

/// 消息分类枚举
///
/// 将消息按功能分类，便于分发到对应的处理器
#[derive(Debug, Clone)]
pub enum MessageCategory {
    /// 应用程序生命周期消息
    App(AppMessage),
    /// 导航相关消息
    Navigation(NavigationMessage),
    /// 域名相关消息
    Domain(DomainMessage),
    /// DNS相关消息
    Dns(DnsMessage),
    /// 同步相关消息
    Sync(SyncMessage),
    /// 托管商相关消息
    Provider(ProviderMessage),
    /// 窗口相关消息
    Window(WindowMessage),
    /// 配置相关消息
    Config(ConfigMessage),
    /// UI相关消息
    Ui(UiMessage),
    /// 控制台消息
    Console(ConsoleMessage),
    /// 通知消息
    Notification(NotificationMessage),
    /// 数据库消息
    Database(DatabaseMessage),
    /// Agent相关消息
    Agent(AgentMessage),
    /// 其他消息
    Other(OtherMessage),
}

/// 应用程序消息
#[derive(Debug, Clone)]
pub enum AppMessage {
    Started,
    Initialize,
    Shutdown,
}

/// 数据库消息
#[derive(Debug, Clone)]
pub enum DatabaseMessage {
    Connected(Result<DatabaseConnection, String>),
    AddAccount(NewAccount),
    AccountAdded(Result<Account, String>),
    UpdateAccount(Account),
    AccountUpdated(Result<(), String>),
    DeleteAccount(i64),
    AccountDeleted(Result<i64, String>),
    AddDomain(crate::models::domain::NewDomain),
    DomainAdded(Result<i64, String>), // 返回 AccountID 以便刷新列表
    DeleteDomain(i64),
    DomainDeleted(Result<i64, String>),
}

/// 导航消息
#[derive(Debug, Clone)]
pub enum NavigationMessage {
    PageChanged(Page),
    ToHelp,
    OpenHelp,
    CloseHelp,
    Back,
    OpenWebPage(WebPage),
    OpenFile(String, FileInfo, fn(String) -> MessageCategory),
    ChangeConsoleTab(ConsoleTab),
    HideModal,
}

/// 域名消息
#[derive(Debug, Clone)]
pub enum DomainMessage {
    Selected(Domain),
    AddFormChanged(AddDomainFormMessage),
    SubmitForm,
    Delete(i64),
    DeleteRequest(i64),
    DeleteCancel,
    Query(String),
    Reload,
    QueryDomainResult(Vec<Domain>),
}

#[derive(Debug, Clone)]
pub enum AddDomainFormMessage {
    Submit,
    ProviderChanged(Option<DnsProvider>),
}

/// DNS消息
#[derive(Debug, Clone)]
pub enum DnsMessage {
    QueryRecord(i64),
    AddRecord {
        domain_id: i64,
        record_type: String,
        name: String,
        value: String,
        ttl: u32,
    },
    DeleteRecord {
        domain_id: i64,
        record_id: i64,
    },
    Delete(i64),
    DeleteRequest(i64),
    DeleteCancel,
    RecordDeleted(i64),
    TestRecord(i64),

    EditRecord(DnsRecordModal),

    FormCancelled,
    FormNameChanged(String),
    FormValueChanged(String),
    FormRecordTypeChanged(Type),
    FormTtlChanged(i32),
    FormSubmit,
    FormSubmitSuccess(i64),

    ProviderSelected(usize),
    ProviderChange(String),
    DnsFilterChanged(Option<String>),
    DnsSearchChanged(String),
    DnsToggleRecord(usize),
    DnsRecordSelected(usize),
    RecordHovered(Option<usize>),
    CopyRecordUrl(i64),

    // 重新加载域名解析
    ReloadDnsRecord(usize),
    DnsRecordReloaded(i64, Vec<DnsRecordModal>),
    QueryDnsResult(Vec<Record>),
}

/// 同步消息
#[derive(Debug, Clone)]
pub enum SyncMessage {
    Start,
    StartAll,
    SyncAllDomains,
    Reload,
    Complete(Result<Vec<DomainModal>, String>),
    AllComplete(Result<(), String>),
    Cancel,
    DataReloaded(ReloadModel),
    DomainSyncComplete(String, Result<Vec<DnsRecordModal>, String>),
}

/// 窗口消息
#[derive(Debug, Clone)]
pub enum WindowMessage {
    Drag,
    Moved(Point),
    Resized(Size),
    ToggleFloating,
    WindowResized(Size),
    WindowMinimize,
    WindowMaximize,
    BackgroundOpacityChange(f32),
    BackgroundToggle,
    DragWindow(Point),
    StartDrag(Id),
    CloseRequest,
    WindowFocused,
    WindowId(Option<Id>),
}

/// 配置消息
#[derive(Debug, Clone)]
pub enum ConfigMessage {
    ChangeLocale(Language),
    ToggleTheme,
    BackgroundOpacityChanged(f32),
    Save,
}

/// 托管商消息
#[derive(Debug, Clone)]
pub enum ProviderMessage {
    Selected(DnsProvider),
    AddFormNameChanged(String),
    AddFormCredentialChanged(CredentialMessage),
    ValidateCredential,
    AddCredential,
    ProviderChange,
    VerificationStatusChanged(VerificationStatus),
    // 新增消息
    ToggleForm(bool),
    Delete(i64),
    Edit(i64),
    ConfirmDelete(i64),
    CancelDelete,
    // 数据加载
    Load,
    Loaded(Result<Vec<Account>, String>),
    // 域名管理
    ToggleExpand(i64),
    ToggleAddDomain(i64, bool),        // AccountID, is_adding
    NewDomainNameChanged(i64, String), // AccountID, name
    ConfirmAddDomain(i64),             // AccountID
    AddDomain(i64),                    // 原有的 AddDomain 保留，可能用于触发添加模式
    DeleteDomain(i64, i64),            // (AccountID, DomainID)
    SyncDomainInfo(i64),               // AccountID
    DomainDeleted(Result<i64, String>),
    DomainAdded(Result<i64, String>), // 域名添加结果
    LoadDomains(i64),                 // AccountID
    DomainsLoaded(i64, Result<Vec<DomainModal>, String>), // AccountID, Result
    ProviderHovered(Option<i64>),     // AccountID, None表示移出
}

/// UI消息
#[derive(Debug, Clone)]
pub enum UiMessage {
    SearchContentChanged(String),
    ShowToast(String),
    HideToast,
    ToggleConsole,
    ClearConsoleLog,
    ConsoleTabChanged(ConsoleTab),
    ChangePage(Page),
    ToggleTheme,
    ToggleLocale,
    Reset,
    Mock,
    MockDataGenerated(Vec<DomainModal>),
    ToggleFloatingWindow,
    ToggleThumbnail,
}

#[derive(Debug, Clone)]
pub enum ConsoleMessage {
    ClearConsoleLogs,
}

#[derive(Debug, Clone)]
pub enum NotificationMessage {
    ClearAllNotifications,
    ShowToast(String),
    HideToast,
}

#[derive(Debug, Clone)]
pub enum OtherMessage {
    MockDataGenerated(Vec<DomainModal>),
}

/// Agent相关消息
#[derive(Debug, Clone)]
pub enum AgentMessage {
    /// 切换添加模式
    ToggleAddMode,
    /// 添加表单名称变更
    AddFormNameChanged(String),
    /// 添加表单端点变更
    AddFormEndpointChanged(String),
    /// 添加表单密钥变更
    AddFormKeyChanged(String),
    /// 保存新Agent
    SaveAgent,
    /// 取消添加
    CancelAdd,
    /// 删除Agent
    DeleteAgent(String),
    /// 加载Agent列表
    LoadAgents,
    /// Agent列表已加载
    AgentsLoaded(Result<Vec<crate::agent::model::Agent>, String>),
    /// 测试连接
    TestConnection,
    /// 选择Agent查看详情
    SelectAgent(String),
    /// 关闭详情视图
    CloseAgentDetail,
    /// 批准Agent
    ApproveAgent(String),
    /// 拒绝Agent
    DenyAgent(String),
}

/// 消息处理器
///
/// 负责将消息分发到对应的专门处理器
#[derive(Debug)]
pub struct MessageHandler {
    domain_handler: DomainHandler,
    dns_handler: DnsHandler,
    sync_handler: SyncHandler,
    window_handler: WindowHandler,
    provider_handler: ProviderHandler,
    ui_handler: UiHandler,
    database_handler: DataStoreHandler,
}

impl MessageHandler {
    /// 创建新的消息处理器
    pub fn new() -> Self {
        Self {
            domain_handler: DomainHandler::new(),
            dns_handler: DnsHandler::new(),
            sync_handler: SyncHandler::new(),
            window_handler: WindowHandler::new(),
            provider_handler: ProviderHandler::new(),
            ui_handler: UiHandler::new(),
            database_handler: DataStoreHandler::new(),
        }
    }

    /// 处理消息
    pub fn handle_message(
        &self,
        state: &mut AppState,
        message: MessageCategory,
    ) -> Task<MessageCategory> {
        let result_message = match message {
            MessageCategory::App(msg) => self.handle_app(state, msg),
            MessageCategory::Navigation(msg) => self.handle_navigation(state, msg),
            MessageCategory::Domain(msg) => self.domain_handler.handle(state, msg).into(),
            MessageCategory::Dns(msg) => self.dns_handler.handle(state, msg).into(),
            MessageCategory::Sync(msg) => self.sync_handler.handle(state, msg).into(),
            MessageCategory::Provider(msg) => self.provider_handler.handle(state, msg).into(),
            MessageCategory::Window(window_message) => {
                self.window_handler.handle(state, window_message).into()
            }
            MessageCategory::Config(msg) => self.handle_config(state, msg),
            MessageCategory::Ui(msg) => self.ui_handler.handle(state, msg).into(),
            MessageCategory::Database(message) => {
                self.database_handler.handle(state, message).into()
            }
            MessageCategory::Agent(msg) => self.handle_agent(state, msg),
            MessageCategory::Console(_) => Task::none(),
            MessageCategory::Notification(_) => Task::none(),
            MessageCategory::Other(_) => Task::none(),
        };
        // 检查 Task 是否为空
        result_message
    }

    /// 处理应用程序消息
    fn handle_app(&self, state: &mut AppState, message: AppMessage) -> Task<MessageCategory> {
        match message {
            AppMessage::Started => {
                // 应用程序启动时触发初始化
                Task::perform(async {}, |_| MessageCategory::App(AppMessage::Started))
            }
            AppMessage::Initialize => {
                // 执行初始化逻辑
                state.update(StateUpdate::Ui(UiUpdate::SetLoading(true)));
                Task::done(MessageCategory::Sync(SyncMessage::Reload))
            }
            AppMessage::Shutdown => {
                // 应用程序关闭时的清理工作
                Task::none()
            }
        }
    }

    /// 处理导航消息
    fn handle_navigation(
        &self,
        state: &mut AppState,
        message: NavigationMessage,
    ) -> Task<MessageCategory> {
        match message {
            NavigationMessage::PageChanged(page) => {
                state.update(StateUpdate::Ui(UiUpdate::NavigateTo(page.clone())));

                // 页面切换时的自动刷新逻辑
                match page {
                    Page::Providers => Task::done(MessageCategory::Provider(ProviderMessage::Load)),
                    Page::Agent => Task::done(MessageCategory::Agent(AgentMessage::LoadAgents)),
                    _ => Task::none(),
                }
            }
            NavigationMessage::ToHelp | NavigationMessage::OpenHelp => {
                state.ui.show_help = true;
                Task::none()
            }
            NavigationMessage::CloseHelp => {
                state.ui.show_help = false;
                Task::none()
            }
            NavigationMessage::Back => {
                state.update(StateUpdate::Ui(UiUpdate::NavigateBack));
                Task::none()
            }
            NavigationMessage::OpenWebPage(web_page) => {
                open_web(&web_page);
                Task::none()
            }
            NavigationMessage::OpenFile(_, _, _) => Task::none(),
            NavigationMessage::ChangeConsoleTab(_) => Task::none(),
            NavigationMessage::HideModal => Task::none(),
        }
    }

    /// 处理配置消息
    fn handle_config(&self, state: &mut AppState, message: ConfigMessage) -> Task<MessageCategory> {
        match message {
            ConfigMessage::ChangeLocale(language) => {
                let locale = match language {
                    Language::ZH => Locale::Chinese,
                    Language::ZH_TW => Locale::Chinese,
                    Language::EN => Locale::English,
                    Language::IT => Locale::English,
                    Language::FR => Locale::English,
                    Language::ES => Locale::English,
                    Language::PL => Locale::English,
                    Language::DE => Locale::English,
                    Language::UK => Locale::English,
                    Language::RO => Locale::English,
                    Language::KO => Locale::English,
                    Language::PT => Locale::English,
                    Language::TR => Locale::English,
                    Language::RU => Locale::English,
                    Language::EL => Locale::English,
                    Language::SV => Locale::English,
                    Language::FI => Locale::English,
                    Language::JA => Locale::English,
                    Language::UZ => Locale::English,
                    Language::VI => Locale::English,
                    Language::ID => Locale::English,
                };
                state.update(StateUpdate::Config(
                    crate::gui::state::app_state::ConfigUpdate::SetLocale(locale),
                ));
                Task::none()
            }
            ConfigMessage::ToggleTheme => {
                state.update(StateUpdate::Ui(UiUpdate::ToggleTheme));
                Task::none()
            }
            ConfigMessage::BackgroundOpacityChanged(opacity) => {
                state.update(StateUpdate::Ui(UiUpdate::SetBackgroundOpacity(opacity)));
                Task::none()
            }
            ConfigMessage::Save => {
                state.update(StateUpdate::Config(
                    crate::gui::state::app_state::ConfigUpdate::Save,
                ));
                Task::none()
            }
        }
    }

    /// 处理Agent消息
    fn handle_agent(&self, state: &mut AppState, message: AgentMessage) -> Task<MessageCategory> {
        match message {
            AgentMessage::ToggleAddMode => {
                state.data.agent_page.toggle_add_mode();
                Task::none()
            }
            AgentMessage::AddFormNameChanged(name) => {
                state.data.agent_page.new_agent_name = name;
                Task::none()
            }
            AgentMessage::AddFormEndpointChanged(endpoint) => {
                state.data.agent_page.new_agent_endpoint = endpoint;
                Task::none()
            }
            AgentMessage::AddFormKeyChanged(key) => {
                state.data.agent_page.new_agent_key = key;
                Task::none()
            }
            AgentMessage::SaveAgent => {
                // 保存Agent到数据库
                tracing::info!("SaveAgent called, database present: {}", state.database.is_some());
                if let Some(conn) = &state.database {
                    let agent = state.data.agent_page.create_agent();
                    tracing::info!("create_agent returned: {:?}, name: '{}'", agent.is_some(), state.data.agent_page.new_agent_name);
                    if let Some(agent) = agent {
                        let conn_clone = conn.clone();
                        tracing::info!("Saving agent: {}", agent.name);
                        return Task::perform(
                            async move {
                                agents::create_agent(&conn_clone, agent)
                                    .await
                                    .map_err(|e| e.to_string())
                            },
                            |result| {
                                match result {
                                    Ok(_) => MessageCategory::Agent(AgentMessage::LoadAgents),
                                    Err(e) => MessageCategory::Ui(UiMessage::ShowToast(format!(
                                        "保存Agent失败: {}",
                                        e
                                    ))),
                                }
                            },
                        );
                    } else {
                        tracing::warn!("create_agent returned None - agent name may be empty");
                        return Task::done(MessageCategory::Ui(UiMessage::ShowToast("请输入Agent名称".to_string())));
                    }
                } else {
                    tracing::warn!("Database connection not available");
                    return Task::done(MessageCategory::Ui(UiMessage::ShowToast("数据库未连接".to_string())));
                }
            }
            AgentMessage::CancelAdd => {
                state.data.agent_page.cancel_add();
                Task::none()
            }
            AgentMessage::DeleteAgent(id) => {
                if let Some(conn) = &state.database {
                    let conn_clone = conn.clone();
                    // 先更新内存中的数据
                    state.data.agent_page.delete_agent(&id);
                    // 然后异步删除数据库中的记录
                    let id_clone = id.clone();
                    return Task::perform(
                        async move {
                            agents::delete_agent(&conn_clone, uuid::Uuid::parse_str(&id_clone).unwrap_or_default())
                                .await
                                .map_err(|e| e.to_string())
                        },
                        |result| {
                            if let Err(e) = result {
                                tracing::error!("删除Agent失败: {}", e);
                            }
                            MessageCategory::Agent(AgentMessage::LoadAgents)
                        },
                    );
                }
                state.data.agent_page.delete_agent(&id);
                Task::none()
            }
            AgentMessage::LoadAgents => {
                if let Some(conn) = &state.database {
                    let conn_clone = conn.clone();
                    return Task::perform(
                        async move {
                            agents::find_all_agents(&conn_clone)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        |result| MessageCategory::Agent(AgentMessage::AgentsLoaded(result)),
                    );
                }
                Task::none()
            }
            AgentMessage::AgentsLoaded(result) => {
                match result {
                    Ok(agents) => {
                        state.data.agent_page.agents = agents;
                    }
                    Err(e) => {
                        tracing::error!("加载Agent失败: {}", e);
                    }
                }
                Task::none()
            }
            AgentMessage::TestConnection => {
                state.data.agent_page.test_connection();
                Task::none()
            }
            AgentMessage::SelectAgent(id) => {
                state.data.agent_page.select_agent(&id);
                Task::none()
            }
            AgentMessage::CloseAgentDetail => {
                state.data.agent_page.close_detail();
                Task::none()
            }
            AgentMessage::ApproveAgent(id) => {
                tracing::info!("批准Agent: {}", id);
                // TODO: 实现批准逻辑 - 更新数据库中的审批状态
                // 目前只是关闭详情视图
                state.data.agent_page.close_detail();
                Task::none()
            }
            AgentMessage::DenyAgent(id) => {
                tracing::info!("拒绝Agent: {}", id);
                // TODO: 实现拒绝逻辑 - 更新数据库中的审批状态
                // 目前只是关闭详情视图
                state.data.agent_page.close_detail();
                Task::none()
            }
        }
    }
}

impl Default for MessageHandler {
    fn default() -> Self {
        Self::new()
    }
}

fn open_web(web_page: &WebPage) {
    let url = web_page.get_url();

    #[cfg(target_os = "windows")]
    let cmd = "explorer";
    #[cfg(target_os = "macos")]
    let cmd = "open";
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    let cmd = "xdg-open";

    process::Command::new(cmd)
        .arg(url)
        .spawn()
        .unwrap()
        .wait()
        .unwrap_or_default();
}
