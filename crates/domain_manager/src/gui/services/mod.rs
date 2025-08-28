//! 业务服务模块
//! 
//! 提供应用程序的核心业务逻辑服务，包括域名管理、DNS操作、
//! 数据同步等功能。这些服务独立于UI层，可以被不同的处理器调用。

pub mod domain_service;
pub mod dns_service;
pub mod sync_service;
pub mod database_service;
pub mod config_service;

use crate::models::{Domain, DnsRecord, DomainProvider};
use std::collections::HashMap;
use async_trait::async_trait;

/// 服务操作结果
#[derive(Debug, Clone)]
pub enum ServiceResult<T> {
    /// 操作成功
    Success(T),
    /// 操作失败
    Error(String),
    /// 操作被取消
    Cancelled,
}

impl<T> ServiceResult<T> {
    /// 检查是否成功
    pub fn is_success(&self) -> bool {
        matches!(self, ServiceResult::Success(_))
    }
    
    /// 检查是否失败
    pub fn is_error(&self) -> bool {
        matches!(self, ServiceResult::Error(_))
    }
    
    /// 检查是否被取消
    pub fn is_cancelled(&self) -> bool {
        matches!(self, ServiceResult::Cancelled)
    }
    
    /// 获取成功结果
    pub fn success(self) -> Option<T> {
        match self {
            ServiceResult::Success(value) => Some(value),
            _ => None,
        }
    }
    
    /// 获取错误信息
    pub fn error(self) -> Option<String> {
        match self {
            ServiceResult::Error(msg) => Some(msg),
            _ => None,
        }
    }
    
    /// 将结果转换为标准Result
    pub fn into_result(self) -> Result<T, String> {
        match self {
            ServiceResult::Success(value) => Ok(value),
            ServiceResult::Error(msg) => Err(msg),
            ServiceResult::Cancelled => Err("操作被取消".to_string()),
        }
    }
    
    /// 从标准Result创建ServiceResult
    pub fn from_result(result: Result<T, String>) -> Self {
        match result {
            Ok(value) => ServiceResult::Success(value),
            Err(msg) => ServiceResult::Error(msg),
        }
    }
}

/// 异步服务特征
/// 
/// 定义所有业务服务的通用接口
#[async_trait]
pub trait AsyncService {
    /// 服务名称
    fn name(&self) -> &'static str;
    
    /// 初始化服务
    async fn initialize(&mut self) -> ServiceResult<()>;
    
    /// 关闭服务
    async fn shutdown(&mut self) -> ServiceResult<()>;
    
    /// 检查服务健康状态
    async fn health_check(&self) -> ServiceResult<bool>;
}

/// 域名服务特征
#[async_trait]
pub trait DomainServiceTrait: AsyncService {
    /// 获取所有域名
    async fn get_all_domains(&self) -> ServiceResult<Vec<Domain>>;
    
    /// 根据ID获取域名
    async fn get_domain_by_id(&self, domain_id: &str) -> ServiceResult<Option<Domain>>;
    
    /// 根据名称获取域名
    async fn get_domain_by_name(&self, domain_name: &str) -> ServiceResult<Option<Domain>>;
    
    /// 添加域名
    async fn add_domain(&self, domain: Domain) -> ServiceResult<Domain>;
    
    /// 更新域名
    async fn update_domain(&self, domain: Domain) -> ServiceResult<Domain>;
    
    /// 删除域名
    async fn delete_domain(&self, domain_id: &str) -> ServiceResult<()>;
    
    /// 搜索域名
    async fn search_domains(&self, query: &str) -> ServiceResult<Vec<Domain>>;
}

/// DNS服务特征
#[async_trait]
pub trait DnsServiceTrait: AsyncService {
    /// 查询域名的DNS记录
    async fn query_dns_records(&self, domain: &str) -> ServiceResult<Vec<DnsRecord>>;
    
    /// 添加DNS记录
    async fn add_dns_record(&self, record: DnsRecord) -> ServiceResult<DnsRecord>;
    
