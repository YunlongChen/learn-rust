//! 窗口处理器
//!
//! 负责处理所有与窗口操作相关的业务逻辑，包括窗口拖拽、大小调整、
//! 最小化、最大化、透明度调整等操作。

use super::message_handler::{MessageCategory, WindowMessage};
use super::{EventHandler, HandlerResult};
use crate::gui::state::app_state::{ConfigUpdate, StateUpdate, UiUpdate};
use crate::gui::state::AppState;
use iced::window::Id;
use iced::{window, Point, Size, Task};
use tracing::{error, info};

/// 窗口处理器
///
/// 专门处理窗口相关的事件和业务逻辑
#[derive(Debug)]
pub struct WindowHandler {
    // 窗口状态相关字段
    is_dragging: bool,
    drag_start_position: Option<Point>,
    last_window_position: Option<Point>,
    last_window_size: Option<Size>,
}

/// 窗口状态
#[derive(Debug, Clone, PartialEq)]
pub enum WindowState {
    /// 正常状态
    Normal,
    /// 最小化
    Minimized,
    /// 最大化
    Maximized,
    /// 悬浮窗模式
    Floating,
}

/// 窗口操作结果
#[derive(Debug, Clone)]
pub enum WindowOperationResult {
    /// 操作成功
    Success(String),
    /// 操作失败
    Failed(String),
    /// 操作被忽略
    Ignored(String),
}

impl WindowHandler {
    /// 创建新的窗口处理器
    pub fn new() -> Self {
        Self {
            is_dragging: false,
            drag_start_position: None,
            last_window_position: None,
            last_window_size: None,
        }
    }

    /// 处理开始拖拽窗口
    fn handle_start_drag(&mut self, state: &mut AppState, id: Id) -> HandlerResult {
        self.is_dragging = true;

        // 记录当前窗口位置
        if let Some(current_pos) = self.last_window_position {
            self.last_window_position = Some(current_pos);
        }

        state.ui.set_message("开始拖拽窗口".to_string());
        HandlerResult::StateUpdated
    }

    /// 处理拖拽窗口
    fn handle_drag_window(&mut self, state: &mut AppState, position: Point) -> HandlerResult {
        if !self.is_dragging {
            return HandlerResult::NoChange;
        }

        if let Some(start_pos) = self.drag_start_position {
            let delta_x = position.x - start_pos.x;
            let delta_y = position.y - start_pos.y;

            // 计算新的窗口位置
            let new_position = if let Some(last_pos) = self.last_window_position {
                Point::new(last_pos.x + delta_x, last_pos.y + delta_y)
            } else {
                Point::new(delta_x, delta_y)
            };

            // 这里应该调用实际的窗口移动API
            // 暂时只更新状态
            self.last_window_position = Some(new_position);

            state.ui.set_message(format!(
                "拖拽窗口到位置: ({:.0}, {:.0})",
                new_position.x, new_position.y
            ));
        }

        HandlerResult::StateUpdated
    }

    /// 处理窗口移动完成
    fn handle_window_moved(&mut self, state: &mut AppState, position: Point) -> HandlerResult {
        self.is_dragging = false;
        self.drag_start_position = None;
        self.last_window_position = Some(position);

        state.ui.set_message(format!(
            "窗口移动到: ({:.0}, {:.0})",
            position.x, position.y
        ));

        HandlerResult::StateUpdated
    }

