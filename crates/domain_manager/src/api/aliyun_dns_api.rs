//! 阿里云DNS API实现模块
//! 
//! 实现阿里云DNS服务的完整API接口
//! 包括DNS记录的查询、创建、更新、删除等操作
//! 
//! 基于阿里云DNS API v2.0规范实现
//! API文档: https://help.aliyun.com/document_detail/29739.html

use crate::api::dns_api::*;
use crate::model::dns_record_response::{Record, Type, Status};
use crate::utils::aliyun_utils::call_api;
use anyhow::Result;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{info, error, debug};
use validator::Validate;

/// 阿里云DNS API客户端
#[derive(Debug, Clone)]
pub struct AliyunDnsApi {
    /// HTTP客户端
    client: Client,
    
    /// Access Key ID
    access_key_id: String,
    
    /// Access Key Secret
    access_key_secret: String,
    
    /// API端点
    endpoint: String,
    
    /// API版本
    version: String,
}

impl AliyunDnsApi {
    /// 创建新的阿里云DNS API客户端
    pub fn new(access_key_id: String, access_key_secret: String) -> Self {
        Self {
            client: Client::new(),
            access_key_id,
            access_key_secret,
            endpoint: "https://alidns.aliyuncs.com".to_string(),
            version: "2015-01-09".to_string(),
        }
    }
    
    /// 设置自定义端点
    pub fn with_endpoint(mut self, endpoint: String) -> Self {
        self.endpoint = endpoint;
        self
    }
    
    /// 调用阿里云API
    async fn call_aliyun_api(&self, action: &str, params: HashMap<String, String>) -> Result<Value, DnsApiError> {
        debug!("调用阿里云DNS API: {}, 参数: {:?}", action, params);
        
        match call_api(
            &self.client,
            &self.access_key_id,
            &self.access_key_secret,
            action,
            &self.version,
            params,
        ).await {
            Ok(response) => {
                debug!("API调用成功: {}", response);
                
                // 解析响应
                let json_value: Value = serde_json::from_str(&response)
                    .map_err(|e| DnsApiError::Other(format!("解析响应JSON失败: {}", e)))?;
                
                // 检查是否有错误
                if let Some(error_code) = json_value.get("Code").and_then(|v| v.as_str()) {
                    let error_message = json_value.get("Message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("未知错误");
                    
                    error!("阿里云API错误: {} - {}", error_code, error_message);
                    return Err(DnsApiUtils::parse_api_error(error_code, error_message));
                }
                
                Ok(json_value)
            },
            Err(e) => {
                error!("调用阿里云API失败: {}", e);
                Err(DnsApiError::NetworkError(e.to_string()))
            }
        }
    }
    
    /// 解析DNS记录响应
    fn parse_dns_records(&self, json_value: &Value) -> Result<Vec<Record>, DnsApiError> {
        let domain_records = json_value.get("DomainRecords")
            .and_then(|v| v.get("Record"))
            .and_then(|v| v.as_array())
            .ok_or_else(|| DnsApiError::Other("响应中缺少DNS记录数据".to_string()))?;
        
        let mut records = Vec::new();
        
        for record_value in domain_records {
            let record = self.parse_single_record(record_value)?;
            records.push(record);
        }
        
        Ok(records)
    }
    
    /// 解析单个DNS记录
    fn parse_single_record(&self, record_value: &Value) -> Result<Record, DnsApiError> {
        let record_id = record_value.get("RecordId")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        
        let rr = record_value.get("RR")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        
        let record_type_str = record_value.get("Type")
            .and_then(|v| v.as_str())
            .unwrap_or("A");
        
        let record_type = match record_type_str {
            "A" => Type::A,
            "AAAA" => Type::AAAA,
            "CNAME" => Type::Cname,
            "MX" => Type::MX,
            "TXT" => Type::TXT,
            "NS" => Type::NS,
            "SOA" => Type::SOA,
            "PTR" => Type::PTR,
            "SRV" => Type::SRV,
            "FORWARD_URL" => Type::ForwardUrl,
            _ => Type::A, // 默认为A记录
        };
        
        let value = record_value.get("Value")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        
        let ttl = record_value.get("TTL")
            .and_then(|v| v.as_u64())
            .unwrap_or(600) as i32;
        
        let priority = record_value.get("Priority")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);
        
        let line = record_value.get("Line")
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_string();
        
        let status = Status::Enable; // 阿里云DNS记录默认启用
        
        let weight = record_value.get("Weight")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);
        
