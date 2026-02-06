//! 托管商处理器
//!
//! 负责处理托管商相关的业务逻辑，包括托管商选择、添加托管商、
//! 凭证验证等功能。

use super::{EventHandler, HandlerResult};
use crate::gui::handlers::message_handler::{
    DatabaseMessage, MessageCategory, ProviderMessage, SyncMessage,
};
use crate::gui::model::domain::DnsProvider;
use crate::gui::pages::domain::{AddDomainProviderForm, VerificationStatus};
use crate::gui::state::app_state::{StateUpdate, UiUpdate};
use crate::gui::state::AppState;
use crate::gui::types::credential::{Credential, CredentialMessage};
use crate::models::account::{Account, NewAccount};
use crate::storage;
use iced::Task;
use tokio::time;
use tokio::time::Duration;
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
                    .provider_page
                    .providers
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
        provider_type: DnsProvider,
    ) -> HandlerResult {
        // 根据提供商类型字符串查找对应的DnsProvider
        let dns_provider = match provider_type {
            DnsProvider::Aliyun => Some(DnsProvider::Aliyun),
            DnsProvider::TencentCloud => Some(DnsProvider::TencentCloud),
            DnsProvider::CloudFlare => Some(DnsProvider::CloudFlare),
            DnsProvider::Tomato => Some(DnsProvider::Tomato),
            DnsProvider::Dnspod => Some(DnsProvider::Dnspod),
            DnsProvider::Aws => Some(DnsProvider::Aws),
            DnsProvider::Google => Some(DnsProvider::Google),
        };

        if let Some(provider) = dns_provider {
            // 更新表单中的提供商类型
            state.data.provider_page.form.provider = Some(provider.clone());
            // 同时设置默认凭证
            state.data.provider_page.form.credential = Some(provider.credential());

            // 自动生成名称
            let base_name = provider.name();
            let mut new_name = base_name.to_string();
            let mut counter = 1;

            // 检查名称是否存在
            let existing_names: Vec<String> = state
                .data
                .provider_page
                .providers
                .iter()
                .map(|p| p.provider_name.clone())
                .collect();

            while existing_names.contains(&new_name) {
                new_name = format!("{} {}", base_name, counter);
                counter += 1;
            }

            state.data.provider_page.form.provider_name = new_name;

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
        state.data.provider_page.form.provider_name = name.clone();
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

        if let Some(credential) = &mut state.data.provider_page.form.credential {
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
        let form = state.data.provider_page.form.clone();

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
        state.data.provider_page.form.verification_status = VerificationStatus::Pending;

        // 启动异步凭证验证任务
        HandlerResult::StateUpdatedWithTask(Task::perform(
            Self::validate_credential_async(form),
            |result| match result {
                Ok(_) => MessageCategory::Provider(ProviderMessage::VerificationStatusChanged(
                    VerificationStatus::Success,
                )),
                Err(err) => MessageCategory::Provider(ProviderMessage::VerificationStatusChanged(
                    VerificationStatus::Failed(err),
                )),
            },
        ))
    }

    /// 处理添加托管商凭证
    fn handle_add_credential(&self, state: &mut AppState) -> HandlerResult {
        let form = &state.data.provider_page.form;

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
            "开始处理域名托管商: {}, 类型: {}",
            new_account.username,
            new_account.provider.name()
        );

        state.ui.set_loading(true);

        if let Some(id) = state.data.provider_page.editing_provider_id {
            let account = Account {
                id,
                username: new_account.username.clone(),
                provider_type: new_account.provider.name().to_string(),
                email: new_account.email.clone(),
                created_at: chrono::Local::now().naive_local().to_string(),
                last_login: None,
                credential_type: new_account.credential.credential_type(),
                credential_data: new_account.credential.raw_data().into(),
                salt: "salt".to_string(),
                api_keys: vec![],
            };

            HandlerResult::StateUpdatedWithTask(Task::done(MessageCategory::Database(
                DatabaseMessage::UpdateAccount(account),
            )))
        } else {
            debug!("新增托管商凭证：{:?}", new_account);
            HandlerResult::StateUpdatedWithTask(Task::done(MessageCategory::Database(
                DatabaseMessage::AddAccount(new_account),
            )))
        }
    }

    /// 处理托管商变更
    fn handle_provider_change(&self, state: &mut AppState) -> HandlerResult {
        info!("托管商配置发生变更，重新加载数据");
        state.ui.set_message("托管商配置已更新".to_string());

        // 触发数据重新加载
        // 同时触发 ProviderMessage::Load
        HandlerResult::StateUpdatedWithTask(Task::perform(async {}, |_| {
            MessageCategory::Provider(ProviderMessage::Load)
        }))
    }

    fn handle_verification_status_changed(
        &self,
        state: &mut AppState,
        status: VerificationStatus,
    ) -> HandlerResult {
        state.data.provider_page.form.verification_status = status;
        match state.data.provider_page.form.verification_status {
            VerificationStatus::Success | VerificationStatus::Failed(_) => {
                state.ui.set_loading(false);
            }
            _ => {}
        }
        HandlerResult::StateUpdated
    }

    /// 异步验证凭证
    async fn validate_credential_async(form: AddDomainProviderForm) -> Result<(), String> {
        time::timeout(Duration::from_secs(10), async move {
            // 获取提供商和凭证信息
            let provider = form
                .provider
                .clone()
                .ok_or("提供商类型未选择".to_string())?;
            let credential = form
                .credential
                .clone()
                .ok_or("凭证信息未提供".to_string())?;

            // 调用实际的API进行凭证验证
            match crate::api::credential_service::CredentialService::validate_credentials(
                provider,
                &credential,
            )
            .await
            {
                Ok(_) => {
                    info!("凭证验证成功: {}", form.provider_name);
                    Ok(())
                }
                Err(err) => {
                    error!("凭证验证失败: {:?}", err);
                    Err(err.to_string())
                }
            }
        })
        .await
        .unwrap_or_else(|_| Err("验证超时，请检查网络连接".to_string()))
    }

    /// 异步添加托管商凭证
    async fn add_credential_async(_new_account: NewAccount) -> Result<(), String> {
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

    fn handle_toggle_form(&self, state: &mut AppState, visible: bool) -> HandlerResult {
        state.data.provider_page.form_visible = visible;
        HandlerResult::StateUpdated
    }

    fn handle_delete_request(&self, state: &mut AppState, id: i64) -> HandlerResult {
        state.data.provider_page.deleting_provider_id = Some(id);
        HandlerResult::StateUpdated
    }

    fn handle_cancel_delete(&self, state: &mut AppState) -> HandlerResult {
        state.data.provider_page.deleting_provider_id = None;
        HandlerResult::StateUpdated
    }

    fn handle_confirm_delete(&self, state: &mut AppState, id: i64) -> HandlerResult {
        state.data.provider_page.deleting_provider_id = None;
        state.ui.set_loading(true);
        HandlerResult::StateUpdatedWithTask(Task::done(MessageCategory::Database(
            DatabaseMessage::DeleteAccount(id),
        )))
    }

    fn handle_edit_request(&self, state: &mut AppState, id: i64) -> HandlerResult {
        if let Some(provider) = state
            .data
            .provider_page
            .providers
            .iter()
            .find(|p| p.account_id == id)
        {
            let form = &mut state.data.provider_page.form;
            form.provider_name = provider.provider_name.clone();
            form.provider = Some(provider.provider.clone());
            form.credential = Some(provider.credential.clone());

            state.data.provider_page.editing_provider_id = Some(id);
            state.data.provider_page.form_visible = true;

            HandlerResult::StateUpdated
        } else {
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                "未找到指定服务商".to_string(),
            )));
            HandlerResult::StateUpdated
        }
    }

    fn handle_load(&self, state: &mut AppState) -> HandlerResult {
        state.data.provider_page.is_loading = true;

        if let Some(conn) = &state.database {
            let conn_clone = conn.clone();
            HandlerResult::StateUpdatedWithTask(Task::perform(
                async move {
                    storage::list_accounts(&conn_clone)
                        .await
                        .map_err(|e| e.to_string())
                },
                |result| MessageCategory::Provider(ProviderMessage::Loaded(result)),
            ))
        } else {
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                "数据库未连接".to_string(),
            )));
            state.data.provider_page.is_loading = false;
            HandlerResult::StateUpdated
        }
    }

    fn handle_loaded(
        &self,
        state: &mut AppState,
        result: Result<Vec<Account>, String>,
    ) -> HandlerResult {
        state.data.provider_page.is_loading = false;
        match result {
            Ok(accounts) => {
                let providers: Vec<crate::gui::pages::domain::DomainProvider> =
                    accounts.into_iter().map(|a| a.into()).collect();
                state.data.provider_page.providers = providers;
            }
            Err(e) => {
                error!("加载服务商失败: {}", e);
                state.update(StateUpdate::Ui(UiUpdate::ShowToast(format!(
                    "加载服务商失败: {}",
                    e
                ))));
            }
        }
        HandlerResult::StateUpdated
    }
}

impl EventHandler<ProviderMessage> for ProviderHandler {
    /// 处理托管商相关消息
    fn handle(&self, state: &mut AppState, message: ProviderMessage) -> HandlerResult {
        match message {
            ProviderMessage::Selected(provider_type) => {
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
            ProviderMessage::VerificationStatusChanged(status) => {
                self.handle_verification_status_changed(state, status)
            }
            ProviderMessage::ToggleForm(visible) => self.handle_toggle_form(state, visible),
            ProviderMessage::Delete(id) => self.handle_delete_request(state, id),
            ProviderMessage::ConfirmDelete(id) => self.handle_confirm_delete(state, id),
            ProviderMessage::CancelDelete => self.handle_cancel_delete(state),
            ProviderMessage::Edit(id) => self.handle_edit_request(state, id),
            ProviderMessage::Load => self.handle_load(state),
            ProviderMessage::Loaded(result) => self.handle_loaded(state, result),
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