    /// 处理窗口大小调整
    fn handle_window_resized(&mut self, state: &mut AppState, size: Size) -> HandlerResult {
        self.last_window_size = Some(size);

        state.ui.set_message(format!(
            "窗口大小调整为: {:.0}x{:.0}",
            size.width, size.height
        ));

        // 检查是否需要调整UI布局
        if size.width < 800.0 || size.height < 600.0 {
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                "窗口尺寸过小，可能影响显示效果".to_string(),
            )));
        }

        HandlerResult::StateUpdated
    }

    /// 处理窗口最小化
    fn handle_window_minimize(&mut self, state: &mut AppState) -> HandlerResult {
        state.ui.set_message("窗口最小化".to_string());
        // 这里可以添加最小化时的特殊处理逻辑
        // 比如暂停某些后台任务等

        // 处理窗口最小化事件
        info!("窗口最小化");
        state.ui.window_minimize = true;
        HandlerResult::StateUpdated
    }

    /// 处理窗口最大化/还原
    fn handle_window_maximize(
        &mut self,
        state: &mut AppState,
        is_maximized: bool,
    ) -> HandlerResult {
        let message = if is_maximized {
            "窗口已最大化"
        } else {
            "窗口已还原"
        };

        state.ui.set_message(message.to_string());
        state.ui.window_maximized = is_maximized;

        HandlerResult::StateUpdated
    }

    /// 处理切换悬浮窗模式
    fn handle_toggle_floating(&mut self, state: &mut AppState) -> HandlerResult {
        let new_floating_mode = !state.ui.floating_mode;
        state.ui.floating_mode = new_floating_mode;

        let message = if new_floating_mode {
            "已切换到悬浮窗模式"
        } else {
            "已退出悬浮窗模式"
        };

        state.ui.set_message(message.to_string());
        state.update(StateUpdate::Ui(UiUpdate::ShowToast(message.to_string())));

        HandlerResult::StateUpdated
    }

    /// 处理背景透明度变更
    fn handle_background_opacity_change(
        &mut self,
        state: &mut AppState,
        opacity: f32,
    ) -> HandlerResult {
        // 限制透明度范围
        let clamped_opacity = opacity.clamp(0.1, 1.0);
        state.ui.background_opacity = clamped_opacity;

        state
            .ui
            .set_message(format!("背景透明度调整为: {:.1}%", clamped_opacity * 100.0));

        HandlerResult::StateUpdated
    }

    /// 处理背景切换
    fn handle_background_toggle(&mut self, state: &mut AppState) -> HandlerResult {
        // 这里可以实现背景图片或颜色的切换逻辑
        state.ui.set_message("背景已切换".to_string());

        HandlerResult::StateUpdated
    }

    /// 处理窗口关闭请求
    fn handle_window_close_request(&mut self, state: &mut AppState) -> HandlerResult {
        // 检查是否有未保存的数据
        if state.data.has_unsaved_changes() {
            state.update(StateUpdate::Ui(UiUpdate::ShowToast(
                "有未保存的更改，请先保存数据".to_string(),
            )));
            return HandlerResult::StateUpdated;
        }

        state.ui.set_message("准备关闭应用程序".to_string());

        // 这里可以添加清理逻辑
        // 比如保存窗口位置、大小等设置

        HandlerResult::StateUpdated
    }

    /// 获取当前窗口状态
    pub fn get_window_state(&self, state: &AppState) -> WindowState {
        if state.ui.floating_mode {
            WindowState::Floating
        } else if state.ui.window_maximized {
            WindowState::Maximized
        } else {
            WindowState::Normal
        }
    }

    /// 获取窗口位置
    pub fn get_window_position(&self) -> Option<Point> {
        self.last_window_position
    }

    /// 获取窗口大小
    pub fn get_window_size(&self) -> Option<Size> {
        self.last_window_size
    }

    /// 检查是否正在拖拽
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    /// 重置拖拽状态
    pub fn reset_drag_state(&mut self) {
        self.is_dragging = false;
        self.drag_start_position = None;
    }

    /// 保存窗口设置
    pub fn save_window_settings(&self, state: &AppState) -> Result<(), String> {
        // 这里应该将窗口设置保存到配置文件
        // 包括位置、大小、透明度等

        let position = self.last_window_position.map(|p| (p.x, p.y));
        let size = self.last_window_size.map(|s| (s.width, s.height));

        let settings = serde_json::json!({
            "position": position,
            "size": size,
            "is_maximized": state.ui.window_maximized,
            "is_floating_mode": state.ui.floating_mode,
            "background_opacity": state.ui.background_opacity,
        });

        // 暂时只是模拟保存
        println!("保存窗口设置: {}", settings);

        Ok(())
    }

    /// 加载窗口设置
    pub fn load_window_settings(&mut self, state: &mut AppState) -> Result<(), String> {
        // 这里应该从配置文件加载窗口设置
        // 暂时使用默认值

        self.last_window_position = Some(Point::new(100.0, 100.0));
        self.last_window_size = Some(Size::new(1200.0, 800.0));
        state.ui.window_maximized = false;
        state.ui.floating_mode = false;
        state.ui.background_opacity = 1.0;

        Ok(())
    }
}

