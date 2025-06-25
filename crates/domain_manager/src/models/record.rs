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
pub struct RecordEntity {
    pub id: i64,
    pub account_id: i64,
    pub domain_id: i64,
    pub record_name: String,
    pub record_type: String,
    pub record_value: String,
    pub ttl: i64,
}

/// 新域名创建模型
pub struct NewRecord {
    pub account_id: i64,
    pub domain_id: i64,
    pub record_name: String,
    pub record_type: String,
    pub record_value: String,
    pub ttl: i64,
}
