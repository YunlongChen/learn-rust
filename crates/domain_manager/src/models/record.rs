use serde::{Deserialize, Serialize};

/// 域名状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DomainStatus {
    Active,
    Expired,
    Pending,
    Suspended,
}

/// 域名模型
#[derive(Debug, Clone)]
pub struct RecordEntity {
    pub id: i64,
    pub domain_id: i64,
    pub record_name: String,
    pub record_type: String,
    pub record_value: String,
    pub ttl: i32,
}

/// 新域名创建模型
pub struct NewRecord {
    pub domain_id: i64,
    pub record_name: String,
    pub record_type: String,
    pub record_value: String,
    pub ttl: i32,
}
