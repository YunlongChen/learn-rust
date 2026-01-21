//! 托管商处理器
//!
//! 负责处理托管商相关的业务逻辑，包括托管商选择、添加托管商、
//! 凭证验证等功能。

use super::{EventHandler, HandlerResult};
use crate::gui::handlers::message_handler::{
    MessageCategory, NotificationMessage, ProviderMessage, SyncMessage,
};
use crate::gui::model::domain::DnsProvider;
use crate::gui::pages::domain::AddDomainProviderForm;
use crate::gui::state::app_state::{StateUpdate, UiUpdate};
use crate::gui::state::AppState;
use crate::gui::types::credential::Credential;
use crate::models::account::NewAccount;
use iced::Task;
use tracing::{debug, error, info, warn};

/// 托管商处理器
///
/// 处理所有与托管商相关的业务逻辑
#[derive(Debug, Clone)]
pub struct ProviderHandler {
    // 可以添加一些状态字段，如缓存等
}

impl ProviderHandler {
    /// 创建新的托管商处理器
    pub fn new() -> Self {
        Self {}
    }

    /// 处理托管商选择
    fn handle_provider_selected(
        &self,
        state: &mut AppState,
        provider_id_option: Option<usize>,
    ) -> HandlerResult {
        match provider_id_option {
            Some(provider_id) => {
                // 查找对应的托管商
                if let Some(provider) = state
                    .data
                    .domain_providers
                    .iter()
                    .find(|provider| provider.account_id as usize == provider_id)
                    .cloned()
                {
                    state.data.selected_provider = Some(provider.clone());
                    state
                        .ui
                        .set_message(format!("已选择托管商: {}", provider.provider_name));

                    info!("托管商选择成功: {}", provider.provider_name);

                    // 触发数据重新加载
                    HandlerResult::StateUpdatedWithTask(Task::perform(async {}, |_| {
                        MessageCategory::Sync(SyncMessage::Reload)
                    }))
                } else {
                    warn!("未找到托管商: {}", provider_id);
                    state.update(StateUpdate::Ui(UiUpdate::ShowToast(format!(
                        "托管商 {} 不存在",
                        provider_id
                    ))));
                    HandlerResult::StateUpdated
                }
            }
            None => {
                // 清除选择
                state.data.selected_provider = None;
                state.ui.set_message("已清除托管商选择".to_string());
                info!("已清除托管商选择");

                // 触发数据重新加载
                HandlerResult::StateUpdatedWithTask(Task::perform(async {}, |_| {
                    MessageCategory::Sync(SyncMessage::Reload)
                }))
            }
        }
    }

