//! 同步处理器
//!
//! 负责处理所有与数据同步相关的业务逻辑，包括域名同步、DNS记录同步、
//! 批量同步等操作。

use super::message_handler::{MessageCategory, NotificationMessage, SyncMessage};
use super::{AsyncEventHandler, EventHandler, HandlerResult};
use crate::api::aliyun_dns_api::AliyunDnsApi;
use crate::api::dns_api::{DnsApiTrait, DnsRecordQuery};
use crate::gui::model::gui::ReloadModel;
use crate::gui::pages::domain::DomainProvider;
use crate::gui::state::app_state::{StateUpdate, UiUpdate};
use crate::gui::state::AppState;
use crate::gui::types::credential::Credential;
use crate::models::record::NewRecord;
use crate::storage::{accounts, domains, records, DnsRecordModal, DomainModal};
use iced::Task;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// 同步处理器
///
/// 专门处理数据同步相关的事件和业务逻辑
#[derive(Debug)]
pub struct SyncHandler {
    // 可以添加同步服务的依赖
}

/// 批量同步结果
#[derive(Debug, Clone)]
pub struct BatchSyncResult {
    pub successful: Vec<SyncResult>,
    pub failed: Vec<SyncResult>,
    pub cancelled: Vec<SyncResult>,
    pub total_domains: usize,
    pub total_records: usize,
}

#[derive(Debug, Clone)]
pub enum SyncResult {
    // 同步失败
    Failed(String),

    /// 同步成功
    Success,

    // 已取消
    Cancelled,
}

impl SyncHandler {
    /// 创建新的同步处理器
    pub fn new() -> Self {
        Self {}
    }

