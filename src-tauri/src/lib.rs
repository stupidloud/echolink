use std::sync::Mutex;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_variables)]
    let state = Mutex::new(AppState::default());

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().level(log::LevelFilter::Info).build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

            let ralt = Shortcut::new(Some(Modifiers::ALT), Code::AltRight);

            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |_app, shortcut, event| {
                        if shortcut == &ralt {
                            match event.state() {
                                ShortcutState::Pressed => {
                                    let _ = _app.emit("recording-state", true);
                                }
                                ShortcutState::Released => {
                                    let _ = _app.emit("recording-state", false);
                                }
                            }
                        }
                    })
                    .build(),
            )?;

            if let Err(e) = app.global_shortcut().register(ralt) {
                eprintln!("Failed to register RAlt shortcut: {}", e);
            }

            let show = MenuItemBuilder::with_id("show", "显示 Echolink").build(app).unwrap();
            let hide = MenuItemBuilder::with_id("hide", "隐藏").build(app).unwrap();
            let quit = MenuItemBuilder::with_id("quit", "退出").build(app).unwrap();
            let menu = MenuBuilder::new(app).items(&[&show, &hide, &quit]).build().unwrap();

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(w) = app.get_webview("main") {
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
                        if let Some(w) = app.get_webview("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)
                .expect("failed to build tray icon");

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
    Ok(s)
}

#[tauri::command]
async fn save_settings(app: tauri::AppHandle, settings: AppSettings) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let value = serde_json::to_value(settings).map_err(|e| e.to_string())?;
    store.set("settings", value);
    store.save();
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
    let conn = db_conn()?;
    conn.execute("DELETE FROM history WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn transcribe_audio(audio_b64: String, settings: AppSettings) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/v1/audio/transcriptions", settings.base_url.trim_end_matches('/'));
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
        return Err(format!("ASR error: {}", resp.status()));
    }
    let text: String = resp.text().await.map_err(|e| e.to_string())?;
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
    let url = format!("{}/v1/audio/asr/sse", settings.base_url.trim_end_matches('/'));

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
            if !line.starts_with("data:") {
                remainder.push_str(raw_line);
                remainder.push('\n');
                continue;
            }
            let data = line.strip_prefix("data:").map(|s| s.trim()).unwrap_or("");
            if data.is_empty() || data == "[DONE]" {
                continue;
            }
            if let Ok(evt) = serde_json::from_str::<StepFunSseEvent>(data) {
                match evt.event_type.as_str() {
                    "transcript.text.delta" => {
                        if let Some(ref d) = evt.delta {
                            full_text.push_str(d);
                            let _ = app.emit("transcript-delta", d.clone());
                        }
                    }
                    "transcript.text.done" => {
                        if let Some(ref t) = evt.text {
                            full_text = t.clone();
                        }
                        let _ = app.emit("transcript-done", full_text.clone());
                    }
                    "error" => {
                        let err_msg = evt.message.clone().unwrap_or_default();
                        let _ = app.emit("transcript-error", err_msg.clone());
                        return Err(format!("ASR SSE error: {}", err_msg));
                    }
                    _ => {}
                }
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

    Ok(full_text)
}

#[tauri::command]
async fn inject_text(app: tauri::AppHandle, text: String) -> Result<(), String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    app.clipboard().write_text(&text).map_err(|e| e.to_string())?;

    #[cfg(target_os = "macos")]
    {
        let _ = app.shell().command("osascript")
            .args(["-e", "tell application \"System Events\" to keystroke \"v\" using command down"])
            .spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = app.shell().command("powershell")
            .args(["-Command", "(New-Object -ComObject WScript.Shell).SendKeys('^v')"])
            .spawn();
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
    match settings.protocol.as_str() {
        "stepfun" => {
            let url = format!("{}/v1/audio/asr/sse", settings.base_url.trim_end_matches('/'));
            let body = serde_json::json!({
                "audio": {
                    "data": "",
                    "input": {
                        "transcription": {
                            "model": settings.model,
                            "language": "zh",
                            "enable_itn": true,
                            "enable_timestamp": false
                        },
                        "format": {
                            "type": "pcm",
                            "codec": "pcm_s16le",
                            "rate": 16000,
                            "bits": 16,
                            "channel": 1
                        }
                    }
                }
            });
            let resp = client
                .post(&url)
                .header("Authorization", format!("Bearer {}", settings.api_key))
                .header("Content-Type", "application/json")
                .header("Accept", "text/event-stream")
                .json(&body)
                .send()
                .await
                .map_err(|e| e.to_string())?;

            match resp.status().as_u16() {
                401 | 403 => Err("❌ API Key 无效或已过期".to_string()),
                200..=299 => Ok("✅ 连接成功，端点与 Key 有效".to_string()),
                _ => Err(format!("❌ 服务端返回错误: {}", resp.status())),
            }
        }
        _ => {
            let url = format!("{}/v1/models", settings.base_url.trim_end_matches('/'));
            let resp = client
                .get(&url)
                .header("Authorization", format!("Bearer {}", settings.api_key))
                .send()
                .await
                .map_err(|e| e.to_string())?;

            match resp.status().as_u16() {
                401 | 403 => Err("❌ API Key 无效或已过期".to_string()),
                200..=299 => Ok("✅ 连接成功，端点与 Key 有效".to_string()),
                _ => Err(format!("❌ 服务端返回错误: {}", resp.status())),
            }
        }
    }
}
    env!("CARGO_PKG_VERSION").to_string()
}
