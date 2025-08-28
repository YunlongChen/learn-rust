//! 事件处理模块
//! 
//! 该模块负责处理应用程序中的各种事件和消息，包括用户交互事件、
//! 系统事件、业务逻辑事件等。通过分离不同类型的事件处理逻辑，
//! 提高代码的可维护性和可测试性。

pub mod message_handler;
pub mod domain_handler;
pub mod dns_handler;
pub mod sync_handler;
pub mod window_handler;

pub use message_handler::MessageHandler;
pub use domain_handler::DomainHandler;
pub use dns_handler::DnsHandler;
pub use sync_handler::SyncHandler;
pub use window_handler::WindowHandler;

use crate::gui::state::AppState;
use crate::gui::Message;
use iced::Task;

/// 事件处理结果
#[derive(Debug)]
pub enum HandlerResult {
    /// 无操作
    None,
    /// 返回任务
    Task(Task<Message>),
    /// 状态已更新
    StateUpdated,
    /// 状态已更新并返回任务
    StateUpdatedWithTask(Task<Message>),
}

/// 事件处理器特征
/// 
/// 定义事件处理器的基本接口
pub trait EventHandler<T> {
    /// 处理事件
    fn handle(&self, state: &mut AppState, event: T) -> HandlerResult;
    
    /// 检查是否可以处理指定类型的事件
    fn can_handle(&self, event: &T) -> bool;
}

/// 异步事件处理器特征
/// 
/// 定义异步事件处理器的基本接口
pub trait AsyncEventHandler<T> {
    /// 异步处理事件
    fn handle_async(&self, state: &mut AppState, event: T) -> Task<Message>;
}