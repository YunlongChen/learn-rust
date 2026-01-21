//! DNS处理器
//!
//! 负责处理所有与DNS记录相关的业务逻辑，包括DNS记录的增删改查、
//! 提供商管理等操作。

use super::message_handler::{DnsMessage, MessageCategory, NavigationMessage, NotificationMessage};
use super::{EventHandler, HandlerResult};
use crate::gui::model::form::AddDnsField;
use crate::gui::pages::Page;
use crate::gui::state::app_state::{StateUpdate, UiUpdate};
use crate::gui::state::AppState;
use crate::model::dns_record_response::Type as RecordType;
use crate::storage::DnsRecordModal;
use iced::Task;
use tracing::{info, warn};

/// DNS处理器
///
/// 专门处理DNS记录相关的事件和业务逻辑
#[derive(Debug)]
pub struct DnsHandler {
    // 可以添加DNS服务的依赖
}

impl DnsHandler {
    /// 创建新的DNS处理器
    pub fn new() -> Self {
        Self {}
    }

    /// 处理查询DNS记录
    fn handle_query_record(&self, state: &mut AppState, domain_id: usize) -> HandlerResult {
        // 检查缓存中是否已有记录
        if state.data.dns_records_cache.contains_key(&domain_id) {
            state
                .ui
                .set_message(format!("DNS记录已缓存: {}", domain_id));
            return HandlerResult::StateUpdated;
        }

        // 设置加载状态
        state
            .ui
            .set_message(format!("正在查询 {} 的DNS记录...", domain_id));

        // 返回查询DNS记录的异步任务
        HandlerResult::StateUpdatedWithTask(Task::perform(
            Self::query_dns_records_async(domain_id.clone()),
            move |result| match result {
                Ok(records) => {
                    MessageCategory::Dns(DnsMessage::DnsRecordReloaded(domain_id, records))
                }
                Err(e) => MessageCategory::Notification(NotificationMessage::ShowToast(format!(
                    "查询DNS记录失败: {}",
                    e
                ))),
            },
        ))
    }

