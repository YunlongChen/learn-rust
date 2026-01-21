//! UI处理器模块
//!
//! 负责处理UI相关的消息和状态更新，包括页面渲染、组件交互等

use super::{EventHandler, HandlerResult};
use crate::gui::components::console::ConsoleTab;
use crate::gui::handlers::message_handler::{
    MessageCategory, NotificationMessage, OtherMessage, UiMessage,
};
use crate::gui::pages::Page;
use crate::gui::state::app_state::{DataUpdate, StateUpdate, UiUpdate};
use crate::gui::state::AppState;
use crate::storage::DomainModal;
use crate::translations::types::locale::Locale;
use iced::{Task, Theme};
use tracing::{debug, info};

/// UI处理器
///
/// 负责处理UI相关的业务逻辑，包括：
/// - 页面切换
/// - 主题切换
/// - 搜索功能
/// - 窗口状态管理
#[derive(Debug, Default)]
pub struct UiHandler;

impl UiHandler {
    /// 创建新的UI处理器实例
    pub fn new() -> Self {
        Self
    }

    /// 处理页面切换
    ///
    /// # 参数
    /// * `state` - 应用状态
    /// * `page` - 目标页面
    fn handle_change_page(&self, state: &mut AppState, page: Page) -> HandlerResult {
        info!("切换到页面: {:?}", page);

        // 保存当前页面为上一页
        let current_page = state.ui.current_page.clone();
        state.update(StateUpdate::Ui(UiUpdate::SetLastPage(current_page)));

        // 切换到新页面
        state.update(StateUpdate::Ui(UiUpdate::SetCurrentPage(page)));

        HandlerResult::StateUpdated
    }

    /// 处理主题切换
    ///
    /// # 参数
    /// * `state` - 应用状态
    fn handle_toggle_theme(&self, state: &mut AppState) -> HandlerResult {
        info!("切换主题");
        // 获取当前主题并切换
        let current_theme = state.ui.theme.clone();
        let new_theme = match current_theme {
            Theme::TokyoNightLight => Theme::SolarizedDark,
            _ => Theme::TokyoNightLight,
        };

        state.update(StateUpdate::Ui(UiUpdate::SetTheme(new_theme)));
        HandlerResult::StateUpdated
    }

    /// 处理搜索内容变更
    ///
    /// # 参数
    /// * `state` - 应用状态
    /// * `search_content` - 搜索内容
    fn handle_search_changed(&self, state: &mut AppState, search_content: String) -> HandlerResult {
        debug!("搜索内容变更: {}", search_content);

        state.update(StateUpdate::Ui(UiUpdate::SetSearchQuery(search_content)));

        HandlerResult::StateUpdated
    }

    /// 处理重置操作
    ///
    /// # 参数
    /// * `state` - 应用状态
    fn handle_reset(&self, state: &mut AppState) -> HandlerResult {
        info!("执行重置操作");

        // 清除搜索查询
        state.update(StateUpdate::Ui(UiUpdate::SetSearchQuery(String::new())));

        // 清除选中的域名和托管商
        state.update(StateUpdate::Ui(UiUpdate::SetSelectedDomain(None)));
        state.update(StateUpdate::Ui(UiUpdate::SetSelectedProvider(None)));

        // 清除错误信息
        state.update(StateUpdate::Ui(UiUpdate::SetError(None)));

        HandlerResult::StateUpdated
    }

    /// 处理模拟数据生成
    ///
    /// # 参数
    /// * `state` - 应用状态
    fn handle_mock(&self, state: &mut AppState) -> HandlerResult {
        info!("生成模拟数据");

        // 设置加载状态
        state.update(StateUpdate::Ui(UiUpdate::SetLoading(true)));

        // 创建异步任务生成模拟数据
        let task = Task::perform(
            async {
                // 模拟异步操作
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                // 生成模拟域名数据
                let mock_domains: Vec<DomainModal> = vec![
                    crate::storage::entities::domain::Model {
                        id: 1,
                        name: "example.com".to_string(),
                        provider_id: 1,
                        status: "active".to_string(),
                        created_at: chrono::Utc::now().naive_utc(),
                        updated_at: Some(chrono::Utc::now().naive_utc()),
                    },
                    crate::storage::entities::domain::Model {
                        id: 2,
                        name: "test.org".to_string(),
                        provider_id: 1,
                        status: "active".to_string(),
                        created_at: chrono::Utc::now().naive_utc(),
                        updated_at: Some(chrono::Utc::now().naive_utc()),
                    },
                ];

                // 将存储实体转换为GUI模型
                let domains: Vec<DomainModal> = mock_domains
                    .into_iter()
                    .map(|domain| {
                        DomainModal {
                            id: domain.id,
                            name: domain.name,
                            provider_id: domain.provider_id,
                            status: domain.status,
                            // expiry_date字段在Model中不存在，暂时注释掉
                            created_at: domain.created_at,
                            updated_at: domain.updated_at,
                        }
                    })
                    .collect();
                domains
            },
            |domains| MessageCategory::Other(OtherMessage::MockDataGenerated(domains)),
        );

        HandlerResult::StateUpdatedWithTask(task)
    }

    /// 处理模拟数据生成完成
    ///
    /// # 参数
    /// * `state` - 应用状态
    /// * `domains` - 生成的模拟域名数据
    fn handle_mock_data_generated(
        &self,
        state: &mut AppState,
        domains: Vec<DomainModal>,
    ) -> HandlerResult {
        info!("模拟数据生成完成，共 {} 个域名", domains.len());
        // 更新域名列表
        state.update(StateUpdate::Data(DataUpdate::SetDomains(domains)));
        // 清除加载状态
        state.update(StateUpdate::Ui(UiUpdate::SetLoading(false)));
        HandlerResult::StateUpdated
    }

