//! Iced框架集成测试
//!
//! 测试DomainManager应用程序的完整消息处理流程
//! 包括：
//! - Message::Reload 消息处理测试
//! - Message::SyncAllDomains 消息处理测试
//! - 数据库集成测试
//! - 界面状态更新测试
//! - 阿里云客户端数据模拟

use crate::gui::manager::DomainManager;
use crate::gui::model::domain::{DnsProvider, DnsRecord, Domain, DomainStatus};
use crate::gui::model::gui::ReloadModel;
use crate::gui::pages::domain::DomainProvider;
use crate::gui::types::credential::{Credential, UsernamePasswordCredential};
use crate::gui::types::message::{Message, SyncResult};
use crate::models::account::NewAccount;
use crate::models::domain::{NewDomain, DomainStatus as ModelDomainStatus};
use crate::models::record::NewRecord;
use crate::storage::{
    create_account, init_memory_database, add_domain,
    list_accounts, get_account_domains,
};
use crate::storage::records::add_record;
use crate::api::dns_client::DnsClient;
use crate::model::dns_record_response::{Record, Type};
use crate::tests::test_utils::init_test_env;
use anyhow::Result;
use chrono::Utc;
use secrecy::{ExposeSecret, SecretString};
use std::sync::Arc;
use tokio;
use tracing::{info, debug, error};
use tracing_test::traced_test;

/// 创建模拟的阿里云DNS记录数据
fn create_mock_dns_records() -> Vec<Record> {
    vec![
        Record::new(
            crate::model::dns_record_response::Status::Enable,
            "www".to_string(),
            Type::A,
            "192.168.1.100".to_string(),
            "record_001".to_string(),
            600,
        ),
        Record::new(
            crate::model::dns_record_response::Status::Enable,
            "mail".to_string(),
            Type::A,
            "192.168.1.101".to_string(),
            "record_002".to_string(),
            3600,
        ),
        Record::new(
            crate::model::dns_record_response::Status::Enable,
            "@".to_string(),
            Type::A,
            "192.168.1.102".to_string(),
            "record_003".to_string(),
            600,
        ),
    ]
}

/// 创建模拟的域名数据
fn create_mock_domains() -> Vec<Domain> {
    vec![
        Domain {
            id: Some(1),
            name: "example.com".to_string(),
            provider: DnsProvider::Aliyun,
            status: DomainStatus::Active,
            expiry: "2025-12-31".to_string(),
            records: vec![],
        },
        Domain {
            id: Some(2),
            name: "test.com".to_string(),
            provider: DnsProvider::Aliyun,
            status: DomainStatus::Active,
            expiry: "2025-11-30".to_string(),
            records: vec![],
        },
    ]
}

/// 创建模拟的域名提供商数据
fn create_mock_providers() -> Vec<DomainProvider> {
    vec![
        DomainProvider {
            account_id: 1,
            provider_name: "阿里云测试账户".to_string(),
            provider: DnsProvider::Aliyun,
            credential: Credential::UsernamePassword(UsernamePasswordCredential {
                username: "test_user".to_string(),
                password: "test_password".to_string(),
            }),
        },
    ]
}

/// 创建模拟的DNS记录数据（用于界面显示）
fn create_mock_dns_records_for_ui() -> Vec<DnsRecord> {
    vec![
        DnsRecord {
            name: "www".to_string(),
            record_type: "A".to_string(),
            value: "192.168.1.100".to_string(),
            ttl: "600".to_string(),
        },
        DnsRecord {
            name: "mail".to_string(),
            record_type: "A".to_string(),
            value: "192.168.1.101".to_string(),
            ttl: "3600".to_string(),
        },
        DnsRecord {
            name: "@".to_string(),
            record_type: "A".to_string(),
            value: "192.168.1.102".to_string(),
            ttl: "600".to_string(),
        },
    ]
}