        Ok(Record::new(
            status,
            rr,
            record_type,
            value,
            record_id,
            ttl,
        ))
    }
}

#[async_trait::async_trait]
impl DnsApiTrait for AliyunDnsApi {
    /// 查询DNS记录列表
    async fn query_dns_records(&self, query: DnsRecordQuery) -> Result<DnsRecordQueryResponse, DnsApiError> {
        // 验证查询参数
        query.validate()
            .map_err(|e| DnsApiError::ValidationError(format!("查询参数验证失败: {}", e)))?;
        
        info!("查询DNS记录: 域名={}, 页码={:?}, 每页={:?}", 
              query.domain_name, query.page_number, query.page_size);
        
        // 构建查询参数
        let params = DnsApiUtils::build_query_params(&query);
        
        // 调用阿里云API
        let response = self.call_aliyun_api("DescribeDomainRecords", params).await?;
        
        // 解析响应
        let total_count = response.get("TotalCount")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        
        let page_number = query.page_number.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        
        let records = self.parse_dns_records(&response)?;
        
        let request_id = response.get("RequestId")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        
        DnsApiUtils::log_operation("查询DNS记录", &query.domain_name, None, true);
        
        Ok(DnsRecordQueryResponse {
            total_count,
            page_number,
            page_size,
            records,
            request_id,
        })
    }
    
    /// 获取单个DNS记录详情
    async fn get_dns_record(&self, record_id: &str) -> Result<Record, DnsApiError> {
        if record_id.is_empty() {
            return Err(DnsApiError::ValidationError("记录ID不能为空".to_string()));
        }
        
        info!("获取DNS记录详情: 记录ID={}", record_id);
        
        let mut params = HashMap::new();
        params.insert("RecordId".to_string(), record_id.to_string());
        
        // 调用阿里云API
        let response = self.call_aliyun_api("DescribeDomainRecordInfo", params).await?;
        
        // 解析单个记录
        let record = self.parse_single_record(&response)?;
        
        DnsApiUtils::log_operation("获取DNS记录详情", "", Some(record_id), true);
        
        Ok(record)
    }
    
    /// 创建DNS记录
    async fn create_dns_record(&self, request: CreateDnsRecordRequest) -> Result<DnsRecordOperationResponse, DnsApiError> {
        // 验证请求参数
        request.validate()
            .map_err(|e| DnsApiError::ValidationError(format!("创建请求参数验证失败: {}", e)))?;
        
        // 验证主机记录格式
        DnsRecordValidator::validate_rr(&request.rr)?;
        
        // 验证记录值格式
        DnsRecordValidator::validate_record_by_type(&request.record_type, &request.value, request.priority)?;
        
        // 验证TTL值
        if let Some(ttl) = request.ttl {
            DnsRecordValidator::validate_ttl(ttl)?;
        }
        
        info!("创建DNS记录: 域名={}, 主机记录={}, 类型={:?}, 值={}", 
              request.domain_name, request.rr, request.record_type, request.value);
        
        // 构建创建参数
        let params = DnsApiUtils::build_create_params(&request);
        
        // 调用阿里云API
        let response = self.call_aliyun_api("AddDomainRecord", params).await?;
        
        // 解析响应
        let record_id = response.get("RecordId")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        
        let request_id = response.get("RequestId")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        
        DnsApiUtils::log_operation("创建DNS记录", &request.domain_name, Some(&record_id), true);
        
        Ok(DnsRecordOperationResponse {
            record_id,
            request_id,
            success: true,
            error_message: None,
        })
    }
    
    /// 更新DNS记录
    async fn update_dns_record(&self, request: UpdateDnsRecordRequest) -> Result<DnsRecordOperationResponse, DnsApiError> {
        // 验证请求参数
        request.validate()
            .map_err(|e| DnsApiError::ValidationError(format!("更新请求参数验证失败: {}", e)))?;
        
        // 验证主机记录格式
        DnsRecordValidator::validate_rr(&request.rr)?;
        
        // 验证记录值格式
        DnsRecordValidator::validate_record_by_type(&request.record_type, &request.value, request.priority)?;
        
        // 验证TTL值
        if let Some(ttl) = request.ttl {
            DnsRecordValidator::validate_ttl(ttl)?;
        }
        
        info!("更新DNS记录: 记录ID={}, 主机记录={}, 类型={:?}, 值={}", 
              request.record_id, request.rr, request.record_type, request.value);
        
        // 构建更新参数
        let params = DnsApiUtils::build_update_params(&request);
        
        // 调用阿里云API
        let response = self.call_aliyun_api("UpdateDomainRecord", params).await?;
        
        // 解析响应
        let request_id = response.get("RequestId")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        
        DnsApiUtils::log_operation("更新DNS记录", "", Some(&request.record_id), true);
        
        Ok(DnsRecordOperationResponse {
            record_id: request.record_id,
            request_id,
            success: true,
            error_message: None,
        })
    }
    
