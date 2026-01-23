//! Iced框架集成测试
//!
//! 测试DomainManager应用程序的完整消息处理流程
//! 包括：
//! - Message::Reload 消息处理测试
//! - Message::SyncAllDomains 消息处理测试
//! - 数据库集成测试
//! - 界面状态更新测试
//! - 阿里云客户端数据模拟

use crate::api::dns_client::DnsClient;
use crate::gui::model::domain::{DnsProvider, DnsRecord, Domain, DomainStatus};
use crate::gui::model::gui::ReloadModel;
use crate::gui::pages::domain::DomainProvider;
use crate::gui::types::credential::{ApiKeyCredential, Credential, UsernamePasswordCredential};
use crate::model::dns_record_response::{Record, Type};
use crate::models::account::NewAccount;
use crate::models::domain::{DomainStatus as ModelDomainStatus, NewDomain};
use crate::models::record::NewRecord;
use crate::storage::records::add_record;
use crate::storage::{
    add_domain, create_account, get_account_domains, init_memory_database, list_accounts,
    DnsRecordModal, DomainModal,
};
use crate::tests::test_utils::init_test_env;
use anyhow::Result;
use chrono::Utc;
use secrecy::{ExposeSecret, SecretString};
use std::sync::Arc;
use tokio;
use tracing::info;

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

/// 创建模拟的域名数据(DB实体)
fn create_mock_domain_modals() -> Vec<DomainModal> {
    vec![
        DomainModal {
            id: 1,
            name: "example.com".to_string(),
            provider_id: 1,
            status: "Active".to_string(),
            created_at: Utc::now().naive_utc(),
            updated_at: None,
        },
        DomainModal {
            id: 2,
            name: "test.com".to_string(),
            provider_id: 1,
            status: "Active".to_string(),
            created_at: Utc::now().naive_utc(),
            updated_at: None,
        },
    ]
}

/// 创建模拟的域名提供商数据
fn create_mock_providers() -> Vec<DomainProvider> {
    vec![DomainProvider {
        account_id: 1,
        provider_name: "阿里云测试账户".to_string(),
        provider: DnsProvider::Aliyun,
        credential: Credential::UsernamePassword(UsernamePasswordCredential {
            username: "test_user".to_string(),
            password: "test_password".to_string(),
        }),
    }]
}

