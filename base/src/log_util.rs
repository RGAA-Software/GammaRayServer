use tracing::level_filters::LevelFilter;
use tracing_log::LogTracer;
use tracing_subscriber::{fmt, Layer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn init_log(path: String, name: String) -> tracing_appender::non_blocking::WorkerGuard{
    LogTracer::builder()
        // .with_max_level(log::LevelFilter::Error)
        .init()
        .expect("LogTracer init failed.");

    let fmt_layer = fmt::layer()
        .with_level(true)
        .with_writer(std::io::stdout)
        .with_filter(LevelFilter::INFO);
    
    let file_appender = tracing_appender::rolling::daily(path, name);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_writer(non_blocking)
        .with_filter(LevelFilter::INFO);
    
    let collector = tracing_subscriber::registry()
        .with(file_layer)
        .with(fmt_layer);
    
    tracing::subscriber::set_global_default(collector).expect("Tracing collect error");

    guard
}