//! DNS记录管理API模块
//!
//! 提供完整的DNS记录查询、添加、更新、删除功能
//! 支持阿里云DNS API，可扩展支持其他DNS服务商
//!
//! 主要功能：
//! - DNS记录查询（支持分页、过滤）
//! - DNS记录添加
//! - DNS记录更新
//! - DNS记录删除
//! - DNS记录批量操作
//! - 错误处理和验证

use crate::model::dns_record_response::{Record, Status, Type};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info};
use validator::Validate;

/// DNS记录查询参数
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DnsRecordQuery {
    /// 域名名称（必填）
    #[validate(length(min = 1, message = "域名名称不能为空"))]
    pub domain_name: String,

    /// 页码，从1开始
    #[validate(range(min = 1, message = "页码必须大于0"))]
    pub page_number: Option<u32>,

    /// 每页记录数，默认20，最大500
    #[validate(range(min = 1, max = 500, message = "每页记录数必须在1-500之间"))]
    pub page_size: Option<u32>,

    /// 主机记录关键字（模糊匹配）
    pub rr_keyword: Option<String>,

    /// 记录类型关键字
    pub type_keyword: Option<String>,

    /// 记录值关键字（模糊匹配）
    pub value_keyword: Option<String>,

    /// 记录类型过滤
    pub record_type: Option<Type>,

    /// 记录状态过滤
    pub status: Option<Status>,

    /// 排序方式：default（按创建时间）
    pub order_by: Option<String>,

    /// 排序方向：DESC（降序）、ASC（升序）
    pub direction: Option<String>,
}

impl Default for DnsRecordQuery {
    fn default() -> Self {
        Self {
            domain_name: String::new(),
            page_number: Some(1),
            page_size: Some(20),
            rr_keyword: None,
            type_keyword: None,
            value_keyword: None,
            record_type: None,
            status: None,
            order_by: Some("default".to_string()),
            direction: Some("DESC".to_string()),
        }
    }
}

/// DNS记录创建请求
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateDnsRecordRequest {
    /// 域名名称
    #[validate(length(min = 1, message = "域名名称不能为空"))]
    pub domain_name: String,

    /// 主机记录
    #[validate(length(min = 1, message = "主机记录不能为空"))]
    pub rr: String,

    /// 记录类型
    pub record_type: Type,

    /// 记录值
    #[validate(length(min = 1, message = "记录值不能为空"))]
    pub value: String,

    /// TTL值，默认600秒
    #[validate(range(min = 1, message = "TTL值必须大于0"))]
    pub ttl: Option<u32>,

    /// MX记录优先级（仅MX记录需要）
    #[validate(range(min = 1, max = 50, message = "MX优先级必须在1-50之间"))]
    pub priority: Option<u32>,

    /// 解析线路，默认default
    pub line: Option<String>,
}

impl Default for CreateDnsRecordRequest {
    fn default() -> Self {
        Self {
            domain_name: String::new(),
            rr: String::new(),
            record_type: Type::A,
            value: String::new(),
            ttl: Some(600),
            priority: None,
            line: Some("default".to_string()),
        }
    }
}

/// DNS记录更新请求
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateDnsRecordRequest {
    /// 记录ID
    #[validate(length(min = 1, message = "记录ID不能为空"))]
    pub record_id: String,

    /// 主机记录
    #[validate(length(min = 1, message = "主机记录不能为空"))]
    pub rr: String,

    /// 记录类型
    pub record_type: Type,

    /// 记录值
    #[validate(length(min = 1, message = "记录值不能为空"))]
    pub value: String,

    /// TTL值
    #[validate(range(min = 1, message = "TTL值必须大于0"))]
    pub ttl: Option<u32>,

    /// MX记录优先级
    #[validate(range(min = 1, max = 50, message = "MX优先级必须在1-50之间"))]
    pub priority: Option<u32>,

    /// 解析线路
    pub line: Option<String>,
}

/// DNS记录删除请求
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DeleteDnsRecordRequest {
    /// 记录ID
    #[validate(length(min = 1, message = "记录ID不能为空"))]
    pub record_id: String,
}

/// DNS记录批量删除请求
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct BatchDeleteDnsRecordRequest {
    /// 域名名称
    #[validate(length(min = 1, message = "域名名称不能为空"))]
    pub domain_name: String,

    /// 主机记录
    #[validate(length(min = 1, message = "主机记录不能为空"))]
    pub rr: String,

    /// 记录类型（可选，不填则删除该主机记录的所有类型）
    pub record_type: Option<Type>,
}

