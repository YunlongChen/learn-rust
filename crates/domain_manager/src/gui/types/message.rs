use crate::api::model::dns_operate::RecordLog;
use crate::gui::model::domain::{DnsProvider, DomainName};
use crate::gui::pages::names::Page;
use crate::gui::pages::types::running::RunningPage;
use crate::gui::pages::types::settings::SettingsPage;
use crate::model::dns_record_response::{Record, Type};
use crate::translations::types::language::Language;
use crate::utils::types::file_info::FileInfo;
use crate::utils::types::web_page::WebPage;
use iced::keyboard::Key;
use iced::window;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Message {
    /// Change the language of the application
    ChangeLocale,
    /// Change the theme of the application
    ToggleTheme,
    ChangePage(Page),
    PageChanged(Page, Page),
    SubmitDomainForm,
    DomainDeleted(DomainName),
    AddDomainFormChanged(String),
    DnsProviderSelected(DnsProvider),
    QueryDomainDnsRecord(DomainName),
    ToHelp,
    KeyInput {
        key: Key,
    },
    CloseHelp,
    OpenHelp {
        last_page: Page,
    },
    QueryDomain,
    QueryDomainResult(Vec<DomainName>),
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
    Reset,
    Quit,
    CloseSettings,
    WindowId(Option<window::Id>),
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "界面")
    }
}