    /// 处理单个域名同步
    fn handle_sync_domain(&self, state: &mut AppState, domain: String) -> HandlerResult {
        // 检查域名是否存在
        let domain_exists = state.data.domain_list.iter().any(|d| d.name == domain);
        if !domain_exists {
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(format!(
                "域名 {} 不存在",
                domain
            ))));
            return HandlerResult::StateUpdated;
        }

        // 设置同步状态
        state.data.set_syncing(&domain, true);
        state.ui.set_message(format!("正在同步域名: {}", domain));

        // 返回同步任务
        let domain_for_async = domain.clone();
        let domain_for_message = domain.clone();

        if let Some(conn) = &state.database {
            let db_arc = conn.clone();

            HandlerResult::StateUpdatedWithTask(Task::perform(
                Self::sync_domain_async(db_arc, domain_for_async),
                move |result| {
                    // ... (保持不变)
                    let converted_dns_record_result = result.map(|records| {
                        let dns_records: Vec<DnsRecordModal> = records
                            .into_iter()
                            .map(|record| DnsRecordModal {
                                id: 0,
                                name: record.name,
                                record_type: record.record_type,
                                value: record.value,
                                ttl: record.ttl,
                                priority: None,
                                enabled: false,
                                created_at: Default::default(),
                                domain_id: 0,
                                updated_at: None,
                            })
                            .collect();

                        dns_records
                    });
                    MessageCategory::Sync(SyncMessage::DomainSyncComplete(
                        domain_for_message.clone(),
                        converted_dns_record_result,
                    ))
                },
            ))
        } else {
            state.ui.set_message("数据库未连接".to_string());
            HandlerResult::StateUpdated
        }
    }

    /// 处理批量同步所有域名
    fn handle_sync_all_domains(&self, state: &mut AppState) -> HandlerResult {
        let domains: Vec<String> = state
            .data
            .domain_list
            .iter()
            .map(|d| d.name.clone())
            .collect();

        if domains.is_empty() {
            warn!("没有域名需要同步");
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                "没有域名需要同步".to_string(),
            )));
            return HandlerResult::StateUpdated;
        }

        // 设置全局同步状态
        state.ui.is_syncing = true;

        // 设置所有域名为同步状态
        for domain in &domains {
            state.data.set_syncing(domain, true);
        }

        state
            .ui
            .set_message(format!("正在同步 {} 个域名...", domains.len()));

        if let Some(conn) = &state.database {
            let conn_clone = conn.clone();

            // 返回批量同步任务
            HandlerResult::StateUpdatedWithTask(Task::perform(
                Self::sync_all_domains_async(conn_clone, domains),
                |result| {
                    // 将BatchSyncResult转换为Result<(), String>
                    if result.failed.is_empty() {
                        MessageCategory::Sync(SyncMessage::AllComplete(Ok(())))
                    } else {
                        let error_msg = format!("同步失败: {} 个域名失败", result.failed.len());
                        MessageCategory::Sync(SyncMessage::AllComplete(Err(error_msg)))
                    }
                },
            ))
        } else {
            state.ui.set_message("数据库未连接".to_string());
            HandlerResult::StateUpdated
        }
    }

    /// 处理同步完成
    fn handle_sync_complete(
        &self,
        state: &mut AppState,
        result: Result<Vec<DomainModal>, String>,
    ) -> HandlerResult {
        match result {
            Ok(domains) => {
                info!("域名同步成功:{}", &domains.len());
                // 更新域名列表
                let gui_domains: Vec<DomainModal> = domains
                    .into_iter()
                    .map(|domain| DomainModal {
                        id: domain.id,
                        name: domain.name,
                        status: "str".into(),
                        created_at: Default::default(),
                        provider_id: 0,
                        updated_at: None,
                    })
                    .collect();

                // 更新应用状态
                state.data.domain_list = gui_domains;
                state.ui.set_message("数据同步完成".to_string());

                // 标记数据已变更
                state.data.mark_changed();
            }
            Err(error) => {
                debug!("同步失败：错误信息：「{}」", error);
                state.ui.set_message(format!("同步失败: {}", error));
            }
        }

        HandlerResult::StateUpdated
    }

    /// 处理批量同步完成
    fn handle_all_sync_complete(
        &self,
        state: &mut AppState,
        result: BatchSyncResult,
    ) -> HandlerResult {
        // 重置全局同步状态
        state.ui.is_syncing = false;

        // 清除所有同步状态
        let domain_names: Vec<String> = state
            .data
            .domain_list
            .iter()
            .map(|d| d.name.clone())
            .collect();
        for domain_name in domain_names {
            state.data.set_syncing(&domain_name, false);
        }

        // 更新成功同步的记录
        for sync_result in &result.successful {
            if let SyncResult::Success = sync_result {
                // TODO: 需要从其他地方获取domain和records_count信息
                // 暂时跳过统计更新
            }
        }

        // 显示同步结果摘要
        let summary = format!(
            "批量同步完成: 成功 {} 个，失败 {} 个，取消 {} 个，共处理 {} 条记录",
            result.successful.len(),
            result.failed.len(),
            result.cancelled.len(),
            result.total_records
        );

        state.ui.set_message(summary.clone());
        state.update(StateUpdate::Ui(UiUpdate::ShowToast(summary)));

        // 标记数据已变更
        if !result.successful.is_empty() {
            state.data.mark_changed();
        }

        HandlerResult::StateUpdated
    }

    /// 处理取消同步
    fn handle_cancel_sync(&self, state: &mut AppState, domain: Option<String>) -> HandlerResult {
        match domain {
            Some(domain_name) => {
                // 取消单个域名的同步
                state.data.set_syncing(&domain_name, false);
                state
                    .ui
                    .set_message(format!("已取消域名 {} 的同步", domain_name));
            }
            None => {
                // 取消所有域名的同步
                let domain_names: Vec<String> = state
                    .data
                    .domain_list
                    .iter()
                    .map(|d| d.name.clone())
                    .collect();
                for domain_name in domain_names {
                    state.data.set_syncing(&domain_name, false);
                }
                state.ui.set_message("已取消所有域名的同步".to_string());
            }
        }

        HandlerResult::StateUpdated
    }

    /// 处理重新加载数据
    fn handle_reload(&self, state: &mut AppState) -> HandlerResult {
        state.ui.set_message("正在重新加载数据...".to_string());

        match &state.database {
            Some(_x) => HandlerResult::NoChange,
            None => {
                error!("数据库未连接！");
                state.ui.set_message("数据库未连接".to_string());
                HandlerResult::NoChange
            }
        }
    }

    /// 异步同步单个域名
    async fn sync_domain_async(
        conn: DatabaseConnection,
        domain_name: String,
    ) -> Result<Vec<DnsRecordModal>, String> {
        // 1. 查找域名以获取 Account ID
        let domain_entity = domains::find_domain_by_name(&conn, &domain_name)
            .await
            .map_err(|e: Box<dyn Error>| e.to_string())?
            .ok_or(format!("域名 {} 不存在", domain_name))?;

        // 2. 获取账户信息
        let account = accounts::get_account_by_id(&conn, domain_entity.account_id)
            .await
            .map_err(|e: Box<dyn Error + Send>| e.to_string())?
            .ok_or(format!("账户 ID {} 不存在", domain_entity.account_id))?;

        // 3. 初始化 API 客户端
        let credential: Credential = account
            .try_into()
            .map_err(|e: anyhow::Error| e.to_string())?;

        let api_client = match credential {
            Credential::ApiKey(key) => AliyunDnsApi::new(key.api_key, key.api_secret),
            _ => return Err("不支持的凭据类型".to_string()),
        };

        // 4. 调用 API
        let query = DnsRecordQuery {
            domain_name: domain_name.clone(),
            ..Default::default()
        };

        let response = api_client
            .query_dns_records(query)
            .await
            .map_err(|e| e.to_string())?;

        // 5. 转换结果
        let records: Vec<DnsRecordModal> = response
            .records
            .into_iter()
            .map(|r| DnsRecordModal {
                id: 0, // 新记录 ID 为 0，插入数据库时会自动生成
                domain_id: domain_entity.id,
                name: r.rr,
                record_type: r.record_type.to_string(),
                value: r.value,
                ttl: r.ttl,
                priority: None,
                enabled: r.status == crate::model::dns_record_response::Status::Enable,
                created_at: chrono::Utc::now().naive_utc(),
                updated_at: None,
            })
            .collect();

        // 6. 更新数据库
        // 先删除旧记录
        records::delete_records_by_domain(&conn, domain_entity.id)
            .await
            .map_err(|e| e.to_string())?;

        // 再插入新记录
        if !records.is_empty() {
            let new_records: Vec<NewRecord> = records
                .iter()
                .map(|r| NewRecord {
                    domain_id: r.domain_id,
                    record_name: r.name.clone(),
                    record_type: r.record_type.clone(),
                    record_value: r.value.clone(),
                    ttl: r.ttl,
                })
                .collect();

            records::add_records_many(&conn, new_records)
                .await
                .map_err(|e| e.to_string())?;
        }

        Ok(records)
    }

    /// 异步批量同步所有域名
    async fn sync_all_domains_async(
        conn: DatabaseConnection,
        domains: Vec<String>,
    ) -> BatchSyncResult {
        info!("正在同步所有域名！");
        let mut successful = Vec::new();
        let mut failed = Vec::new();
        let cancelled = Vec::new();
        let mut total_records = 0;

        // 并发同步所有域名（限制并发数）
        let semaphore = Arc::new(tokio::sync::Semaphore::new(3)); // 最多3个并发
        let mut tasks = Vec::new();

        for domain in domains.iter().cloned() {
            let permit = semaphore.clone();
            let domain_clone = domain.clone();
            let conn_clone = conn.clone();
            let task = tokio::spawn(async move {
                let _permit = permit.acquire().await.unwrap();
                let result = Self::sync_domain_async(conn_clone, domain_clone.clone()).await;
                (domain_clone, result)
            });
            tasks.push(task);
        }

        // 等待所有任务完成
        for task in tasks {
            match task.await {
                Ok((_domain, result)) => match result {
                    Ok(records) => {
                        let count = records.len();
                        total_records += count;
                        successful.push(SyncResult::Success);
                    }
                    Err(error) => {
                        failed.push(SyncResult::Failed(error));
                    }
                },
                Err(_) => {
                    // 任务被取消或出现其他错误
                    // 这里无法获取域名，所以跳过
                }
            }
        }

        BatchSyncResult {
            successful,
            failed,
            cancelled,
            total_domains: domains.len(),
            total_records,
        }
    }

    /// 异步重新加载数据
    async fn reload_data_async(
        conn: DatabaseConnection,
    ) -> Result<
        (
            Vec<DomainModal>,
            HashMap<String, Vec<DnsRecordModal>>,
            Vec<DomainProvider>,
        ),
        String,
    > {
        info!("reloading data async");
        info!("reload data");
        // 1. 加载账户 (Providers)
        let accounts = accounts::list_accounts(&conn)
            .await
            .map_err(|e| e.to_string())?;
        let providers: Vec<DomainProvider> = accounts.into_iter().map(|acc| acc.into()).collect();

        // 2. 加载域名
        let domain_entities = domains::list_domains(&conn)
            .await
            .map_err(|e| e.to_string())?;
        let mut domain_modals = Vec::new();
        let mut records_map = HashMap::new();

        for domain in domain_entities {
            // 转换 DomainEntity -> DomainModal
            let modal = DomainModal {
                id: domain.id,
                name: domain.domain_name.clone(),
                provider_id: domain.account_id,
                status: domain.status.to_string().into(),
                created_at: chrono::NaiveDateTime::parse_from_str(
                    &domain.created_at,
                    "%Y-%m-%d %H:%M:%S",
                )
                .unwrap_or_default(),
                updated_at: None,
            };
            domain_modals.push(modal);

            // 3. 加载每个域名的 DNS 记录
            let record_entities = records::get_records_by_domain(&conn, Some(domain.id))
                .await
                .unwrap_or_default();
            let dns_records: Vec<DnsRecordModal> = record_entities
                .into_iter()
                .map(|r| DnsRecordModal {
                    id: r.id,
                    domain_id: r.domain_id,
                    name: r.record_name,
                    record_type: r.record_type,
                    value: r.record_value,
                    ttl: r.ttl,
                    priority: None,
                    enabled: true,
                    created_at: Default::default(),
                    updated_at: None,
                })
                .collect();

            records_map.insert(domain.domain_name, dns_records);
        }

        Ok((domain_modals, records_map, providers))
    }
}