/// DNS记录查询响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecordQueryResponse {
    /// 总记录数
    pub total_count: u32,

    /// 当前页码
    pub page_number: u32,

    /// 每页记录数
    pub page_size: u32,

    /// DNS记录列表
    pub records: Vec<Record>,

    /// 请求ID
    pub request_id: String,
}

/// DNS记录操作响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecordOperationResponse {
    /// 记录ID
    pub record_id: String,

    /// 请求ID
    pub request_id: String,

    /// 操作是否成功
    pub success: bool,

    /// 错误信息（如果有）
    pub error_message: Option<String>,
}

/// DNS记录批量操作响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchDnsRecordOperationResponse {
    /// 成功操作的记录数
    pub success_count: u32,

    /// 失败操作的记录数
    pub failed_count: u32,

    /// 操作详情
    pub operations: Vec<DnsRecordOperationResponse>,

    /// 请求ID
    pub request_id: String,
}

/// DNS API错误类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DnsApiError {
    /// 验证错误
    ValidationError(String),

    /// 网络错误
    NetworkError(String),

    /// API错误
    ApiError { code: String, message: String },

    /// 记录不存在
    RecordNotFound(String),

    /// 域名不存在
    DomainNotFound(String),

    /// 权限不足
    PermissionDenied(String),

    /// 配额超限
    QuotaExceeded(String),

    /// 其他错误
    Other(String),
}

impl std::fmt::Display for DnsApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DnsApiError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            DnsApiError::NetworkError(msg) => write!(f, "网络错误: {}", msg),
            DnsApiError::ApiError { code, message } => write!(f, "API错误 [{}]: {}", code, message),
            DnsApiError::RecordNotFound(id) => write!(f, "DNS记录不存在: {}", id),
            DnsApiError::DomainNotFound(domain) => write!(f, "域名不存在: {}", domain),
            DnsApiError::PermissionDenied(msg) => write!(f, "权限不足: {}", msg),
            DnsApiError::QuotaExceeded(msg) => write!(f, "配额超限: {}", msg),
            DnsApiError::Other(msg) => write!(f, "其他错误: {}", msg),
        }
    }
}

impl std::error::Error for DnsApiError {}

/// DNS API 接口定义
#[async_trait]
pub trait DnsApiTrait: Send + Sync {
    /// 查询DNS记录列表
    async fn query_dns_records(
        &self,
        query: DnsRecordQuery,
    ) -> Result<DnsRecordQueryResponse, DnsApiError>;

    /// 获取单个DNS记录详情
    async fn get_dns_record(&self, record_id: &str) -> Result<Record, DnsApiError>;

    /// 创建DNS记录
    async fn create_dns_record(
        &self,
        request: CreateDnsRecordRequest,
    ) -> Result<DnsRecordOperationResponse, DnsApiError>;

    /// 更新DNS记录
    async fn update_dns_record(
        &self,
        request: UpdateDnsRecordRequest,
    ) -> Result<DnsRecordOperationResponse, DnsApiError>;

    /// 删除DNS记录
    async fn delete_dns_record(
        &self,
        request: DeleteDnsRecordRequest,
    ) -> Result<DnsRecordOperationResponse, DnsApiError>;

    /// 批量删除DNS记录
    async fn batch_delete_dns_records(
        &self,
        request: BatchDeleteDnsRecordRequest,
    ) -> Result<BatchDnsRecordOperationResponse, DnsApiError>;

    /// 验证DNS记录格式
    fn validate_dns_record(&self, record_type: &Type, value: &str) -> Result<(), DnsApiError>;
}

/// DNS记录验证工具
pub struct DnsRecordValidator;

impl DnsRecordValidator {
    /// 验证A记录
    pub fn validate_a_record(value: &str) -> Result<(), DnsApiError> {
        use std::net::Ipv4Addr;
        value
            .parse::<Ipv4Addr>()
            .map_err(|_| DnsApiError::ValidationError(format!("无效的IPv4地址: {}", value)))?;
        Ok(())
    }

    /// 验证AAAA记录
    pub fn validate_aaaa_record(value: &str) -> Result<(), DnsApiError> {
        use std::net::Ipv6Addr;
        value
            .parse::<Ipv6Addr>()
            .map_err(|_| DnsApiError::ValidationError(format!("无效的IPv6地址: {}", value)))?;
        Ok(())
    }