    /// 删除DNS记录
    async fn delete_dns_record(&self, request: DeleteDnsRecordRequest) -> Result<DnsRecordOperationResponse, DnsApiError> {
        // 验证请求参数
        request.validate()
            .map_err(|e| DnsApiError::ValidationError(format!("删除请求参数验证失败: {}", e)))?;
        
        info!("删除DNS记录: 记录ID={}", request.record_id);
        
        let mut params = HashMap::new();
        params.insert("RecordId".to_string(), request.record_id.clone());
        
        // 调用阿里云API
        let response = self.call_aliyun_api("DeleteDomainRecord", params).await?;
        
        // 解析响应
        let request_id = response.get("RequestId")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        
        DnsApiUtils::log_operation("删除DNS记录", "", Some(&request.record_id), true);
        
        Ok(DnsRecordOperationResponse {
            record_id: request.record_id,
            request_id,
            success: true,
            error_message: None,
        })
    }
    
    /// 批量删除DNS记录
    async fn batch_delete_dns_records(&self, request: BatchDeleteDnsRecordRequest) -> Result<BatchDnsRecordOperationResponse, DnsApiError> {
        // 验证请求参数
        request.validate()
            .map_err(|e| DnsApiError::ValidationError(format!("批量删除请求参数验证失败: {}", e)))?;
        
        info!("批量删除DNS记录: 域名={}, 主机记录={}, 类型={:?}", 
              request.domain_name, request.rr, request.record_type);
        
        let mut params = HashMap::new();
        params.insert("DomainName".to_string(), request.domain_name.clone());
        params.insert("RR".to_string(), request.rr.clone());
        
        if let Some(ref record_type) = request.record_type {
            params.insert("Type".to_string(), record_type.get_value().to_string());
        }
        
        // 调用阿里云API
        let response = self.call_aliyun_api("DeleteSubDomainRecords", params).await?;
        
        // 解析响应
        let request_id = response.get("RequestId")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        
        // 阿里云批量删除API不返回详细的操作结果，假设全部成功
        let operations = vec![DnsRecordOperationResponse {
            record_id: format!("{}.{}", request.rr, request.domain_name),
            request_id: request_id.clone(),
            success: true,
            error_message: None,
        }];
        
        DnsApiUtils::log_operation("批量删除DNS记录", &request.domain_name, None, true);
        
        Ok(BatchDnsRecordOperationResponse {
            success_count: 1,
            failed_count: 0,
            operations,
            request_id,
        })
    }
    
    /// 验证DNS记录格式
    fn validate_dns_record(&self, record_type: &Type, value: &str) -> Result<(), DnsApiError> {
        DnsRecordValidator::validate_record_by_type(record_type, value, None)
    }
}

/// 阿里云DNS API扩展功能
impl AliyunDnsApi {
    /// 查询域名列表
    pub async fn query_domain_list(&self, page_number: Option<u32>, page_size: Option<u32>) -> Result<Value, DnsApiError> {
        info!("查询域名列表: 页码={:?}, 每页={:?}", page_number, page_size);
        
        let mut params = HashMap::new();
        
        if let Some(page_number) = page_number {
            params.insert("PageNumber".to_string(), page_number.to_string());
        }
        
        if let Some(page_size) = page_size {
            params.insert("PageSize".to_string(), page_size.to_string());
        }
        
        let response = self.call_aliyun_api("DescribeDomains", params).await?;
        
        DnsApiUtils::log_operation("查询域名列表", "", None, true);
        
        Ok(response)
    }
    
