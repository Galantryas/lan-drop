use axum::{
    extract::{ConnectInfo, Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

use crate::types::{ClipboardPayload, ReceivedFileRecord, TransferProgressPayload};
use crate::AppState;

pub fn get_download_dir() -> PathBuf {
    let base = dirs::download_dir().unwrap_or_else(|| PathBuf::from("."));
    let landrop_dir = base.join("LANDrop");
    let _ = std::fs::create_dir_all(&landrop_dir);
    landrop_dir
}

pub async fn bind_listener(local_ip: &str, preferred_port: u16) -> Result<(u16, tokio::net::TcpListener), String> {
    let addr_str = format!("{}:{}", local_ip, preferred_port);
    let listener = match tokio::net::TcpListener::bind(&addr_str).await {
        Ok(l) => l,
        Err(_) => {
            let fallback_str = format!("{}:0", local_ip);
            tokio::net::TcpListener::bind(&fallback_str)
                .await
                .map_err(|e| format!("Failed to bind HTTP server to {}: {}", fallback_str, e))?
        }
    };

    let bound_port = listener
        .local_addr()
        .map_err(|e| e.to_string())?
        .port();

    Ok((bound_port, listener))
}

pub fn serve_listener(app_handle: AppHandle, listener: tokio::net::TcpListener) {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/ping", get(handle_ping))
        .route("/upload", post(handle_upload))
        .route("/clipboard", post(handle_clipboard))
        .layer(cors)
        .with_state(app_handle);

    tokio::spawn(async move {
        if let Err(err) = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        {
            eprintln!("Axum HTTP server error: {}", err);
        }
    });
}

async fn handle_ping() -> impl IntoResponse {
    let device_name = hostname::get()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|_| "Unknown Device".to_string());
    let os = std::env::consts::OS.to_string();

    Json(json!({
        "device_name": device_name,
        "os": os
    }))
}

async fn handle_clipboard(
    State(app_handle): State<AppHandle>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<ClipboardPayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let client_ip = addr.ip().to_string();

    if let Some(state) = app_handle.try_state::<AppState>() {
        let blocked = state.blocked_peers.read().await;
        if blocked.contains(&client_ip) {
            return Err((
                StatusCode::FORBIDDEN,
                "Device is blocked/disconnected by recipient".to_string(),
            ));
        }
    }

    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        let _ = clipboard.set_text(&payload.content);
    }

    let _ = app_handle.emit("clipboard-received", payload);

    Ok(Json(json!({ "status": "success" })))
}

async fn handle_upload(
    State(app_handle): State<AppHandle>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let client_ip = addr.ip().to_string();

    let mut limit_kbps: Option<u32> = None;

    if let Some(state) = app_handle.try_state::<AppState>() {
        let blocked = state.blocked_peers.read().await;
        if blocked.contains(&client_ip) {
            return Err((
                StatusCode::FORBIDDEN,
                "Device is blocked/disconnected by recipient".to_string(),
            ));
        }

        let limits = state.bandwidth_limits.read().await;
        limit_kbps = limits.get(&client_ip).copied();
    }

    let download_dir = get_download_dir();
    let mut sender_name = "Peer".to_string();
    let mut total_files_saved = 0;

    while let Ok(Some(mut field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "sender_name" {
            if let Ok(text) = field.text().await {
                sender_name = text;
            }
            continue;
        }

        let original_file_name = field
            .file_name()
            .unwrap_or("received_file")
            .to_string();

        let safe_name = PathBuf::from(&original_file_name)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| format!("file_{}", Uuid::new_v4()));

        let target_path = download_dir.join(&safe_name);

        let mut file = File::create(&target_path)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create file: {}", e)))?;

        let transfer_id = Uuid::new_v4().to_string();
        let mut transferred_bytes: u64 = 0;
        let start_time = Instant::now();
        let mut last_emit = Instant::now();

        while let Ok(Some(chunk)) = field.chunk().await {
            let chunk_start = Instant::now();
            let chunk_len = chunk.len();

            file.write_all(&chunk)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed writing chunk: {}", e)))?;

            transferred_bytes += chunk_len as u64;

            if let Some(kbps) = limit_kbps {
                if kbps > 0 {
                    let bytes_per_sec = (kbps as f64) * 1024.0;
                    let target_chunk_time_secs = chunk_len as f64 / bytes_per_sec;
                    let elapsed = chunk_start.elapsed().as_secs_f64();
                    if elapsed < target_chunk_time_secs {
                        let sleep_duration = Duration::from_secs_f64(target_chunk_time_secs - elapsed);
                        tokio::time::sleep(sleep_duration).await;
                    }
                }
            }

            if last_emit.elapsed().as_millis() >= 100 {
                let elapsed_secs = start_time.elapsed().as_secs_f64();
                let speed = if elapsed_secs > 0.0 {
                    transferred_bytes as f64 / elapsed_secs
                } else {
                    0.0
                };

                let progress = TransferProgressPayload {
                    transfer_id: transfer_id.clone(),
                    file_name: safe_name.clone(),
                    total_bytes: transferred_bytes,
                    transferred_bytes,
                    speed_bytes_per_sec: speed,
                    is_complete: false,
                    is_error: false,
                    error_message: None,
                    direction: "incoming".to_string(),
                    peer_name: sender_name.clone(),
                };

                let _ = app_handle.emit("transfer-progress", progress);
                last_emit = Instant::now();
            }
        }

        file.flush()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed flushing file: {}", e)))?;

        let elapsed_secs = start_time.elapsed().as_secs_f64();
        let final_speed = if elapsed_secs > 0.0 {
            transferred_bytes as f64 / elapsed_secs
        } else {
            0.0
        };

        let final_progress = TransferProgressPayload {
            transfer_id: transfer_id.clone(),
            file_name: safe_name.clone(),
            total_bytes: transferred_bytes,
            transferred_bytes,
            speed_bytes_per_sec: final_speed,
            is_complete: true,
            is_error: false,
            error_message: None,
            direction: "incoming".to_string(),
            peer_name: sender_name.clone(),
        };
        let _ = app_handle.emit("transfer-progress", final_progress);

        let record = ReceivedFileRecord {
            id: transfer_id,
            file_name: safe_name,
            file_path: target_path.to_string_lossy().to_string(),
            file_size: transferred_bytes,
            sender_name: sender_name.clone(),
            timestamp: chrono_now_string(),
        };
        let _ = app_handle.emit("file-received", record);

        total_files_saved += 1;
    }

    Ok(Json(json!({
        "status": "success",
        "files_saved": total_files_saved
    })))
}

fn chrono_now_string() -> String {
    let now = std::time::SystemTime::now();
    let datetime: ChronoLight = now.into();
    datetime.format_time()
}

struct ChronoLight;
impl From<std::time::SystemTime> for ChronoLight {
    fn from(_: std::time::SystemTime) -> Self {
        ChronoLight
    }
}
impl ChronoLight {
    fn format_time(&self) -> String {
        format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
    }
}