impl EventHandler<WindowMessage> for WindowHandler {
    fn handle(&self, state: &mut AppState, event: WindowMessage) -> HandlerResult {
        // 由于需要修改self，这里需要特殊处理
        // 在实际实现中，可能需要使用RefCell或其他方式
        match event {
            WindowMessage::Drag => HandlerResult::StateUpdated,
            WindowMessage::Moved(position) => {
                // 处理窗口移动事件，更新配置中的窗口位置
                info!("窗口移动到位置: ({}, {})", position.x, position.y);
                state.update(StateUpdate::Config(ConfigUpdate::UpdateWindowConfig(
                    Size::new(state.ui.window_state.width, state.ui.window_state.height),
                    Point::new(position.x, position.y),
                )));
                HandlerResult::StateUpdated
            }
            WindowMessage::Resized(size) => HandlerResult::StateUpdated,
            WindowMessage::Maximize => HandlerResult::StateUpdated,
            WindowMessage::DragWindow(position) => {
                HandlerResult::Task(
                    // 获取最旧的窗口并拖动
                    window::get_oldest().then(|id_option| {
                        if let Some(id) = id_option {
                            Task::done(MessageCategory::Window(WindowMessage::StartDrag(id)))
                        } else {
                            Task::none()
                        }
                    }),
                )
            }
            WindowMessage::StartDrag(id) => {
                // 这里需要可变引用，实际实现时需要调整
                HandlerResult::Task(window::drag(id))
            }
            WindowMessage::WindowResized(size) => HandlerResult::StateUpdated,
            WindowMessage::WindowMinimize => {
                state.ui.set_message("窗口最小化".to_string());
                info!("窗口最小化");
                state.ui.window_minimize = true;
                HandlerResult::StateUpdated
            }
            WindowMessage::WindowMaximize(is_maximized) => HandlerResult::StateUpdated,
            WindowMessage::ToggleFloating => HandlerResult::StateUpdated,
            WindowMessage::BackgroundOpacityChange(opacity) => HandlerResult::StateUpdated,
            WindowMessage::BackgroundToggle => HandlerResult::StateUpdated,
            WindowMessage::CloseRequest => HandlerResult::StateUpdated,
            WindowMessage::WindowFocused => HandlerResult::StateUpdated,
            WindowMessage::WindowId(_) => HandlerResult::StateUpdated,
        }
    }

    /// 检查是否可以处理该消息
    fn can_handle(&self, _event: &WindowMessage) -> bool {
        true // WindowHandler可以处理所有WindowMessage
    }
}

impl Default for WindowHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// 窗口处理器的可变版本
///
/// 由于窗口处理器需要维护内部状态，提供一个可变的包装器
pub struct MutableWindowHandler {
    handler: std::cell::RefCell<WindowHandler>,
}

impl MutableWindowHandler {
    pub fn new() -> Self {
        Self {
            handler: std::cell::RefCell::new(WindowHandler::new()),
        }
    }

    pub fn handle(&self, state: &mut AppState, event: WindowMessage) -> HandlerResult {
        let mut handler = self.handler.borrow_mut();
        match event {
            WindowMessage::Drag => HandlerResult::StateUpdated,
            // StartDrag不再需要position参数
            WindowMessage::StartDrag(id) => handler.handle_start_drag(state, id),
            WindowMessage::Moved(position) => handler.handle_window_moved(state, position),
            WindowMessage::Resized(size) => handler.handle_window_resized(state, size),
            WindowMessage::Maximize => HandlerResult::StateUpdated,
            WindowMessage::DragWindow(position) => handler.handle_drag_window(state, position),
            WindowMessage::WindowResized(size) => handler.handle_window_resized(state, size),
            WindowMessage::WindowMinimize => handler.handle_window_minimize(state),
            WindowMessage::WindowMaximize(is_maximized) => {
                handler.handle_window_maximize(state, is_maximized)
            }
            WindowMessage::ToggleFloating => handler.handle_toggle_floating(state),
            WindowMessage::BackgroundOpacityChange(opacity) => {
                handler.handle_background_opacity_change(state, opacity)
            }
            WindowMessage::BackgroundToggle => handler.handle_background_toggle(state),
            WindowMessage::CloseRequest => handler.handle_window_close_request(state),
            WindowMessage::WindowFocused => todo!(),
            WindowMessage::WindowId(_) => todo!(),
        }
    }

    pub fn get_window_state(&self, state: &AppState) -> WindowState {
        self.handler.borrow().get_window_state(state)
    }

    pub fn save_window_settings(&self, state: &AppState) -> Result<(), String> {
        self.handler.borrow().save_window_settings(state)
    }

    pub fn load_window_settings(&self, state: &mut AppState) -> Result<(), String> {
        self.handler.borrow_mut().load_window_settings(state)
    }
}

impl Default for MutableWindowHandler {
    fn default() -> Self {
        Self::new()
    }
}
