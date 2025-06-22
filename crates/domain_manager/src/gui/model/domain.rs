use crate::gui::components::credential_form::CredentialForm;
use crate::gui::types::credential::{
    ApiKeyCredential, Credential, TokenCredential, UsernamePasswordCredential,
};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Deserialize, Serialize)]
pub enum DnsProvider {
    Aliyun,
    TencentCloud,
    CloudFlare,
    Tomato,
    Dnspod,
    Aws,
    Google,
}

impl DnsProvider {
    pub(crate) const ALL: [DnsProvider; 7] = [
        DnsProvider::Aliyun,
        DnsProvider::TencentCloud,
        DnsProvider::CloudFlare,
        DnsProvider::Tomato,
        DnsProvider::Google,
        DnsProvider::Aws,
        DnsProvider::Dnspod,
    ];
}

impl From<String> for DnsProvider {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Aliyun" => DnsProvider::Aliyun,
            "TencentCloud" => DnsProvider::TencentCloud,
            "CloudFlare" => DnsProvider::CloudFlare,
            "Tomato" => DnsProvider::Tomato,
            "Google" => DnsProvider::Google,
            "Aws" => DnsProvider::Aws,
            "Dnspod" => DnsProvider::Dnspod,
            _ => panic!("Unknown dns provider: {}", s),
        }
    }
}

impl DnsProvider {
    pub(crate) fn credential(self) -> Credential {
        match self {
            DnsProvider::Aliyun => Credential::ApiKey(ApiKeyCredential::default()),
            DnsProvider::CloudFlare => Credential::Token(TokenCredential::default()),
            DnsProvider::Aws => Credential::ApiKey(ApiKeyCredential::default()),
            DnsProvider::Google => Credential::ApiKey(ApiKeyCredential::default()),
            _ => Credential::UsernamePassword(UsernamePasswordCredential::default()),
        }
    }

    pub(crate) fn get_credential_form(&self) -> Option<Box<dyn CredentialForm>> {
        match self.credential() {
            Credential::UsernamePassword(username_password_credential) => {
                Some(Box::new(username_password_credential))
            }
            Credential::Token(token_credential) => Some(Box::new(token_credential)),
            Credential::ApiKey(api_key_credential) => Some(Box::new(api_key_credential)),
        }
    }
    pub fn value(&self) -> &str {
        match self {
            DnsProvider::CloudFlare => "CloudFlare",
            DnsProvider::Aliyun => "Aliyun",
            DnsProvider::TencentCloud => "TencentCloud",
            DnsProvider::Dnspod => "Dnspod",
            DnsProvider::Aws => "Aws",
            DnsProvider::Google => "Google",
            DnsProvider::Tomato => "Tomato",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            DnsProvider::CloudFlare => "Cloudflare",
            DnsProvider::Aliyun => "阿里云",
            DnsProvider::TencentCloud => "腾讯云",
            DnsProvider::Dnspod => "DNSPod",
            DnsProvider::Aws => "Amazon Route 53",
            DnsProvider::Google => "Google Domains",
            DnsProvider::Tomato => "Tomato DNS",
        }
    }

    pub(crate) fn icon(&self) -> char {
        match self {
            DnsProvider::CloudFlare => 'C',
            DnsProvider::Aliyun => 'A',
            DnsProvider::TencentCloud => 'T',
            DnsProvider::Dnspod => 'D',
            DnsProvider::Aws => 'S',
            DnsProvider::Google => 'G',
            DnsProvider::Tomato => 'T',
        }
    }

    pub(crate) fn features(&self) -> Vec<&str> {
        match self {
            DnsProvider::CloudFlare => vec!["安全设置", "性能优化", "DNSSEC", "刷新DNS"],
            DnsProvider::Aliyun => vec!["域名解析", "安全加速", "域名转移"],
            DnsProvider::TencentCloud => vec!["DNS管理", "安全防护", "CDN加速"],
            DnsProvider::Dnspod => vec!["域名解析", "智能解析", "安全监控"],
            DnsProvider::Aws => vec!["路由策略", "健康检查", "地理路由"],
            DnsProvider::Google => vec!["域名转移", "隐私保护", "DNS配置"],
            DnsProvider::Tomato => vec!["域名解析", "安全防护", "DNS缓存"],
        }
    }
}

impl Display for DnsProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DnsProvider::Aliyun => write!(f, "Aliyun"),
            DnsProvider::TencentCloud => write!(f, "TencentCloud"),
            DnsProvider::CloudFlare => write!(f, "CloudFlare"),
            DnsProvider::Tomato => write!(f, "Tomato"),
            DnsProvider::Dnspod => write!(f, "Dnspod"),
            DnsProvider::Aws => write!(f, "Aws"),
            DnsProvider::Google => write!(f, "Google"),
        }
    }
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct Domain {
    pub name: String,
    pub provider: DnsProvider,
    pub status: DomainStatus,
    pub expiry: String,
}

impl Default for Domain {
    fn default() -> Self {
        Self {
            name: String::new(),
            provider: DnsProvider::Aliyun,
            status: DomainStatus::Active,
            expiry: String::new(),
        }
    }
}

impl From<String> for Domain {
    fn from(value: String) -> Self {
        Self {
            name: value,
            provider: DnsProvider::Tomato,
            expiry: String::new(),
            status: DomainStatus::Active,
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
pub struct DnsRecord {
    pub name: String,
    pub record_type: String,
    pub value: String,
    pub ttl: String,
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Serialize, Eq, Deserialize)]
pub enum DomainStatus {
    Active,
    Warning,
    Suspended,
}

impl DomainStatus {
    pub(crate) fn text(&self) -> &str {
        match self {
            DomainStatus::Active => "正常",
            DomainStatus::Warning => "即将到期",
            DomainStatus::Suspended => "暂停",
        }
    }

    pub(crate) fn color(&self) -> iced::Color {
        match self {
            DomainStatus::Active => iced::Color::from_rgb(0.1, 0.8, 0.2), // 绿色
            DomainStatus::Warning => iced::Color::from_rgb(1.0, 0.6, 0.0), // 橙色
            DomainStatus::Suspended => iced::Color::from_rgb(1.0, 0.2, 0.2), // 红色
        }
    }
}

#[derive(Debug, Clone)]
struct DomainStats {
    total: usize,
    expiring: usize,
    providers: usize,
}

impl Default for DomainStats {
    fn default() -> Self {
        Self {
            total: 42,
            expiring: 5,
            providers: 6,
        }
    }
}
