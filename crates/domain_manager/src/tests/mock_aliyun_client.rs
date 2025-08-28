//! 模拟阿里云DNS客户端
//!
//! 用于测试环境中模拟阿里云DNS API的响应数据
//! 提供一致的测试数据，确保测试的可重复性和可靠性

use crate::api::dns_client::DnsClientTrait;
use crate::api::model::domain::DomainQueryResponse;
use crate::gui::model::domain::Domain;
use crate::gui::model::domain::{DnsProvider, DomainName};
use crate::model::dns_record_response::{Record, Type, Status};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::{info, debug};

/// 模拟的阿里云DNS客户端
/// 提供预定义的测试数据，用于集成测试
pub struct MockAliyunDnsClient {
    /// 模拟的域名数据
    mock_domains: HashMap<String, Vec<Domain>>,
    /// 模拟的DNS记录数据
    mock_records: HashMap<String, Vec<Record>>,
    /// 是否模拟API调用失败
    simulate_failure: bool,
}

impl MockAliyunDnsClient {
    /// 创建新的模拟客户端实例
    pub fn new() -> Self {
        let mut client = Self {
            mock_domains: HashMap::new(),
            mock_records: HashMap::new(),
            simulate_failure: false,
        };
        
        // 初始化模拟数据
        client.setup_mock_data();
        client
    }
    
    /// 创建会模拟失败的客户端实例
    pub fn new_with_failure() -> Self {
        let mut client = Self::new();
        client.simulate_failure = true;
        client
    }
    
    /// 设置模拟数据
    fn setup_mock_data(&mut self) {
        // 设置模拟域名数据 - 简化为DomainName结构
        // 注意：这里暂时使用空的Domain vec，因为Domain结构体字段较复杂
        let mock_domains = vec![];
        
        // 为测试创建简单的域名记录
        // 实际的Domain结构体包含复杂的阿里云API字段，这里简化处理
        
        self.mock_domains.insert("default".to_string(), mock_domains);
        
        // 设置example.com的DNS记录
        let example_records = vec![
            Record::new(
                Status::Enable,
                "www".to_string(),
                Type::A,
                "192.168.1.100".to_string(),
                "record_001".to_string(),
                600,
            ),
            Record::new(
                Status::Enable,
                "mail".to_string(),
                Type::A,
                "192.168.1.101".to_string(),
                "record_002".to_string(),
                600,
            ),
            Record::new(
                Status::Enable,
                "@".to_string(),
                Type::A,
                "192.168.1.102".to_string(),
                "record_003".to_string(),
                600,
            ),
            Record::new(
                Status::Enable,
                "ftp".to_string(),
                Type::Cname,
                "www.example.com".to_string(),
                "record_004".to_string(),
                1800,
            ),
            Record::new(
                Status::Enable,
                "@".to_string(),
                Type::MX,
                "10 mail.example.com".to_string(),
                "record_003".to_string(),
                3600,
            ),
        ];
        
        self.mock_records.insert("example.com".to_string(), example_records);
        
        // 设置test.com的DNS记录
        let test_records = vec![
            Record::new(
                Status::Enable,
                "www".to_string(),
                Type::A,
                "10.0.0.100".to_string(),
                "record_101".to_string(),
                300,
            ),
            Record::new(
                Status::Enable,
                "api".to_string(),
                Type::A,
                "10.0.0.101".to_string(),
                "record_102".to_string(),
                600,
            ),
            Record::new(
                Status::Enable,
                "blog".to_string(),
                Type::Cname,
                "www.test.com".to_string(),
                "record_103".to_string(),
                1200,
            ),
        ];
        
        self.mock_records.insert("test.com".to_string(), test_records);
        
        // 设置demo.org的DNS记录
        let demo_records = vec![
            Record::new(
                Status::Enable,
                "@".to_string(),
                Type::A,
                "203.0.113.100".to_string(),
                "record_201".to_string(),
                3600,
            ),
            Record::new(
                Status::Enable,
                "www".to_string(),
                Type::Cname,
                "demo.org".to_string(),
                "record_202".to_string(),
                1800,
            ),
        ];
        
        self.mock_records.insert("demo.org".to_string(), demo_records);
        
        info!("模拟阿里云DNS客户端数据初始化完成");
        debug!("模拟域名数量: {}", self.mock_domains.len());
        debug!("模拟DNS记录数量: {}", self.mock_records.len());
    }
    
    /// 添加自定义域名数据
    pub fn add_mock_domain(&mut self, account_key: &str, domain: Domain) {
        self.mock_domains
            .entry(account_key.to_string())
            .or_insert_with(Vec::new)
            .push(domain);
    }
    
    /// 添加自定义DNS记录数据
    pub fn add_mock_records(&mut self, domain_name: &str, records: Vec<Record>) {
        self.mock_records.insert(domain_name.to_string(), records);
    }
    
    /// 设置是否模拟失败
    pub fn set_simulate_failure(&mut self, simulate: bool) {
        self.simulate_failure = simulate;
    }
    
    /// 获取所有模拟域名
    pub fn get_mock_domains(&self) -> &HashMap<String, Vec<Domain>> {
        &self.mock_domains
    }
    
    /// 获取指定域名的模拟DNS记录
    pub fn get_mock_records(&self, domain_name: &str) -> Option<&Vec<Record>> {
        self.mock_records.get(domain_name)
    }
    
    /// 清除所有模拟数据
    pub fn clear_mock_data(&mut self) {
        self.mock_domains.clear();
        self.mock_records.clear();
    }
}

