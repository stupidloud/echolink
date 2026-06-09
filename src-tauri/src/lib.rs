use std::sync::Mutex;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_store::StoreExt;

#[derive(Default)]
struct AppState {
    is_recording: bool,
}

fn db_conn() -> Result<rusqlite::Connection, String> {
    let path = dirs::data_dir()
        .ok_or_else(|| "cannot get data dir".to_string())?
        .join("echolink")
        .join("echolink.db");
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    rusqlite::Connection::open(path).map_err(|e| e.to_string())
}

fn api_url(base: &str, path: &str) -> String {
    let base = base.trim_end_matches('/');
    let path = path.trim_start_matches('/');

    let base_last = base.split('/').last().unwrap_or("");
    let path_first = path.split('/').next().unwrap_or("");

    if !base_last.is_empty() && base_last == path_first {
        let rest = path.strip_prefix(base_last).unwrap_or("");
        format!("{}{}", base, rest)
    } else {
        format!("{}/{}", base, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_url_keeps_no_v1_base() {
        let url = api_url("https://api.openai.com", "/v1/models");
        assert_eq!(url, "https://api.openai.com/v1/models");
    }

    #[test]
    fn api_url_strips_any_duplicate_segment() {
        let url = api_url("https://api.stepfun.com/v1", "/v1/audio/asr/sse");
        assert_eq!(url, "https://api.stepfun.com/v1/audio/asr/sse");
    }

    #[test]
    fn api_url_strips_v1beta_duplicate() {
        let url = api_url("https://api.gemini.com/v1beta", "/v1beta/models");
        assert_eq!(url, "https://api.gemini.com/v1beta/models");
    }

    #[test]
    fn api_url_handles_trailing_slash() {
        let url = api_url("https://api.openai.com/v1/", "/v1/models");
        assert_eq!(url, "https://api.openai.com/v1/models");
    }

    #[test]
    fn api_url_adds_slash_when_no_common_segment() {
        let url = api_url("https://example.com", "v1/models");
        assert_eq!(url, "https://example.com/v1/models");
    }

    #[test]
    fn api_url_empty_path() {
        let url = api_url("https://api.openai.com", "");
        assert_eq!(url, "https://api.openai.com/");
    }

    #[test]
    fn api_url_root_path() {
        let url = api_url("https://api.openai.com/v1", "/");
        assert_eq!(url, "https://api.openai.com/v1/");
    }

    #[test]
    fn api_url_no_v1_in_base_but_path_has_v1() {
        let url = api_url("https://custom.com", "/v1/chat");
        assert_eq!(url, "https://custom.com/v1/chat");
    }

    #[test]
    fn app_settings_defaults() {
        let s = AppSettings::default();
        assert_eq!(s.base_url, "https://api.stepfun.com");
        assert_eq!(s.model, "stepaudio-2.5-asr");
        assert_eq!(s.protocol, "stepfun");
        assert!(s.api_key.is_empty());
    }

    #[test]
    fn app_settings_deserialize_camel_case() {
        let json = r#"{"baseUrl":"https://oai.com","apiKey":"sk-xxx","model":"whisper-1","protocol":"openai"}"#;
        let s: AppSettings = serde_json::from_str(json).unwrap();
        assert_eq!(s.base_url, "https://oai.com");
        assert_eq!(s.api_key, "sk-xxx");
        assert_eq!(s.model, "whisper-1");
        assert_eq!(s.protocol, "openai");
    }

    #[test]
    fn app_settings_serialize_camel_case() {
        let s = AppSettings {
            base_url: "https://test.com".into(),
            api_key: "key".into(),
            model: "m".into(),
            protocol: "p".into(),
        };
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("\"baseUrl\""));
        assert!(json.contains("\"apiKey\""));
        assert!(json.contains("\"model\""));
        assert!(json.contains("\"protocol\""));
        assert!(!json.contains("\"base_url\""));
        assert!(!json.contains("\"api_key\""));
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_variables)]
    let state = Mutex::new(AppState::default());

    tauri::Builder::default()
        .device_event_filter(tauri::DeviceEventFilter::Never)
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Stdout,
                ))
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Webview,
                ))
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .setup(|app| {
            log::info!("Rust backend started");

            #[cfg(target_os = "windows")]
            {
                let _ = prevent_alt_win_menu::start(Default::default())
                    .map_err(|e| log::error!("prevent-alt-win-menu: {:?}", e));
            }

            let handle = app.handle().clone();
            std::thread::spawn(move || {
                log::info!("rdev thread started");
                let cb_handle = handle.clone();
                let mut alt_down = false;
                if let Err(e) = rdev::listen(move |event| {
                    match event.event_type {
                        rdev::EventType::KeyPress(rdev::Key::AltGr) => {
                            if !alt_down {
                                alt_down = true;
                                log::info!("AltGr pressed");
                                if let Some(ov) = cb_handle.get_webview_window("overlay") {
                                    let _ = ov.show();
                                }
                                let _ = cb_handle.emit("recording-state", true);
                            }
                        }
                        rdev::EventType::KeyRelease(rdev::Key::AltGr) => {
                            if alt_down {
                                alt_down = false;
                                log::info!("AltGr released");
                                if let Some(ov) = cb_handle.get_webview_window("overlay") {
                                    let _ = ov.hide();
                                }
                                let _ = cb_handle.emit("recording-state", false);
                            }
                        }
                        _ => {}
                    }
                }) {
                    log::error!("rdev listen error: {:?}", e);
                }
            });

            let show = MenuItemBuilder::with_id("show", "显示 Echolink").build(app).unwrap();
            let hide = MenuItemBuilder::with_id("hide", "隐藏").build(app).unwrap();
            let quit = MenuItemBuilder::with_id("quit", "退出").build(app).unwrap();
            let menu = MenuBuilder::new(app).items(&[&show, &hide, &quit]).build().unwrap();

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.hide();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)
                .expect("failed to build tray icon");

            // Overlay recording indicator window
            let _overlay = WebviewWindowBuilder::new(
                app,
                "overlay",
                WebviewUrl::App("index.html#/overlay".into()),
            )
            .title("")
            .decorations(false)
            .always_on_top(true)
            .inner_size(120.0, 40.0)
            .position(0.0, 0.0)
            .build()
            .expect("failed to build overlay window");
            let _ = _overlay.set_background_color(Some(tauri::window::Color { r: 0, g: 0, b: 0, a: 0 }));
            let _ = _overlay.hide();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            get_history,
            insert_history,
            delete_history,
            transcribe_audio,
            transcribe_audio_sse,
            verify_connection,
            inject_text,
            get_app_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub protocol: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            base_url: "https://api.stepfun.com".to_string(),
            api_key: String::new(),
            model: "stepaudio-2.5-asr".to_string(),
            protocol: "stepfun".to_string(),
        }
    }
}