    /// 验证CNAME记录
    pub fn validate_cname_record(value: &str) -> Result<(), DnsApiError> {
        if value.is_empty() {
            return Err(DnsApiError::ValidationError(
                "CNAME记录值不能为空".to_string(),
            ));
        }

        // 简单的域名格式验证
        if !value
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '-')
        {
            return Err(DnsApiError::ValidationError(format!(
                "无效的CNAME记录值: {}",
                value
            )));
        }

        Ok(())
    }

    /// 验证MX记录
    pub fn validate_mx_record(value: &str, priority: Option<u32>) -> Result<(), DnsApiError> {
        if value.is_empty() {
            return Err(DnsApiError::ValidationError("MX记录值不能为空".to_string()));
        }

        if priority.is_none() {
            return Err(DnsApiError::ValidationError(
                "MX记录必须指定优先级".to_string(),
            ));
        }

        let priority = priority.unwrap();
        if !(1..=50).contains(&priority) {
            return Err(DnsApiError::ValidationError(
                "MX记录优先级必须在1-50之间".to_string(),
            ));
        }

        // 验证邮件服务器域名格式
        Self::validate_cname_record(value)?;

        Ok(())
    }

    /// 验证TXT记录
    pub fn validate_txt_record(value: &str) -> Result<(), DnsApiError> {
        if value.is_empty() {
            return Err(DnsApiError::ValidationError(
                "TXT记录值不能为空".to_string(),
            ));
        }

        // TXT记录长度限制（通常为255字符）
        if value.len() > 255 {
            return Err(DnsApiError::ValidationError(
                "TXT记录值长度不能超过255字符".to_string(),
            ));
        }

        Ok(())
    }

    /// 验证主机记录格式
    pub fn validate_rr(rr: &str) -> Result<(), DnsApiError> {
        if rr.is_empty() {
            return Err(DnsApiError::ValidationError("主机记录不能为空".to_string()));
        }

        // 主机记录格式验证
        if rr != "@"
            && !rr
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(DnsApiError::ValidationError(format!(
                "无效的主机记录格式: {}",
                rr
            )));
        }

        Ok(())
    }

    /// 验证TTL值
    pub fn validate_ttl(ttl: u32) -> Result<(), DnsApiError> {
        // TTL值通常在1-86400秒之间（1秒到1天）
        if !(1..=86400).contains(&ttl) {
            return Err(DnsApiError::ValidationError(
                "TTL值必须在1-86400秒之间".to_string(),
            ));
        }

        Ok(())
    }

    /// 根据记录类型验证记录值
    pub fn validate_record_by_type(
        record_type: &Type,
        value: &str,
        priority: Option<u32>,
    ) -> Result<(), DnsApiError> {
        match record_type {
            Type::A => Self::validate_a_record(value),
            Type::AAAA => Self::validate_aaaa_record(value),
            Type::Cname => Self::validate_cname_record(value),
            Type::MX => Self::validate_mx_record(value, priority),
            Type::TXT => Self::validate_txt_record(value),
            Type::NS => Self::validate_cname_record(value), // NS记录格式类似CNAME
            Type::SOA => Ok(()),                            // SOA记录格式复杂，暂时跳过验证
            Type::PTR => Self::validate_cname_record(value), // PTR记录格式类似CNAME
            Type::SRV => Ok(()),                            // SRV记录格式复杂，暂时跳过验证
            Type::ForwardUrl => Ok(()),                     // URL转发记录，暂时跳过验证
        }
    }
}

/// DNS API工具函数
pub struct DnsApiUtils;

impl DnsApiUtils {
    /// 构建查询参数
    pub fn build_query_params(query: &DnsRecordQuery) -> HashMap<String, String> {
        let mut params = HashMap::new();

        params.insert("DomainName".to_string(), query.domain_name.clone());

        if let Some(page_number) = query.page_number {
            params.insert("PageNumber".to_string(), page_number.to_string());
        }

        if let Some(page_size) = query.page_size {
            params.insert("PageSize".to_string(), page_size.to_string());
        }

        if let Some(ref rr_keyword) = query.rr_keyword {
            params.insert("RRKeyWord".to_string(), rr_keyword.clone());
        }

        if let Some(ref type_keyword) = query.type_keyword {
            params.insert("TypeKeyWord".to_string(), type_keyword.clone());
        }

        if let Some(ref value_keyword) = query.value_keyword {
            params.insert("ValueKeyWord".to_string(), value_keyword.clone());
        }

        if let Some(ref record_type) = query.record_type {
            params.insert("Type".to_string(), record_type.get_value().to_string());
        }

        if let Some(ref status) = query.status {
            params.insert(
                "Status".to_string(),
                match status {
                    Status::Enable => "Enable".to_string(),
                },
            );
        }

        if let Some(ref order_by) = query.order_by {
            params.insert("OrderBy".to_string(), order_by.clone());
        }

        if let Some(ref direction) = query.direction {
            params.insert("Direction".to_string(), direction.clone());
        }

        params
    }