impl EventHandler<SyncMessage> for SyncHandler {
    fn handle(&self, state: &mut AppState, event: SyncMessage) -> HandlerResult {
        match event {
            SyncMessage::Start => {
                info!("开始同步单个域名");
                HandlerResult::NoChange
            }
            SyncMessage::StartAll => {
                info!("开始同步所有域名");
                self.handle_sync_all_domains(state)
            }
            SyncMessage::SyncAllDomains => {
                info!("同步所有域名");
                self.handle_sync_all_domains(state)
            }
            SyncMessage::Reload => {
                info!("重新加载数据");
                self.handle_reload(state)
            }
            SyncMessage::Complete(result) => {
                info!("同步完成",);
                self.handle_sync_complete(state, result)
            }
            SyncMessage::AllComplete(result) => {
                info!("所有同步完成");
                // 重置全局同步状态
                state.ui.is_syncing = false;

                // 清除所有域名的同步状态
                let domain_names: Vec<String> = state
                    .data
                    .domain_list
                    .iter()
                    .map(|d| d.name.clone())
                    .collect();
                for domain_name in domain_names {
                    state.data.set_syncing(&domain_name, false);
                }

                match result {
                    Ok(_) => {
                        state.ui.set_message("所有域名同步完成".to_string());
                    }
                    Err(e) => {
                        state.ui.set_message(e.clone());
                        state.update(StateUpdate::Ui(UiUpdate::ShowToast(e)));
                    }
                }
                HandlerResult::StateUpdated
            }
            SyncMessage::Cancel => {
                info!("取消同步");
                HandlerResult::StateUpdated
            }
            SyncMessage::DataReloaded(model) => {
                info!("数据加载完成");
                // 更新应用状态
                state.data.provider_page.providers = model.providers;
                state.data.domain_list = model.domains;
                state.data.current_dns_records = model.records;
                state.ui.set_message("数据加载完成".to_string());
                HandlerResult::StateUpdated
            }
            SyncMessage::DomainSyncComplete(_, _) => todo!(),
        }
    }

