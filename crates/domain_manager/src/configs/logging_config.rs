use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 日志输出配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LogOutput {
    /// 输出到控制台
    Console,
    /// 输出到文件
    File {
        path: PathBuf,
        max_size_mb: Option<u64>,
        max_files: Option<u32>,
    },
    /// 同时输出到控制台和文件
    Both {
        file_path: PathBuf,
        max_size_mb: Option<u64>,
        max_files: Option<u32>,
    },
}

/// 日志格式配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LogFormat {
    /// 简单文本格式
    Text,
    /// JSON格式
    Json,
    /// 紧凑格式
    Compact,
}

/// 日志级别配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<&str> for LogLevel {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "TRACE" => LogLevel::Trace,
            "DEBUG" => LogLevel::Debug,
            "INFO" => LogLevel::Info,
            "WARN" => LogLevel::Warn,
            "ERROR" => LogLevel::Error,
            _ => LogLevel::Info, // 默认级别
        }
    }
}

impl From<LogLevel> for tracing::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => tracing::Level::TRACE,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}

/// 完整的日志配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoggingConfig {
    /// 日志级别
    pub level: LogLevel,
    /// 输出配置
    pub output: LogOutput,
    /// 日志格式
    pub format: LogFormat,
    /// 是否启用颜色（仅对控制台输出有效）
    pub enable_colors: bool,
    /// 模块过滤（可选）
    pub module_filter: Option<Vec<String>>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig {
            level: LogLevel::Info,
            output: LogOutput::Console,
            format: LogFormat::Text,
            enable_colors: true,
            module_filter: None,
        }
    }
}