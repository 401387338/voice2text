use crate::AppState;
use arboard::Clipboard;
use tauri::State;

/// 开始录音
#[tauri::command]
pub fn start_recording(state: State<AppState>) -> Result<(), String> {
    let mut is_recording = state.is_recording.lock().map_err(|e| e.to_string())?;
    if *is_recording {
        return Err("已经在录音中".into());
    }
    state.audio_recorder.start().map_err(|e| e.to_string())?;
    *is_recording = true;
    Ok(())
}

/// 停止录音并返回识别结果
#[tauri::command]
pub fn stop_recording(state: State<AppState>) -> Result<String, String> {
    let mut is_recording = state.is_recording.lock().map_err(|e| e.to_string())?;
    if !*is_recording {
        return Err("当前未在录音".into());
    }
    *is_recording = false;

    let audio_samples = state.audio_recorder.stop().map_err(|e| e.to_string())?;
    let sample_rate = state.audio_recorder.sample_rate();

    if audio_samples.is_empty() {
        return Err("没有录制到音频数据".into());
    }

    let mut engine = state.whisper_engine.lock().map_err(|e| e.to_string())?;
    engine
        .transcribe(&audio_samples, sample_rate)
        .map_err(|e| e.to_string())
}

/// 复制文本到剪贴板（前端调用）
#[tauri::command]
pub fn copy_to_clipboard(text: String) -> Result<(), String> {
    if text.is_empty() {
        return Err("文本为空".into());
    }
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(&text).map_err(|e| e.to_string())?;
    Ok(())
}

/// 获取当前录音状态
#[tauri::command]
pub fn get_recording_status(state: State<AppState>) -> Result<bool, String> {
    state
        .is_recording
        .lock()
        .map(|b| *b)
        .map_err(|e| e.to_string())
}

/// 获取音频设备信息
#[tauri::command]
pub fn get_audio_info(state: State<AppState>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "sampleRate": state.audio_recorder.sample_rate(),
    }))
}

// === 以下命令保留兼容旧前端，实际已不再使用 ===

#[tauri::command]
pub fn get_transcribed_text() -> Result<String, String> {
    Ok(String::new())
}

#[tauri::command]
pub fn clear_text() -> Result<(), String> {
    Ok(())
}
