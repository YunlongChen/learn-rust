use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Deserialize, Serialize)]
pub enum DnsProvider {
    Aliyun,
    TencentCloud,
    CloudFlare,
    Tomato,
}

impl DnsProvider {
    pub(crate) const ALL: [DnsProvider; 4] = [
        DnsProvider::Aliyun,
        DnsProvider::TencentCloud,
        DnsProvider::CloudFlare,
        DnsProvider::Tomato,
    ];
}

impl Display for DnsProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DnsProvider::Aliyun => write!(f, "Aliyun"),
            DnsProvider::TencentCloud => write!(f, "TencentCloud"),
            DnsProvider::CloudFlare => write!(f, "CloudFlare"),
            DnsProvider::Tomato => write!(f, "Tomato"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Deserialize, Serialize)]
pub struct DomainName {
    pub provider: DnsProvider,
    pub name: String, // e.g. example.com ,
    pub dns_record: Vec<DnsRecord>,
}

impl DomainName {
    pub(crate) fn get_domain_name(&self) -> &str {
        &self.name
    }
}

impl From<String> for DomainName {
    fn from(value: String) -> Self {
        DomainName {
            name: value,
            provider: DnsProvider::Tomato,
            dns_record: vec![],
        }
    }
}

impl Display for DomainName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Default for DomainName {
    fn default() -> Self {
        DomainName {
            name: String::from(""),
            provider: DnsProvider::Tomato,
            dns_record: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Deserialize, Serialize)]
struct DnsRecord {
    domain_name: String,
    dns_name: String,
    dns_type: String,
    dns_value: String,
    ttl: i64,
}
