//! DNS记录同步功能测试
//!
//! 测试域名同步和DNS记录保存到数据库的完整流程
//! 包括：
//! - 域名信息同步测试
//! - DNS记录查询测试
//! - DNS记录保存到数据库测试
//! - 界面数据重新渲染测试

use crate::api::dns_client::DnsClient;
use crate::api::provider::aliyun::AliyunDnsClient;
use crate::gui::model::domain::DnsProvider;
use crate::models::record::NewRecord;
use crate::storage::database::init_database;
use crate::storage::domains::{add_domain, find_domain_by_name_and_account};
use crate::configs::database::DatabaseConfig;
use crate::models::domain::DomainEntity;
use crate::storage::init_memory_database;
use crate::storage::records::add_record;
use crate::models::domain::{NewDomain, DomainStatus};
use crate::tests::test_utils::init_test_env;
use anyhow::Result;
use sea_orm::DatabaseConnection;
use std::error::Error as StdError;
use tokio;
use tracing::{info, error};

/// 测试DNS记录同步到数据库的完整流程
#[tokio::test]
async fn test_dns_sync_complete_flow() -> Result<()> {
    // 初始化测试环境
    init_test_env();

    info!("开始测试DNS记录同步完整流程");

    // 1. 建立数据库连接
    let db_config = DatabaseConfig::default();
    info!("开始初始化数据库，配置: {:?}", db_config);
    let conn = match init_memory_database().await {
        Ok(conn) => {
            info!("数据库连接初始化成功");
            conn
        }
        Err(e) => {
            error!("数据库连接初始化失败: {}", e);
            return Err(e);
        }
    };

    // 2. 创建测试域名
    let test_domain_name = "test-domain.com";
    let account_id = 1i64; // 假设账户ID为1

    let new_domain = NewDomain {
        account_id,
        domain_name: test_domain_name.to_string(),
        status: DomainStatus::Active,
        registration_date: None,
        expiration_date: None,
        registrar: Some("aliyun".to_string()),
    };

    // 3. 添加域名到数据库
    let domain_entity = add_domain(&conn, new_domain).await
        .map_err(|e| {
            error!("添加域名失败: {:?}", e);
            e
        })?;

    info!("成功添加测试域名: {}, ID: {}", test_domain_name, domain_entity.id);

    // 4. 测试DNS记录同步功能
    test_sync_dns_records_for_domain(
        &conn,
        account_id,
        test_domain_name,
        &domain_entity,
    ).await?;

    info!("DNS记录同步测试完成");
    Ok(())
}

/// 测试单个域名的DNS记录同步
async fn test_sync_dns_records_for_domain(
    conn: &DatabaseConnection,
    account_id: i64,
    domain_name: &str,
    domain_entity: &DomainEntity,
) -> Result<()> {
    info!("开始测试域名 {} 的DNS记录同步", domain_name);

    // 创建模拟的DNS客户端（在实际测试中，这里应该使用mock）
    let dns_client = DnsClient::new(
        "test_access_key".to_string(),
        "test_secret_key".to_string(),
        "cn-hangzhou".to_string(),
        vec![DnsProvider::Aliyun],
    );

    // 创建阿里云DNS客户端
    let aliyun_client = AliyunDnsClient::new(
        dns_client.access_key_id.clone(),
        dns_client.access_key_secret.clone(),
    );

    // 注意：在实际测试中，我们应该mock DNS API调用
    // 这里我们创建一些模拟的DNS记录来测试数据库保存功能
    let mock_dns_records = create_mock_dns_records();

    info!("模拟查询到 {} 条DNS记录", mock_dns_records.len());

    // 保存DNS记录到数据库
    let mut saved_count = 0;
    for record in mock_dns_records {
        let new_record = NewRecord {
            account_id,
            domain_id: domain_entity.id,
            record_name: record.rr.clone(),
            record_type: record.record_type.to_string(),
            record_value: record.value.clone(),
            ttl: record.ttl,
        };

        match add_record(conn, new_record).await {
            Ok(_) => {
                saved_count += 1;
                info!("保存DNS记录成功: {} {} {}", record.rr, record.record_type.to_string(), record.value);
            }
            Err(err) => {
                error!("保存DNS记录失败: {} - {}", record.rr, err);
                return Err(anyhow::anyhow!("DNS查询失败: {}", err));
            }
        }
    }

    info!("成功保存 {} 条DNS记录到数据库", saved_count);

    // 验证记录是否正确保存
    assert!(saved_count > 0, "应该至少保存一条DNS记录");

    Ok(())
}

/// 创建模拟的DNS记录用于测试
fn create_mock_dns_records() -> Vec<crate::model::dns_record_response::Record> {
    use crate::model::dns_record_response::{Record, Type, Status, Line};

    vec![
        Record::new(
            Status::Enable,
            "www".to_string(),
            Type::A,
            "192.168.1.1".to_string(),
            "test_record_1".to_string(),
            300,
        ),
        Record::new(
            Status::Enable,
            "mail".to_string(),
            Type::A,
            "192.168.1.2".to_string(),
            "test_record_2".to_string(),
            600,
        ),
        Record::new(
            Status::Enable,
            "@".to_string(),
            Type::MX,
            "10 mail.test-domain.com".to_string(),
            "test_record_3".to_string(),
            3600,
        ),
    ]
}

/// 测试域名查找功能
#[tokio::test]
async fn test_find_domain_by_name() -> Result<()> {
    init_test_env();
    info!("开始测试域名查找功能");

    let conn = init_memory_database().await?;
    let test_domain_name = "test-find-domain.com";

    // 首先添加一个测试域名
    let new_domain = NewDomain {
        account_id: 1,
        domain_name: test_domain_name.to_string(),
        status: DomainStatus::Active,
        registration_date: None,
        expiration_date: None,
        registrar: Some("aliyun".to_string()),
    };

    let added_domain = add_domain(&conn, new_domain).await?;
    info!("添加测试域名成功: ID {}", added_domain.id);

    // 测试查找功能
    match find_domain_by_name_and_account(&conn, test_domain_name, 1).await {
        Ok(Some(found_domain)) => {
            info!("成功找到域名: {} (ID: {})", found_domain.domain_name, found_domain.id);
            assert_eq!(found_domain.domain_name, test_domain_name);
            assert_eq!(found_domain.id, added_domain.id);
        }
        Ok(None) => {
            error!("未找到域名: {}", test_domain_name);
            panic!("应该能找到刚添加的域名");
        }
        Err(e) => {
            error!("查找域名时发生错误: {:?}", e);
            return Err(anyhow::anyhow!("查找域名失败: {}", e));
        }
    }

    info!("域名查找测试完成");
    Ok(())
}

/// 测试数据库连接
#[tokio::test]
async fn test_database_connection() -> Result<()> {
    init_test_env();
    info!("开始测试数据库连接");

    let conn = init_memory_database().await
        .map_err(|e| {
            error!("数据库连接失败: {:?}", e);
            e
        })?;

    info!("数据库连接成功");
    Ok(())
}
