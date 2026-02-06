use crate::gui::handlers::message_handler::{
    AppMessage, DatabaseMessage, MessageCategory, SyncMessage,
};
use crate::gui::handlers::{EventHandler, HandlerResult};
use crate::gui::pages::Page;
use crate::gui::state::app_state::{StateUpdate, UiUpdate};
use crate::gui::state::AppState;
use iced::Task;
use tracing::{debug, error, info};

#[derive(Debug, Default)]
pub struct DataStoreHandler;

impl DataStoreHandler {
    /// 创建新的UI处理器实例
    pub fn new() -> Self {
        Self
    }
}

impl EventHandler<DatabaseMessage> for DataStoreHandler {
    fn handle(&self, state: &mut AppState, message: DatabaseMessage) -> HandlerResult {
        debug!("数据库相关的事件:{:?}", message);
        match message {
            DatabaseMessage::Connected(result) => {
                if state.database.is_some() {
                    // 已经存在数据库连接，忽略重复的连接操作
                    info!("数据库连接已存在，忽略重复连接");
                    return HandlerResult::NoChange;
                }
                match result {
                    Ok(conn) => {
                        info!("数据库连接成功");
                        state.database = Some(conn);
                        state.initialize(); // 标记初始化完成
                        HandlerResult::Task(Task::done(MessageCategory::App(
                            AppMessage::Initialize,
                        )))
                        // 触发数据重载
                    }
                    Err(e) => {
                        error!("数据库连接失败: {}", e);
                        state.update(StateUpdate::Ui(UiUpdate::SetError(Some(format!(
                            "数据库连接失败: {}",
                            e
                        )))));
                        HandlerResult::None
                    }
                }
            }
            DatabaseMessage::AddAccount(new_account) => {
                info!("收到添加账户请求: {}", new_account.username);
                if let Some(conn) = &state.database {
                    let account = new_account.clone();
                    let conn_clone = conn.clone();

                    HandlerResult::Task(Task::perform(
                        async move {
                            use crate::storage::create_account;
                            match create_account(&conn_clone, account).await {
                                Ok(acc) => MessageCategory::Database(
                                    DatabaseMessage::AccountAdded(Ok(acc)),
                                ),
                                Err(e) => {
                                    MessageCategory::Database(DatabaseMessage::AccountAdded(Err(e)))
                                }
                            }
                        },
                        |msg| msg,
                    ))
                    // 启动异步任务执行数据库操作
                } else {
                    error!("数据库未连接，无法添加账户");
                    HandlerResult::Task(Task::done(MessageCategory::Database(
                        DatabaseMessage::AccountAdded(Err("数据库未连接".to_string())),
                    )))
                }
            }
            DatabaseMessage::AccountAdded(result) => {
                // 处理添加账户结果
                state.update(StateUpdate::Ui(UiUpdate::SetLoading(false)));
                match result {
                    Ok(acc) => {
                        info!("账户添加成功: {}", acc.username);
                        // 清空表单
                        state.data.provider_page.form.clear();
                        // 提示成功
                        state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                            "服务商添加成功".to_string(),
                        )));
                        // 重新加载数据
                        HandlerResult::Task(Task::done(MessageCategory::Sync(SyncMessage::Reload)))
                    }
                    Err(e) => {
                        error!("账户添加失败: {}", e);
                        state.update(StateUpdate::Ui(UiUpdate::ShowToast(format!(
                            "添加失败: {}",
                            e
                        ))));
                        HandlerResult::None
                    }
                }
            }
            DatabaseMessage::DeleteAccount(id) => {
                info!("收到删除账户请求: {}", id);
                if let Some(conn) = &state.database {
                    let account_id = id;
                    let conn_clone = conn.clone();
                    HandlerResult::Task(Task::perform(
                        async move {
                            match crate::storage::delete_account(&conn_clone, account_id).await {
                                Ok(_) => MessageCategory::Database(DatabaseMessage::AccountDeleted(Ok(
                                    account_id,
                                ))),
                                Err(e) => MessageCategory::Database(DatabaseMessage::AccountDeleted(
                                    Err(e.to_string()),
                                )),
                            }
                        },
                        |msg| msg,
                    ))
                } else {
                    HandlerResult::Task(Task::done(MessageCategory::Database(
                        DatabaseMessage::AccountDeleted(Err("数据库未连接".to_string())),
                    )))
                }
            }
            DatabaseMessage::UpdateAccount(account) => {
                info!("收到更新账户请求: {}", account.username);
                if let Some(conn) = &state.database {
                    let account_clone = account.clone();

                    let conn_clone = conn.clone();
                    let handler_result = HandlerResult::Task(Task::perform(
                        async move {
                            match crate::storage::update_account(&conn_clone, &account_clone).await
                            {
                                Ok(_) => MessageCategory::Database(
                                    DatabaseMessage::AccountUpdated(Ok(())),
                                ),
                                Err(e) => MessageCategory::Database(
                                    DatabaseMessage::AccountUpdated(Err(e.to_string())),
                                ),
                            }
                        },
                        |msg| msg,
                    ));
                    return handler_result;
                }
                HandlerResult::None
            }
            DatabaseMessage::AccountUpdated(result) => {
                state.ui.set_loading(false);
                match result {
                    Ok(_) => {
                        state.ui.set_message("账户更新成功".to_string());
                        state.data.provider_page.editing_provider_id = None;
                        state.data.provider_page.form.clear();
                        // 刷新列表
                        return HandlerResult::Task(Task::done(MessageCategory::Sync(
                            SyncMessage::Reload,
                        )));
                    }
                    Err(e) => {
                        state.ui.set_message(format!("账户更新失败: {}", e));
                    }
                }
                HandlerResult::Task(Task::none())
            }
            DatabaseMessage::AccountDeleted(result) => {
                state.update(StateUpdate::Ui(UiUpdate::SetLoading(false)));
                match result {
                    Ok(id) => {
                        info!("账户删除成功: {}", id);
                        // 更新本地列表
                        state
                            .data
                            .provider_page
                            .providers
                            .retain(|p| p.account_id != id);
                        state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                            "服务商删除成功".to_string(),
                        )));
                        HandlerResult::Task(Task::none())
                    }
                    Err(e) => {
                        error!("账户删除失败: {}", e);
                        state.update(StateUpdate::Ui(UiUpdate::ShowToast(format!(
                            "删除失败: {}",
                            e
                        ))));
                        HandlerResult::Task(Task::none())
                    }
                }
            }
            DatabaseMessage::DeleteDomain(id) => {
                info!("收到删除域名请求: {}", id);
                if let Some(conn) = &state.database {
                    let domain_id = id;
                    let conn_clone = conn.clone();
                    HandlerResult::Task(Task::perform(
                        async move {
                            match crate::storage::delete_domain(&conn_clone, domain_id).await {
                                Ok(_) => MessageCategory::Provider(
                                    crate::gui::handlers::message_handler::ProviderMessage::DomainDeleted(
                                        Ok(domain_id),
                                    ),
                                ),
                                Err(e) => MessageCategory::Provider(
                                    crate::gui::handlers::message_handler::ProviderMessage::DomainDeleted(
                                        Err(e.to_string()),
                                    ),
                                ),
                            }
                        },
                        |msg| msg,
                    ))
                } else {
                    HandlerResult::Task(Task::done(MessageCategory::Provider(
                        crate::gui::handlers::message_handler::ProviderMessage::DomainDeleted(Err(
                            "数据库未连接".to_string(),
                        )),
                    )))
                }
            }
            DatabaseMessage::DomainDeleted(_) => {
                // 这个消息似乎多余了，因为我们在DeleteDomain中直接返回了ProviderMessage::DomainDeleted
                // 但如果需要统一处理，可以在这里处理
                HandlerResult::None
            }
            DatabaseMessage::AddDomain(new_domain) => {
                info!("收到添加域名请求: {}", new_domain.domain_name);
                if let Some(conn) = &state.database {
                    let conn_clone = conn.clone();
                    let account_id = new_domain.account_id;
                    HandlerResult::Task(Task::perform(
                        async move {
                            match crate::storage::add_domain(&conn_clone, new_domain).await {
                                Ok(_) => MessageCategory::Provider(
                                    crate::gui::handlers::message_handler::ProviderMessage::DomainAdded(
                                        Ok(account_id),
                                    ),
                                ),
                                Err(e) => MessageCategory::Provider(
                                    crate::gui::handlers::message_handler::ProviderMessage::DomainAdded(
                                        Err(e.to_string()),
                                    ),
                                ),
                            }
                        },
                        |msg| msg,
                    ))
                } else {
                    HandlerResult::Task(Task::done(MessageCategory::Provider(
                        crate::gui::handlers::message_handler::ProviderMessage::DomainAdded(Err(
                            "数据库未连接".to_string(),
                        )),
                    )))
                }
            }
            DatabaseMessage::DomainAdded(_) => {
                 HandlerResult::None
            }
        }
    }

    fn can_handle(&self, _message: &DatabaseMessage) -> bool {
        todo!()
    }
}
