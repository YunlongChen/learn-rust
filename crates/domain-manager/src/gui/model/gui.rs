use crate::gui::pages::domain::DomainProvider;
use crate::storage::{DnsRecordModal, DomainModal};

#[derive(Debug, Clone)]
pub enum ReloadType {
    Account,
    Domain,
    Record,
}

#[derive(Debug, Clone)]
pub struct ReloadModel {
    pub reload_types: Vec<ReloadType>,
    pub providers: Vec<DomainProvider>,
    pub domains: Vec<DomainModal>,
    pub records: Vec<DnsRecordModal>,
    pub message: String,
    pub total_count: usize,
}

impl ReloadModel {
    pub fn new_from(
        providers: Vec<DomainProvider>,
        domains: Vec<DomainModal>,
        dns_records: Vec<DnsRecordModal>,
        total_count: usize,
    ) -> Self {
        ReloadModel {
            reload_types: vec![],
            providers,
            domains,
            records: dns_records,
            message: "".to_string(),
            total_count,
        }
    }
}

impl Default for ReloadModel {
    fn default() -> Self {
        ReloadModel {
            reload_types: vec![],
            providers: vec![],
            domains: vec![],
            records: vec![],
            message: "".to_string(),
            total_count: 0,
        }
    }
}
