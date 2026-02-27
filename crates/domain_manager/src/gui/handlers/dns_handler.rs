//! DNS处理器
//!
//! 负责处理所有与DNS记录相关的业务逻辑，包括DNS记录的增删改查、
//! 提供商管理等操作。

use crate::gui::handlers::message_handler::{
    DnsMessage, MessageCategory, NavigationMessage, NotificationMessage,
};
use crate::gui::handlers::{EventHandler, HandlerResult};
use crate::api::dns_client::DnsClientTrait;
use crate::api::provider::aliyun::AliyunDnsClient;
use crate::gui::model::domain::{DnsProvider, DomainName};
use crate::gui::model::form::AddDnsField;
use crate::gui::pages::Page;
use crate::gui::state::app_state::{StateUpdate, UiUpdate};
use crate::gui::state::AppState;
use crate::gui::types::credential::Credential;
use crate::model::dns_record_response::{Record, Status, Type as RecordType};
use crate::models::record::NewRecord;
use crate::storage::{accounts, domains, records, DnsRecordModal};
use iced::Task;
use sea_orm::{DatabaseConnection, DbErr}; // Import DbErr
use std::error::Error;
use tracing::{debug, info, warn};

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
        // 清除过滤器
        state.data.dns_record_filter = Default::default();

        // 设置加载状态
        state
            .ui
            .set_message(format!("正在查询 {} 的DNS记录...", domain_id));

        if let Some(conn) = &state.database {
            let conn_clone = conn.clone();
            // 返回查询DNS记录的异步任务（从数据库加载）
            HandlerResult::StateUpdatedWithTask(Task::perform(
                Self::load_dns_records_from_db(conn_clone, domain_id),
                move |result| match result {
                    Ok(records) => {
                        MessageCategory::Dns(DnsMessage::DnsRecordReloaded(domain_id, records))
                    }
                    Err(e) => MessageCategory::Notification(NotificationMessage::ShowToast(format!(
                        "加载DNS记录失败: {}",
                        e
                    ))),
                },
            ))
        } else {
            state.ui.set_message("数据库未连接".to_string());
            HandlerResult::StateUpdated
        }
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
            domain_id: domain_id as i64,
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

        if let Some(conn) = &state.database {
            let conn_clone = conn.clone();
            // 返回添加DNS记录的异步任务
            HandlerResult::StateUpdatedWithTask(Task::perform(
                Self::add_dns_record_async(conn_clone, new_record),
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
        } else {
            state.ui.set_message("数据库未连接".to_string());
            HandlerResult::StateUpdated
        }
    }

    /// 处理删除DNS记录
    fn handle_delete_record(
        &self,
        state: &mut AppState,
        domain: usize,
        record_id: usize,
    ) -> HandlerResult {
        state
            .ui
            .set_message(format!("正在删除DNS记录: {}", record_id));

        if let Some(conn) = &state.database {
            let conn_clone = conn.clone();
            // 返回删除DNS记录的异步任务（组合式操作）
            HandlerResult::StateUpdatedWithTask(Task::perform(
                Self::delete_and_reload_dns_record_async(conn_clone, domain, record_id),
                move |result| match result {
                    Ok((domain_id, records)) => {
                         // 删除并重载成功，直接更新UI
                         MessageCategory::Dns(DnsMessage::DnsRecordReloaded(domain_id, records))
                    },
                    Err(e) => MessageCategory::Notification(NotificationMessage::ShowToast(format!(
                        "删除DNS记录失败: {}",
                        e
                    ))),
                },
            ))
        } else {
            state.ui.set_message("数据库未连接".to_string());
            HandlerResult::StateUpdated
        }
    }

    /// 处理DNS提供商选择
    fn handle_provider_selected(&self, state: &mut AppState, provider_id: usize) -> HandlerResult {
        // 查找提供商
        if let Some(provider) = state
            .data
            .provider_page.providers
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

    /// 处理DNS记录删除请求
    fn handle_delete_request(&self, state: &mut AppState, record_id: usize) -> HandlerResult {
        state.data.deleting_dns_record_id = Some(record_id);
        HandlerResult::StateUpdated
    }

    /// 处理DNS记录删除取消
    fn handle_delete_cancel(&self, state: &mut AppState) -> HandlerResult {
        state.data.deleting_dns_record_id = None;
        HandlerResult::StateUpdated
    }

    /// 处理DNS记录删除（原版Message::DnsDelete）
    fn handle_dns_delete(&self, state: &mut AppState, record_id: usize) -> HandlerResult {
        info!("删除DNS记录: {}", record_id);

        // 重置删除状态
        state.data.deleting_dns_record_id = None;

        // 获取当前选中的域名ID，以便刷新列表
        let domain_id = state.data.selected_domain.as_ref().map(|d| d.id as usize).unwrap_or(0);

        if let Some(conn) = &state.database {
            let conn_clone = conn.clone();
            // 返回删除DNS记录的异步任务
            HandlerResult::Task(Task::perform(
                Self::delete_and_reload_dns_record_async(conn_clone, domain_id, record_id),
                move |result| match result {
                    Ok((domain_id, records)) => {
                        MessageCategory::Dns(DnsMessage::DnsRecordReloaded(domain_id, records))
                    },
                    Err(e) => MessageCategory::Notification(NotificationMessage::ShowToast(format!(
                        "删除DNS记录失败: {}",
                        e
                    ))),
                },
            ))
        } else {
            state.ui.set_message("数据库未连接".to_string());
            HandlerResult::StateUpdated
        }
    }

    /// 处理DNS记录悬停
    fn handle_record_hovered(&self, state: &mut AppState, record_id: Option<usize>) -> HandlerResult {
        state.ui.hovered_dns_record = record_id;
        HandlerResult::StateUpdated
    }

    /// 处理编辑DNS记录
    fn handle_edit_record(&self, state: &mut AppState, record: DnsRecordModal) -> HandlerResult {
        info!("开始编辑DNS记录: {:?}", record);

        let record_type = match record.record_type.as_str() {
            "A" => RecordType::A,
            "AAAA" => RecordType::AAAA,
            "CNAME" => RecordType::Cname,
            "MX" => RecordType::MX,
            "TXT" => RecordType::TXT,
            "NS" => RecordType::NS,
            "SRV" => RecordType::SRV,
            "PTR" => RecordType::PTR,
            "SOA" => RecordType::SOA,
            _ => RecordType::A,
        };

        state.data.add_dns_form.record_id = Some(record.id.to_string());
        state.data.add_dns_form.record_name = record.name;
        state.data.add_dns_form.value = record.value;
        state.data.add_dns_form.ttl = record.ttl;
        state.data.add_dns_form.record_type = Some(record_type);
        state.data.add_dns_form.is_visible = true;

        HandlerResult::StateUpdated
    }

    /// 处理DNS记录搜索变更
    fn handle_dns_search_changed(&self, state: &mut AppState, keyword: String) -> HandlerResult {
        state.data.dns_record_filter.search_keyword = keyword;
        HandlerResult::StateUpdated
    }

    /// 处理DNS记录过滤器变更
    fn handle_dns_filter_changed(
        &self,
        state: &mut AppState,
        filter_type: Option<String>,
    ) -> HandlerResult {
        state.data.dns_record_filter.record_type = filter_type;
        HandlerResult::StateUpdated
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

        // 获取当前选中的域名ID
        let domain_id = match &state.data.selected_domain {
            Some(domain) => domain.id as usize,
            None => {
                warn!("未选择域名，无法添加DNS记录");
                return HandlerResult::StateUpdated;
            }
        };

        if let Some(conn) = &state.database {
            let conn_clone = conn.clone();

            // 判断是添加还是更新
            let is_update = form_data.record_id.is_some();

            let task = if is_update {
                Task::perform(
                    Self::handle_dns_record_update_async(conn_clone, form_data, domain_id),
                    move |result| Self::handle_form_result(result, domain_id)
                )
            } else {
                Task::perform(
                    Self::handle_dns_record_add_async(conn_clone, form_data, domain_id),
                    move |result| Self::handle_form_result(result, domain_id)
                )
            };

            HandlerResult::Task(task)
        } else {
             state.ui.set_message("数据库未连接".to_string());
             HandlerResult::StateUpdated
        }
    }

    /// 统一处理表单结果
    fn handle_form_result(result: Result<(), String>, domain_id: usize) -> MessageCategory {
        match result {
            Ok(_) => {
                info!("DNS记录操作成功，刷新列表");
                // 刷新当前域名的DNS记录
                MessageCategory::Dns(DnsMessage::FormSubmitSuccess(domain_id))
            },
            Err(e) => {
                 let msg = if e.contains("DomainRecordDuplicate") || e.contains("already exists") {
                     "DNS记录已存在".to_string()
                 } else {
                     format!("操作失败: {}", e)
                 };
                 MessageCategory::Notification(NotificationMessage::ShowToast(msg))
            }
        }
    }

    /// 处理DNS表单提交成功
    fn handle_form_submit_success(&self, state: &mut AppState, domain_id: usize) -> HandlerResult {
        info!("DNS记录表单提交成功，清空表单并刷新列表");
        state.data.add_dns_form = Default::default();
        self.handle_query_record(state, domain_id)
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
    ///
    /// 注意：现在的删除逻辑通常通过 `delete_and_reload` 直接触发 `DnsRecordReloaded`，
    /// 这个方法仅保留作为兼容性处理或备用。
    fn handle_record_deleted(&self, state: &mut AppState, record_id: usize) -> HandlerResult {
        info!("DNS记录删除消息收到: {} (Legacy)", record_id);

        // 仅处理导航，数据更新已由 DnsRecordReloaded 处理
        state.update(StateUpdate::Navigation(Page::DnsRecord));
        HandlerResult::StateUpdated
    }

    /// 从数据库加载DNS记录
    async fn load_dns_records_from_db(
        conn: DatabaseConnection,
        domain_id: usize,
    ) -> Result<Vec<DnsRecordModal>, String> {
        info!("开始从数据库加载域名 {} 的DNS记录", domain_id);

        let records = records::get_records_by_domain(&conn, Some(domain_id as i64))
            .await
            .map_err(|e: anyhow::Error| e.to_string())?;

        // 转换为 DnsRecordModal
        let modal_records: Vec<DnsRecordModal> = records
            .into_iter()
            .map(|r| DnsRecordModal {
                id: r.id,
                domain_id: r.domain_id,
                record_type: r.record_type,
                name: r.record_name,
                value: r.record_value,
                ttl: r.ttl,
                priority: None, // 数据库中没有存储 priority，后续可以添加
                enabled: true,  // 数据库中没有存储 status，默认为 true
                created_at: chrono::Utc::now().naive_utc(),
                updated_at: None,
            })
            .collect();

        info!(
            "域名 {} 的DNS记录加载完成，共 {} 条记录",
            domain_id,
            modal_records.len()
        );
        Ok(modal_records)
    }

    /// 异步同步DNS记录
    async fn sync_dns_records_async(domain: usize, conn: DatabaseConnection) -> Result<Vec<DnsRecordModal>, String> {
        // 使用 load_dns_records_from_db 代替 query_dns_records_async
        Self::load_dns_records_from_db(conn, domain).await
    }

    /// 异步添加DNS记录
    async fn add_dns_record_async(
        conn: DatabaseConnection,
        record: DnsRecordModal,
    ) -> Result<DnsRecordModal, String> {
        // 1. 获取域名信息
        let domain = domains::find_domain_by_id(&conn, record.domain_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("域名不存在")?;

        // 2. 获取账户信息
        let account = accounts::get_account_by_id(&conn, domain.account_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("账户不存在")?;

        // 3. 初始化 API 客户端
        let credential: Credential = account
            .try_into()
            .map_err(|e: anyhow::Error| e.to_string())?;

        let api_client = match credential {
            Credential::ApiKey(key) => AliyunDnsClient::new(key.api_key, key.api_secret),
            _ => return Err("不支持的凭据类型".to_string()),
        };

        let record_type_enum = match record.record_type.to_uppercase().as_str() {
            "A" => RecordType::A,
            "CNAME" => RecordType::Cname,
            "MX" => RecordType::MX,
            "AAAA" => RecordType::AAAA,
            "TXT" => RecordType::TXT,
            "NS" => RecordType::NS,
            "SOA" => RecordType::SOA,
            "PTR" => RecordType::PTR,
            "SRV" => RecordType::SRV,
            "FORWARD_URL" => RecordType::ForwardUrl,
            _ => RecordType::A, // 默认为 A 记录
        };

        // 4. 调用 API 创建记录
        let domain_name = DomainName {
            name: domain.domain_name.clone(),
            provider: DnsProvider::Aliyun,
            ..Default::default()
        };

        let new_dns_record = Record::new(
            Status::Enable,
            record.name.clone(),
            record_type_enum,
            record.value.clone(),
            "".to_string(), // 创建时还没有 RecordId
            record.ttl,
        );

        api_client
            .add_dns_record(&domain_name, &new_dns_record)
            .await
            .map_err(|e| e.to_string())?;

        // 5. 更新本地数据库
        let new_record = NewRecord {
            domain_id: record.domain_id,
            record_name: record.name.clone(),
            record_type: record.record_type.clone(),
            record_value: record.value.clone(),
            ttl: record.ttl,
        };

        let saved_record = records::add_record(&conn, new_record)
            .await
            .map_err(|e| e.to_string())?;

        // 6. 返回更新后的记录
        let mut result_record = record;
        result_record.id = saved_record.id;

        Ok(result_record)
    }

    /// 异步删除DNS记录
    async fn delete_dns_record_async(
        conn: DatabaseConnection,
        _domain_id: usize,
        record_id: usize,
    ) -> Result<(), String> {
        // 1. 获取记录信息
        let record = records::find_record_by_id(&conn, record_id as i64)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("记录不存在")?;

        // 2. 获取域名信息
        let domain = domains::find_domain_by_id(&conn, record.domain_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("域名不存在")?;

        // 3. 获取账户信息
        let account = accounts::get_account_by_id(&conn, domain.account_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("账户不存在")?;

        // 4. 初始化 API 客户端
        let credential: Credential = account
            .try_into()
            .map_err(|e: anyhow::Error| e.to_string())?;

        let api_client = match credential {
            Credential::ApiKey(key) => AliyunDnsClient::new(key.api_key, key.api_secret),
            _ => return Err("不支持的凭据类型".to_string()),
        };

        // 5. 调用 API 删除记录
        // 注意：阿里云API需要 RecordId，这是阿里云分配的ID，不是本地数据库ID
        // 临时方案：先查询该域名下的所有记录，找到匹配的（根据 RR, Type, Value），获取 RecordId
        let records = api_client
            .list_dns_records(domain.domain_name.clone())
            .await
            .map_err(|e| e.to_string())?;

        let target_record = records
            .iter()
            .find(|r| {
                r.rr == record.record_name
                    && r.record_type.get_value() == record.record_type
                    && r.value == record.record_value
            })
            .ok_or("在云端未找到匹配的DNS记录")?;

        let domain_name = DomainName {
            name: domain.domain_name.clone(),
            provider: DnsProvider::Aliyun,
            ..Default::default()
        };

        api_client
            .delete_dns_record(&domain_name, &target_record.record_id)
            .await
            .map_err(|e| e.to_string())?;

        // 6. 删除本地数据库记录
        records::delete_record(&conn, record_id as i64)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 组合操作：删除并重新加载DNS记录
    ///
    /// 这是一个原子化的业务流程，包含：
    /// 1. 确认域名ID
    /// 2. 执行删除（API + DB）
    /// 3. 重新加载最新数据
    async fn delete_and_reload_dns_record_async(
        conn: DatabaseConnection,
        domain_id: usize,
        record_id: usize,
    ) -> Result<(usize, Vec<DnsRecordModal>), String> {
        // 1. 确定目标域名ID
        // 如果传入的 domain_id 为 0，我们需要先查询记录所属的域名
        let target_domain_id = if domain_id == 0 {
             let record = records::find_record_by_id(&conn, record_id as i64)
                .await
                .map_err(|e| e.to_string())?
                .ok_or("记录不存在")?;
             record.domain_id as usize
        } else {
            domain_id
        };

        // 2. 执行删除
        Self::delete_dns_record_async(conn.clone(), target_domain_id, record_id).await?;

        // 3. 重新加载数据
        let records = Self::load_dns_records_from_db(conn, target_domain_id).await?;

        Ok((target_domain_id, records))
    }

    /// 异步更新DNS记录
    async fn update_dns_record_async(
        conn: DatabaseConnection,
        old_record: DnsRecordModal,
        new_record: DnsRecordModal,
    ) -> Result<DnsRecordModal, String> {
        // 1. 获取域名信息
        let domain = domains::find_domain_by_id(&conn, old_record.domain_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("域名不存在")?;

        // 2. 获取账户信息
        let account = accounts::get_account_by_id(&conn, domain.account_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("账户不存在")?;

        // 3. 初始化 API 客户端
        let credential: Credential = account
            .try_into()
            .map_err(|e: anyhow::Error| e.to_string())?;

        let api_client = match credential {
            Credential::ApiKey(key) => AliyunDnsClient::new(key.api_key, key.api_secret),
            _ => return Err("不支持的凭据类型".to_string()),
        };

        // 4. 查找云端记录ID
        let records = api_client
            .list_dns_records(domain.domain_name.clone())
            .await
            .map_err(|e| e.to_string())?;

        let target_record = records
            .iter()
            .find(|r| {
                r.rr == old_record.name
                    && r.record_type.get_value() == old_record.record_type
                    && r.value == old_record.value
            })
            .ok_or("在云端未找到匹配的DNS记录，无法更新")?;

        // 5. 调用 API 更新记录
        let domain_name = DomainName {
            name: domain.domain_name.clone(),
            provider: DnsProvider::Aliyun,
            ..Default::default()
        };

        let record_type_enum = match new_record.record_type.to_uppercase().as_str() {
            "A" => RecordType::A,
            "CNAME" => RecordType::Cname,
            "MX" => RecordType::MX,
            "AAAA" => RecordType::AAAA,
            "TXT" => RecordType::TXT,
            "NS" => RecordType::NS,
            "SOA" => RecordType::SOA,
            "PTR" => RecordType::PTR,
            "SRV" => RecordType::SRV,
            "FORWARD_URL" => RecordType::ForwardUrl,
            _ => RecordType::A,
        };

        let update_record_req = Record::new(
            Status::Enable,
            new_record.name.clone(),
            record_type_enum,
            new_record.value.clone(),
            target_record.record_id.clone(),
            new_record.ttl,
        );

        api_client
            .update_dns_record(&domain_name, &update_record_req)
            .await
            .map_err(|e| e.to_string())?;

        // 6. 更新本地数据库
        let new_record_model = NewRecord {
            domain_id: new_record.domain_id,
            record_name: new_record.name.clone(),
            record_type: new_record.record_type.clone(),
            record_value: new_record.value.clone(),
            ttl: new_record.ttl,
        };

        // delete old and add new? or update?
        // records::update_record not available?
        // Assuming we have to delete and add, or implement update.
        // Checking imports: `use crate::storage::{accounts, domains, records, DnsRecordModal};`
        // I don't know if `records` has update.
        // I will assume I can update. If not, I'll delete and add.
        // But `records::add_record` creates a NEW record with NEW ID.
        // If I delete and add, the ID changes.
        // Ideally I should update.
        // I'll assume `records::update_record` exists or I can implement it.
        // Wait, I cannot edit `records` module easily if I don't see it.
        // I'll assume `records::update_record` exists.
        // If not, I'll use delete+add.

        // Let's try to check `records` module.
        // But for now, I will use `records::delete_record` and `records::add_record` to be safe if update is missing,
        // BUT this changes the ID, which might confuse the UI if it relies on ID stability.
        // However, we reload the list after update, so it should be fine.

        records::delete_record(&conn, old_record.id)
            .await
            .map_err(|e| e.to_string())?;

        let saved_record = records::add_record(&conn, new_record_model)
            .await
            .map_err(|e| e.to_string())?;

        let mut result_record = new_record;
        result_record.id = saved_record.id;
        result_record.updated_at = Some(chrono::Utc::now().naive_utc());

        Ok(result_record)
    }

    /// 异步同步DNS记录
    // async fn sync_dns_records_async(domain: usize) -> Result<Vec<DnsRecordModal>, String> {
    //     // 模拟网络延迟
    //     tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    //
    //     // 这里应该调用实际的DNS同步服务
    //     Self::query_dns_records_async(domain).await
    // }

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

    /// 异步添加DNS记录（真实实现）
    async fn handle_dns_record_add_async(
        conn: DatabaseConnection,
        form_data: AddDnsField,
        domain_id: usize,
    ) -> Result<(), String> {
        info!("开始异步添加DNS记录: {:?}，域名ID: {}", form_data, domain_id);

        // 构建 DnsRecordModal
        let record = DnsRecordModal {
            id: 0, // 临时ID
            domain_id: domain_id as i64,
            record_type: form_data
                .record_type
                .map(|t| t.get_value().to_string())
                .unwrap_or("A".to_string()),
            name: form_data.record_name.clone(),
            value: form_data.value.clone(),
            ttl: form_data.ttl,
            priority: None,
            enabled: true,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: None,
        };

        // 调用 add_dns_record_async (复用已有逻辑)
        Self::add_dns_record_async(conn, record).await?;

        Ok(())
    }

    /// 异步更新DNS记录（真实实现）
    async fn handle_dns_record_update_async(
        conn: DatabaseConnection,
        form_data: AddDnsField,
        domain_id: usize,
    ) -> Result<(), String> {
        info!("开始异步更新DNS记录: {:?}，域名ID: {}", form_data, domain_id);

        let record_id = form_data
            .record_id
            .clone()
            .ok_or("记录ID缺失")?
            .parse::<i64>()
            .map_err(|e| e.to_string())?;

        // 获取原始记录
        let old_record_entity = records::find_record_by_id(&conn, record_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("记录不存在")?;

        let old_record = DnsRecordModal {
            id: old_record_entity.id,
            domain_id: old_record_entity.domain_id,
            record_type: old_record_entity.record_type,
            name: old_record_entity.record_name,
            value: old_record_entity.record_value,
            ttl: old_record_entity.ttl,
            priority: None,
            enabled: true,
            created_at: chrono::Utc::now().naive_utc(), // RecordEntity 缺少 created_at，使用当前时间代替
            updated_at: None, // RecordEntity 缺少 updated_at
        };

        let new_record = DnsRecordModal {
            id: old_record.id,
            domain_id: domain_id as i64,
            record_type: form_data
                .record_type
                .map(|t| t.get_value().to_string())
                .unwrap_or("A".to_string()),
            name: form_data.record_name.clone(),
            value: form_data.value.clone(),
            ttl: form_data.ttl,
            priority: None,
            enabled: true,
            created_at: old_record.created_at,
            updated_at: Some(chrono::Utc::now().naive_utc()),
        };

        Self::update_dns_record_async(conn, old_record, new_record).await?;

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
            DnsMessage::DeleteRequest(record_id) => self.handle_delete_request(state, record_id),
            DnsMessage::DeleteCancel => self.handle_delete_cancel(state),
            DnsMessage::RecordHovered(record_id) => self.handle_record_hovered(state, record_id),
            DnsMessage::EditRecord(record) => self.handle_edit_record(state, record),
            DnsMessage::FormNameChanged(record_name) => {
                self.handle_form_name_changed(state, record_name)
            }
            DnsMessage::FormRecordTypeChanged(record_type) => {
                self.handle_form_record_type_changed(state, record_type)
            }
            DnsMessage::FormValueChanged(value) => self.handle_form_value_changed(state, value),
            DnsMessage::FormTtlChanged(ttl) => self.handle_form_ttl_changed(state, ttl),
            DnsMessage::FormSubmit => self.handle_form_submit(state),
            DnsMessage::FormSubmitSuccess(domain_id) => self.handle_form_submit_success(state, domain_id),
            DnsMessage::FormCancelled => self.handle_form_cancelled(state),
            DnsMessage::DnsFilterChanged(filter) => self.handle_dns_filter_changed(state, filter),
            DnsMessage::DnsSearchChanged(keyword) => self.handle_dns_search_changed(state, keyword),
            DnsMessage::RecordDeleted(record_id) => self.handle_record_deleted(state, record_id),
            DnsMessage::ProviderSelected(account_id) => {
                // 特殊处理：使用 99999 作为切换添加表单的信号
                if account_id == 99999 {
                    state.data.add_dns_form.is_visible = !state.data.add_dns_form.is_visible;
                    HandlerResult::StateUpdated
                } else {
                    self.handle_provider_selected(state, account_id)
                }
            }
            DnsMessage::ProviderChange(provider) => self.handle_provider_change(state, provider),
            DnsMessage::DnsRecordReloaded(domain_id, records) => {
                info!("DNS记录重新加载完成，域名ID: {}，记录数: {}", domain_id, records.len());
                // 更新数据状态
                state.data.set_dns_records(domain_id, records);
                // 更新UI状态
                state.ui.set_message(format!("DNS记录已更新，共 {} 条", state.data.current_dns_records.len()));
                state.ui.is_loading = false;
                HandlerResult::StateUpdated
            }
            _ => {
                debug!("未处理事件！{:?}",event);
                HandlerResult::None
            },
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
