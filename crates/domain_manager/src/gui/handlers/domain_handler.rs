//! 域名处理器
//! 
//! 负责处理所有与域名相关的业务逻辑，包括域名的增删改查、
//! 选择、搜索等操作。

use super::{HandlerResult, EventHandler};
use super::message_handler::DomainMessage;
use crate::gui::state::{AppState, StateUpdate, DataUpdate, UiUpdate};
use crate::gui::Message;
use crate::models::Domain;
use crate::storage::DatabaseConnection;
use iced::Task;

/// 域名处理器
/// 
/// 专门处理域名相关的事件和业务逻辑
pub struct DomainHandler {
    // 可以添加域名服务的依赖
}

impl DomainHandler {
    /// 创建新的域名处理器
    pub fn new() -> Self {
        Self {}
    }
    
    /// 处理域名选择
    fn handle_domain_selected(&self, state: &mut AppState, domain_name: String) -> HandlerResult {
        // 查找域名
        if let Some(domain) = state.data.domain_list.iter().find(|d| d.name == domain_name).cloned() {
            state.update(StateUpdate::Data(DataUpdate::SelectDomain(domain)));
            
            // 如果有DNS记录缓存，直接显示；否则需要查询
            if !state.data.dns_records_cache.contains_key(&domain_name) {
                // 返回查询DNS记录的任务
                return HandlerResult::StateUpdatedWithTask(
                    Task::perform(
                        Self::query_dns_records_async(domain_name.clone()),
                        move |records| Message::QueryDnsResult(domain_name.clone(), records)
                    )
                );
            }
            
            HandlerResult::StateUpdated
        } else {
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                format!("域名 {} 不存在", domain_name)
            )));
            HandlerResult::StateUpdated
        }
    }
    
    /// 处理添加域名表单变更
    fn handle_add_form_changed(&self, state: &mut AppState, content: String) -> HandlerResult {
        // 更新表单内容（这里可能需要在状态中添加表单状态）
        state.ui.set_message(format!("域名表单内容: {}", content));
        HandlerResult::StateUpdated
    }
    
    /// 处理提交域名表单
    fn handle_submit_form(&self, state: &mut AppState) -> HandlerResult {
        // 这里需要从表单状态中获取域名信息
        // 暂时使用示例数据
        let domain_name = "example.com".to_string();
        
        // 检查域名是否已存在
        if state.data.domain_list.iter().any(|d| d.name == domain_name) {
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                format!("域名 {} 已存在", domain_name)
            )));
            return HandlerResult::StateUpdated;
        }
        
        // 创建新域名
        let new_domain = Domain {
            id: None,
            name: domain_name.clone(),
            provider: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        // 返回添加域名的异步任务
        HandlerResult::Task(
            Task::perform(
                Self::add_domain_async(new_domain),
                |result| match result {
                    Ok(domain) => Message::DomainAdded(domain),
                    Err(e) => Message::ShowToast(format!("添加域名失败: {}", e)),
                }
            )
        )
    }
    
    /// 处理删除域名
    fn handle_delete_domain(&self, state: &mut AppState, domain_name: String) -> HandlerResult {
        // 检查域名是否存在
        if !state.data.domain_list.iter().any(|d| d.name == domain_name) {
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                format!("域名 {} 不存在", domain_name)
            )));
            return HandlerResult::StateUpdated;
        }
        
        // 返回删除域名的异步任务
        HandlerResult::Task(
            Task::perform(
                Self::delete_domain_async(domain_name.clone()),
                move |result| match result {
                    Ok(_) => Message::DomainDeleteComplete(domain_name),
                    Err(e) => Message::ShowToast(format!("删除域名失败: {}", e)),
                }
            )
        )
    }
    
    /// 处理查询域名
    fn handle_query_domain(&self, _state: &mut AppState, domain_name: String) -> HandlerResult {
        // 返回查询域名的异步任务
        HandlerResult::Task(
            Task::perform(
                Self::query_domain_async(domain_name),
                |result| match result {
                    Ok(domain) => Message::QueryDomainResult(Ok(domain)),
                    Err(e) => Message::QueryDomainResult(Err(e)),
                }
            )
        )
    }
    
    /// 异步查询DNS记录
    async fn query_dns_records_async(domain_name: String) -> Result<Vec<crate::models::DnsRecord>, String> {
        // 这里应该调用实际的DNS查询服务
        // 暂时返回空结果
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        Ok(Vec::new())
    }
    
    /// 异步添加域名
    async fn add_domain_async(domain: Domain) -> Result<Domain, String> {
        // 这里应该调用数据库服务添加域名
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        Ok(domain)
    }
    
    /// 异步删除域名
    async fn delete_domain_async(domain_name: String) -> Result<(), String> {
        // 这里应该调用数据库服务删除域名
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        Ok(())
    }
    
    /// 异步查询域名
    async fn query_domain_async(domain_name: String) -> Result<Domain, String> {
        // 这里应该调用域名查询服务
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        Ok(Domain {
            id: Some(1),
            name: domain_name,
            provider: Some("阿里云".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
}

impl EventHandler<DomainMessage> for DomainHandler {
    fn handle(&self, state: &mut AppState, event: DomainMessage) -> HandlerResult {
        match event {
            DomainMessage::Selected(domain_name) => {
                self.handle_domain_selected(state, domain_name)
            },
            DomainMessage::AddFormChanged(content) => {
                self.handle_add_form_changed(state, content)
            },
            DomainMessage::SubmitForm => {
                self.handle_submit_form(state)
            },
            DomainMessage::Delete(domain_name) => {
                self.handle_delete_domain(state, domain_name)
            },
            DomainMessage::Query(domain_name) => {
                self.handle_query_domain(state, domain_name)
            },
        }
    }
    
    fn can_handle(&self, event: &DomainMessage) -> bool {
        // 域名处理器可以处理所有域名相关的消息
        true
    }
}

impl Default for DomainHandler {
    fn default() -> Self {
        Self::new()
    }
}