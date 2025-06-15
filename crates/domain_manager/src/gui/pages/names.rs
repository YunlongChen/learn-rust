use crate::utils::i18_utils::get_text;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum Page {
    DomainPage,
    AddProvider,
    AddDomain,
    DnsRecord,
    AddRecord,
    Help,
    Demo(DemoPage),
}

impl Display for Page {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Page::AddProvider => write!(f, "添加服务商"),
            Page::DomainPage => write!(f, "page_domain_manage"),
            Page::AddDomain => write!(f, "page_add_domain"),
            Page::DnsRecord => write!(f, "DnsRecord"),
            Page::AddRecord => write!(f, "{}", get_text("add_record")),
            Page::Help => write!(f, "Help"),
            Page::Demo(demo) => write!(f, "Demo:{:?}", demo),
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
