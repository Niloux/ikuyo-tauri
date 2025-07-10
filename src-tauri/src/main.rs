// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(e) = ikuyo_app_lib::run() {
        eprintln!("应用程序发生致命错误: {:?}", e);
        std::process::exit(1);
    }
}
