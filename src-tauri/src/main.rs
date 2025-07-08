// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing_subscriber::{filter::EnvFilter, fmt, prelude::*, registry};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

fn main() {
    // 1. 定义日志文件路径和名称
    let file_appender = RollingFileAppender::new(Rotation::DAILY, "logs", "ikuyo.log");
    let (non_blocking_file_appender, _guard) = tracing_appender::non_blocking(file_appender);

    // 2. 配置控制台输出
    let console_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .pretty(); // 漂亮的控制台输出

    // 3. 配置文件输出
    let file_layer = fmt::layer()
        .with_writer(non_blocking_file_appender)
        .json(); // JSON 格式的文件输出，方便机器解析

    // 4. 配置环境变量过滤器
    // 允许通过 RUST_LOG 环境变量控制日志级别，例如 RUST_LOG=info,ikuyo_app_lib=debug
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info")); // 默认日志级别为 info

    // 5. 组合所有层并初始化日志系统
    registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    tracing::info!("main入口启动，日志系统已初始化并验证");
    ikuyo_app_lib::run();
}