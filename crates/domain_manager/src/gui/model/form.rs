#![allow(dead_code)]
use crate::gui::model::domain::DnsProvider;
use crate::model::dns_record_response::Type;
use tracing::info;

#[derive(Debug, Clone)]
pub struct AddDnsField {
    pub record_id: Option<String>,
    pub domain_name: String,
    pub record_name: String,
    pub value: String,
    pub ttl: i32,
    pub record_type: Option<Type>,
}

impl AddDnsField {
    /// update_input
    pub fn update_value(&mut self) {
        self.record_name = String::new();
    }

    pub(crate) fn validate(&self) -> bool {
        info!("验证输入是否合法！{:?}", &self);
        true
    }
}

impl Default for AddDnsField {
    fn default() -> Self {
        AddDnsField {
            record_id: None,
            record_name: String::new(),
            domain_name: String::new(),
            ttl: 600,
            record_type: Some(Type::A),
            value: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AddDomainField {
    pub domain_name: String,
    pub provider: Option<DnsProvider>,
}

impl Default for AddDomainField {
    fn default() -> Self {
        AddDomainField {
            domain_name: String::new(),
            provider: Some(DnsProvider::Aliyun),
        }
    }
}

impl AddDomainField {
    /// 清空表单字段
    pub fn clear(&mut self) {
        self.domain_name.clear();
        self.provider = Some(DnsProvider::Aliyun);
    }
}
