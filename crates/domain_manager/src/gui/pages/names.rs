use crate::gui::model::domain::DnsRecord;
use crate::gui::pages::types::settings::SettingsPage;
use crate::utils::i18_utils::get_text;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum Page {
    DomainPage,
    AddProvider,
    AddDomain,
    DnsRecord,
    AddRecord,
    EditRecord(DnsRecord),
    Help,
    Settings(SettingsPage),
    Demo(DemoPage),
    Console,
    Dashboard,
    EditDomain,
}

impl Display for Page {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Page::AddProvider => write!(f, "添加托管商"),
            Page::DomainPage => write!(f, "page_domain_manage"),
            Page::AddDomain => write!(f, "page_add_domain"),
            Page::DnsRecord => write!(f, "DnsRecord"),
            Page::AddRecord => write!(f, "{}", get_text("add_record")),
            Page::EditRecord(_) => write!(f, "{}", get_text("edit_record")),
            Page::Help => write!(f, "Help"),
            Page::Settings(settings_page) => write!(f, "Settings: {:?}", settings_page),
            Page::Demo(demo) => write!(f, "Demo:{:?}", demo),
            Page::Console => write!(f, "控制台"),
            Page::Dashboard => write!(f, "dashboard"),
            Page::EditDomain => write!(f, "编辑域名"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum DemoPage {
    Scrollers,
}

impl Display for DemoPage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DemoPage::Scrollers => write!(f, "Scrollers"),
        }
    }
}
