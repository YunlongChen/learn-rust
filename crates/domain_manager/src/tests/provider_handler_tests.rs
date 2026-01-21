//! 托管商处理器单元测试
//!
//! 测试ProviderHandler的逻辑，包括表单验证、消息处理等

use crate::gui::handlers::message_handler::{DatabaseMessage, MessageCategory, ProviderMessage};
use crate::gui::handlers::provider_handler::ProviderHandler;
use crate::gui::handlers::EventHandler;
use crate::gui::model::domain::DnsProvider;
use crate::gui::state::app_state::AppState;
use crate::gui::types::credential::{Credential, TokenCredential};
use crate::models::account::NewAccount;
use crate::tests::test_utils::init_test_env;
use iced::Task;
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
async fn test_handle_add_form_provider_changed() {
    init_test_env();
    let handler = ProviderHandler::new();
    let mut state = AppState::default();

    let provider_type = "Aliyun".to_string();
    let message = ProviderMessage::AddFormProviderChanged(provider_type.clone());

    handler.handle(&mut state, message);

    assert!(
        state.data.add_domain_provider_form.provider.is_some(),
        "Provider type should be set"
    );
    assert_eq!(
        state
            .data
            .add_domain_provider_form
            .provider
            .as_ref()
            .unwrap()
            .name(),
        "阿里云",
        "Provider type should match"
    );
    assert!(
        state.data.add_domain_provider_form.credential.is_some(),
        "Credential should be initialized"
    );
}

#[tokio::test]
async fn test_handle_add_credential_validation() {
    init_test_env();
    let handler = ProviderHandler::new();
    let mut state = AppState::default();

    // Case 1: Empty form
    handler.handle(&mut state, ProviderMessage::AddCredential);
    // Should show toast error (we can't easily check toast here without mocking UI update logic details,
    // but we can check if loading state is NOT set)
    assert!(
        !state.ui.is_loading,
        "Should not start loading if validation fails"
    );

    // Case 2: Partial form (Provider set, but name empty)
    state.data.add_domain_provider_form.provider = Some(DnsProvider::Aliyun);
    handler.handle(&mut state, ProviderMessage::AddCredential);
    assert!(
        !state.ui.is_loading,
        "Should not start loading if name is empty"
    );

    // Case 3: Partial form (Provider & Name set, but credential empty)
    state.data.add_domain_provider_form.provider_name = "Test Provider".to_string();
    // Credential is None by default
    handler.handle(&mut state, ProviderMessage::AddCredential);
    assert!(
        !state.ui.is_loading,
        "Should not start loading if credential is missing"
    );

    // Case 4: Valid form
    state.data.add_domain_provider_form.credential =
        Some(Credential::Token(TokenCredential::default()));
    let result = handler.handle(&mut state, ProviderMessage::AddCredential);

    // Should start loading and return a task
    assert!(
        state.ui.is_loading,
        "Should start loading when form is valid"
    );

    // Verify it returns a Task (HandlerResult::StateUpdatedWithTask)
    // We can't easily inspect the Task content, but we know it should be there.
    match result {
        crate::gui::handlers::HandlerResult::StateUpdatedWithTask(_) => {}
        _ => panic!("Should return StateUpdatedWithTask"),
    }
}
