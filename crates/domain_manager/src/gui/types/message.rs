use crate::api::model::dns_operate::RecordLog;
use crate::gui::model::domain::{DnsProvider, Domain};
use crate::gui::pages::domain::DomainProvider;
use crate::gui::pages::names::Page;
use crate::gui::pages::types::running::RunningPage;
use crate::gui::pages::types::settings::SettingsPage;
use crate::gui::types::credential::Credential;
use crate::model::dns_record_response::{Record, Type};
use crate::translations::types::language::Language;
use crate::translations::types::locale::Locale;
use crate::utils::types::file_info::FileInfo;
use crate::utils::types::web_page::WebPage;
use iced::keyboard::Key;
use iced::window;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Message {
    /// Change the language of the application
    ChangeLocale(Locale),
    LocaleChanged(Locale),
    /// Change the theme of the application
    ToggleTheme,
    ChangePage(Page),
    PageChanged(Page, Page),
    SubmitDomainForm,
    DomainDeleted(Domain),
    AddDomainFormChanged(String),
    SyncAllDomains,
    SyncAllDomainsComplete(SyncResult),
    SyncAllDomainsFailed,
    CancelSyncDomain,
    DnsProviderSelected(DnsProvider),
    QueryDomainDnsRecord(Domain),
    AddProviderFormProviderChanged(DnsProvider),
    AddProviderFormNameChanged(String),
    AddProviderFormCredentialChanged(Credential),
    ToHelp,
    KeyInput {
        key: Key,
    },
    CloseHelp,
    OpenHelp {
        last_page: Option<Page>,
    },
    QueryDomain,
    QueryDomainResult(Vec<Domain>),
    QueryDnsResult(Vec<Record>),
    QueryDnsLogResult(Vec<RecordLog>),
    DnsDelete(String),
    AddDnsRecord,
    DnsFormNameChanged(String),
    DnsFormRecordTypeChanged(Type),
    DnsFormValueChanged(String),
    DnsFormTtlChanged(i32),
    AddDnsFormSubmit,
    AddDnsFormCancelled,
    DnsRecordDeleted(String),
    ToggleThumbnail(bool),
    OpenSettings(SettingsPage),
    ResetButtonPressed,
    ChangeRunningPage(RunningPage),
    /// 选择语言
    LanguageSelection(Language),
    /// Emit when the main window be focused
    WindowFocused,
    WindowMoved(f32, f32),
    /// The app window size has been changed
    WindowResized(f32, f32),
    /// Wrapper around the Quit message
    QuitWrapper,
    ClearAllNotifications,
    HideModal,
    OpenFile(String, FileInfo, fn(String) -> Message),
    OpenWebPage(WebPage),
    Start,
    Reset,
    Quit,
    CloseSettings,
    WindowId(Option<window::Id>),

    /// 试验品
    ProviderSelected(DomainProvider),
    DomainSelected(Domain),
    SearchChanged(String),
    AddDomain,
    // AddDnsRecord,
    EditDnsRecord(usize),
    DeleteDnsRecord(usize),
    Filter,
    Export,
    ReSet,
    FeatureClicked(String),
    AddDnsProvider,
    ValidateCredential,
    DnsProviderChange
}

#[derive(Debug, Clone)]
pub enum SyncResult {
    // 同步失败
    Failed(String),

    /// 同步成功
    Success,

    // 已取消
    Cancelled,
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "界面")
    }
}
