//! DNS API集成模块
//! 
//! 将DNS API集成到现有的域名管理系统中
//! 提供统一的DNS操作接口和错误处理
//! 
//! 主要功能：
//! - DNS API客户端管理
//! - 统一的错误处理
//! - 配置管理
//! - 日志记录

use crate::api::dns_api::*;
use crate::api::aliyun_dns_api::AliyunDnsApi;
use crate::model::dns_record_response::{Record, Type};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error, warn, debug};
use validator::Validate;
use chrono;

/// DNS服务提供商类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DnsProvider {
    /// 阿里云DNS
    Aliyun,
    /// Cloudflare DNS
    Cloudflare,
    /// 其他DNS服务商
    Other(String),
}

impl Default for DnsProvider {
    fn default() -> Self {
        DnsProvider::Aliyun
    }
}

impl std::fmt::Display for DnsProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DnsProvider::Aliyun => write!(f, "阿里云DNS"),
            DnsProvider::Cloudflare => write!(f, "Cloudflare DNS"),
            DnsProvider::Other(name) => write!(f, "{}", name),
        }
    }
}

/// DNS API配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsApiConfig {
    /// DNS服务提供商
    pub provider: DnsProvider,
    
    /// Access Key ID
    pub access_key_id: String,
    
    /// Access Key Secret
    pub access_key_secret: String,
    
    /// API端点（可选）
    pub endpoint: Option<String>,
    
    /// 超时时间（秒）
    pub timeout: Option<u64>,
    
    /// 重试次数
    pub retry_count: Option<u32>,
    
    /// 是否启用日志
    pub enable_logging: bool,
}

impl Default for DnsApiConfig {
    fn default() -> Self {
        Self {
            provider: DnsProvider::Aliyun,
            access_key_id: String::new(),
            access_key_secret: String::new(),
            endpoint: None,
            timeout: Some(30),
            retry_count: Some(3),
            enable_logging: true,
        }
    }
}

/// DNS客户端枚举
#[derive(Debug)]
pub enum DnsClient {
    Aliyun(AliyunDnsApi),
    // 可以添加其他DNS服务商
}

#[async_trait::async_trait]
impl DnsApiTrait for DnsClient {
    /// 查询DNS记录列表
    async fn query_dns_records(&self, query: DnsRecordQuery) -> Result<DnsRecordQueryResponse, DnsApiError> {
        match self {
            DnsClient::Aliyun(client) => {
                client.query_dns_records(query).await
            }
        }
    }

    /// 获取单个DNS记录详情
    async fn get_dns_record(&self, record_id: &str) -> Result<Record, DnsApiError> {
        match self {
            DnsClient::Aliyun(client) => {
                client.get_dns_record(record_id).await
            }
        }
    }

    /// 创建DNS记录
    async fn create_dns_record(&self, request: CreateDnsRecordRequest) -> Result<DnsRecordOperationResponse, DnsApiError> {
        match self {
            DnsClient::Aliyun(client) => {
                client.create_dns_record(request).await
            }
        }
    }

    /// 更新DNS记录
    async fn update_dns_record(&self, request: UpdateDnsRecordRequest) -> Result<DnsRecordOperationResponse, DnsApiError> {
        match self {
            DnsClient::Aliyun(client) => {
                client.update_dns_record(request).await
            }
        }
    }

    /// 删除DNS记录
    async fn delete_dns_record(&self, request: DeleteDnsRecordRequest) -> Result<DnsRecordOperationResponse, DnsApiError> {
        match self {
            DnsClient::Aliyun(client) => {
                client.delete_dns_record(request).await
            }
        }
    }

    /// 批量删除DNS记录
    async fn batch_delete_dns_records(&self, request: BatchDeleteDnsRecordRequest) -> Result<BatchDnsRecordOperationResponse, DnsApiError> {
        match self {
            DnsClient::Aliyun(client) => {
                client.batch_delete_dns_records(request).await
            }
        }
    }

    /// 验证DNS记录格式
    fn validate_dns_record(&self, record_type: &Type, value: &str) -> Result<(), DnsApiError> {
        match self {
            DnsClient::Aliyun(client) => {
                client.validate_dns_record(record_type, value)
            }
        }
    }
}