    /// 处理添加DNS记录
    fn handle_add_record(
        &self,
        state: &mut AppState,
        domain_id: usize,
        record_type: String,
        name: String,
        value: String,
        ttl: u32,
    ) -> HandlerResult {
        // 验证输入
        if record_type.is_empty() || value.is_empty() {
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                "请填写完整的DNS记录信息".to_string(),
            )));
            return HandlerResult::StateUpdated;
        }

        // 创建新的DNS记录
        let new_record = DnsRecordModal {
            id: 0,        // 临时ID，数据库会自动生成
            domain_id: 0, // 需要根据域名查找对应的domain_id
            record_type: record_type.clone(),
            name: name.clone(),
            value: value.clone(),
            ttl: ttl.try_into().unwrap(),
            priority: None,
            enabled: true,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: Some(chrono::Utc::now().naive_utc()),
        };

        state.ui.set_message(format!(
            "正在添加DNS记录: {} {} {}",
            record_type, name, value
        ));

        // 返回添加DNS记录的异步任务
        HandlerResult::StateUpdatedWithTask(Task::perform(
            Self::add_dns_record_async(new_record),
            move |result| match result {
                Ok(_record) => MessageCategory::Notification(NotificationMessage::ShowToast(
                    format!("DNS记录已添加到域名: {}", domain_id),
                )),
                Err(e) => MessageCategory::Notification(NotificationMessage::ShowToast(format!(
                    "添加DNS记录失败: {}",
                    e
                ))),
            },
        ))
    }

    /// 处理删除DNS记录
    fn handle_delete_record(
        &self,
        state: &mut AppState,
        domain: usize,
        record_id: usize,
    ) -> HandlerResult {
        // 检查记录是否存在
        let record_exists = state
            .data
            .dns_records_cache
            .get(&domain)
            .map_or(false, |records| {
                records
                    .iter()
                    .any(|record_model| record_model.id as usize == record_id)
            });

        if !record_exists {
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                "DNS记录不存在".to_string(),
            )));
            return HandlerResult::StateUpdated;
        }

        state
            .ui
            .set_message(format!("正在删除DNS记录: {}", record_id));

        // 返回删除DNS记录的异步任务
        let domain_for_async = domain.clone();
        let domain_for_message = domain.clone();
        HandlerResult::StateUpdatedWithTask(Task::perform(
            Self::delete_dns_record_async(domain_for_async, record_id.clone()),
            move |result| match result {
                Ok(_) => MessageCategory::Dns(DnsMessage::Delete(domain_for_message.clone())),
                Err(e) => MessageCategory::Notification(NotificationMessage::ShowToast(format!(
                    "删除DNS记录失败: {}",
                    e
                ))),
            },
        ))
    }

    /// 处理DNS提供商选择
    fn handle_provider_selected(&self, state: &mut AppState, provider_id: usize) -> HandlerResult {
        // 查找提供商
        if let Some(provider) = state
            .data
            .domain_providers
            .iter()
            .find(|provider| provider.account_id as usize == provider_id)
            .cloned()
        {
            state.data.selected_provider = Some(provider.clone());
            state.ui.set_message(format!(
                "已选择DNS提供商，标识：{},名称：{}",
                provider.account_id, provider.provider_name
            ));
            HandlerResult::StateUpdated
        } else {
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(format!(
                "DNS提供商 {} 不存在",
                provider_id
            ))));
            HandlerResult::StateUpdated
        }
    }

    /// 处理DNS提供商变更
    fn handle_provider_change(&self, state: &mut AppState, provider_name: String) -> HandlerResult {
        // 更新当前选中的提供商
        state
            .ui
            .set_message(format!("切换到DNS提供商: {}", provider_name));

        // 这里可能需要重新加载相关数据
        HandlerResult::StateUpdated
    }

    /// 处理DNS记录删除（原版Message::DnsDelete）
    fn handle_dns_delete(&self, _state: &mut AppState, record_id: usize) -> HandlerResult {
        info!("删除DNS记录: {}", record_id);

        // 返回删除DNS记录的异步任务
        HandlerResult::Task(Task::perform(
            Self::handle_dns_record_delete_async(record_id.clone()),
            move |result| match result {
                Some(deleted_id) => MessageCategory::Dns(DnsMessage::RecordDeleted(deleted_id)),
                None => MessageCategory::Navigation(NavigationMessage::Back),
            },
        ))
    }

    /// 处理DNS表单名称变更
    fn handle_form_name_changed(&self, state: &mut AppState, record_name: String) -> HandlerResult {
        info!("DNS记录表单名称变更: {}", record_name);
        state.data.add_dns_form.record_name = record_name;
        HandlerResult::StateUpdated
    }

    /// 处理DNS表单记录类型变更
    fn handle_form_record_type_changed(
        &self,
        state: &mut AppState,
        record_type: RecordType,
    ) -> HandlerResult {
        info!("DNS记录表单类型变更: {:?}", record_type);
        state.data.add_dns_form.record_type = Some(record_type);
        HandlerResult::StateUpdated
    }

    /// 处理DNS表单值变更
    fn handle_form_value_changed(&self, state: &mut AppState, value: String) -> HandlerResult {
        info!("DNS记录表单值变更: {}", value);
        state.data.add_dns_form.value = value;
        HandlerResult::StateUpdated
    }

    /// 处理DNS表单TTL变更
    fn handle_form_ttl_changed(&self, state: &mut AppState, ttl: i32) -> HandlerResult {
        info!("DNS记录表单TTL变更: {}", ttl);
        state.data.add_dns_form.ttl = ttl;
        HandlerResult::StateUpdated
    }

    /// 处理DNS表单提交
    fn handle_form_submit(&self, state: &mut AppState) -> HandlerResult {
        info!("提交DNS记录表单");

        // 验证表单
        if !state.data.add_dns_form.validate() {
            warn!("DNS记录表单验证失败");
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                "请填写完整的DNS记录信息".to_string(),
            )));
            return HandlerResult::StateUpdated;
        }

        // 返回添加DNS记录的异步任务
        let form_data = state.data.add_dns_form.clone();
        HandlerResult::Task(Task::perform(
            Self::handle_dns_record_add_async(form_data),
            |result| {
                info!("DNS记录添加结果: {:?}", result);
                MessageCategory::Navigation(NavigationMessage::PageChanged(Page::AddRecord))
            },
        ))
    }

    /// 处理DNS表单取消
    fn handle_form_cancelled(&self, state: &mut AppState) -> HandlerResult {
        info!("取消DNS记录表单");

        // 清空表单
        state.data.add_dns_form = Default::default();

        // 返回到DNS记录页面
        state.update(StateUpdate::Navigation(Page::DnsRecord));
        HandlerResult::StateUpdated
    }

    /// 处理DNS记录删除完成
    fn handle_record_deleted(&self, state: &mut AppState, record_id: usize) -> HandlerResult {
        info!("DNS记录删除完成: {}", record_id);

        // 从DNS记录列表中移除
        state
            .data
            .dns_list
            .retain(|record| record.id as usize != record_id);

        // 返回到DNS记录页面
        state.update(StateUpdate::Navigation(Page::DnsRecord));
        HandlerResult::StateUpdated
    }

    /// 异步查询DNS记录
    async fn query_dns_records_async(domain_id: usize) -> Result<Vec<DnsRecordModal>, String> {
        // 模拟网络延迟
        tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

        // 这里应该调用实际的DNS查询服务
        // 暂时返回模拟数据
        let records = vec![
            DnsRecordModal {
                id: 1,
                domain_id: domain_id.clone() as i64,
                record_type: "A".to_string(),
                name: "@".to_string(),
                value: "192.168.1.1".to_string(),
                ttl: 600,
                priority: None,
                enabled: true,
                created_at: chrono::Utc::now().naive_utc(),
                updated_at: Some(chrono::Utc::now().naive_utc()),
            },
            DnsRecordModal {
                id: 2,
                domain_id: domain_id.clone() as i64,
                record_type: "CNAME".to_string(),
                name: "www".to_string(),
                value: "127.0.0.1".to_string(),
                ttl: 600,
                priority: None,
                enabled: true,
                created_at: chrono::Utc::now().naive_utc(),
                updated_at: Some(chrono::Utc::now().naive_utc()),
            },
        ];

        Ok(records)
    }

    /// 异步添加DNS记录
    async fn add_dns_record_async(record: DnsRecordModal) -> Result<DnsRecordModal, String> {
        // 模拟网络延迟
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // 这里应该调用实际的DNS服务API
        // 暂时直接返回记录
        Ok(record)
    }

    /// 异步删除DNS记录
    async fn delete_dns_record_async(_domain_id: usize, _record_id: usize) -> Result<(), String> {
        // 模拟网络延迟
        tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;

        // 这里应该调用实际的DNS服务API
        // 暂时直接返回成功
        Ok(())
    }

    /// 异步更新DNS记录
    async fn update_dns_record_async(record: DnsRecordModal) -> Result<DnsRecordModal, String> {
        // 模拟网络延迟
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // 这里应该调用实际的DNS服务API
        let mut updated_record = record;
        updated_record.updated_at = Some(chrono::Utc::now().naive_utc());

        Ok(updated_record)
    }

    /// 异步同步DNS记录
    async fn sync_dns_records_async(domain: usize) -> Result<Vec<DnsRecordModal>, String> {
        // 模拟网络延迟
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        // 这里应该调用实际的DNS同步服务
        Self::query_dns_records_async(domain).await
    }

    /// 异步删除DNS记录（原版handle_dns_record_delete）
    async fn handle_dns_record_delete_async(record_id: usize) -> Option<usize> {
        info!("开始异步删除DNS记录: {}", record_id);

        // 模拟网络延迟
        tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;

        // 这里应该调用实际的DNS服务API删除记录
        // 暂时模拟删除成功
        info!("DNS记录删除成功: {}", record_id);
        Some(record_id)
    }

    /// 异步添加DNS记录（原版handle_dns_record_add）
    async fn handle_dns_record_add_async(form_data: AddDnsField) -> Result<(), String> {
        info!("开始异步添加DNS记录: {:?}", form_data);

        // 模拟网络延迟
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // 这里应该调用实际的DNS服务API添加记录
        // 暂时模拟添加成功
        info!(
            "DNS记录添加成功: {} {} {}",
            form_data.record_name,
            form_data
                .record_type
                .map_or("Unknown".to_string(), |t| t.get_value().to_string()),
            form_data.value
        );
        Ok(())
    }
}