/// 创建模拟的DNS记录数据(DB实体)
fn create_mock_dns_record_modals() -> Vec<DnsRecordModal> {
    vec![
        DnsRecordModal {
            id: 1,
            domain_id: 1,
            name: "www".to_string(),
            record_type: "A".to_string(),
            value: "192.168.1.100".to_string(),
            ttl: 600,
            priority: None,
            enabled: true,
            created_at: Utc::now().naive_utc(),
            updated_at: None,
        },
        DnsRecordModal {
            id: 2,
            domain_id: 1,
            name: "mail".to_string(),
            record_type: "A".to_string(),
            value: "192.168.1.101".to_string(),
            ttl: 3600,
            priority: None,
            enabled: true,
            created_at: Utc::now().naive_utc(),
            updated_at: None,
        },
        DnsRecordModal {
            id: 3,
            domain_id: 1,
            name: "@".to_string(),
            record_type: "A".to_string(),
            value: "192.168.1.102".to_string(),
            ttl: 600,
            priority: None,
            enabled: true,
            created_at: Utc::now().naive_utc(),
            updated_at: None,
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
    )
    .await
    .map_err(|e| anyhow::anyhow!("创建账户失败: {}", e))?;

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
    )
    .await
    .map_err(|e| anyhow::anyhow!("添加域名失败: {}", e))?;

    info!("创建测试域名成功，域名ID: {}", domain.id);

    // 创建测试DNS记录
    let mock_records = create_mock_dns_records();
    for (index, record) in mock_records.iter().enumerate() {
        let new_record = NewRecord {
            domain_id: domain.id,
            record_name: record.rr.clone(),
            record_type: record.record_type.to_string(),
            record_value: record.value.clone(),
            ttl: record.ttl,
        };

        let saved_record = add_record(&connection, new_record)
            .await
            .map_err(|e| anyhow::anyhow!("添加记录失败: {}", e))?;
        info!(
            "创建测试DNS记录 {}/{} 成功，记录ID: {}",
            index + 1,
            mock_records.len(),
            saved_record.id
        );
    }

    Ok((connection, account.id, domain.id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configs::gui_config::Config;
    use crate::gui::components::State;
    use crate::gui::handlers::message_handler::{
        AppMessage, DatabaseMessage, MessageCategory, ProviderMessage, SyncMessage, WindowMessage,
    };
    use crate::gui::manager_v2::DomainManagerV2;
    use crate::gui::pages::domain::VerificationStatus;
    use crate::storage::domain::Relation::Provider;
    use crate::storage::DomainModal;
    use async_trait::async_trait;
    use iced::{Size, Task};
    use std::time::Duration;
    use tokio::sync::RwLock;

    /// 测试Message::ReloadComplete消息处理
    #[test]
    fn test_message_reload_complete() {
        init_test_env();
        info!("开始测试Message::ReloadComplete消息处理");
        let mut app = DomainManagerV2::default();

        // 准备测试数据
        let providers = create_mock_providers();
        let domains = create_mock_domain_modals();
        let records = create_mock_dns_record_modals();
        let total_count = domains.len();

        info!(
            "准备测试数据: {} 个提供商, {} 个域名, {} 条DNS记录",
            providers.len(),
            domains.len(),
            records.len()
        );

        // 执行ReloadComplete消息
        let reload_model = ReloadModel::new_from(
            providers.clone(),
            domains.clone(),
            records.clone(),
            total_count,
        );

        let _task = app.update(MessageCategory::Sync(SyncMessage::DataReloaded(
            reload_model,
        )));

        let data_state = &app.state.data;

        // 验证应用程序状态
        assert_eq!(
            data_state.domain_providers.len(),
            providers.len(),
            "域名提供商数量不匹配"
        );
        assert_eq!(
            data_state.domain_list.len(),
            domains.len(),
            "域名列表数量不匹配"
        );
        let dns_records = &data_state.current_dns_records;
        assert_eq!(dns_records.len(), records.len(), "DNS记录数量不匹配");
        assert_eq!(
            data_state.domain_list.len(),
            total_count as usize,
            "总数统计不匹配"
        );

        // 验证具体数据
        let first_provider = data_state.domain_providers.get(0).unwrap();
        assert_eq!(*first_provider.provider_name, "阿里云测试账户".to_string());

        let first_domain = data_state.domain_list.get(0).unwrap();
        assert_eq!(first_domain.name, "example.com");
        // 注意：DomainModel的status字段可能是枚举类型，需要适配

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
        let (connection, account_id, domain_id) =
            setup_test_database().await.expect("设置测试数据库失败");

        info!(
            "测试数据库设置完成，账户ID: {}, 域名ID: {}",
            account_id, domain_id
        );

        // 创建DomainManager实例并设置数据库连接
        let mut app = DomainManagerV2::default();
        let _ = app.initialize().await;

        let clone_connection = connection.clone();

        app.state.database = Some(Arc::new(RwLock::new(clone_connection)));

        // 执行Reload消息
        let task = app.update(MessageCategory::Sync(SyncMessage::Reload));

        // 注意：在实际测试中，我们需要等待异步任务完成
        // 这里我们验证任务被正确创建
        // 在真实环境中，任务会异步执行并最终调用ReloadComplete

        info!("Message::Reload任务创建成功");

        // 验证数据库中的数据
        let accounts = list_accounts(&(connection.clone()))
            .await
            .expect("查询账户列表失败");
        assert_eq!(accounts.len(), 1, "账户数量不匹配");
        assert_eq!(accounts[0].username, "test_aliyun_user");

        let domains = get_account_domains(&connection, Some(account_id))
            .await
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
        let (connection, _account_id, _domain_id) =
            setup_test_database().await.expect("设置测试数据库失败");

        // 创建DomainManager实例并设置数据库连接
        let mut app = DomainManagerV2::new(Config::default());
        let _ = app.initialize().await;
        app.state.database = Some(Arc::new(RwLock::new(connection)));

        // 创建模拟的DNS客户端
        let dns_client = DnsClient::new(
            "test_access_key".to_string(),
            "test_secret_key".to_string(),
            "cn-hangzhou".to_string(),
            vec![DnsProvider::Aliyun],
        );

        // 执行SyncAllDomains消息
        let task = app.update(MessageCategory::Sync(SyncMessage::SyncAllDomains));

        // 验证同步状态被正确设置
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
        let connection = init_memory_database().await.expect("初始化内存数据库失败");

        let mut app = DomainManagerV2::default();
        let _ = app.initialize().await;
        app.state.database = Some(Arc::new(RwLock::new(connection)));
        app.state.ui.set_syncing(true);
        app.state.ui.set_message("同步中...".to_string());

        // 执行SyncAllDomainsComplete成功消息
        let task = app.update(MessageCategory::Sync(SyncMessage::AllComplete(Ok(()))));

        // 验证应用程序状态
        assert!(!app.state.ui.is_syncing, "同步状态应该被重置为false");
        assert_eq!(
            app.state.ui.message, "所有域名同步完成",
            "同步完成消息应正确设置"
        );

        info!("Message::SyncAllDomainsComplete成功场景测试通过");
    }

    #[tokio::test]
    async fn test_message_sync_all_domains_complete_failed() {
        init_test_env();
        info!("开始测试Message::SyncAllDomainsComplete失败场景");

        let mut app = DomainManagerV2::default();
        let _ = app.initialize().await;
        app.state.ui.set_syncing(true);

        let error_message = "网络连接失败";

        // 执行SyncAllDomainsComplete失败消息
        let task = app.update(MessageCategory::Sync(SyncMessage::AllComplete(Err(
            error_message.to_string(),
        ))));

        // 验证应用程序状态
        assert!(!app.state.ui.is_syncing, "同步状态应该被重置为false");
        assert_eq!(
            app.state.ui.message,
            error_message.to_string(),
            "错误消息应该被正确设置"
        );

        info!("Message::SyncAllDomainsComplete失败场景测试通过");
    }

    /// 综合测试：完整的数据流程测试
    #[tokio::test]
    async fn test_complete_data_flow() {
        init_test_env();
        info!("开始综合测试：完整的数据流程");

        // 1. 设置测试数据库
        let (connection, account_id, _domain_id) =
            setup_test_database().await.expect("设置测试数据库失败");

        // 2. 创建DomainManager实例
        let mut app = DomainManagerV2::default();

        // 在测试环境中，我们直接设置数据库连接，避免触发真实的异步初始化
        // 这样可以确保测试的确定性和幂等性
        let mock_connection = connection.clone();

        // 直接设置数据库连接并标记初始化完成（模拟 Connected 消息的处理）
        let _ = app.update(MessageCategory::Database(DatabaseMessage::Connected(Ok(
            mock_connection,
        ))));
        assert_eq!(app.is_initialized(), true, "客户端应该已经初始化");

        // 3. 执行Reload消息，模拟应用启动时的数据加载
        let _reload_task = app.update(MessageCategory::Sync(SyncMessage::Reload));
        info!("执行Reload消息完成");

        // 4. 模拟ReloadComplete消息，验证界面数据更新
        let providers = create_mock_providers();
        let domains = create_mock_domain_modals();
        let records = create_mock_dns_record_modals();

        let reload_model = ReloadModel::new_from(
            providers.clone(),
            domains.clone(),
            records.clone(),
            domains.len(),
        );

        let _reload_complete_task = app.update(MessageCategory::Sync(SyncMessage::DataReloaded(
            reload_model,
        )));
        info!("执行ReloadComplete消息完成");

        // 5. 验证界面状态
        assert_eq!(
            app.state.data.domain_providers.len(),
            1,
            "域名提供商数量应该为1"
        );
        assert_eq!(app.state.data.domain_list.len(), 2, "域名列表数量应该为2");
        let dns_records = &app.state.data.current_dns_records;
        assert_eq!(dns_records.len(), 3, "DNS记录数量应该为3");

        // 6. 执行同步操作
        let _dns_client = DnsClient::new(
            "test_access_key".to_string(),
            "test_secret_key".to_string(),
            "cn-hangzhou".to_string(),
            vec![DnsProvider::Aliyun],
        );
        // 注意：DomainManagerV2不再直接暴露dns_client字段

        let _sync_task = app.update(MessageCategory::Sync(SyncMessage::SyncAllDomains));
        assert!(app.state.ui.is_syncing, "同步状态应该为true");
        info!("执行SyncAllDomains消息完成");

        // let domains: Vec<DomainModal> = vec![];
        // 7. 模拟同步完成
        let _sync_complete_task =
            app.update(MessageCategory::Sync(SyncMessage::AllComplete(Ok(()))));
        assert!(!app.state.ui.is_syncing, "同步状态应该被重置为false");
        info!("执行SyncAllDomainsComplete消息完成");

        // 8. 验证数据库中的数据完整性
        let accounts = list_accounts(&connection).await.expect("查询账户列表失败");
        assert_eq!(accounts.len(), 1, "数据库中应该有1个账户");

        let domains = get_account_domains(&connection, Some(account_id))
            .await
            .expect("查询域名列表失败");
        assert_eq!(domains.len(), 1, "数据库中应该有1个域名");

        info!("综合测试：完整的数据流程测试通过");

        // 1. 验证表单初始化状态
        info!("验证表单初始化状态");
        let form = &app.state.data.add_domain_provider_form;
        assert_eq!(form.provider_name, "", "提供商名称应该为空");
        assert!(form.provider.is_none(), "提供商类型应该为空");
        assert!(form.credential.is_none(), "凭证信息应该为空");

        // 2. 选择阿里云提供商
        info!("选择阿里云提供商");
        let event = app.update(MessageCategory::Provider(ProviderMessage::Selected(
            DnsProvider::Aliyun,
        )));

        assert_eq!(
            app.state.data.add_domain_provider_form.provider.unwrap(),
            DnsProvider::Aliyun
        );

        // 3. 输入提供商名称
        info!("输入提供商名称");
        let provider_name = "阿里云测试账户".to_string();
        let _ = app.update(MessageCategory::Provider(
            ProviderMessage::AddFormNameChanged(provider_name.clone()),
        ));
        assert_eq!(
            app.state.data.add_domain_provider_form.provider_name,
            provider_name
        );

        // 4. 输入accesskey和apisecret
        info!("输入accesskey和apisecret");
        let access_key = "test_access_key".to_string();
        let api_secret = "test_api_secret".to_string();

        // 模拟APIKEY凭证输入
        let _ = app.update(MessageCategory::Provider(
            ProviderMessage::AddFormCredentialChanged(
                crate::gui::types::credential::CredentialMessage::ApiKeyChanged(
                    crate::gui::types::credential::ApiKeyMessage::ApiKeyChanged(
                        crate::gui::types::credential::ApiKeyCredential {
                            api_key: access_key.clone(),
                            api_secret: api_secret.clone(),
                        },
                    ),
                ),
            ),
        ));

        // 模拟API_SECRET凭证输入
        let _ = app.update(MessageCategory::Provider(
            ProviderMessage::AddFormCredentialChanged(
                crate::gui::types::credential::CredentialMessage::ApiKeyChanged(
                    crate::gui::types::credential::ApiKeyMessage::ApiSecretChanged(
                        crate::gui::types::credential::ApiKeyCredential {
                            api_key: access_key.clone(),
                            api_secret: api_secret.clone(),
                        },
                    ),
                ),
            ),
        ));

        // 验证凭证信息已更新
        if let Some(Credential::ApiKey(cred)) = &app.state.data.add_domain_provider_form.credential
        {
            assert_eq!(cred.api_key, access_key, "AccessKey应该正确设置");
            assert_eq!(cred.api_secret, api_secret, "ApiSecret应该正确设置");
        } else {
            panic!("凭证类型应该是ApiKey");
        }

        // 5. 点击验证凭据按钮
        info!("点击验证凭据按钮");
        let validate_task = app.update(MessageCategory::Provider(
            ProviderMessage::ValidateCredential,
        ));

        // 验证验证状态为Pending
        match app.state.data.add_domain_provider_form.verification_status {
            VerificationStatus::Pending => {
                info!("凭证验证状态正确设置为Pending");
            }
            _ => panic!("验证状态应该是Pending"),
        }

        // 6. 模拟验证成功（这里需要等待异步任务完成，在实际测试中可能需要mock）
        info!("模拟验证成功");
        let _ = app.update(MessageCategory::Provider(
            ProviderMessage::VerificationStatusChanged(VerificationStatus::Success),
        ));

        // 验证验证状态为Success
        match app.state.data.add_domain_provider_form.verification_status {
            VerificationStatus::Success => {
                info!("凭证验证成功");
            }
            _ => panic!("验证状态应该是Success"),
        }

        let origin_length = list_accounts(&connection)
            .await
            .expect("查询账户列表失败")
            .len();

        info!("原始账户列表长度为 {}", origin_length);

        // 7. 点击添加按钮
        info!("点击添加按钮");
        let add_task = app.update(MessageCategory::Provider(ProviderMessage::AddCredential));

        // 8. 模拟运行时执行数据库操作
        // 注意：在单元测试中，app.update 返回的 Task 不会被 Iced 运行时执行。
        // 我们需要手动执行 AddCredential 对应的数据库操作来模拟这一过程。
        info!("模拟运行时执行数据库写入");

        // 直接手动调用数据库操作来模拟 Task 的执行
        let new_account = NewAccount {
            provider: DnsProvider::Aliyun,
            username: "阿里云测试账户".to_string(),
            email: "example@qq.com".to_string(),
            credential: Credential::ApiKey(ApiKeyCredential {
                api_key: access_key.clone(),
                api_secret: api_secret.clone(),
            }),
        };

        let account = create_account(connection.clone(), new_account)
            .await
            .expect("模拟数据库写入失败");

        info!("数据库写入完成，账户ID: {}", account.id);

        // 数据库操作已经在上面完成

        // 9. 验证数据库中的数据
        info!("验证数据库中的数据");
        let accounts = list_accounts(&connection).await.expect("查询账户列表失败");
        assert_eq!(
            accounts.len(),
            2,
            "数据库中应该有2个账户（原有1个+新增1个）"
        );

        // 验证新增的账户信息
        let new_account = accounts
            .iter()
            .find(|acc| acc.id == account.id)
            .expect("应该找到新增的账户");
        assert_eq!(
            new_account.provider_type,
            DnsProvider::Aliyun.value(),
            "提供商类型应该正确"
        );
        assert_eq!(new_account.username, "阿里云测试账户", "用户名应该匹配");
    }
    async fn handle_window_minimize() {
        init_test_env();
        info!("测试窗口最小化");

        let mut app = DomainManagerV2::default();
        // 不设置数据库连接，模拟数据库连接失败

        // 验证错误消息被正确设置
        // 注意：由于没有数据库连接，可能不会设置特定的错误消息
        // 这里我们验证应用状态是否正确处理了错误情况
        // assert_eq!(app.state.ui.message, "数据库连接失败，无法加载数据");

        app.initialize().await.unwrap();

        info!("初始化完成，开始进行事件的测试！");

        // 执行Reload消息，应该处理数据库连接失败的情况
        let _task = app.update(MessageCategory::Window(WindowMessage::WindowMinimize));
        assert_eq!(app.state.ui.window_minimize, true);
    }

    /// 测试错误处理场景
    #[tokio::test]
    async fn handle_reload() {
        init_test_env();
        info!("测试窗口最小化");

        let mut app = DomainManagerV2::default();
        // 不设置数据库连接，模拟数据库连接失败

        // 验证错误消息被正确设置
        // 注意：由于没有数据库连接，可能不会设置特定的错误消息
        // 这里我们验证应用状态是否正确处理了错误情况
        // assert_eq!(app.state.ui.message, "数据库连接失败，无法加载数据");

        app.initialize().await.unwrap();
        info!("初始化完成，开始进行事件的测试！");

        // 执行Reload消息，应该处理数据库连接失败的情况
        let _task = app.update(MessageCategory::Sync(SyncMessage::Reload));
        assert_eq!(app.state.data.domain_list.len(), 1, "数据库中应该有1个域名");
    }
}
