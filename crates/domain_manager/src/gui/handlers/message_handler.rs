//! 消息处理器
//!
//! 负责分发和处理应用程序中的各种消息，将复杂的消息处理逻辑
//! 分解为更小、更专门的处理器。

use super::{
    DnsHandler, DomainHandler, EventHandler, HandlerResult, ProviderHandler, SyncHandler,
    UiHandler, WindowHandler,
};
use crate::gui::components::console::ConsoleTab;
use crate::gui::model::domain::{DnsProvider, DnsRecord, Domain};
use crate::gui::model::gui::ReloadModel;
use crate::gui::pages::domain::AddDomainProviderForm;
use crate::gui::pages::Page;
use crate::gui::state::app_state::{DataUpdate, StateUpdate, UiUpdate};
use crate::gui::state::AppState;
use crate::gui::types::credential::Credential;
use crate::model::dns_record_response::{Record, Type};
use crate::storage::{DnsRecordModal, DomainModal};
use crate::translations::types::language::Language;
use crate::translations::types::locale::Locale;
use crate::utils::types::file_info::FileInfo;
use crate::utils::types::web_page::WebPage;
use iced::{window, Point, Size, Task};
use std::process;
use tracing::{debug, info};
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
    Delete(usize),
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
    QueryRecord(usize),
    AddRecord {
        domain_id: usize,
        record_type: String,
        name: String,
        value: String,
        ttl: u32,
    },
    DeleteRecord {
        domain_id: usize,
        record_id: usize,
    },
    Delete(usize),
    RecordDeleted(usize),
    TestRecord(usize),

    EditRecord(DnsRecordModal),

    FormCancelled,
    FormNameChanged(String),
    FormValueChanged(String),
    FormRecordTypeChanged(Type),
    FormTtlChanged(i32),
    FormSubmit,

    ProviderSelected(usize),
    ProviderChange(String),
    DnsFilterChanged(Option<String>),
    DnsSearchChanged(String),
    DnsToggleRecord(usize),
    DnsRecordSelected(usize),

    // 重新加载域名解析
    ReloadDnsRecord(usize),
    DnsRecordReloaded(usize, Vec<DnsRecordModal>),
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
    Maximize,
    ToggleFloating,
    WindowResized(Size),
    WindowMinimize,
    WindowMaximize(bool),
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
    Selected(Option<DnsProvider>),
    AddFormProviderChanged(String),
    AddFormNameChanged(String),
    AddFormCredentialChanged(Credential),
    ValidateCredential,
    AddCredential,
    ProviderChange,
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
    ToggleLocale(Locale),
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
        }
    }

    /// 处理消息
    pub fn handle_message(
        &self,
        state: &mut AppState,
        message: MessageCategory,
    ) -> Task<MessageCategory> {
        match message {
            MessageCategory::App(msg) => self.handle_app(state, msg),
            MessageCategory::Navigation(msg) => self.handle_navigation(state, msg),
            MessageCategory::Domain(msg) => match self.domain_handler.handle(state, msg) {
                HandlerResult::None => Task::none(),
                HandlerResult::Task(task) => task,
                HandlerResult::StateUpdated => Task::none(),
                HandlerResult::StateUpdatedWithTask(task) => task,
                HandlerResult::NoChange => Task::none(),
            },
            MessageCategory::Dns(msg) => match self.dns_handler.handle(state, msg) {
                HandlerResult::None => Task::none(),
                HandlerResult::Task(task) => task,
                HandlerResult::StateUpdated => Task::none(),
                HandlerResult::StateUpdatedWithTask(task) => task,
                HandlerResult::NoChange => Task::none(),
            },
            MessageCategory::Sync(msg) => match self.sync_handler.handle(state, msg) {
                HandlerResult::None => Task::none(),
                HandlerResult::Task(task) => task,
                HandlerResult::StateUpdated => {
                    info!("同步状态发生了变化");
                    Task::none()
                }
                HandlerResult::StateUpdatedWithTask(task) => task,
                HandlerResult::NoChange => Task::none(),
            },
            MessageCategory::Provider(msg) => match self.provider_handler.handle(state, msg) {
                HandlerResult::None => Task::none(),
                HandlerResult::Task(task) => task,
                HandlerResult::StateUpdated => Task::none(),
                HandlerResult::StateUpdatedWithTask(task) => task,
                HandlerResult::NoChange => Task::none(),
            },
            MessageCategory::Window(msg) => match self.window_handler.handle(state, msg) {
                HandlerResult::None => Task::none(),
                HandlerResult::Task(task) => task,
                HandlerResult::StateUpdated => Task::none(),
                HandlerResult::StateUpdatedWithTask(task) => task,
                HandlerResult::NoChange => Task::none(),
            },
            MessageCategory::Config(msg) => self.handle_config(state, msg),
            MessageCategory::Ui(msg) => match self.ui_handler.handle(state, msg) {
                HandlerResult::None => Task::none(),
                HandlerResult::Task(task) => task,
                HandlerResult::StateUpdated => Task::none(),
                HandlerResult::StateUpdatedWithTask(task) => task,
                HandlerResult::NoChange => Task::none(),
            },
            MessageCategory::Console(_) => Task::none(),
            MessageCategory::Notification(_) => Task::none(),
            MessageCategory::Other(_) => Task::none(),
        }
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
                Task::none()
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
                state.update(StateUpdate::Ui(UiUpdate::NavigateTo(page)));
                Task::none()
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

    /// 处理UI消息
    fn handle_ui(&self, state: &mut AppState, message: UiMessage) -> Task<MessageCategory> {
        match message {
            UiMessage::SearchContentChanged(content) => {
                state.update(StateUpdate::Data(DataUpdate::SetSearchContent(content)));
                Task::none()
            }
            UiMessage::ShowToast(message) => {
                state.update(StateUpdate::Ui(UiUpdate::ShowToast(message)));
                Task::none()
            }
            UiMessage::HideToast => {
                state.update(StateUpdate::Ui(UiUpdate::HideToast));
                Task::none()
            }
            UiMessage::ToggleConsole => {
                state.update(StateUpdate::Ui(UiUpdate::ToggleConsole));
                Task::none()
            }
            UiMessage::ClearConsoleLog => {
                // 清除控制台日志的逻辑
                state.ui.set_message("控制台日志已清除".to_string());
                Task::none()
            }
            UiMessage::ConsoleTabChanged(tab) => {
                state.ui.switch_console_tab(tab);
                Task::none()
            }
            UiMessage::ChangePage(page) => {
                state.update(StateUpdate::Ui(UiUpdate::SetCurrentPage(page)));
                Task::none()
            }
            UiMessage::ToggleTheme => {
                state.update(StateUpdate::Ui(UiUpdate::ToggleTheme));
                Task::none()
            }
            UiMessage::Reset => {
                // 重置应用状态的逻辑
                Task::none()
            }
            UiMessage::Mock => {
                // 模拟数据的逻辑
                Task::none()
            }
            UiMessage::MockDataGenerated(domains) => {
                // 处理生成的模拟数据
                state.update(StateUpdate::Data(DataUpdate::SetDomains(domains)));
                Task::none()
            }
            UiMessage::ToggleFloatingWindow => {
                state.update(StateUpdate::Ui(UiUpdate::ToggleFloatingWindow));
                Task::none()
            }
            UiMessage::ToggleThumbnail => {
                state.update(StateUpdate::Ui(UiUpdate::ToggleThumbnail));
                Task::none()
            }
            UiMessage::ToggleLocale(_) => todo!(),
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