    /// 更新DNS记录
    async fn update_dns_record(&self, record: DnsRecord) -> ServiceResult<DnsRecord>;
    
    /// 删除DNS记录
    async fn delete_dns_record(&self, domain: &str, record_id: &str) -> ServiceResult<()>;
    
    /// 批量操作DNS记录
    async fn batch_dns_operations(&self, operations: Vec<DnsOperation>) -> ServiceResult<Vec<DnsOperationResult>>;
}

/// 同步服务特征
#[async_trait]
pub trait SyncServiceTrait: AsyncService {
    /// 同步单个域名
    async fn sync_domain(&self, domain: &str) -> ServiceResult<Vec<DnsRecord>>;
    
    /// 同步所有域名
    async fn sync_all_domains(&self, domains: Vec<String>) -> ServiceResult<HashMap<String, Vec<DnsRecord>>>;
    
    /// 检查同步状态
    async fn get_sync_status(&self, domain: &str) -> ServiceResult<SyncStatus>;
    
    /// 取消同步
    async fn cancel_sync(&self, domain: Option<String>) -> ServiceResult<()>;
}

/// 数据库服务特征
#[async_trait]
pub trait DatabaseServiceTrait: AsyncService {
    /// 保存域名
    async fn save_domain(&self, domain: &Domain) -> ServiceResult<()>;
    
    /// 保存DNS记录
    async fn save_dns_records(&self, domain: &str, records: &[DnsRecord]) -> ServiceResult<()>;
    
    /// 加载域名列表
    async fn load_domains(&self) -> ServiceResult<Vec<Domain>>;
    
    /// 加载DNS记录
    async fn load_dns_records(&self, domain: &str) -> ServiceResult<Vec<DnsRecord>>;
    
    /// 删除域名数据
    async fn delete_domain_data(&self, domain: &str) -> ServiceResult<()>;
    
    /// 清理过期数据
    async fn cleanup_expired_data(&self) -> ServiceResult<usize>;
}

/// 配置服务特征
#[async_trait]
pub trait ConfigServiceTrait: AsyncService {
    /// 获取配置值
    async fn get_config<T>(&self, key: &str) -> ServiceResult<Option<T>>
    where
        T: serde::de::DeserializeOwned + Send;
    
    /// 设置配置值
    async fn set_config<T>(&self, key: &str, value: &T) -> ServiceResult<()>
    where
        T: serde::Serialize + Send + Sync;
    
    /// 删除配置
    async fn remove_config(&self, key: &str) -> ServiceResult<()>;
    
    /// 获取所有配置
    async fn get_all_configs(&self) -> ServiceResult<HashMap<String, serde_json::Value>>;
    
    /// 保存配置到文件
    async fn save_to_file(&self, path: &str) -> ServiceResult<()>;
    
    /// 从文件加载配置
    async fn load_from_file(&self, path: &str) -> ServiceResult<()>;
}

/// DNS操作类型
#[derive(Debug, Clone)]
pub enum DnsOperation {
    Add(DnsRecord),
    Update(DnsRecord),
    Delete { domain: String, record_id: String },
}

/// DNS操作结果
#[derive(Debug, Clone)]
pub enum DnsOperationResult {
    Added(DnsRecord),
    Updated(DnsRecord),
    Deleted { domain: String, record_id: String },
    Failed { operation: String, error: String },
}

/// 同步状态
#[derive(Debug, Clone, PartialEq)]
pub enum SyncStatus {
    /// 空闲状态
    Idle,
    /// 同步中
    Syncing,
    /// 同步完成
    Completed,
    /// 同步失败
    Failed(String),
    /// 同步被取消
    Cancelled,
}

/// 服务管理器
/// 
/// 管理所有业务服务的生命周期
pub struct ServiceManager {
    domain_service: Box<dyn DomainServiceTrait + Send + Sync>,
    dns_service: Box<dyn DnsServiceTrait + Send + Sync>,
    sync_service: Box<dyn SyncServiceTrait + Send + Sync>,
    database_service: Box<dyn DatabaseServiceTrait + Send + Sync>,
    config_service: Box<dyn ConfigServiceTrait + Send + Sync>,
}

