use crate::configs::gui_config::Config;
use tracing::{info, Level};

#[cfg(feature = "logging")]
use crate::configs::logging_config::{LogLevel, LogOutput, LogFormat};

/// 初始化日志系统
pub fn init_logging(config: &Config) {
    info!("初始化日志框架");

    #[cfg(feature = "logging")]
    {
        if let Some(logging_config) = &config.logging_config {
            init_advanced_logging(logging_config);
            return;
        }
    }

    // 回退到简单日志配置
    init_simple_logging(config);
}

/// 初始化高级日志配置（支持文件输出、多级别等）
#[cfg(feature = "logging")]
fn init_advanced_logging(config: &crate::configs::logging_config::LoggingConfig) {
    let max_level: Level = config.level.clone().into();

    // 配置输出
    match &config.output {
        LogOutput::Console => {
            init_console_logging(&config, max_level);
        }
        LogOutput::File { path, max_size_mb: _, max_files: _ } => {
            init_file_logging(&config, max_level, path);
        }
        LogOutput::Both { file_path, max_size_mb: _, max_files: _ } => {
            init_both_logging(&config, max_level, file_path);
        }
    }
}

/// 初始化控制台日志
#[cfg(feature = "logging")]
fn init_console_logging(config: &crate::configs::logging_config::LoggingConfig, max_level: Level) {
    let subscriber = tracing_subscriber::fmt()
        .with_target(true)
        .with_level(true)
        .with_ansi(config.enable_colors)
        .with_max_level(max_level)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("设置默认订阅者失败");
}

/// 初始化文件日志（简化版本）
#[cfg(feature = "logging")]
fn init_file_logging(
    config: &crate::configs::logging_config::LoggingConfig,
    max_level: Level,
    path: &std::path::PathBuf,
) {
    // 确保日志目录存在
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // 创建文件
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap_or_else(|_| panic!("无法打开日志文件: {:?}", path));

    let subscriber = tracing_subscriber::fmt()
        .with_target(true)
        .with_level(true)
        .with_ansi(false) // 文件输出不需要ANSI颜色
        .with_writer(file)
        .with_max_level(max_level)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("设置默认订阅者失败");
}

/// 初始化同时输出到控制台和文件（简化版本）
#[cfg(feature = "logging")]
fn init_both_logging(
    config: &crate::configs::logging_config::LoggingConfig,
    max_level: Level,
    file_path: &std::path::PathBuf,
) {
    // 确保日志目录存在
    if let Some(parent) = file_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // 创建文件
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
        .unwrap_or_else(|_| panic!("无法打开日志文件: {:?}", file_path));

    // 使用Layer API来实现同时输出
    use tracing_subscriber::layer::SubscriberExt;

    let console_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_ansi(config.enable_colors);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_ansi(false)
        .with_writer(file);

    let subscriber = tracing_subscriber::registry()
        .with(console_layer.with_max_level(max_level))
        .with(file_layer.with_max_level(max_level));

    tracing::subscriber::set_global_default(subscriber).expect("设置默认订阅者失败");
}

/// 初始化简单日志配置（向后兼容）
fn init_simple_logging(config: &Config) {
    let max_level = match config.logging_config.as_ref() {
        Some(level_str) => {
            #[cfg(not(feature = "logging"))]
            {
                match level_str.as_str() {
                    "TRACE" => Level::TRACE,
                    "DEBUG" => Level::DEBUG,
                    "INFO" => Level::INFO,
                    "WARN" => Level::WARN,
                    "ERROR" => Level::ERROR,
                    _ => Level::INFO,
                }
            }
            #[cfg(feature = "logging")]
            {
                Level::INFO // 默认级别
            }
        }
        None => Level::INFO,
    };

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(max_level)
        .with_test_writer()
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("设置默认订阅者失败");
}