    /// 查询DNS操作日志
    pub async fn query_dns_operation_logs(&self, domain_name: &str, page_number: Option<u32>, page_size: Option<u32>) -> Result<Value, DnsApiError> {
        if domain_name.is_empty() {
            return Err(DnsApiError::ValidationError("域名不能为空".to_string()));
        }
        
        info!("查询DNS操作日志: 域名={}, 页码={:?}, 每页={:?}", domain_name, page_number, page_size);
        
        let mut params = HashMap::new();
        params.insert("DomainName".to_string(), domain_name.to_string());
        
        if let Some(page_number) = page_number {
            params.insert("PageNumber".to_string(), page_number.to_string());
        }
        
        if let Some(page_size) = page_size {
            params.insert("PageSize".to_string(), page_size.to_string());
        }
        
        let response = self.call_aliyun_api("DescribeRecordLogs", params).await?;
        
        DnsApiUtils::log_operation("查询DNS操作日志", domain_name, None, true);
        
        Ok(response)
    }
    
    /// 设置DNS记录状态
    pub async fn set_dns_record_status(&self, record_id: &str, status: bool) -> Result<DnsRecordOperationResponse, DnsApiError> {
        if record_id.is_empty() {
            return Err(DnsApiError::ValidationError("记录ID不能为空".to_string()));
        }
        
        info!("设置DNS记录状态: 记录ID={}, 状态={}", record_id, status);
        
        let mut params = HashMap::new();
        params.insert("RecordId".to_string(), record_id.to_string());
        params.insert("Status".to_string(), if status { "Enable" } else { "Disable" }.to_string());
        
        let response = self.call_aliyun_api("SetDomainRecordStatus", params).await?;
        
        let request_id = response.get("RequestId")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        
        DnsApiUtils::log_operation("设置DNS记录状态", "", Some(record_id), true);
        
        Ok(DnsRecordOperationResponse {
            record_id: record_id.to_string(),
            request_id,
            success: true,
            error_message: None,
        })
    }
    
    /// 获取DNS记录解析统计
    pub async fn get_dns_record_statistics(&self, domain_name: &str, start_date: &str, end_date: &str) -> Result<Value, DnsApiError> {
        if domain_name.is_empty() {
            return Err(DnsApiError::ValidationError("域名不能为空".to_string()));
        }
        
        info!("获取DNS记录解析统计: 域名={}, 开始日期={}, 结束日期={}", domain_name, start_date, end_date);
        
        let mut params = HashMap::new();
        params.insert("DomainName".to_string(), domain_name.to_string());
        params.insert("StartDate".to_string(), start_date.to_string());
        params.insert("EndDate".to_string(), end_date.to_string());
        
        let response = self.call_aliyun_api("DescribeDomainStatistics", params).await?;
        
        DnsApiUtils::log_operation("获取DNS记录解析统计", domain_name, None, true);
        
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use super::*;
    
    fn create_test_client() -> AliyunDnsApi {
        AliyunDnsApi::new(
            "test_access_key_id".to_string(),
            "test_access_key_secret".to_string(),
        )
    }
    
    #[test]
    fn test_create_client() {
        let client = create_test_client();
        assert_eq!(client.access_key_id, "test_access_key_id");
        assert_eq!(client.access_key_secret, "test_access_key_secret");
        assert_eq!(client.endpoint, "https://alidns.aliyuncs.com");
        assert_eq!(client.version, "2015-01-09");
    }
    
    #[test]
    fn test_with_endpoint() {
        let client = create_test_client()
            .with_endpoint("https://custom.endpoint.com".to_string());
        assert_eq!(client.endpoint, "https://custom.endpoint.com");
    }
    
    #[test]
    fn test_parse_single_record() {
        let client = create_test_client();
        let record_json = json!({
            "RecordId": "12345",
            "RR": "www",
            "Type": "A",
            "Value": "192.168.1.1",
            "TTL": 600,
            "Priority": null,
            "Line": "default"
        });
        
        let record = client.parse_single_record(&record_json).unwrap();
        assert_eq!(record.record_id, "12345");
        assert_eq!(record.rr, "www");
        assert_eq!(record.value, "192.168.1.1");
        assert_eq!(record.ttl, 600);
    }
    
    #[tokio::test]
    async fn test_validate_dns_record() {
        let client = create_test_client();
        
        // 测试A记录验证
        assert!(client.validate_dns_record(&Type::A, "192.168.1.1").is_ok());
        assert!(client.validate_dns_record(&Type::A, "invalid-ip").is_err());
        
        // 测试CNAME记录验证
        assert!(client.validate_dns_record(&Type::Cname, "example.com").is_ok());
        assert!(client.validate_dns_record(&Type::Cname, "").is_err());
    }
}