//! DNS处理器
//! 
//! 负责处理所有与DNS记录相关的业务逻辑，包括DNS记录的增删改查、
//! 提供商管理等操作。

use super::{HandlerResult, EventHandler};
use super::message_handler::DnsMessage;
use crate::gui::state::{AppState, StateUpdate, DataUpdate, UiUpdate};
use crate::gui::Message;
use crate::models::DnsRecord;
use crate::dns::DnsProvider;
use iced::Task;

/// DNS处理器
/// 
/// 专门处理DNS记录相关的事件和业务逻辑
pub struct DnsHandler {
    // 可以添加DNS服务的依赖
}

impl DnsHandler {
    /// 创建新的DNS处理器
    pub fn new() -> Self {
        Self {}
    }
    
    /// 处理查询DNS记录
    fn handle_query_record(&self, state: &mut AppState, domain: String) -> HandlerResult {
        // 检查缓存中是否已有记录
        if state.data.dns_records_cache.contains_key(&domain) {
            state.ui.set_message(format!("DNS记录已缓存: {}", domain));
            return HandlerResult::StateUpdated;
        }
        
        // 设置加载状态
        state.ui.set_message(format!("正在查询 {} 的DNS记录...", domain));
        
        // 返回查询DNS记录的异步任务
        HandlerResult::StateUpdatedWithTask(
            Task::perform(
                Self::query_dns_records_async(domain.clone()),
                move |result| match result {
                    Ok(records) => Message::DnsRecordsLoaded(domain, records),
                    Err(e) => Message::ShowToast(format!("查询DNS记录失败: {}", e)),
                }
            )
        )
    }
    
