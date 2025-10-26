#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod config;
mod input_method;

use commands::{paste, paste_instant};
use config::{get_hotkey, set_hotkey, AppConfig};
use input_method::{switch_to_english_ime, get_current_layout_info};
use std::env;
use std::sync::Mutex;
use tauri::{GlobalShortcutManager, Manager};

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // 应用启动时切换到英文输入法
            #[cfg(target_os = "windows")]
            {
                println!("应用启动，切换输入法...");
                println!("状态: {}", get_current_layout_info());
                
                // 延迟一小段时间，确保窗口已完全创建
                std::thread::sleep(std::time::Duration::from_millis(100));
                
                match switch_to_english_ime() {
                    Ok(_) => {
                        println!("✓ 已自动切换到英文输入法");
                    }
                    Err(e) => {
                        eprintln!("✗ 切换输入法失败: {}", e);
                    }
                }
            }
            
            // 加载配置
            let config = AppConfig::load();
            let hotkey = config.hotkey.clone();
            let hotkey_for_closure = hotkey.clone();
            
            // 注册全局快捷键
            let app_handle = app.handle();
            let mut shortcut_manager = app.app_handle().global_shortcut_manager();
            
            match shortcut_manager.register(&hotkey, move || {
                println!("全局快捷键被触发: {}", hotkey_for_closure);
                
                // 获取窗口以读取当前配置
                if let Some(window) = app_handle.get_window("main") {
                    // 触发前端的快捷键粘贴事件
                    let _ = window.emit("shortcut-paste", ());
                }
            }) {
                Ok(_) => println!("✓ 全局快捷键已注册: {}", hotkey),
                Err(e) => eprintln!("✗ 注册全局快捷键失败: {}", e),
            }
            
            // 保存快捷键到应用状态（用于后续更新快捷键）
            app.manage(Mutex::new(hotkey));
            
            // 使用 tauri::async_runtime 来延迟注册窗口事件
            let app_handle_clone = app.handle();
            tauri::async_runtime::spawn(async move {
                std::thread::sleep(std::time::Duration::from_millis(200));
                
                if let Some(window) = app_handle_clone.get_window("main") {
                    window.on_window_event(move |event| {
                        if let tauri::WindowEvent::Focused(focused) = event {
                            if *focused {
                                #[cfg(target_os = "windows")]
                                {
                                    println!("窗口获得焦点，切换到英文输入法");
                                    let _ = switch_to_english_ime();
                                }
                            }
                        }
                    });
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            paste,
            paste_instant,
            get_hotkey,
            set_hotkey
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
