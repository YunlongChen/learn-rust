//! 测试工具模块
//!
//! 提供测试中使用的通用工具函数，包括：
//! - 统一的日志初始化
//! - 测试数据库设置
//! - 测试数据生成

use std::sync::Once;
use tracing::info;

static INIT: Once = Once::new();

/// 初始化测试环境
/// 
/// 确保日志系统只初始化一次，避免Once poisoned错误
pub fn init_test_env() {
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();
        info!("测试环境初始化完成");
    });
}