    /// 构建创建记录参数
    pub fn build_create_params(request: &CreateDnsRecordRequest) -> HashMap<String, String> {
        let mut params = HashMap::new();

        params.insert("DomainName".to_string(), request.domain_name.clone());
        params.insert("RR".to_string(), request.rr.clone());
        params.insert(
            "Type".to_string(),
            request.record_type.get_value().to_string(),
        );
        params.insert("Value".to_string(), request.value.clone());

        if let Some(ttl) = request.ttl {
            params.insert("TTL".to_string(), ttl.to_string());
        }

        if let Some(priority) = request.priority {
            params.insert("Priority".to_string(), priority.to_string());
        }

        if let Some(ref line) = request.line {
            params.insert("Line".to_string(), line.clone());
        }

        params
    }

    /// 构建更新记录参数
    pub fn build_update_params(request: &UpdateDnsRecordRequest) -> HashMap<String, String> {
        let mut params = HashMap::new();

        params.insert("RecordId".to_string(), request.record_id.clone());
        params.insert("RR".to_string(), request.rr.clone());
        params.insert(
            "Type".to_string(),
            request.record_type.get_value().to_string(),
        );
        params.insert("Value".to_string(), request.value.clone());

        if let Some(ttl) = request.ttl {
            params.insert("TTL".to_string(), ttl.to_string());
        }

        if let Some(priority) = request.priority {
            params.insert("Priority".to_string(), priority.to_string());
        }

        if let Some(ref line) = request.line {
            params.insert("Line".to_string(), line.clone());
        }

        params
    }

    /// 解析API错误响应
    pub fn parse_api_error(error_code: &str, error_message: &str) -> DnsApiError {
        match error_code {
            "InvalidDomainName.NoExist" => DnsApiError::DomainNotFound(error_message.to_string()),
            "InvalidRecordId.NotExist" => DnsApiError::RecordNotFound(error_message.to_string()),
            "Forbidden.RAM" | "Forbidden.Risk" => {
                DnsApiError::PermissionDenied(error_message.to_string())
            }
            "QuotaExceeded.DomainRecord" => DnsApiError::QuotaExceeded(error_message.to_string()),
            _ => DnsApiError::ApiError {
                code: error_code.to_string(),
                message: error_message.to_string(),
            },
        }
    }

    /// 记录操作日志
    pub fn log_operation(operation: &str, domain: &str, record_id: Option<&str>, success: bool) {
        if success {
            info!(
                "DNS操作成功: {} - 域名: {}, 记录ID: {:?}",
                operation, domain, record_id
            );
        } else {
            error!(
                "DNS操作失败: {} - 域名: {}, 记录ID: {:?}",
                operation, domain, record_id
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_a_record() {
        assert!(DnsRecordValidator::validate_a_record("192.168.1.1").is_ok());
        assert!(DnsRecordValidator::validate_a_record("invalid-ip").is_err());
    }

    #[test]
    fn test_validate_cname_record() {
        assert!(DnsRecordValidator::validate_cname_record("example.com").is_ok());
        assert!(DnsRecordValidator::validate_cname_record("sub.example.com").is_ok());
        assert!(DnsRecordValidator::validate_cname_record("").is_err());
    }

    #[test]
    fn test_validate_mx_record() {
        assert!(DnsRecordValidator::validate_mx_record("mail.example.com", Some(10)).is_ok());
        assert!(DnsRecordValidator::validate_mx_record("mail.example.com", None).is_err());
        assert!(DnsRecordValidator::validate_mx_record("mail.example.com", Some(100)).is_err());
    }

    #[test]
    fn test_validate_rr() {
        assert!(DnsRecordValidator::validate_rr("@").is_ok());
        assert!(DnsRecordValidator::validate_rr("www").is_ok());
        assert!(DnsRecordValidator::validate_rr("api-v1").is_ok());
        assert!(DnsRecordValidator::validate_rr("").is_err());
    }

    #[test]
    fn test_validate_ttl() {
        assert!(DnsRecordValidator::validate_ttl(600).is_ok());
        assert!(DnsRecordValidator::validate_ttl(0).is_err());
        assert!(DnsRecordValidator::validate_ttl(100000).is_err());
    }
}
