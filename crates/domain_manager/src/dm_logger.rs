use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

pub fn init_logging() {
    info!("初始化日志框架");
    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::INFO)
        // completes the builder.
        .with_test_writer()
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("设置默认订阅者失败");
}