    /// 检查是否可以处理该消息
    fn can_handle(&self, _event: &SyncMessage) -> bool {
        true // SyncHandler可以处理所有SyncMessage
    }
}

impl AsyncEventHandler<SyncMessage> for SyncHandler {
    fn handle_async(&self, state: &mut AppState, event: SyncMessage) -> Task<MessageCategory> {
        match event {
            SyncMessage::SyncAllDomains => {
                let domains: Vec<String> = state
                    .data
                    .domain_list
                    .iter()
                    .map(|d| d.name.clone())
                    .collect();

                if let Some(conn) = &state.database {
                    let conn = conn.clone();
                    Task::perform(Self::sync_all_domains_async(conn, domains), |result| {
                        // 将BatchSyncResult转换为Result<(), String>
                        if result.failed.is_empty() {
                            MessageCategory::Sync(SyncMessage::AllComplete(Ok(())))
                        } else {
                            let error_msg = format!("同步失败: {} 个域名失败", result.failed.len());
                            MessageCategory::Sync(SyncMessage::AllComplete(Err(error_msg)))
                        }
                    })
                } else {
                    Task::done(MessageCategory::Sync(SyncMessage::AllComplete(Err(
                        "数据库未连接".to_string(),
                    ))))
                }
            }
            SyncMessage::Reload => {
                if let Some(conn) = &state.database {
                    let conn = conn.clone();
                    Task::perform(Self::reload_data_async(conn), |result| {
                        match result {
                            Ok((domains, records_map, providers)) => {
                                // 将HashMap<String, Vec<DnsRecordModal>>转换为Vec<DnsRecordModal>
                                let records: Vec<DnsRecordModal> = records_map
                                    .into_iter()
                                    .flat_map(|(_, records)| records)
                                    .collect();

                                let total_count = domains.len() + records.len();

                                MessageCategory::Sync(SyncMessage::DataReloaded(
                                    ReloadModel::new_from(providers, domains, records, total_count),
                                ))
                            }
                            Err(e) => MessageCategory::Notification(
                                NotificationMessage::ShowToast(format!("重新加载数据失败: {}", e)),
                            ),
                        }
                    })
                } else {
                    Task::done(MessageCategory::Notification(
                        NotificationMessage::ShowToast(
                            "重新加载数据失败: 数据库未连接".to_string(),
                        ),
                    ))
                }
            }
            _ => Task::none(),
        }
    }
}

impl Default for SyncHandler {
    fn default() -> Self {
        Self::new()
    }
}
