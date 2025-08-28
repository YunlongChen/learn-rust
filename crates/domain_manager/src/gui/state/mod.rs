//! 状态管理模块
//! 
//! 该模块负责管理应用程序的所有状态，包括UI状态、数据状态和配置状态。
//! 通过分离不同类型的状态，提高代码的可维护性和可测试性。

pub mod app_state;
pub mod ui_state;
pub mod data_state;

pub use app_state::AppState;
pub use ui_state::UiState;
pub use data_state::DataState;