    /// 处理添加托管商表单的提供商类型变更
    fn handle_add_form_provider_changed(
        &self,
        state: &mut AppState,
        provider_type: String,
    ) -> HandlerResult {
        // 根据提供商类型字符串查找对应的DnsProvider
        let dns_provider = match provider_type.as_str() {
            "Aliyun" => Some(DnsProvider::Aliyun),
            "TencentCloud" => Some(DnsProvider::TencentCloud),
            "CloudFlare" => Some(DnsProvider::CloudFlare),
            "Tomato" => Some(DnsProvider::Tomato),
            "Dnspod" => Some(DnsProvider::Dnspod),
            "Aws" => Some(DnsProvider::Aws),
            "Google" => Some(DnsProvider::Google),
            _ => {
                warn!("未知的托管商类型: {}", provider_type);
                None
            }
        };

        if let Some(provider) = dns_provider {
            // 更新表单中的提供商类型
            state.data.add_domain_provider_form.provider = Some(provider.clone());
            // 同时设置默认凭证
            state.data.add_domain_provider_form.credential = Some(provider.credential());

            debug!("添加托管商表单提供商类型已更新: {}", provider_type);
            state
                .ui
                .set_message(format!("已选择托管商类型: {}", provider_type));
            HandlerResult::StateUpdated
        } else {
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(format!(
                "不支持的托管商类型: {}",
                provider_type
            ))));
            HandlerResult::StateUpdated
        }
    }

    /// 处理添加托管商表单的名称变更
    fn handle_add_form_name_changed(&self, state: &mut AppState, name: String) -> HandlerResult {
        state.data.add_domain_provider_form.provider_name = name.clone();
        debug!("添加托管商表单名称已更新: {}", name);
        HandlerResult::StateUpdated
    }

    /// 处理添加托管商表单的凭证变更
    fn handle_add_form_credential_changed(
        &self,
        state: &mut AppState,
        credential_info: Credential,
    ) -> HandlerResult {
        // 这里可以根据实际需要处理凭证信息的更新
        // 由于凭证结构比较复杂，这里先做简单处理
        debug!(
            "添加托管商表单凭证信息已更新：「{}」",
            credential_info.credential_type()
        );
        state.ui.set_message("凭证信息已更新".to_string());
        HandlerResult::StateUpdated
    }

    /// 处理凭证验证
    fn handle_validate_credential(&self, state: &mut AppState) -> HandlerResult {
        let form = &state.data.add_domain_provider_form;

        // 检查表单完整性
        if form.provider.is_none() {
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                "请先选择托管商类型".to_string(),
            )));
            return HandlerResult::StateUpdated;
        }

        if form.provider_name.is_empty() {
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                "请输入托管商名称".to_string(),
            )));
            return HandlerResult::StateUpdated;
        }

        if form.credential.is_none() {
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                "请配置凭证信息".to_string(),
            )));
            return HandlerResult::StateUpdated;
        }

        info!("开始验证托管商凭证: {}", form.provider_name);
        state.ui.set_loading(true);

        // 启动异步凭证验证任务
        HandlerResult::StateUpdatedWithTask(Task::perform(
            Self::validate_credential_async(form.clone()),
            |result| match result {
                Ok(_) => MessageCategory::Notification(NotificationMessage::ShowToast(
                    "凭证验证成功".to_string(),
                )),
                Err(err) => MessageCategory::Notification(NotificationMessage::ShowToast(format!(
                    "凭证验证失败: {}",
                    err
                ))),
            },
        ))
    }

    /// 处理添加托管商凭证
    fn handle_add_credential(&self, state: &mut AppState) -> HandlerResult {
        let form = &state.data.add_domain_provider_form;

        // 参数校验
        if form.provider.is_none() {
            error!("提供商类型未选择");
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                "请选择提供商类型".to_string(),
            )));
            return HandlerResult::StateUpdated;
        }

        if form.provider_name.is_empty() {
            error!("提供商名称为空");
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                "请输入提供商名称".to_string(),
            )));
            return HandlerResult::StateUpdated;
        }

        if form.credential.is_none() {
            error!("凭证信息未提供");
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                "请提供凭证信息".to_string(),
            )));
            return HandlerResult::StateUpdated;
        }

        let new_account = NewAccount {
            provider: form.provider.clone().unwrap(),
            username: form.provider_name.clone(),
            email: "example@qq.com".to_string(), // 这里可以从表单获取或使用默认值
            credential: form.credential.clone().unwrap(),
        };

        info!(
            "开始添加域名托管商: {}, 类型: {}",
            new_account.username,
            new_account.provider.name()
        );

        state.ui.set_loading(true);

        // 启动异步添加托管商任务
        HandlerResult::StateUpdatedWithTask(Task::perform(
            Self::add_credential_async(new_account),
            |result| match result {
                Ok(_) => {
                    info!("托管商添加成功，准备刷新界面");
                    MessageCategory::Sync(SyncMessage::Reload)
                }
                Err(err) => {
                    error!("托管商添加失败: {:?}", err);
                    MessageCategory::Notification(NotificationMessage::ShowToast(format!(
                        "添加托管商失败: {}",
                        err
                    )))
                }
            },
        ))
    }

    /// 处理托管商变更
    fn handle_provider_change(&self, state: &mut AppState) -> HandlerResult {
        info!("托管商配置发生变更，重新加载数据");
        state.ui.set_message("托管商配置已更新".to_string());

        // 触发数据重新加载
        HandlerResult::StateUpdatedWithTask(Task::perform(async {}, |_| {
            MessageCategory::Sync(SyncMessage::Reload)
        }))
    }

    /// 异步验证凭证
    async fn validate_credential_async(form: AddDomainProviderForm) -> Result<(), String> {
        // 模拟凭证验证过程
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        // 这里应该调用实际的API进行凭证验证
        // 目前返回模拟结果
        if form.provider_name.contains("test") {
            Err("测试凭证验证失败".to_string())
        } else {
            info!("凭证验证成功: {}", form.provider_name);
            Ok(())
        }
    }

    /// 异步添加托管商凭证
    async fn add_credential_async(new_account: NewAccount) -> Result<(), String> {
        // 模拟数据库操作
        tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

        // 这里应该调用实际的数据库操作
        // 目前返回模拟结果
        info!("模拟添加托管商到数据库: {}", new_account.username);
        Ok(())
    }
}

impl EventHandler<ProviderMessage> for ProviderHandler {
    /// 处理托管商相关消息
    fn handle(&self, state: &mut AppState, message: ProviderMessage) -> HandlerResult {
        match message {
            ProviderMessage::AddFormProviderChanged(provider_type) => {
                self.handle_add_form_provider_changed(state, provider_type)
            }
            ProviderMessage::AddFormNameChanged(name) => {
                self.handle_add_form_name_changed(state, name)
            }
            ProviderMessage::AddFormCredentialChanged(credential_info) => {
                self.handle_add_form_credential_changed(state, credential_info)
            }
            ProviderMessage::ValidateCredential => self.handle_validate_credential(state),
            ProviderMessage::AddCredential => self.handle_add_credential(state),
            ProviderMessage::ProviderChange => self.handle_provider_change(state),
            _ => HandlerResult::NoChange,
        }
    }

    /// 检查是否可以处理该消息
    fn can_handle(&self, _event: &ProviderMessage) -> bool {
        true // ProviderHandler可以处理所有ProviderMessage
    }
}

impl Default for ProviderHandler {
    fn default() -> Self {
        Self::new()
    }
}
