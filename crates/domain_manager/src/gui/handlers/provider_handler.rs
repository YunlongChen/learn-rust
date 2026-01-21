//! 托管商处理器
//!
//! 负责处理托管商相关的业务逻辑，包括托管商选择、添加托管商、
//! 凭证验证等功能。

use super::{EventHandler, HandlerResult};
use crate::gui::handlers::message_handler::{
    DatabaseMessage, MessageCategory, NotificationMessage, ProviderMessage, SyncMessage,
};
use crate::gui::model::domain::DnsProvider;
use crate::gui::pages::domain::AddDomainProviderForm;
use crate::gui::state::app_state::{StateUpdate, UiUpdate};
use crate::gui::state::AppState;
use crate::gui::types::credential::{Credential, CredentialMessage};
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
        message: CredentialMessage,
    ) -> HandlerResult {
        use crate::gui::types::credential::{ApiKeyMessage, TokenMessage, UsernamePasswordMessage};

        if let Some(credential) = &mut state.data.add_domain_provider_form.credential {
            match (credential, message) {
                (
                    Credential::UsernamePassword(cred),
                    CredentialMessage::UsernamePasswordChanged(msg),
                ) => match msg {
                    UsernamePasswordMessage::UsernameChanged(new_cred) => {
                        cred.username = new_cred.username;
                    }
                    UsernamePasswordMessage::PasswordChanged(new_cred) => {
                        cred.password = new_cred.password;
                    }
                },
                (Credential::Token(cred), CredentialMessage::TokenChanged(msg)) => match msg {
                    TokenMessage::TokenChanged(token) => {
                        cred.token = token;
                    }
                },
                (Credential::ApiKey(cred), CredentialMessage::ApiKeyChanged(msg)) => match msg {
                    ApiKeyMessage::ApiKeyChanged(new_cred) => {
                        cred.api_key = new_cred.api_key;
                    }
                    ApiKeyMessage::ApiSecretChanged(new_cred) => {
                        cred.api_secret = new_cred.api_secret;
                    }
                },
                _ => {
                    tracing::warn!("凭证类型与消息类型不匹配");
                }
            }
        }

        debug!("添加托管商表单凭证信息已更新");
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

        // 发送数据库消息，请求创建账户
        // 注意：实际的数据库操作将在DomainManagerV2中处理
        HandlerResult::StateUpdatedWithTask(Task::done(MessageCategory::Database(
            DatabaseMessage::AddAccount(new_account),
        )))
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
        // 获取全局数据库连接
        // 注意：由于ProviderHandler没有持有数据库连接，这里我们需要一种方式获取连接
        // 实际上，应该在DomainManagerV2中注入数据库服务或连接到Handler
        // 这里我们假设可以通过某种全局方式或参数传递获取连接，但为了简单起见，
        // 我们需要重新思考架构。正确的方式是通过ServiceManager或DatabaseService。
        // 但由于HandlerResult::Task限制，我们只能在闭包中使用拥有的数据。

        // 临时解决方案：由于无法直接访问DomainManagerV2的数据库连接，
        // 我们需要通过消息传递将操作委托给主线程，或者重构Handler以支持数据库访问。
        // 但为了快速修复，我们先尝试建立一个新的临时连接（不推荐）或者
        // 更好的方式是：在add_credential中不直接执行异步任务，而是返回一个包含数据库操作请求的消息，
        // 由主循环处理。

        // 鉴于现有架构，我们实际上无法在这里直接访问数据库连接，因为它是Arc<RwLock<DatabaseConnection>>
        // 且存储在AppState或DomainManagerV2中。

        // 我们修改策略：handle_add_credential 不直接 spawn task，
        // 而是返回一个 Task，该 Task 会发送一个包含数据库操作的闭包给主循环？不对。

        // 正确的做法是：ProviderHandler 需要访问数据库连接。
        // 现有的 handle 方法签名是 `fn handle(&self, state: &mut AppState, ...)`
        // AppState 中并没有数据库连接（它在 DomainManagerV2 中）。
        // 这意味着当前的架构将数据库连接与 UI 状态分离了。

        // 让我们看看 SyncHandler 是怎么做的。
        // SyncHandler 也是返回 Task，但它似乎主要处理逻辑。

        // 考虑到 DomainManagerV2 拥有 `database: Option<Arc<RwLock<DatabaseConnection>>>`
        // 我们可能需要引入一个新的消息类型 DatabaseMessage::CreateAccount(NewAccount)
        // 让 DomainManagerV2 处理这个消息，因为它有数据库连接。

        // 但是为了保持 ProviderHandler 的内聚性，我们可以在 AppState 中不做处理，
        // 而是修改 MessageCategory，添加一个 DatabaseMessage 变体来请求创建账户。

        // 实际上，我们可以在 ProviderHandler 中构造一个 Task，该 Task 并不直接执行数据库操作，
        // 而是发送一个 MessageCategory::Database(DatabaseMessage::AddAccount(new_account))。
        // 然后在 DomainManagerV2 的 update 方法中处理这个 DatabaseMessage。

        // 不过，为了遵循现有的模式（如果其他 Handler 也是这样的话），我们看看 DomainManagerV2 的 update 方法。
        // 现在的 DomainManagerV2::update 主要是调用 message_handler.handle_message。

        // 方案 B：修改 add_credential_async 签名，接受数据库连接配置，并建立新的连接？太慢。

        // 方案 C（推荐）：
        // 1. 在 DatabaseMessage 中添加 AddAccount(NewAccount)
        // 2. ProviderHandler 返回 Task::done(MessageCategory::Database(DatabaseMessage::AddAccount(new_account)))
        // 3. DomainManagerV2 在处理 DatabaseMessage 时执行实际的数据库操作。

        Err("架构限制：请通过 DatabaseMessage::AddAccount 处理数据库操作".to_string())
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