    /// 处理Toast通知显示
    ///
    /// # 参数
    /// * `state` - 应用状态
    /// * `message` - 通知消息
    fn handle_show_toast(&self, state: &mut AppState, message: String) -> HandlerResult {
        debug!("显示Toast通知: {}", message);

        state.update(StateUpdate::Ui(UiUpdate::SetToastMessage(Some(message))));
        state.update(StateUpdate::Ui(UiUpdate::SetToastVisible(true)));

        // 3秒后自动隐藏Toast
        let task = Task::perform(
            async {
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            },
            |_| MessageCategory::Notification(NotificationMessage::HideToast),
        );
        HandlerResult::StateUpdatedWithTask(task)
    }

    /// 处理Toast通知隐藏
    ///
    /// # 参数
    /// * `state` - 应用状态
    fn handle_hide_toast(&self, state: &mut AppState) -> HandlerResult {
        debug!("隐藏Toast通知");

        state.update(StateUpdate::Ui(UiUpdate::SetToastVisible(false)));
        state.update(StateUpdate::Ui(UiUpdate::SetToastMessage(None)));

        HandlerResult::StateUpdated
    }

    /// 处理窗口状态切换
    ///
    /// # 参数
    /// * `state` - 应用状态
    fn handle_toggle_floating_window(&self, state: &mut AppState) -> HandlerResult {
        info!("切换悬浮窗状态");

        let current_state = state.ui.floating_window_enabled;
        state.update(StateUpdate::Ui(UiUpdate::SetFloatingWindowEnabled(
            !current_state,
        )));

        HandlerResult::StateUpdated
    }

    /// 处理缩略图模式切换
    ///
    /// # 参数
    /// * `state` - 应用状态
    fn handle_toggle_thumbnail(&self, state: &mut AppState) -> HandlerResult {
        info!("切换缩略图模式");

        let current_state = state.ui.thumbnail_mode;
        state.update(StateUpdate::Ui(UiUpdate::SetThumbnailMode(!current_state)));

        HandlerResult::StateUpdated
    }

    /// 处理控制台切换
    ///
    /// # 参数
    /// * `state` - 应用状态
    fn handle_toggle_console(&self, state: &mut AppState) -> HandlerResult {
        info!("切换控制台显示状态");
        let current_state = state.ui.console_visible;
        state.update(StateUpdate::Ui(UiUpdate::SetConsoleVisible(!current_state)));
        HandlerResult::StateUpdated
    }

    /// 处理清除控制台日志
    ///
    /// # 参数
    /// * `state` - 应用状态
    fn handle_clear_console_log(&self, state: &mut AppState) -> HandlerResult {
        info!("清除控制台日志");

        state.update(StateUpdate::Ui(UiUpdate::ClearConsoleLogs));

        HandlerResult::StateUpdated
    }

    /// 处理控制台标签页切换
    ///
    /// # 参数
    /// * `state` - 应用状态
    /// * `tab` - 目标标签页
    fn handle_console_tab_changed(&self, state: &mut AppState, tab: ConsoleTab) -> HandlerResult {
        info!("切换控制台标签页: {:?}", tab);
        state.update(StateUpdate::Ui(UiUpdate::ToggleConsole));
        HandlerResult::StateUpdated
    }

    /// 处理控制台标签页切换
    ///
    /// # 参数
    /// * `state` - 应用状态
    /// * `tab` - 目标标签页
    fn handle_toggle_locale(&self, state: &mut AppState, locale: Locale) -> HandlerResult {
        info!("切换语言标签页: {:?}", locale);
        state.update(StateUpdate::Ui(UiUpdate::ToggleLocale(locale)));
        HandlerResult::StateUpdated
    }
}

impl EventHandler<UiMessage> for UiHandler {
    fn handle(&self, state: &mut AppState, message: UiMessage) -> HandlerResult {
        match message {
            UiMessage::ChangePage(page) => self.handle_change_page(state, page),
            UiMessage::ToggleTheme => self.handle_toggle_theme(state),
            UiMessage::SearchContentChanged(search_content) => {
                self.handle_search_changed(state, search_content)
            }
            UiMessage::Reset => self.handle_reset(state),
            UiMessage::Mock => self.handle_mock(state),
            UiMessage::MockDataGenerated(domains) => {
                self.handle_mock_data_generated(state, domains)
            }
            UiMessage::ShowToast(message) => self.handle_show_toast(state, message),
            UiMessage::HideToast => self.handle_hide_toast(state),
            UiMessage::ToggleFloatingWindow => self.handle_toggle_floating_window(state),
            UiMessage::ToggleThumbnail => self.handle_toggle_thumbnail(state),
            UiMessage::ToggleConsole => self.handle_toggle_console(state),
            UiMessage::ClearConsoleLog => self.handle_clear_console_log(state),
            UiMessage::ConsoleTabChanged(tab) => self.handle_console_tab_changed(state, tab),
            UiMessage::ToggleLocale(locale) => self.handle_toggle_locale(state, locale),
        }
    }

    /// 检查是否可以处理该消息
    fn can_handle(&self, _event: &UiMessage) -> bool {
        true // UiHandler可以处理所有UiMessage
    }
}
