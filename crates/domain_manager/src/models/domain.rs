use sea_orm::prelude::DateTime;
use crate::models::account::Account;

/// 域名状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainStatus {
    Active,
    Expired,
    Pending,
    Suspended,
}

impl DomainStatus {
    pub fn to_string(&self) -> &'static str {
        match self {
            DomainStatus::Active => "active",
            DomainStatus::Expired => "expired",
            DomainStatus::Pending => "pending",
            DomainStatus::Suspended => "suspended",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "active" => Some(DomainStatus::Active),
            "expired" => Some(DomainStatus::Expired),
            "pending" => Some(DomainStatus::Pending),
            "suspended" => Some(DomainStatus::Suspended),
            _ => None,
        }
    }
}

/// 域名模型
#[derive(Debug, Clone)]
pub struct DomainEntity {
    pub id: i64,
    pub account_id: i64,
    pub domain_name: String,
    pub registration_date: Option<String>,
    pub expiration_date: Option<String>,
    pub registrar: Option<String>,
    pub status: DomainStatus,
    pub created_at: String,
    pub updated_at: Option<DateTime>,
}

/// 新域名创建模型
pub struct NewDomain {
    pub domain_name: String,
    pub registration_date: Option<String>,
    pub expiration_date: Option<String>,
    pub registrar: Option<String>,
    pub status: DomainStatus,
    pub account_id: i64
}