impl ServiceManager {
    /// 创建新的服务管理器
    pub fn new(
        domain_service: Box<dyn DomainServiceTrait + Send + Sync>,
        dns_service: Box<dyn DnsServiceTrait + Send + Sync>,
        sync_service: Box<dyn SyncServiceTrait + Send + Sync>,
        database_service: Box<dyn DatabaseServiceTrait + Send + Sync>,
        config_service: Box<dyn ConfigServiceTrait + Send + Sync>,
    ) -> Self {
        Self {
            domain_service,
            dns_service,
            sync_service,
            database_service,
            config_service,
        }
    }
    
    /// 获取域名服务
    pub fn domain_service(&self) -> &dyn DomainServiceTrait {
        self.domain_service.as_ref()
    }
    
    /// 获取DNS服务
    pub fn dns_service(&self) -> &dyn DnsServiceTrait {
        self.dns_service.as_ref()
    }
    
    /// 获取同步服务
    pub fn sync_service(&self) -> &dyn SyncServiceTrait {
        self.sync_service.as_ref()
    }
    
    /// 获取数据库服务
    pub fn database_service(&self) -> &dyn DatabaseServiceTrait {
        self.database_service.as_ref()
    }
    
    /// 获取配置服务
    pub fn config_service(&self) -> &dyn ConfigServiceTrait {
        self.config_service.as_ref()
    }
    
    /// 初始化所有服务
    pub async fn initialize_all(&mut self) -> ServiceResult<()> {
        let services = [
            ("domain", self.domain_service.as_mut()),
            ("dns", self.dns_service.as_mut()),
            ("sync", self.sync_service.as_mut()),
            ("database", self.database_service.as_mut()),
            ("config", self.config_service.as_mut()),
        ];
        
        for (name, service) in services {
            match service.initialize().await {
                ServiceResult::Success(_) => {
                    println!("服务 {} 初始化成功", name);
                },
                ServiceResult::Error(e) => {
                    return ServiceResult::Error(format!("服务 {} 初始化失败: {}", name, e));
                },
                ServiceResult::Cancelled => {
                    return ServiceResult::Error(format!("服务 {} 初始化被取消", name));
                }
            }
        }
        
        ServiceResult::Success(())
    }
    
    /// 关闭所有服务
    pub async fn shutdown_all(&mut self) -> ServiceResult<()> {
        let services = [
            ("config", self.config_service.as_mut()),
            ("database", self.database_service.as_mut()),
            ("sync", self.sync_service.as_mut()),
            ("dns", self.dns_service.as_mut()),
            ("domain", self.domain_service.as_mut()),
        ];
        
        let mut errors = Vec::new();
        
        for (name, service) in services {
            match service.shutdown().await {
                ServiceResult::Success(_) => {
                    println!("服务 {} 关闭成功", name);
                },
                ServiceResult::Error(e) => {
                    errors.push(format!("服务 {} 关闭失败: {}", name, e));
                },
                ServiceResult::Cancelled => {
                    errors.push(format!("服务 {} 关闭被取消", name));
                }
            }
        }
        
        if errors.is_empty() {
            ServiceResult::Success(())
        } else {
            ServiceResult::Error(errors.join("; "))
        }
    }
    
    /// 检查所有服务的健康状态
    pub async fn health_check_all(&self) -> ServiceResult<HashMap<String, bool>> {
        let mut results = HashMap::new();
        
        let services = [
            ("domain", self.domain_service.as_ref()),
            ("dns", self.dns_service.as_ref()),
            ("sync", self.sync_service.as_ref()),
            ("database", self.database_service.as_ref()),
            ("config", self.config_service.as_ref()),
        ];
        
        for (name, service) in services {
            match service.health_check().await {
                ServiceResult::Success(healthy) => {
                    results.insert(name.to_string(), healthy);
                },
                _ => {
                    results.insert(name.to_string(), false);
                }
            }
        }
        
        ServiceResult::Success(results)
    }
}