//! 托管商处理器单元测试
//!
//! 测试ProviderHandler的逻辑，包括表单验证、消息处理、自动命名、删除流程等

use crate::gui::handlers::message_handler::ProviderMessage;
use crate::gui::handlers::provider_handler::ProviderHandler;
use crate::gui::handlers::EventHandler;
use crate::gui::model::domain::DnsProvider;
use crate::gui::pages::domain::DomainProvider;
use crate::gui::state::app_state::AppState;
use crate::gui::types::credential::{Credential, TokenCredential};
use crate::tests::test_utils::init_test_env;
use tokio;

#[tokio::test]
async fn test_handle_add_form_name_changed() {
    init_test_env();
    let handler = ProviderHandler::new();
    let mut state = AppState::default();

    let name = "New Provider".to_string();
    let message = ProviderMessage::AddFormNameChanged(name.clone());

    handler.handle(&mut state, message);

    assert_eq!(
        state.data.add_domain_provider_form.provider_name, name,
        "Provider name should be updated"
    );
}

#[tokio::test]
async fn test_handle_add_form_provider_changed_basic() {
    init_test_env();
    let handler = ProviderHandler::new();
    let mut state = AppState::default();

    let message = ProviderMessage::Selected(DnsProvider::Aliyun);

    handler.handle(&mut state, message);

    assert!(
        state.data.add_domain_provider_form.provider.is_some(),
        "Provider type should be set"
    );
    assert_eq!(
        state.data.add_domain_provider_form.provider_name, "阿里云",
        "Should auto-fill name"
    );
}

#[tokio::test]
async fn test_handle_auto_naming() {
    init_test_env();
    let handler = ProviderHandler::new();
    let mut state = AppState::default();

    // 模拟已存在的服务商
    state.data.domain_providers.push(DomainProvider {
        account_id: 1,
        provider_name: "阿里云".to_string(),
        provider: DnsProvider::Aliyun,
        credential: Credential::Token(TokenCredential::default()),
    });

    // 触发选择 Aliyun
    let message = ProviderMessage::Selected(DnsProvider::Aliyun);
    handler.handle(&mut state, message);

    // 应该自动命名为 "阿里云 1"
    assert_eq!(
        state.data.add_domain_provider_form.provider_name,
        "阿里云 1"
    );
}

#[tokio::test]
async fn test_delete_flow() {
    init_test_env();
    let handler = ProviderHandler::new();
    let mut state = AppState::default();

    let provider_id = 123;

    // 1. 请求删除
    handler.handle(&mut state, ProviderMessage::Delete(provider_id));
    assert_eq!(state.ui.deleting_provider_id, Some(provider_id));

    // 2. 取消删除
    handler.handle(&mut state, ProviderMessage::CancelDelete);
    assert_eq!(state.ui.deleting_provider_id, None);

    // 3. 再次请求删除
    handler.handle(&mut state, ProviderMessage::Delete(provider_id));

    // 4. 确认删除
    let result = handler.handle(&mut state, ProviderMessage::ConfirmDelete(provider_id));
    assert_eq!(state.ui.deleting_provider_id, None);
    assert!(state.ui.is_loading);

    // 验证返回了 Task
    match result {
        crate::gui::handlers::HandlerResult::StateUpdatedWithTask(_) => {}
        _ => panic!("Should return StateUpdatedWithTask"),
    }
}

#[tokio::test]
async fn test_edit_flow() {
    init_test_env();
    let handler = ProviderHandler::new();
    let mut state = AppState::default();

    let provider_id = 123;
    let provider_name = "My Provider".to_string();

    // 添加一个服务商
    state.data.domain_providers.push(DomainProvider {
        account_id: provider_id,
        provider_name: provider_name.clone(),
        provider: DnsProvider::Aliyun,
        credential: Credential::Token(TokenCredential::default()),
    });

    // 请求编辑
    handler.handle(&mut state, ProviderMessage::Edit(provider_id));

    // 验证状态
    assert_eq!(state.ui.editing_provider_id, Some(provider_id));
    assert!(state.ui.provider_form_visible);

    // 验证表单填充
    assert_eq!(
        state.data.add_domain_provider_form.provider_name,
        provider_name
    );
    assert!(state.data.add_domain_provider_form.provider.is_some());
}
