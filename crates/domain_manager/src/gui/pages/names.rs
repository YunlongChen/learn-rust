use crate::utils::i18_utils::get_text;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Page {
    DomainPage,
    AddDomain,
    DnsRecord,
    AddRecord,
    Help,
}

impl Display for Page {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Page::DomainPage => write!(f, "page_domain_manage"),
            Page::AddDomain => write!(f, "page_add_domain"),
            Page::DnsRecord => write!(f, "DnsRecord"),
            Page::AddRecord => write!(f, "{}", get_text("add_record")),
            Page::Help => write!(f, "Help"),
        }
    }
}
