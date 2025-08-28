//! 同步处理器
//! 
//! 负责处理所有与数据同步相关的业务逻辑，包括域名同步、DNS记录同步、
//! 批量同步等操作。

use super::{HandlerResult, EventHandler, AsyncEventHandler};
use super::message_handler::SyncMessage;
use crate::gui::state::{AppState, StateUpdate, DataUpdate, UiUpdate};
use crate::gui::Message;
use crate::models::{Domain, DnsRecord};
use iced::Task;
use std::collections::HashMap;
use tokio::time::{Duration, sleep};

/// 同步处理器
/// 
/// 专门处理数据同步相关的事件和业务逻辑
pub struct SyncHandler {
    // 可以添加同步服务的依赖
}

/// 同步结果
#[derive(Debug, Clone)]
pub enum SyncResult {
    /// 同步成功
    Success {
        domain: String,
        records_count: usize,
    },
    /// 同步失败
    Failed {
        domain: String,
        error: String,
    },
    /// 同步取消
    Cancelled {
        domain: String,
    },
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

impl SyncHandler {
    /// 创建新的同步处理器
    pub fn new() -> Self {
        Self {}
    }
    
    /// 处理单个域名同步
    fn handle_sync_domain(&self, state: &mut AppState, domain: String) -> HandlerResult {
        // 检查域名是否存在
        let domain_exists = state.data.domains.iter().any(|d| d.domain_name == domain);
        if !domain_exists {
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                format!("域名 {} 不存在", domain)
            )));
            return HandlerResult::StateUpdated;
        }
        
        // 设置同步状态
        state.data.set_syncing(&domain, true);
        state.ui.set_message(format!("正在同步域名: {}", domain));
        
        // 返回同步任务
        HandlerResult::StateUpdatedWithTask(
            Task::perform(
                Self::sync_domain_async(domain.clone()),
                move |result| Message::DomainSyncComplete(domain, result)
            )
        )
    }
    
    /// 处理批量同步所有域名
    fn handle_sync_all_domains(&self, state: &mut AppState) -> HandlerResult {
        let domains: Vec<String> = state.data.domains
            .iter()
            .map(|d| d.domain_name.clone())
            .collect();
        
        if domains.is_empty() {
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                "没有域名需要同步".to_string()
            )));
            return HandlerResult::StateUpdated;
        }
        
        // 设置所有域名为同步状态
        for domain in &domains {
            state.data.set_syncing(domain, true);
        }
        
        state.ui.set_message(format!("正在同步 {} 个域名...", domains.len()));
        
        // 返回批量同步任务
        HandlerResult::StateUpdatedWithTask(
            Task::perform(
                Self::sync_all_domains_async(domains),
                |result| Message::AllDomainsSyncComplete(result)
            )
        )
    }
    
    /// 处理同步完成
    fn handle_sync_complete(
        &self,
        state: &mut AppState,
        domain: String,
        result: Result<Vec<DnsRecord>, String>,
    ) -> HandlerResult {
        // 清除同步状态
        state.data.set_syncing(&domain, false);
        
        match result {
            Ok(records) => {
                // 更新DNS记录缓存
                state.data.dns_records_cache.insert(domain.clone(), records.clone());
                
                // 更新统计信息
                if let Some(stats) = state.data.domain_stats.get_mut(&domain) {
                    stats.dns_records_count = records.len();
                    stats.last_sync = Some(chrono::Utc::now());
                }
                
                state.ui.set_message(format!(
                    "域名 {} 同步完成，共 {} 条DNS记录",
                    domain,
                    records.len()
                ));
                
                // 标记数据已变更
                state.data.mark_changed();
            },
            Err(error) => {
                state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                    format!("域名 {} 同步失败: {}", domain, error)
                )));
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
        // 清除所有同步状态
        for domain in state.data.domains.iter() {
            state.data.set_syncing(&domain.domain_name, false);
        }
        
        // 更新成功同步的记录
        for sync_result in &result.successful {
            if let SyncResult::Success { domain, records_count } = sync_result {
                if let Some(stats) = state.data.domain_stats.get_mut(domain) {
                    stats.dns_records_count = *records_count;
                    stats.last_sync = Some(chrono::Utc::now());
                }
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
                state.ui.set_message(format!("已取消域名 {} 的同步", domain_name));
            },
            None => {
                // 取消所有域名的同步
                for domain in state.data.domains.iter() {
                    state.data.set_syncing(&domain.domain_name, false);
                }
                state.ui.set_message("已取消所有域名的同步".to_string());
            }
        }
        
        HandlerResult::StateUpdated
    }
    
    /// 处理重新加载数据
    fn handle_reload(&self, state: &mut AppState) -> HandlerResult {
        state.ui.set_message("正在重新加载数据...".to_string());
        
        // 返回重新加载任务
        HandlerResult::StateUpdatedWithTask(
            Task::perform(
                Self::reload_data_async(),
                |result| match result {
                    Ok((domains, records)) => Message::DataReloaded(domains, records),
                    Err(e) => Message::ShowToast(format!("重新加载数据失败: {}", e)),
                }
            )
        )
    }
    
    /// 异步同步单个域名
    async fn sync_domain_async(domain: String) -> Result<Vec<DnsRecord>, String> {
        // 模拟网络延迟
        sleep(Duration::from_millis(1200)).await;
        
        // 这里应该调用实际的DNS同步服务
        // 暂时返回模拟数据
        let records = vec![
            DnsRecord {
                record_id: format!("{}-a-sync", domain),
                domain_name: domain.clone(),
                record_type: "A".to_string(),
                name: "@".to_string(),
                value: "192.168.1.100".to_string(),
                ttl: 600,
                status: "Active".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            DnsRecord {
                record_id: format!("{}-mx-sync", domain),
                domain_name: domain.clone(),
                record_type: "MX".to_string(),
                name: "@".to_string(),
                value: format!("mail.{}", domain),
                ttl: 3600,
                status: "Active".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ];
        
        // 模拟可能的失败情况
        if domain.contains("error") {
            return Err("模拟同步失败".to_string());
        }
        
        Ok(records)
    }
    
    /// 异步批量同步所有域名
    async fn sync_all_domains_async(domains: Vec<String>) -> BatchSyncResult {
        let mut successful = Vec::new();
        let mut failed = Vec::new();
        let mut cancelled = Vec::new();
        let mut total_records = 0;
        
        // 并发同步所有域名（限制并发数）
        let semaphore = tokio::sync::Semaphore::new(3); // 最多3个并发
        let mut tasks = Vec::new();
        
        for domain in domains.iter().cloned() {
            let permit = semaphore.clone();
            let task = tokio::spawn(async move {
                let _permit = permit.acquire().await.unwrap();
                let result = Self::sync_domain_async(domain.clone()).await;
                (domain, result)
            });
            tasks.push(task);
        }
        
        // 等待所有任务完成
        for task in tasks {
            match task.await {
                Ok((domain, result)) => {
                    match result {
                        Ok(records) => {
                            let count = records.len();
                            total_records += count;
                            successful.push(SyncResult::Success {
                                domain,
                                records_count: count,
                            });
                        },
                        Err(error) => {
                            failed.push(SyncResult::Failed { domain, error });
                        }
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
    async fn reload_data_async() -> Result<(Vec<Domain>, HashMap<String, Vec<DnsRecord>>), String> {
        // 模拟网络延迟
        sleep(Duration::from_millis(800)).await;
        
        // 这里应该从数据库或API重新加载数据
        // 暂时返回模拟数据
        let domains = vec![
            Domain {
                domain_id: "reload-1".to_string(),
                domain_name: "reloaded-example.com".to_string(),
                provider: "Aliyun".to_string(),
                status: "Active".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ];
        
        let mut records = HashMap::new();
        for domain in &domains {
            let domain_records = vec![
                DnsRecord {
                    record_id: format!("{}-reload-1", domain.domain_name),
                    domain_name: domain.domain_name.clone(),
                    record_type: "A".to_string(),
                    name: "@".to_string(),
                    value: "192.168.1.200".to_string(),
                    ttl: 600,
                    status: "Active".to_string(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                },
            ];
            records.insert(domain.domain_name.clone(), domain_records);
        }
        
        Ok((domains, records))
    }
}

impl EventHandler<SyncMessage> for SyncHandler {
    fn handle(&self, state: &mut AppState, event: SyncMessage) -> HandlerResult {
        match event {
            SyncMessage::SyncDomain(domain) => {
                self.handle_sync_domain(state, domain)
            },
            SyncMessage::SyncAllDomains => {
                self.handle_sync_all_domains(state)
            },
            SyncMessage::SyncComplete { domain, result } => {
                self.handle_sync_complete(state, domain, result)
            },
            SyncMessage::AllSyncComplete(result) => {
                self.handle_all_sync_complete(state, result)
            },
            SyncMessage::CancelSync(domain) => {
                self.handle_cancel_sync(state, domain)
            },
            SyncMessage::Reload => {
                self.handle_reload(state)
            },
        }
    }
    
    fn can_handle(&self, event: &SyncMessage) -> bool {
        // 同步处理器可以处理所有同步相关的消息
        true
    }
}

impl AsyncEventHandler<SyncMessage> for SyncHandler {
    fn handle_async(&self, state: &mut AppState, event: SyncMessage) -> Task<Message> {
        match event {
            SyncMessage::SyncDomain(domain) => {
                Task::perform(
                    Self::sync_domain_async(domain.clone()),
                    move |result| Message::DomainSyncComplete(domain, result)
                )
            },
            SyncMessage::SyncAllDomains => {
                let domains: Vec<String> = state.data.domains
                    .iter()
                    .map(|d| d.domain_name.clone())
                    .collect();
                
                Task::perform(
                    Self::sync_all_domains_async(domains),
                    |result| Message::AllDomainsSyncComplete(result)
                )
            },
            SyncMessage::Reload => {
                Task::perform(
                    Self::reload_data_async(),
                    |result| match result {
                        Ok((domains, records)) => Message::DataReloaded(domains, records),
                        Err(e) => Message::ShowToast(format!("重新加载数据失败: {}", e)),
                    }
                )
            },
            _ => {
                // 其他消息不需要异步处理
                Task::none()
            }
        }
    }
}

impl Default for SyncHandler {
    fn default() -> Self {
        Self::new()
    }
}