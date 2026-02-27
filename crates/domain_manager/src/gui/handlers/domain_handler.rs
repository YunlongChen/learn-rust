//! 域名处理器
//!
//! 负责处理所有与域名相关的业务逻辑，包括域名的增删改查、
//! 选择、搜索等操作。

use super::message_handler::{
    AddDomainFormMessage, DnsMessage, DomainMessage, MessageCategory, NotificationMessage,
};
use super::{EventHandler, HandlerResult};
use crate::gui::model::domain::{DnsProvider, Domain};
use crate::gui::pages::domain::DomainProvider;
use crate::gui::pages::names::Page;
use crate::gui::state::app_state::{DataUpdate, StateUpdate, UiUpdate};
use crate::gui::state::AppState;
use crate::gui::types::credential::{ApiKeyCredential, Credential};
use crate::model::dns_record_response::{Line, Record};
use crate::storage::DomainModal;
use iced::Task;
use tracing::{info, warn};

/// 域名处理器
///
/// 专门处理域名相关的事件和业务逻辑
#[derive(Debug)]
pub struct DomainHandler {
    // 可以添加域名服务的依赖
}

impl DomainHandler {
    /// 创建新的域名处理器
    pub fn new() -> Self {
        Self {}
    }

    /// 处理域名选择
    fn handle_domain_selected(&self, state: &mut AppState, domain_id: usize) -> HandlerResult {
        info!("选择域名: {}", domain_id);

        // 查找域名
        if let Some(domain) = state
            .data
            .domain_list
            .iter()
            .find(|d| d.id as usize == domain_id)
            .cloned()
        {
            // 更新选中的域名
            state.update(StateUpdate::Data(DataUpdate::SelectDomain(domain.clone())));

            // 设置过滤器中的选中域名
            state.ui.selected_domain = Some(domain.clone());

            // 切换到DNS记录页面并查询DNS记录
            state.ui.current_page = Page::DnsRecord;

            // 触发DNS记录查询消息，交给DnsHandler处理
            info!("触发DNS记录查询消息: {}", domain_id);
            return HandlerResult::Task(Task::done(MessageCategory::Dns(DnsMessage::QueryRecord(
                domain_id,
            ))));
        } else {
            warn!("域名 {} 不存在", domain_id);
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(format!(
                "域名 {} 不存在",
                domain_id
            ))));
            HandlerResult::StateUpdated
        }
    }

    /// 处理添加域名表单变更
    fn handle_add_form_changed(
        &self,
        state: &mut AppState,
        message: AddDomainFormMessage,
    ) -> HandlerResult {
        match message {
            AddDomainFormMessage::Submit => {
                info!("提交创建域名表单");
            }
            AddDomainFormMessage::ProviderChanged(provider) => {
                info!("域名提供商发生了变更");
                // 更新添加域名表单的域名字段
                state.ui.add_domain_field.provider = provider;
            }
        }
        HandlerResult::StateUpdated
    }

    /// 处理提交域名表单
    fn handle_submit_form(&self, state: &mut AppState) -> HandlerResult {
        let domain_name = state.ui.add_domain_field.domain_name.clone();
        let provider = state.ui.add_domain_field.provider.clone();

        info!(
            "提交域名表单：添加域名「{}」，托管商类型：「{:?}」",
            domain_name, provider
        );

        // 验证表单数据
        if domain_name.is_empty() {
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                "请输入域名".to_string(),
            )));
            return HandlerResult::StateUpdated;
        }

        if provider.is_none() {
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                "请选择托管商".to_string(),
            )));
            return HandlerResult::StateUpdated;
        }

        // 检查域名是否已存在
        if state.data.domain_list.iter().any(|d| d.name == domain_name) {
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(format!(
                "域名 {} 已存在",
                domain_name
            ))));
            return HandlerResult::StateUpdated;
        }

        // 创建新域名
        let new_domain = DomainModal {
            id: 0, // 临时ID，实际应该由数据库生成
            name: domain_name.clone(),
            provider_id: 1, // 临时值，应该从选中的provider获取
            status: "active".to_string(),
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: Some(chrono::Utc::now().naive_utc()),
        };

        // 添加到域名列表
        state.data.domain_list.push(new_domain);

        // 清空表单
        state.ui.add_domain_field.clear();

        // 切换到域名页面
        state.ui.current_page = Page::DomainPage;

        info!("域名 {} 添加成功", domain_name);
        state.update(StateUpdate::Ui(UiUpdate::ShowToast(format!(
            "域名 {} 添加成功",
            domain_name
        ))));

        HandlerResult::StateUpdated
    }

    /// 处理域名删除请求
    fn handle_delete_request(&self, state: &mut AppState, domain_id: usize) -> HandlerResult {
        state.data.deleting_domain_id = Some(domain_id);
        HandlerResult::StateUpdated
    }

    /// 处理域名删除取消
    fn handle_delete_cancel(&self, state: &mut AppState) -> HandlerResult {
        state.data.deleting_domain_id = None;
        HandlerResult::StateUpdated
    }

    /// 处理删除域名
    fn handle_delete_domain(&self, state: &mut AppState, domain_id: usize) -> HandlerResult {
        info!("删除域名: {}", domain_id);

        // 重置删除状态
        state.data.deleting_domain_id = None;

        // 检查域名是否存在
        if !state
            .data
            .domain_list
            .iter()
            .any(|domain| domain.id as usize == domain_id)
        {
            warn!("尝试删除不存在的域名: {}", domain_id);
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(format!(
                "域名 {} 不存在",
                domain_id
            ))));
            return HandlerResult::StateUpdated;
        }

        // 从域名列表中移除
        state
            .data
            .domain_list
            .retain(|d| d.id as usize != domain_id);

        // 清除DNS记录缓存
        state.data.dns_records_cache.remove(&domain_id);

        // 如果删除的是当前选中的域名，清除选中状态
        if let Some(selected) = &state.data.selected_domain {
            if selected.id as usize == domain_id {
                state.data.selected_domain = None;
                state.ui.selected_domain = None;
                state.data.dns_list.clear();
            }
        }

        info!("域名 {} 删除成功", domain_id);
        state.update(StateUpdate::Ui(UiUpdate::ShowToast(format!(
            "域名 {} 删除成功",
            domain_id
        ))));

        HandlerResult::StateUpdated
    }

    /// 处理查询域名
    fn handle_query_domain(&self, state: &mut AppState, domain_name: String) -> HandlerResult {
        info!("查询域名: {}", domain_name);

        // 设置查询状态
        state.ui.in_query = true;

        // 返回查询域名的异步任务
        HandlerResult::StateUpdatedWithTask(Task::perform(
            Self::query_domain_async(domain_name.clone()),
            move |result| match result {
                Ok(domains) => {
                    let _provider = DomainProvider {
                        account_id: 1,
                        provider_name: "阿里云".to_string(),
                        provider: DnsProvider::Aliyun,
                        credential: Credential::ApiKey(ApiKeyCredential {
                            api_key: "mock_key".to_string(),
                            api_secret: "mock_secret".to_string(),
                        }),
                        is_expanded: false,
                        is_adding_domain: false,
                        new_domain_name: String::new(),
                        domains: vec![],
                        status: crate::gui::pages::domain::ProviderStatus::Inactive,
                        last_synced_at: None,
                        domain_count: 0,
                    };
                    MessageCategory::Domain(DomainMessage::QueryDomainResult(domains))
                }
                Err(e) => MessageCategory::Notification(NotificationMessage::ShowToast(format!(
                    "查询域名失败: {}",
                    e
                ))),
            },
        ))
    }

    /// 异步查询DNS记录
    async fn query_dns_records_async(domain_id: usize) -> Result<Vec<Record>, String> {
        use crate::model::dns_record_response::Record;

        info!("开始异步查询域名 {} 的DNS记录", domain_id);

        // 模拟DNS查询延迟
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // 这里应该调用实际的DNS查询服务
        // 暂时返回模拟数据
        let mock_records = vec![
            Record {
                status: crate::model::dns_record_response::Status::Enable,
                rr: "@".to_string(),
                record_type: crate::model::dns_record_response::Type::A,
                value: "192.168.1.1".to_string(),
                record_id: "1".to_string(),
                ttl: 600,
                line: Line::Default,
                locked: false,
                update_timestamp: Some(123),
                create_timestamp: 1223,
                weight: Some(1),
            },
            Record {
                status: crate::model::dns_record_response::Status::Enable,
                rr: "www".to_string(),
                record_type: crate::model::dns_record_response::Type::Cname,
                value: "127.0.0.1".to_string(),
                record_id: "2".to_string(),
                ttl: 600,
                line: Line::Default,
                locked: false,
                update_timestamp: None,
                create_timestamp: chrono::Utc::now().timestamp(),
                weight: None,
            },
        ];

        info!(
            "域名 {} 的DNS记录查询完成，共 {} 条记录",
            domain_id,
            mock_records.len()
        );
        Ok(mock_records)
    }

    /// 异步查询域名
    async fn query_domain_async(domain_name: String) -> Result<Vec<Domain>, String> {
        info!("开始异步查询域名: {}", domain_name);

        // 模拟域名查询延迟
        tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

        // 这里应该调用实际的域名查询服务
        // 暂时返回模拟数据
        let mock_domain = Domain {
            id: 1,
            name: domain_name.clone(),
            provider: crate::gui::model::domain::DnsProvider::Aliyun,
            status: crate::gui::model::domain::DomainStatus::Active,
            expiry: "2025-12-31".to_string(),
            records: vec![],
        };
        let mock_domains = vec![mock_domain];

        info!("域名查询完成，找到 {} 个域名", mock_domains.len());
        Ok(mock_domains)
    }
}

impl EventHandler<DomainMessage> for DomainHandler {
    fn handle(&self, state: &mut AppState, event: DomainMessage) -> HandlerResult {
        match event {
            DomainMessage::Selected(domain) => {
                self.handle_domain_selected(state, domain.id as usize)
            }
            DomainMessage::AddFormChanged(message) => self.handle_add_form_changed(state, message),
            DomainMessage::SubmitForm => self.handle_submit_form(state),
            DomainMessage::Delete(domain_id) => self.handle_delete_domain(state, domain_id),
            DomainMessage::DeleteRequest(domain_id) => self.handle_delete_request(state, domain_id),
            DomainMessage::DeleteCancel => self.handle_delete_cancel(state),
            DomainMessage::Query(domain_name) => self.handle_query_domain(state, domain_name),
            DomainMessage::Reload => HandlerResult::StateUpdated,
            DomainMessage::QueryDomainResult(_) => todo!(),
        }
    }

    /// 检查是否可以处理该消息
    fn can_handle(&self, _event: &DomainMessage) -> bool {
        true // DomainHandler可以处理所有DomainMessage
    }
}

impl Default for DomainHandler {
    fn default() -> Self {
        Self::new()
    }
}
