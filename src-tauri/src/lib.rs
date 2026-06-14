use std::sync::Mutex;
use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

mod audio_capture;
mod cloud_api;
mod commands;
mod whisper_engine;

#[cfg(target_os = "windows")]
extern "system" {
    fn Beep(dwFreq: u32, dwDuration: u32) -> i32;
}

use audio_capture::AudioRecorder;
use cloud_api::{CloudApiClient, CloudApiConfig};
use whisper_engine::WhisperEngine;

pub struct AppState {
    pub is_recording: Mutex<bool>,
    pub audio_recorder: AudioRecorder,
    pub whisper_engine: Mutex<WhisperEngine>,
    pub cloud_client: Mutex<CloudApiClient>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            is_recording: Mutex::new(false),
            audio_recorder: AudioRecorder::new().expect("无法初始化音频录音器"),
            whisper_engine: Mutex::new(
                WhisperEngine::new("models/ggml-medium.bin")
                    .unwrap_or_else(|_| WhisperEngine::new("").unwrap()),
            ),
            cloud_client: Mutex::new(CloudApiClient::new(CloudApiConfig::default())),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    println!("[Voice2Text] 启动中...");

    let app_state = AppState::new();
    println!("[Voice2Text] 引擎初始化完成");

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .setup(|app| {
            // ========================================
            // 系统托盘
            // ========================================
            let menu = tauri::menu::MenuBuilder::new(app)
                .text("show", "显示窗口")
                .separator()
                .text("quit", "退出")
                .build()?;

            let _tray = TrayIconBuilder::new()
                .tooltip("Voice2Text — Ctrl+Alt+Z 开始 | Ctrl+Alt+X 停止")
                .menu(&menu)
                .on_tray_icon_event(|tray_handle, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        if let Some(window) = tray_handle.app_handle().get_webview_window("main")
                        {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .on_menu_event(|tray_handle, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) =
                            tray_handle.app_handle().get_webview_window("main")
                        {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        tray_handle.app_handle().exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            // ========================================
            // 全局快捷键
            //  Ctrl+Shift+R = 开始录音
            //  Ctrl+Shift+S = 停止录音 + 识别 + 粘贴
            // ========================================
            {

                // 开始录音快捷键: Ctrl+Alt+Z
                app.global_shortcut()
                    .on_shortcut("Ctrl+Alt+Z", move |app, _shortcut, event| {
                        if event.state() != ShortcutState::Pressed {
                            return;
                        }
                        let state = app.state::<AppState>();
                        let mut is_rec = state.is_recording.lock().unwrap();
                        if *is_rec {
                            return; // 已在录音中
                        }
                        println!("[热键] ▶ 开始录音");
                        match state.audio_recorder.start() {
                            Ok(()) => {
                                *is_rec = true;
                                if let Some(tray) = app.tray_by_id("main") {
                                    let _ = tray.set_tooltip(Some("🔴 录音中… Ctrl+Alt+X 停止"));
                                }
                                beep(880, 150);
                                if let Some(w) = app.get_webview_window("main") {
                                    let _ = w.emit("recording-started", ());
                                }
                            }
                            Err(e) => eprintln!("[热键] 录音启动失败: {}", e),
                        }
                    })
                    .expect("注册开始快捷键失败");

                // 停止录音快捷键: Ctrl+Alt+X
                app.global_shortcut()
                    .on_shortcut("Ctrl+Alt+X", move |app, _shortcut, event| {
                        if event.state() != ShortcutState::Pressed {
                            return;
                        }
                        let state = app.state::<AppState>();
                        let mut is_rec = state.is_recording.lock().unwrap();
                        if !*is_rec {
                            return; // 未在录音
                        }
                        println!("[热键] ■ 停止录音");
                        *is_rec = false;
                        if let Some(tray) = app.tray_by_id("main") {
                            let _ = tray.set_tooltip(Some("Voice2Text — Ctrl+Alt+Z 开始"));
                        }
                        beep(660, 100);
                        std::thread::sleep(std::time::Duration::from_millis(80));
                        beep(880, 150);

                            let audio_samples = match state.audio_recorder.stop() {
                                Ok(data) => data,
                                Err(e) => {
                                    eprintln!("[热键] 停止录音失败: {}", e);
                                    return;
                                }
                            };

                            let sample_rate = state.audio_recorder.sample_rate();
                            if audio_samples.is_empty() {
                                println!("[热键] 无录音数据");
                                return;
                            }

                            // 识别
                            let text = {
                                let mut engine = state.whisper_engine.lock().unwrap();
                                match engine.transcribe(&audio_samples, sample_rate) {
                                    Ok(t) => t,
                                    Err(e) => {
                                        eprintln!("[热键] 识别失败: {}", e);
                                        return;
                                    }
                                }
                            };

                            let preview: String = text.chars().take(60).collect();
                            println!(
                                "[热键] 识别完成 ({} 字): {}...",
                                text.chars().count(),
                                preview
                            );

                            // 复制到剪贴板
                            match arboard::Clipboard::new() {
                                Ok(mut clip) => {
                                    if let Err(e) = clip.set_text(&text) {
                                        eprintln!("[热键] 剪贴板失败: {}", e);
                                    } else {
                                        println!("[热键] 已复制到剪贴板");
                                    }
                                }
                                Err(e) => eprintln!("[热键] 剪贴板错误: {}", e),
                            }

                            // 模拟 Ctrl+V 粘贴到活动窗口
                            std::thread::spawn(|| {
                                std::thread::sleep(std::time::Duration::from_millis(100));
                                simulate_paste();
                            });

                            // 通知前端
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.emit("recording-stopped", &text);
                            }
                    })
                    .expect("注册停止快捷键失败");

                println!("[Voice2Text] 快捷键 Ctrl+Alt+Z 开始 / Ctrl+Alt+X 停止 已注册");
            }

            // 启动时隐藏主窗口（后台运行）
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_recording,
            commands::stop_recording,
            commands::get_transcribed_text,
            commands::clear_text,
            commands::copy_to_clipboard,
            commands::get_recording_status,
            commands::get_audio_info,
        ])
        .run(tauri::generate_context!())
        .expect("启动应用失败");
}

/// 播放系统提示音
fn beep(freq: u32, duration_ms: u32) {
    #[cfg(target_os = "windows")]
    unsafe {
        Beep(freq, duration_ms);
    }
    #[cfg(not(target_os = "windows"))]
    {
        eprint!("\x07");
    }
}

/// 模拟 Ctrl+V 粘贴
fn simulate_paste() {
    #[cfg(target_os = "windows")]
    {
        // PowerShell SendKeys 模拟 Ctrl+V
        let script = r#"
            Add-Type -AssemblyName System.Windows.Forms
            [System.Windows.Forms.SendKeys]::SendWait('^v')
        "#;
        match std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .output()
        {
            Ok(out) => {
                if out.status.success() {
                    println!("[粘贴] Ctrl+V 已发送");
                } else {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    eprintln!("[粘贴] PowerShell 失败: {}", stderr.trim());
                }
            }
            Err(e) => eprintln!("[粘贴] 启动 PowerShell 失败: {}", e),
        }
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: osascript 模拟 Cmd+V
        let script = r#"tell application "System Events" to keystroke "v" using command down"#;
        match std::process::Command::new("osascript").arg("-e").arg(script).output() {
            Ok(out) => {
                if out.status.success() {
                    println!("[粘贴] Cmd+V 已发送");
                } else {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    eprintln!("[粘贴] osascript 失败: {}", stderr.trim());
                }
            }
            Err(e) => eprintln!("[粘贴] osascript 失败: {}", e),
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: xdotool 模拟 Ctrl+V
        match std::process::Command::new("xdotool")
            .args(["key", "ctrl+v"])
            .output()
        {
            Ok(out) => {
                if out.status.success() {
                    println!("[粘贴] Ctrl+V 已发送");
                } else {
                    eprintln!("[粘贴] xdotool 失败（可能需要安装 xdotool）");
                }
            }
            Err(e) => eprintln!("[粘贴] xdotool 不可用: {}", e),
        }
    }
}