/// DNS API管理器
#[derive(Debug)]
pub struct DnsApiManager {
    /// 当前配置
    config: Arc<RwLock<DnsApiConfig>>,
    
    /// DNS API客户端
    client: Arc<RwLock<Option<DnsClient>>>,
}

impl DnsApiManager {
    /// 创建新的DNS API管理器
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(DnsApiConfig::default())),
            client: Arc::new(RwLock::new(None)),
        }
    }
    
    /// 使用配置创建DNS API管理器
    pub async fn with_config(config: DnsApiConfig) -> Result<Self> {
        let manager = Self {
            config: Arc::new(RwLock::new(config.clone())),
            client: Arc::new(RwLock::new(None)),
        };
        
        manager.initialize_client().await?;
        
        Ok(manager)
    }
    
    /// 更新配置
    pub async fn update_config(&self, new_config: DnsApiConfig) -> Result<()> {
        info!("更新DNS API配置: 提供商={}", new_config.provider);
        
        // 更新配置
        {
            let mut config = self.config.write().await;
            *config = new_config;
        }
        
        // 重新初始化客户端
        self.initialize_client().await?;
        
        Ok(())
    }
    
    /// 获取当前配置
    pub async fn get_config(&self) -> DnsApiConfig {
        self.config.read().await.clone()
    }
    
    /// 初始化DNS API客户端
    async fn initialize_client(&self) -> Result<()> {
        let config = self.config.read().await.clone();
        
        if config.access_key_id.is_empty() || config.access_key_secret.is_empty() {
            return Err(anyhow!("DNS API配置不完整：缺少访问密钥"));
        }
        
        let client = match config.provider {
            DnsProvider::Aliyun => {
                info!("初始化阿里云DNS客户端");
                
                let mut aliyun_client = AliyunDnsApi::new(
                    config.access_key_id.clone(),
                    config.access_key_secret.clone(),
                );
                
                if let Some(endpoint) = config.endpoint {
                    aliyun_client = aliyun_client.with_endpoint(endpoint);
                }
                
                DnsClient::Aliyun(aliyun_client)
            },
            DnsProvider::Cloudflare => {
                warn!("Cloudflare DNS客户端尚未实现");
                return Err(anyhow!("Cloudflare DNS API暂未实现"));
            },
            DnsProvider::Other(ref name) => {
                warn!("不支持的DNS服务提供商: {}", name);
                return Err(anyhow!("不支持的DNS服务提供商: {}", name));
            },
        };
        
        // 更新客户端
        {
            let mut client_guard = self.client.write().await;
            *client_guard = Some(client);
        }
        
        info!("DNS API客户端初始化成功: 提供商={}", config.provider);
        
        Ok(())
    }
    
    /// 获取DNS API客户端
    async fn get_client(&self) -> Result<Arc<RwLock<Option<DnsClient>>>, anyhow::Error> {
        let client_guard = self.client.read().await;
        if client_guard.is_none() {
            drop(client_guard);
            self.initialize_client().await?;
        }
        
        Ok(self.client.clone())
    }
    
    /// 查询DNS记录
    pub async fn query_dns_records(&self, query: DnsRecordQuery) -> Result<DnsRecordQueryResponse, DnsApiError> {
        debug!("查询DNS记录: 域名={}", query.domain_name);
        
        let client_arc = self.get_client().await
            .map_err(|e| DnsApiError::Other(format!("获取DNS客户端失败: {}", e)))?;
        
        let client_guard = client_arc.read().await;
        let client = client_guard.as_ref()
            .ok_or_else(|| DnsApiError::Other("DNS客户端未初始化".to_string()))?;
        
        client.query_dns_records(query).await
    }
    
    /// 获取DNS记录详情
    pub async fn get_dns_record(&self, record_id: &str) -> Result<Record, DnsApiError> {
        debug!("获取DNS记录详情: 记录ID={}", record_id);
        
        let client_arc = self.get_client().await
            .map_err(|e| DnsApiError::Other(format!("获取DNS客户端失败: {}", e)))?;
        
        let client_guard = client_arc.read().await;
        let client = client_guard.as_ref()
            .ok_or_else(|| DnsApiError::Other("DNS客户端未初始化".to_string()))?;
        
        client.get_dns_record(record_id).await
    }
    
    /// 创建DNS记录
    pub async fn create_dns_record(&self, request: CreateDnsRecordRequest) -> Result<DnsRecordOperationResponse, DnsApiError> {
        info!("创建DNS记录: 域名={}, 主机记录={}", request.domain_name, request.rr);
        
        let client_arc = self.get_client().await
            .map_err(|e| DnsApiError::Other(format!("获取DNS客户端失败: {}", e)))?;
        
        let client_guard = client_arc.read().await;
        let client = client_guard.as_ref()
            .ok_or_else(|| DnsApiError::Other("DNS客户端未初始化".to_string()))?;
        
        client.create_dns_record(request).await
    }
    
    /// 更新DNS记录
    pub async fn update_dns_record(&self, request: UpdateDnsRecordRequest) -> Result<DnsRecordOperationResponse, DnsApiError> {
        info!("更新DNS记录: 记录ID={}", request.record_id);
        
        let client_arc = self.get_client().await
            .map_err(|e| DnsApiError::Other(format!("获取DNS客户端失败: {}", e)))?;
        
        let client_guard = client_arc.read().await;
        let client = client_guard.as_ref()
            .ok_or_else(|| DnsApiError::Other("DNS客户端未初始化".to_string()))?;
        
        client.update_dns_record(request).await
    }
    
    /// 删除DNS记录
    pub async fn delete_dns_record(&self, request: DeleteDnsRecordRequest) -> Result<DnsRecordOperationResponse, DnsApiError> {
        info!("删除DNS记录: 记录ID={}", request.record_id);
        
        let client_arc = self.get_client().await
            .map_err(|e| DnsApiError::Other(format!("获取DNS客户端失败: {}", e)))?;
        
        let client_guard = client_arc.read().await;
        let client = client_guard.as_ref()
            .ok_or_else(|| DnsApiError::Other("DNS客户端未初始化".to_string()))?;
        
        client.delete_dns_record(request).await
    }
    
    /// 批量删除DNS记录
    pub async fn batch_delete_dns_records(&self, request: BatchDeleteDnsRecordRequest) -> Result<BatchDnsRecordOperationResponse, DnsApiError> {
        info!("批量删除DNS记录: 域名={}, 主机记录={}", request.domain_name, request.rr);
        
        let client_arc = self.get_client().await
            .map_err(|e| DnsApiError::Other(format!("获取DNS客户端失败: {}", e)))?;
        
        let client_guard = client_arc.read().await;
        let client = client_guard.as_ref()
            .ok_or_else(|| DnsApiError::Other("DNS客户端未初始化".to_string()))?;
        
        client.batch_delete_dns_records(request).await
    }
    
    /// 验证DNS记录
    pub async fn validate_dns_record(&self, record_type: &crate::model::dns_record_response::Type, value: &str) -> Result<(), DnsApiError> {
        let client_arc = self.get_client().await
            .map_err(|e| DnsApiError::Other(format!("获取DNS客户端失败: {}", e)))?;
        
        let client_guard = client_arc.read().await;
        let client = client_guard.as_ref()
            .ok_or_else(|| DnsApiError::Other("DNS客户端未初始化".to_string()))?;
        
        client.validate_dns_record(record_type, value)
    }
    
    /// 测试DNS API连接
    pub async fn test_connection(&self) -> Result<bool, DnsApiError> {
        info!("测试DNS API连接");
        
        let config = self.get_config().await;
        
        // 创建一个简单的查询来测试连接
        let test_query = DnsRecordQuery {
            domain_name: "test.example.com".to_string(),
            page_number: Some(1),
            page_size: Some(1),
            ..Default::default()
        };
        
        match self.query_dns_records(test_query).await {
            Ok(_) => {
                info!("DNS API连接测试成功");
                Ok(true)
            },
            Err(DnsApiError::DomainNotFound(_)) => {
                // 域名不存在是正常的，说明连接正常
                info!("DNS API连接测试成功（测试域名不存在）");
                Ok(true)
            },
            Err(e) => {
                error!("DNS API连接测试失败: {}", e);
                Err(e)
            }
        }
    }
    
    /// 获取支持的DNS记录类型
    pub fn get_supported_record_types(&self) -> Vec<crate::model::dns_record_response::Type> {
        vec![
            crate::model::dns_record_response::Type::A,
            crate::model::dns_record_response::Type::AAAA,
            crate::model::dns_record_response::Type::Cname,
            crate::model::dns_record_response::Type::MX,
            crate::model::dns_record_response::Type::TXT,
            crate::model::dns_record_response::Type::NS,
            crate::model::dns_record_response::Type::SOA,
            crate::model::dns_record_response::Type::PTR,
            crate::model::dns_record_response::Type::SRV,
            crate::model::dns_record_response::Type::ForwardUrl,
        ]
    }
    
    /// 获取DNS API统计信息
    pub async fn get_api_statistics(&self) -> Result<DnsApiStatistics, DnsApiError> {
        // TODO: 实现API统计信息收集
        Ok(DnsApiStatistics {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time: 0.0,
            last_request_time: None,
        })
    }
}