impl EventHandler<DnsMessage> for DnsHandler {
    fn handle(&self, state: &mut AppState, event: DnsMessage) -> HandlerResult {
        match event {
            DnsMessage::QueryRecord(domain_id) => self.handle_query_record(state, domain_id),
            DnsMessage::AddRecord {
                domain_id,
                record_type,
                name,
                value,
                ttl,
            } => self.handle_add_record(state, domain_id, record_type, name, value, ttl),
            DnsMessage::DeleteRecord {
                domain_id: domain,
                record_id,
            } => self.handle_delete_record(state, domain, record_id),
            DnsMessage::Delete(record_id) => self.handle_dns_delete(state, record_id),
            DnsMessage::FormNameChanged(record_name) => {
                self.handle_form_name_changed(state, record_name)
            }
            DnsMessage::FormRecordTypeChanged(record_type) => {
                self.handle_form_record_type_changed(state, record_type)
            }
            DnsMessage::FormValueChanged(value) => self.handle_form_value_changed(state, value),
            DnsMessage::FormTtlChanged(ttl) => self.handle_form_ttl_changed(state, ttl),
            DnsMessage::FormSubmit => self.handle_form_submit(state),
            DnsMessage::FormCancelled => self.handle_form_cancelled(state),
            DnsMessage::RecordDeleted(record_id) => self.handle_record_deleted(state, record_id),
            DnsMessage::ProviderSelected(account_id) => {
                self.handle_provider_selected(state, account_id)
            }
            DnsMessage::ProviderChange(provider) => self.handle_provider_change(state, provider),
            _ => HandlerResult::None,
        }
    }

    /// 检查是否可以处理该消息
    fn can_handle(&self, _event: &DnsMessage) -> bool {
        // DNS处理器可以处理所有DNS相关的消息
        true
    }
}

impl Default for DnsHandler {
    fn default() -> Self {
        Self::new()
    }
}
