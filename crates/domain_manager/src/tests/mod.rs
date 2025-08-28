//! 单元测试模块
//! 
//! 包含域名管理系统的各种单元测试
//! 主要测试功能：
//! - DNS记录同步测试
//! - 数据库操作测试
//! - 界面渲染测试
//! - Iced框架集成测试
//! - 阿里云客户端模拟测试

pub mod dns_sync_tests;
pub mod iced_integration_tests;
pub mod mock_aliyun_client;
pub mod test_utils;