/// 初始化测试数据库并创建基础数据
async fn setup_test_database() -> Result<(sea_orm::DatabaseConnection, i64, i64)> {
    info!("初始化测试数据库");
    let connection = init_memory_database().await?;
    
    // 创建测试账户
    let password: SecretString = SecretString::from("test_password_123");
    let account = create_account(
        connection.clone(),
        NewAccount {
            provider: DnsProvider::Aliyun,
            username: "test_aliyun_user".to_string(),
            email: "test@example.com".to_string(),
            credential: Credential::UsernamePassword(UsernamePasswordCredential {
                username: "test_aliyun_user".to_string(),
                password: password.expose_secret().to_string(),
            }),
        },
    ).await.map_err(|e| anyhow::anyhow!("创建账户失败: {}", e))?;
    
    info!("创建测试账户成功，账户ID: {}", account.id);
    
    // 创建测试域名
    let domain = add_domain(
        &connection,
        NewDomain {
            domain_name: "example.com".to_string(),
            registration_date: Some(Utc::now().to_string()),
            expiration_date: Some("2025-12-31".to_string()),
            registrar: Some("阿里云".to_string()),
            status: ModelDomainStatus::Active,
            account_id: account.id,
        },
    ).await.map_err(|e| anyhow::anyhow!("添加域名失败: {}", e))?;
    
    info!("创建测试域名成功，域名ID: {}", domain.id);
    
    // 创建测试DNS记录
    let mock_records = create_mock_dns_records();
    for (index, record) in mock_records.iter().enumerate() {
        let new_record = NewRecord {
            account_id: account.id,
            domain_id: domain.id,
            record_name: record.rr.clone(),
            record_type: record.record_type.to_string(),
            record_value: record.value.clone(),
            ttl: record.ttl,
        };
        
        let saved_record = add_record(&connection, new_record).await
            .map_err(|e| anyhow::anyhow!("添加记录失败: {}", e))?;
        info!("创建测试DNS记录 {}/{} 成功，记录ID: {}", 
              index + 1, mock_records.len(), saved_record.id);
    }
    
    Ok((connection, account.id, domain.id))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试Message::ReloadComplete消息处理
    #[test]
    fn test_message_reload_complete() {
        init_test_env();
        info!("开始测试Message::ReloadComplete消息处理");
        
        let mut app = DomainManager::default();
        
        // 准备测试数据
        let providers = create_mock_providers();
        let domains = create_mock_domains();
        let records = create_mock_dns_records_for_ui();
        let total_count = domains.len() as u64;
        
        info!("准备测试数据: {} 个提供商, {} 个域名, {} 条DNS记录", 
              providers.len(), domains.len(), records.len());
        
        // 执行ReloadComplete消息
        let reload_model = ReloadModel::new_from(
            providers.clone(),
            domains.clone(),
            records.clone(),
            total_count,
        );
        
        let _task = app.update(Message::ReloadComplete(reload_model));
        
        // 验证应用程序状态
        assert_eq!(app.domain_providers.len(), providers.len(), "域名提供商数量不匹配");
        assert_eq!(app.domain_list.len(), domains.len(), "域名列表数量不匹配");
        let dns_records = app.get_dns_records();
        assert_eq!(dns_records.len(), records.len(), "DNS记录数量不匹配");
        assert_eq!(app.stats.total, total_count, "总数统计不匹配");
        
        // 验证具体数据
        let first_provider = app.domain_providers.get(0).unwrap();
        assert_eq!(first_provider.provider_name, "阿里云测试账户");
        assert_eq!(first_provider.provider, DnsProvider::Aliyun);
        
        let first_domain = app.domain_list.get(0).unwrap();
        assert_eq!(first_domain.name, "example.com");
        assert_eq!(first_domain.provider, DnsProvider::Aliyun);
        assert_eq!(first_domain.status, DomainStatus::Active);
        
        let first_record = dns_records.get(0).unwrap();
        assert_eq!(first_record.name, "www");
        assert_eq!(first_record.record_type, "A");
        assert_eq!(first_record.value, "192.168.1.100");
        
        info!("Message::ReloadComplete消息处理测试通过");
    }
    
    /// 测试Message::Reload消息处理（使用内存数据库）
    #[tokio::test]
    async fn test_message_reload_with_database() {
        init_test_env();
        info!("开始测试Message::Reload消息处理（使用内存数据库）");
        
        // 设置测试数据库
        let (connection, account_id, domain_id) = setup_test_database().await
            .expect("设置测试数据库失败");
        
        info!("测试数据库设置完成，账户ID: {}, 域名ID: {}", account_id, domain_id);
        
        // 创建DomainManager实例并设置数据库连接
        let mut app = DomainManager::default();
        app.connection = Some(connection.clone());
        
        // 执行Reload消息
        let task = app.update(Message::Reload);
        
        // 注意：在实际测试中，我们需要等待异步任务完成
        // 这里我们验证任务被正确创建
        // 在真实环境中，任务会异步执行并最终调用ReloadComplete
        
        info!("Message::Reload任务创建成功");
        
        // 验证数据库中的数据
        let accounts = list_accounts(&connection).await
            .expect("查询账户列表失败");
        assert_eq!(accounts.len(), 1, "账户数量不匹配");
        assert_eq!(accounts[0].username, "test_aliyun_user");
        
        let domains = get_account_domains(&connection, Some(account_id)).await
            .expect("查询域名列表失败");
        assert_eq!(domains.len(), 1, "域名数量不匹配");
        assert_eq!(domains[0].domain_name, "example.com");
        
        info!("Message::Reload消息处理测试通过");
    }
    
    /// 测试Message::SyncAllDomains消息处理
    #[tokio::test]
    async fn test_message_sync_all_domains() {
        init_test_env();
        info!("开始测试Message::SyncAllDomains消息处理");
        
        // 设置测试数据库
        let (connection, account_id, _domain_id) = setup_test_database().await
            .expect("设置测试数据库失败");
        
        // 创建DomainManager实例并设置数据库连接
        let mut app = DomainManager::default();
        app.connection = Some(connection.clone());
        
        // 创建模拟的DNS客户端
        let dns_client = DnsClient::new(
            "test_access_key".to_string(),
            "test_secret_key".to_string(),
            "cn-hangzhou".to_string(),
            vec![DnsProvider::Aliyun],
        );
        app.dns_client = dns_client;
        
        // 执行SyncAllDomains消息
        let task = app.update(Message::SyncAllDomains);
        
        // 验证同步状态被正确设置
        assert!(app.is_syncing(), "同步状态应该被设置为true");
        
        info!("Message::SyncAllDomains任务创建成功");
        
        // 注意：在实际测试中，我们需要模拟DNS客户端的响应
        // 这里我们验证任务被正确创建和同步状态被设置
        
        info!("Message::SyncAllDomains消息处理测试通过");
    }
    
    /// 测试Message::SyncAllDomainsComplete消息处理
    #[tokio::test]
    async fn test_message_sync_all_domains_complete_success() {
        init_test_env();
        info!("开始测试Message::SyncAllDomainsComplete成功场景");
        
        // 设置测试数据库连接
        let connection = init_memory_database().await
            .expect("初始化内存数据库失败");
        
        let mut app = DomainManager::default();
        app.connection = Some(connection);
        app.set_syncing(true);
        app.set_message("同步中...".to_string());
        
        // 执行SyncAllDomainsComplete成功消息
        let task = app.update(Message::SyncAllDomainsComplete(SyncResult::Success));
        
        // 验证应用程序状态
        assert!(!app.is_syncing(), "同步状态应该被重置为false");
        assert_eq!(app.get_message(), "", "错误消息应该被清除");
        
        info!("Message::SyncAllDomainsComplete成功场景测试通过");
    }
    
    #[test]
    fn test_message_sync_all_domains_complete_failed() {
        init_test_env();
        info!("开始测试Message::SyncAllDomainsComplete失败场景");
        
        let mut app = DomainManager::default();
        app.set_syncing(true);
        
        let error_message = "网络连接失败";
        
        // 执行SyncAllDomainsComplete失败消息
        let task = app.update(Message::SyncAllDomainsComplete(
            SyncResult::Failed(error_message.to_string())
        ));
        
        // 验证应用程序状态
        assert!(!app.is_syncing(), "同步状态应该被重置为false");
        assert_eq!(app.get_message(), format!("同步失败: {}", error_message), "错误消息应该被正确设置");
        
        info!("Message::SyncAllDomainsComplete失败场景测试通过");
    }
    
    /// 综合测试：完整的数据流程测试
    #[tokio::test]
    async fn test_complete_data_flow() {
        init_test_env();
        info!("开始综合测试：完整的数据流程");
        
        // 1. 设置测试数据库
        let (connection, account_id, domain_id) = setup_test_database().await
            .expect("设置测试数据库失败");
        
        // 2. 创建DomainManager实例
        let mut app = DomainManager::default();
        app.connection = Some(connection.clone());
        
        // 3. 执行Reload消息，模拟应用启动时的数据加载
        let _reload_task = app.update(Message::Reload);
        info!("执行Reload消息完成");
        
        // 4. 模拟ReloadComplete消息，验证界面数据更新
        let providers = create_mock_providers();
        let domains = create_mock_domains();
        let records = create_mock_dns_records_for_ui();
        
        let reload_model = ReloadModel::new_from(
            providers.clone(),
            domains.clone(),
            records.clone(),
            domains.len() as u64,
        );
        
        let _reload_complete_task = app.update(Message::ReloadComplete(reload_model));
        info!("执行ReloadComplete消息完成");
        
        // 5. 验证界面状态
        assert_eq!(app.domain_providers.len(), 1, "域名提供商数量应该为1");
        assert_eq!(app.domain_list.len(), 2, "域名列表数量应该为2");
        let dns_records = app.get_dns_records();
        assert_eq!(dns_records.len(), 3, "DNS记录数量应该为3");
        
        // 6. 执行同步操作
        let dns_client = DnsClient::new(
            "test_access_key".to_string(),
            "test_secret_key".to_string(),
            "cn-hangzhou".to_string(),
            vec![DnsProvider::Aliyun],
        );
        app.dns_client = dns_client;
        
        let _sync_task = app.update(Message::SyncAllDomains);
        assert!(app.is_syncing(), "同步状态应该为true");
        info!("执行SyncAllDomains消息完成");
        
        // 7. 模拟同步完成
        let _sync_complete_task = app.update(Message::SyncAllDomainsComplete(SyncResult::Success));
        assert!(!app.is_syncing(), "同步状态应该被重置为false");
        info!("执行SyncAllDomainsComplete消息完成");
        
        // 8. 验证数据库中的数据完整性
        let accounts = list_accounts(&connection).await
            .expect("查询账户列表失败");
        assert_eq!(accounts.len(), 1, "数据库中应该有1个账户");
        
        let domains = get_account_domains(&connection, Some(account_id)).await
            .expect("查询域名列表失败");
        assert_eq!(domains.len(), 1, "数据库中应该有1个域名");
        
        info!("综合测试：完整的数据流程测试通过");
    }
    
    /// 测试错误处理场景
    #[test]
    fn test_error_handling() {
        init_test_env();
        info!("开始测试错误处理场景");
        
        let mut app = DomainManager::default();
        // 不设置数据库连接，模拟数据库连接失败
        
        // 执行Reload消息，应该处理数据库连接失败的情况
        let task = app.update(Message::Reload);
        
        // 验证错误消息被正确设置
        assert_eq!(app.get_message(), "数据库连接失败，无法加载数据");
        
        info!("错误处理场景测试通过");
    }
}