#[tauri::command]
async fn get_settings(app: tauri::AppHandle) -> Result<AppSettings, String> {
    use tauri_plugin_store::StoreExt;
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let key = "settings";
    let default = AppSettings::default();
    let s: AppSettings = store
        .get(key)
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or(default);
    log::info!("get_settings → protocol={}, model={}", s.protocol, s.model);
    Ok(s)
}

#[tauri::command]
async fn save_settings(app: tauri::AppHandle, settings: AppSettings) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let value = serde_json::to_value(settings).map_err(|e| e.to_string())?;
    store.set("settings", value);
    store.save();
    log::info!("save_settings → saved");
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct HistoryRow {
    pub id: String,
    pub timestamp: String,
    pub text: String,
    pub protocol: String,
    pub target_app: String,
}

#[tauri::command]
async fn get_history(
    _app: tauri::AppHandle,
    limit: Option<i32>,
) -> Result<Vec<HistoryRow>, String> {
    let limit = limit.unwrap_or(50);
    let conn = db_conn()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, timestamp, text, protocol, target_app FROM history ORDER BY timestamp DESC LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows: Vec<HistoryRow> = stmt
        .query_map([limit], |row| {
            Ok(HistoryRow {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                text: row.get(2)?,
                protocol: row.get(3)?,
                target_app: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
async fn insert_history(
    _app: tauri::AppHandle,
    text: String,
    protocol: String,
    target_app: String,
) -> Result<(), String> {
    let conn = db_conn()?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS history (id TEXT PRIMARY KEY, timestamp TEXT, text TEXT, protocol TEXT, target_app TEXT)",
        [],
    )
    .map_err(|e| e.to_string())?;
    log::info!("insert_history → protocol={}, text_len={}", protocol, text.len());
    conn.execute(
        "INSERT INTO history (id, timestamp, text, protocol, target_app) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            uuid::Uuid::new_v4().to_string(),
            chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
            text,
            protocol,
            target_app
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn delete_history(_app: tauri::AppHandle, id: String) -> Result<(), String> {
    log::info!("delete_history → id={}", id);
    let conn = db_conn()?;
    conn.execute("DELETE FROM history WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn transcribe_audio(audio_b64: String, settings: AppSettings) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = api_url(&settings.base_url, "/v1/audio/transcriptions");
    let audio_size = audio_b64.len();
    log::info!("transcribe_audio → url={}, audio_b64_len={}", url, audio_size);
    let audio_bytes = base64::decode(&audio_b64).map_err(|e| e.to_string())?;
    let part = reqwest::multipart::Part::bytes(audio_bytes)
        .file_name("audio.webm")
        .mime_str("audio/webm")
        .map_err(|e| e.to_string())?;
    let form = reqwest::multipart::Form::new().part("file", part).text("model", settings.model).text("response_format", "text");

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", settings.api_key))
        .multipart(form)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        log::error!("transcribe_audio HTTP {}", resp.status());
        return Err(format!("ASR error: {}", resp.status()));
    }
    let text: String = resp.text().await.map_err(|e| e.to_string())?;
    log::info!("transcribe_audio → OK, len={}", text.len());
    Ok(text)
}

#[derive(serde::Serialize, serde::Deserialize)]
struct StepFunAsrRequest {
    audio: StepFunAudio,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct StepFunAudio {
    data: String,
    input: StepFunInput,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct StepFunInput {
    transcription: StepFunTranscription,
    format: StepFunFormat,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct StepFunTranscription {
    model: String,
    language: String,
    enable_itn: bool,
    enable_timestamp: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct StepFunFormat {
    #[serde(rename = "type")]
    kind: String,
    codec: String,
    rate: u32,
    bits: u32,
    channel: u32,
}

#[derive(serde::Deserialize)]
struct StepFunSseEvent {
    #[serde(rename = "type")]
    event_type: String,
    delta: Option<String>,
    text: Option<String>,
    message: Option<String>,
}

#[tauri::command]
async fn transcribe_audio_sse(audio_b64: String, settings: AppSettings, app: tauri::AppHandle) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = api_url(&settings.base_url, "/v1/audio/asr/sse");
    let audio_size = audio_b64.len();
    log::info!("transcribe_audio_sse → url={}, audio_b64_len={}", url, audio_size);

    let body = StepFunAsrRequest {
        audio: StepFunAudio {
            data: audio_b64,
            input: StepFunInput {
                transcription: StepFunTranscription {
                    model: settings.model.clone(),
                    language: "zh".to_string(),
                    enable_itn: true,
                    enable_timestamp: false,
                },
                format: StepFunFormat {
                    kind: "pcm".to_string(),
                    codec: "pcm_s16le".to_string(),
                    rate: 16000,
                    bits: 16,
                    channel: 1,
                },
            },
        },
    };

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", settings.api_key))
        .header("Content-Type", "application/json")
        .header("Accept", "text/event-stream")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("ASR SSE error: {}", resp.status()));
    }

    let mut full_text = String::new();
    let mut stream = resp.bytes_stream();
    let mut buf = String::new();

    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        buf.push_str(String::from_utf8_lossy(&chunk).as_ref());

        let mut remainder = String::new();
        for raw_line in buf.split('\n') {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(data) = line.strip_prefix("data:") {
                let data = data.trim();
                if data.is_empty() || data == "[DONE]" {
                    continue;
                }
                match serde_json::from_str::<StepFunSseEvent>(data) {
                    Ok(evt) => match evt.event_type.as_str() {
                        "transcript.text.delta" => {
                            if let Some(ref d) = evt.delta {
                                full_text.push_str(d);
                                log::info!("transcript.text.delta → +{} chars", d.len());
                                let _ = app.emit("transcript-delta", d.clone());
                            }
                        }
                        "transcript.text.done" => {
                            if let Some(ref t) = evt.text {
                                full_text = t.clone();
                            }
                            log::info!("transcript.text.done → total={} chars", full_text.len());
                        }
                        "error" => {
                            let err_msg = evt.message.clone().unwrap_or_default();
                            log::error!("transcript.error → {}", err_msg);
                            let _ = app.emit("transcript-error", err_msg.clone());
                            return Err(format!("ASR SSE error: {}", err_msg));
                        }
                        _ => {}
                    },
                    Err(_) => {
                        remainder.push_str(raw_line);
                        remainder.push('\n');
                    }
                }
            } else {
                remainder.push_str(raw_line);
                remainder.push('\n');
            }
        }
        buf = remainder;
    }

    for line in buf.lines() {
        let line = line.trim();
        if line.starts_with("data:") {
            let data = line.strip_prefix("data:").unwrap_or(line).trim();
            if !data.is_empty() && data != "[DONE]" {
                if let Ok(evt) = serde_json::from_str::<StepFunSseEvent>(data) {
                    match evt.event_type.as_str() {
                        "transcript.text.delta" => {
                            if let Some(d) = evt.delta { full_text.push_str(&d); }
                        }
                        "transcript.text.done" => {
                            if let Some(t) = evt.text { full_text = t; }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    log::info!("transcribe_audio_sse → final_len={}", full_text.len());
    if !full_text.is_empty() {
        let _ = app.emit("transcript-done", full_text.clone());
    }
    Ok(full_text)
}

#[tauri::command]
async fn inject_text(app: tauri::AppHandle, text: String) -> Result<(), String> {
    log::info!("inject_text → len={} chars", text.len());

    #[cfg(target_os = "windows")]
    {
        use std::mem::size_of;
        use windows::Win32::UI::Input::KeyboardAndMouse::*;

        for ch in text.encode_utf16() {
            let ki = KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: ch,
                dwFlags: KEYEVENTF_UNICODE,
                time: 0,
                dwExtraInfo: 0,
            };
            unsafe {
                SendInput(
                    &[INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 { ki },
                    }],
                    size_of::<INPUT>() as i32,
                );
            }

            let ki = KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: ch,
                dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                time: 0,
                dwExtraInfo: 0,
            };
            unsafe {
                SendInput(
                    &[INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 { ki },
                    }],
                    size_of::<INPUT>() as i32,
                );
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        use std::ffi::c_void;

        #[link(name = "CoreGraphics", kind = "framework")]
        extern "C" {
            fn CGEventCreateKeyboardEvent(
                source: *const c_void,
                keycode: u16,
                keydown: bool,
            ) -> *mut c_void;
            fn CGEventKeyboardSetUnicodeString(
                event: *mut c_void,
                length: usize,
                string: *const u16,
            );
            fn CGEventPost(tap: u32, event: *mut c_void);
            fn CFRelease(event: *mut c_void);
        }

        const K_CG_HID_EVENT_TAP: u32 = 0;

        let utf16: Vec<u16> = text.encode_utf16().collect();
        unsafe {
            for chunk in utf16.chunks(20) {
                let event = CGEventCreateKeyboardEvent(std::ptr::null(), 0, true);
                CGEventKeyboardSetUnicodeString(event, chunk.len(), chunk.as_ptr());
                CGEventPost(K_CG_HID_EVENT_TAP, event);
                CFRelease(event);

                std::thread::sleep(std::time::Duration::from_millis(4));

                let event = CGEventCreateKeyboardEvent(std::ptr::null(), 0, false);
                CGEventKeyboardSetUnicodeString(event, chunk.len(), chunk.as_ptr());
                CGEventPost(K_CG_HID_EVENT_TAP, event);
                CFRelease(event);
            }
        }
    }

    Ok(())
}

#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
async fn verify_connection(settings: AppSettings) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = api_url(&settings.base_url, "/v1/models");
    log::info!("verify_connection → url={}, protocol={}", url, settings.protocol);

    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", settings.api_key))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    match resp.status().as_u16() {
        401 | 403 => {
            log::error!("verify_connection → 401/403");
            Err("❌ API Key 无效或已过期".to_string())
        }
        200..=299 => {
            let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let models = body["data"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| m["id"].as_str())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            if models.contains(&settings.model.as_str()) {
                Ok(format!(
                    "✅ 连接成功，模型 {} 可用",
                    settings.model
                ))
            } else {
                let available = models.join(", ");
                Err(format!(
                    "⚠️ 端点与 Key 有效，但模型 '{}' 不在可用列表中：{}",
                    settings.model, available
                ))
            }
        }
        _ => Err(format!("❌ 服务端返回错误: {}", resp.status())),
    }
}