impl DnsClientTrait for MockAliyunDnsClient {
    /// 模拟查询域名列表
    async fn list_domains(&self, _page_num: u32, _page_size: u32) -> Result<Vec<DomainName>> {
        if self.simulate_failure {
            return Err(anyhow::anyhow!("模拟API调用失败"));
        }
        
        info!("模拟查询域名列表");
        
        // 返回模拟的DomainName列表
        let mut domain_names = vec![
            DomainName {
                name: "example.com".to_string(),
                provider: DnsProvider::Aliyun,
                ..DomainName::default()
            },
            DomainName {
                name: "test.com".to_string(),
                provider: DnsProvider::Aliyun,
                ..DomainName::default()
            },
            DomainName {
                name: "demo.org".to_string(),
                provider: DnsProvider::Aliyun,
                ..DomainName::default()
            },
        ];
        
        // 添加动态添加的域名
        for domains in self.mock_domains.values() {
            for domain in domains {
                domain_names.push(DomainName {
                    name: domain.name.clone(),
                    provider: DnsProvider::Aliyun,
                    ..DomainName::default()
                });
            }
        }
            
        debug!("返回 {} 个模拟域名", domain_names.len());
        Ok(domain_names)
    }
    
    fn query_domain(&self, _domain: &crate::gui::model::domain::Domain) -> Result<DomainQueryResponse> {
        if self.simulate_failure {
            return Err(anyhow::anyhow!("模拟API调用失败"));
        }
        todo!("Mock query_domain not implemented")
    }
    
    /// 模拟查询DNS记录列表
    async fn list_dns_records(&self, domain_name: String) -> Result<Vec<Record>> {
        if self.simulate_failure {
            return Err(anyhow::anyhow!("模拟DNS记录查询失败"));
        }
        
        info!("模拟查询域名 {} 的DNS记录", domain_name);
        
        let records = self.mock_records
            .get(&domain_name)
            .cloned()
            .unwrap_or_default();
        
        debug!("返回域名 {} 的 {} 条DNS记录", domain_name, records.len());
        Ok(records)
    }
    
    fn add_dns_record(&self, _domain_name: &DomainName, _record: &Record) -> Result<()> {
        if self.simulate_failure {
            return Err(anyhow::anyhow!("模拟API调用失败"));
        }
        info!("模拟添加DNS记录");
        Ok(())
    }
    
    fn delete_dns_record(&self, _domain_name: &DomainName, _record_id: &str) -> Result<()> {
        if self.simulate_failure {
            return Err(anyhow::anyhow!("模拟API调用失败"));
        }
        info!("模拟删除DNS记录");
        Ok(())
    }
    
    fn update_dns_record(&self, _domain_name: &DomainName, _record: &Record) -> Result<()> {
        if self.simulate_failure {
            return Err(anyhow::anyhow!("模拟API调用失败"));
        }
        info!("模拟更新DNS记录");
        Ok(())
    }
}

/// 创建默认的模拟阿里云DNS客户端
pub fn create_mock_aliyun_client() -> MockAliyunDnsClient {
    MockAliyunDnsClient::new()
}

/// 创建会失败的模拟阿里云DNS客户端
pub fn create_failing_mock_aliyun_client() -> MockAliyunDnsClient {
    MockAliyunDnsClient::new_with_failure()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mock_client_list_domains() {
        let client = create_mock_aliyun_client();
        let domains = client.list_domains(1, 10).await.unwrap();
        
        assert_eq!(domains.len(), 3);
        assert_eq!(domains[0].name, "example.com");
        assert_eq!(domains[1].name, "test.com");
        assert_eq!(domains[2].name, "demo.org");
    }
    
    #[tokio::test]
    async fn test_mock_client_list_dns_records() {
        let client = create_mock_aliyun_client();
        
        // 测试example.com的DNS记录
        let records = client.list_dns_records("example.com".to_string()).await.unwrap();
        assert_eq!(records.len(), 5);
        assert_eq!(records[0].rr, "www");
        assert_eq!(records[0].record_type, Type::A);
        assert_eq!(records[0].value, "192.168.1.100");
        
        // 测试test.com的DNS记录
        let records = client.list_dns_records("test.com".to_string()).await.unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].rr, "www");
        assert_eq!(records[0].value, "10.0.0.100");
        
        // 测试不存在的域名
        let records = client.list_dns_records("nonexistent.com".to_string()).await.unwrap();
        assert_eq!(records.len(), 0);
    }
    
    #[tokio::test]
    async fn test_mock_client_failure_simulation() {
        let client = create_failing_mock_aliyun_client();
        
        // 测试域名查询失败
        let result = client.list_domains(1, 10).await;
        assert!(result.is_err());
        
        // 测试DNS记录查询失败
        let result = client.list_dns_records("example.com".to_string()).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_mock_client_custom_data() {
        let mut client = create_mock_aliyun_client();
        
        // 添加自定义域名
        let custom_domain = Domain {
            id: Some(1),
            name: "custom.com".to_string(),
            provider: DnsProvider::Aliyun,
            status: crate::gui::model::domain::DomainStatus::Active,
            expiry: "2025-04-01".to_string(),
            records: vec![],
        };
        
        client.add_mock_domain("default", custom_domain);
        
        // 添加自定义DNS记录
        let custom_records = vec![
            Record::new(
                Status::Enable,
                "www".to_string(),
                Type::A,
                "1.2.3.4".to_string(),
                "custom_record_001".to_string(),
                300,
            ),
        ];
        
        client.add_mock_records("custom.com", custom_records);
        
        // 验证自定义数据
        let domains = client.list_domains(1, 10).await.unwrap();
        assert_eq!(domains.len(), 4); // 原来3个 + 新增1个
        
        let records = client.list_dns_records("custom.com".to_string()).await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].value, "1.2.3.4");
    }
}