    /// 处理添加DNS记录
    fn handle_add_record(
        &self,
        state: &mut AppState,
        domain: String,
        record_type: String,
        name: String,
        value: String,
        ttl: u32,
    ) -> HandlerResult {
        // 验证输入
        if domain.is_empty() || record_type.is_empty() || value.is_empty() {
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                "请填写完整的DNS记录信息".to_string()
            )));
            return HandlerResult::StateUpdated;
        }
        
        // 创建新的DNS记录
        let new_record = DnsRecord {
            record_id: format!("{}-{}-{}", domain, record_type, chrono::Utc::now().timestamp()),
            domain_name: domain.clone(),
            record_type: record_type.clone(),
            name: name.clone(),
            value: value.clone(),
            ttl,
            status: "Active".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        state.ui.set_message(format!("正在添加DNS记录: {} {} {}", record_type, name, value));
        
        // 返回添加DNS记录的异步任务
        HandlerResult::StateUpdatedWithTask(
            Task::perform(
                Self::add_dns_record_async(new_record),
                move |result| match result {
                    Ok(record) => Message::DnsRecordAdded(domain, record),
                    Err(e) => Message::ShowToast(format!("添加DNS记录失败: {}", e)),
                }
            )
        )
    }
    
    /// 处理删除DNS记录
    fn handle_delete_record(
        &self,
        state: &mut AppState,
        domain: String,
        record_id: String,
    ) -> HandlerResult {
        // 检查记录是否存在
        let record_exists = state.data.dns_records_cache
            .get(&domain)
            .map_or(false, |records| records.iter().any(|r| r.record_id == record_id));
        
        if !record_exists {
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                "DNS记录不存在".to_string()
            )));
            return HandlerResult::StateUpdated;
        }
        
        state.ui.set_message(format!("正在删除DNS记录: {}", record_id));
        
        // 返回删除DNS记录的异步任务
        HandlerResult::StateUpdatedWithTask(
            Task::perform(
                Self::delete_dns_record_async(domain.clone(), record_id.clone()),
                move |result| match result {
                    Ok(_) => Message::DnsRecordDeleted(domain, record_id),
                    Err(e) => Message::ShowToast(format!("删除DNS记录失败: {}", e)),
                }
            )
        )
    }
    
    /// 处理DNS提供商选择
    fn handle_provider_selected(&self, state: &mut AppState, provider_name: String) -> HandlerResult {
        // 查找提供商
        if let Some(provider) = state.data.domain_providers.iter().find(|p| p.name == provider_name).cloned() {
            state.data.selected_provider = Some(provider.clone());
            state.ui.set_message(format!("已选择DNS提供商: {}", provider.name));
            HandlerResult::StateUpdated
        } else {
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                format!("DNS提供商 {} 不存在", provider_name)
            )));
            HandlerResult::StateUpdated
        }
    }
    
    /// 处理DNS提供商变更
    fn handle_provider_change(&self, state: &mut AppState, provider_name: String) -> HandlerResult {
        // 更新当前选中的提供商
        state.ui.set_message(format!("切换到DNS提供商: {}", provider_name));
        
        // 这里可能需要重新加载相关数据
        HandlerResult::StateUpdated
    }
    
    /// 异步查询DNS记录
    async fn query_dns_records_async(domain: String) -> Result<Vec<DnsRecord>, String> {
        // 模拟网络延迟
        tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
        
        // 这里应该调用实际的DNS查询服务
        // 暂时返回模拟数据
        let records = vec![
            DnsRecord {
                record_id: format!("{}-a-1", domain),
                domain_name: domain.clone(),
                record_type: "A".to_string(),
                name: "@".to_string(),
                value: "192.168.1.1".to_string(),
                ttl: 600,
                status: "Active".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            DnsRecord {
                record_id: format!("{}-cname-1", domain),
                domain_name: domain.clone(),
                record_type: "CNAME".to_string(),
                name: "www".to_string(),
                value: domain.clone(),
                ttl: 600,
                status: "Active".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ];
        
        Ok(records)
    }
    
    /// 异步添加DNS记录
    async fn add_dns_record_async(record: DnsRecord) -> Result<DnsRecord, String> {
        // 模拟网络延迟
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        // 这里应该调用实际的DNS服务API
        // 暂时直接返回记录
        Ok(record)
    }
    
    /// 异步删除DNS记录
    async fn delete_dns_record_async(domain: String, record_id: String) -> Result<(), String> {
        // 模拟网络延迟
        tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;
        
        // 这里应该调用实际的DNS服务API
        // 暂时直接返回成功
        Ok(())
    }
    
    /// 异步更新DNS记录
    async fn update_dns_record_async(record: DnsRecord) -> Result<DnsRecord, String> {
        // 模拟网络延迟
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        // 这里应该调用实际的DNS服务API
        let mut updated_record = record;
        updated_record.updated_at = chrono::Utc::now();
        
        Ok(updated_record)
    }
    
    /// 异步同步DNS记录
    async fn sync_dns_records_async(domain: String) -> Result<Vec<DnsRecord>, String> {
        // 模拟网络延迟
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        
        // 这里应该调用实际的DNS同步服务
        Self::query_dns_records_async(domain).await
    }
}

impl EventHandler<DnsMessage> for DnsHandler {
    fn handle(&self, state: &mut AppState, event: DnsMessage) -> HandlerResult {
        match event {
            DnsMessage::QueryRecord(domain) => {
                self.handle_query_record(state, domain)
            },
            DnsMessage::AddRecord { domain, record_type, name, value, ttl } => {
                self.handle_add_record(state, domain, record_type, name, value, ttl)
            },
            DnsMessage::DeleteRecord { domain, record_id } => {
                self.handle_delete_record(state, domain, record_id)
            },
            DnsMessage::ProviderSelected(provider) => {
                self.handle_provider_selected(state, provider)
            },
            DnsMessage::ProviderChange(provider) => {
                self.handle_provider_change(state, provider)
            },
        }
    }
    
    fn can_handle(&self, event: &DnsMessage) -> bool {
        // DNS处理器可以处理所有DNS相关的消息
        true
    }
}

impl Default for DnsHandler {
    fn default() -> Self {
        Self::new()
    }
}