use berry_core::control::{socket_path, Request, Response};
use serde::{Deserialize, Serialize};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceDto {
    pub id: String,
    pub name: String,
    pub url: String,
    pub protocol: String,
}

async fn ensure_daemon() {
    let sock = socket_path();
    if !sock.exists() {
        let _ = std::process::Command::new("berryd").spawn();
        for _ in 0..20 {
            if sock.exists() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
}

async fn send(req: Request) -> Result<Response, String> {
    ensure_daemon().await;
    let sock = socket_path();
    let stream = tokio::net::UnixStream::connect(&sock)
        .await
        .map_err(|e| e.to_string())?;
    let (r, mut w) = stream.into_split();
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    let mut reader = BufReader::new(r);
    let mut line = String::new();
    let mut out = serde_json::to_string(&req).map_err(|e| e.to_string())?;
    out.push('\n');
    w.write_all(out.as_bytes()).await.map_err(|e| e.to_string())?;
    reader.read_line(&mut line).await.map_err(|e| e.to_string())?;
    serde_json::from_str(line.trim()).map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_sources() -> Result<serde_json::Value, String> {
    let resp = send(Request::ListSources).await?;
    if resp.ok {
        Ok(resp.data.unwrap_or(serde_json::json!([])))
    } else {
        Err(resp.message)
    }
}

#[tauri::command]
async fn add_source(name: String, url: String) -> Result<serde_json::Value, String> {
    let resp = send(Request::AddSource { name, url }).await?;
    if resp.ok {
        Ok(resp.data.unwrap_or(serde_json::json!({})))
    } else {
        Err(resp.message)
    }
}

#[tauri::command]
async fn remove_source(id: String) -> Result<String, String> {
    let resp = send(Request::RemoveSource { id }).await?;
    if resp.ok {
        Ok(resp.message)
    } else {
        Err(resp.message)
    }
}

#[tauri::command]
async fn start_source(id: String) -> Result<String, String> {
    let resp = send(Request::Start { id }).await?;
    if resp.ok {
        Ok(resp.message)
    } else {
        Err(resp.message)
    }
}

#[tauri::command]
async fn stop_source() -> Result<String, String> {
    let resp = send(Request::Stop).await?;
    if resp.ok {
        Ok(resp.message)
    } else {
        Err(resp.message)
    }
}

#[tauri::command]
async fn status() -> Result<serde_json::Value, String> {
    let resp = send(Request::Status).await?;
    if resp.ok {
        Ok(resp.data.unwrap_or(serde_json::json!({})))
    } else {
        Err(resp.message)
    }
}

#[tauri::command]
async fn snapshot() -> Result<serde_json::Value, String> {
    let resp = send(Request::Snapshot).await?;
    if resp.ok {
        Ok(resp.data.unwrap_or(serde_json::json!({})))
    } else {
        Err(resp.message)
    }
}

#[tauri::command]
async fn doctor() -> Result<serde_json::Value, String> {
    let resp = send(Request::Doctor).await?;
    if resp.ok {
        Ok(resp.data.unwrap_or(serde_json::json!({})))
    } else {
        Err(resp.message)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_sources,
            add_source,
            remove_source,
            start_source,
            stop_source,
            status,
            snapshot,
            doctor
        ])
        .setup(|app| {
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
