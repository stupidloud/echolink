use std::sync::Mutex;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, Emitter,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

#[derive(Default)]
struct AppState {
    is_recording: bool,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Mutex::new(AppState::default());

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().level(log::LevelFilter::Info).build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let show = MenuItemBuilder::with_id("show", "显示 Echolink").build(app).unwrap();
            let hide = MenuItemBuilder::with_id("hide", "隐藏").build(app).unwrap();
            let quit = MenuItemBuilder::with_id("quit", "退出").build(app).unwrap();
            let menu = MenuBuilder::new(app).items(&[&show, &hide, &quit]).build().unwrap();

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
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

            // Register global shortcut Right Alt (push-to-talk)
            let ralt = Shortcut::new(Some(Modifiers::ALT), Code::AltRight);
            if let Err(e) = app.global_shortcut().register(ralt) {
                eprintln!("Failed to register RAlt shortcut: {}", e);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            get_history,
            insert_history,
            delete_history,
            transcribe_audio,
            inject_text,
            get_app_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Settings {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub protocol: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            base_url: "https://api.openai.com".to_string(),
            api_key: String::new(),
            model: "gpt-4o-mini-transcribe".to_string(),
            protocol: "openai".to_string(),
        }
    }
}

#[tauri::command]
async fn get_settings(app: tauri::AppHandle) -> Result<Settings, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let key = "settings";
    let default = Settings::default();
    let s: Settings = store
        .get(key)
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or(default);
    Ok(s)
}

#[tauri::command]
async fn save_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store
        .set("settings", serde_json::to_value(settings).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    store.save().map_err(|e| e.to_string())?;
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
async fn get_history(app: tauri::AppHandle, limit: Option<i32>) -> Result<Vec<HistoryRow>, String> {
    let limit = limit.unwrap_or(50);
    let conn = app
        .db("sqlite:echolink.db")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "no db connection".to_string())?;
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
    app: tauri::AppHandle,
    text: String,
    protocol: String,
    target_app: String,
) -> Result<(), String> {
    let conn = app
        .db("sqlite:echolink.db")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "no db connection".to_string())?;
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
async fn delete_history(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let conn = app
        .db("sqlite:echolink.db")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "no db connection".to_string())?;
    conn.execute("DELETE FROM history WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn transcribe_audio(audio_b64: String, settings: Settings) -> Result<String, String> {
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

#[tauri::command]
async fn inject_text(text: String) -> Result<(), String> {
    use enigo::{Enigo, KeyboardControllable, Key};
    let mut enigo = Enigo::new();
    for ch in text.chars() {
        if let Ok(key) = Key::from_char(ch) {
            let _ = enigo.key_click(key);
        }
    }
    Ok(())
}

#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