impl Default for DnsApiManager {
    fn default() -> Self {
        Self::new()
    }
}

/// DNS API统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsApiStatistics {
    /// 总请求数
    pub total_requests: u64,
    
    /// 成功请求数
    pub successful_requests: u64,
    
    /// 失败请求数
    pub failed_requests: u64,
    
    /// 平均响应时间（毫秒）
    pub average_response_time: f64,
    
    /// 最后请求时间
    pub last_request_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// DNS API工厂
pub struct DnsApiFactory;

impl DnsApiFactory {
    /// 创建DNS API管理器
    pub async fn create_manager(config: DnsApiConfig) -> Result<DnsApiManager> {
        DnsApiManager::with_config(config).await
    }
    
    /// 从配置文件创建DNS API管理器
    pub async fn create_from_config_file(config_path: &str) -> Result<DnsApiManager> {
        let config_content = tokio::fs::read_to_string(config_path).await
            .map_err(|e| anyhow!("读取配置文件失败: {}", e))?;
        
        let config: DnsApiConfig = toml::from_str(&config_content)
            .map_err(|e| anyhow!("解析配置文件失败: {}", e))?;
        
        Self::create_manager(config).await
    }
    
    /// 创建默认的阿里云DNS API管理器
    pub async fn create_aliyun_manager(access_key_id: String, access_key_secret: String) -> Result<DnsApiManager> {
        let config = DnsApiConfig {
            provider: DnsProvider::Aliyun,
            access_key_id,
            access_key_secret,
            ..Default::default()
        };
        
        DnsApiManager::with_config(config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dns_provider_display() {
        assert_eq!(DnsProvider::Aliyun.to_string(), "阿里云DNS");
        assert_eq!(DnsProvider::Cloudflare.to_string(), "Cloudflare DNS");
        assert_eq!(DnsProvider::Other("Custom".to_string()).to_string(), "Custom");
    }
    
    #[test]
    fn test_dns_api_config_default() {
        let config = DnsApiConfig::default();
        assert_eq!(config.provider, DnsProvider::Aliyun);
        assert_eq!(config.timeout, Some(30));
        assert_eq!(config.retry_count, Some(3));
        assert!(config.enable_logging);
    }
    
    #[tokio::test]
    async fn test_dns_api_manager_creation() {
        let manager = DnsApiManager::new();
        let config = manager.get_config().await;
        assert_eq!(config.provider, DnsProvider::Aliyun);
    }
    
    #[tokio::test]
    async fn test_dns_api_manager_config_update() {
        let manager = DnsApiManager::new();
        
        let new_config = DnsApiConfig {
            provider: DnsProvider::Cloudflare,
            access_key_id: "test_key".to_string(),
            access_key_secret: "test_secret".to_string(),
            ..Default::default()
        };
        
        // 注意：这个测试会失败，因为Cloudflare DNS API未实现
        // 但可以测试配置更新逻辑
        let result = manager.update_config(new_config.clone()).await;
        assert!(result.is_err()); // 预期失败，因为Cloudflare未实现
        
        // 验证配置已更新
        let updated_config = manager.get_config().await;
        assert_eq!(updated_config.provider, DnsProvider::Cloudflare);
        assert_eq!(updated_config.access_key_id, "test_key");
    }
    
    #[test]
    fn test_supported_record_types() {
        let manager = DnsApiManager::new();
        let types = manager.get_supported_record_types();
        assert!(!types.is_empty());
        assert!(types.contains(&crate::model::dns_record_response::Type::A));
        assert!(types.contains(&crate::model::dns_record_response::Type::Cname));
    }
}