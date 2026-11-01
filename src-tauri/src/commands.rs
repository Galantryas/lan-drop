use futures_util::StreamExt;
use reqwest::multipart::{Form, Part};
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::netcut::get_default_gateway;
use crate::scanner::scan_local_subnet;
use crate::types::{LocalNodeInfo, NetworkDevice, PeerNode, TransferProgressPayload};
use crate::AppState;

#[tauri::command]
pub async fn get_active_peers(state: State<'_, AppState>) -> Result<Vec<PeerNode>, String> {
    let store = state.peer_store.read().await;
    Ok(store.values().cloned().collect())
}

#[tauri::command]
pub fn get_local_info(state: State<'_, AppState>) -> Result<LocalNodeInfo, String> {
    Ok(state.local_info.clone())
}

#[tauri::command]
pub async fn scan_subnet(state: State<'_, AppState>) -> Result<Vec<NetworkDevice>, String> {
    let local_ip = state.local_info.ip.clone();
    let peer_store = state.peer_store.clone();
    let blocked_peers = state.blocked_peers.read().await.clone();
    let bandwidth_limits = state.bandwidth_limits.read().await.clone();

    let devices = scan_local_subnet(&local_ip, peer_store, blocked_peers, bandwidth_limits).await;
    Ok(devices)
}

#[tauri::command]
pub async fn cut_wifi_device(
    state: State<'_, AppState>,
    target_ip: String,
) -> Result<(), String> {
    let local_ip = state.local_info.ip.clone();
    let gateway_ip = get_default_gateway(&local_ip);

    state
        .netcut_manager
        .cut_wifi(&target_ip, &gateway_ip)
        .await
}

#[tauri::command]
pub async fn restore_wifi_device(
    state: State<'_, AppState>,
    target_ip: String,
) -> Result<(), String> {
    state.netcut_manager.restore_wifi(&target_ip).await
}

#[tauri::command]
pub async fn disconnect_peer(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    target_ip: String,
) -> Result<(), String> {
    let mut blocked = state.blocked_peers.write().await;
    blocked.insert(target_ip.clone());

    let mut store = state.peer_store.write().await;
    for peer in store.values_mut() {
        if peer.ip == target_ip {
            peer.is_blocked = true;
        }
    }

    let peer_list: Vec<PeerNode> = store.values().cloned().collect();
    let _ = app_handle.emit("peers-updated", peer_list);

    Ok(())
}

#[tauri::command]
pub async fn reconnect_peer(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    target_ip: String,
) -> Result<(), String> {
    let mut blocked = state.blocked_peers.write().await;
    blocked.remove(&target_ip);

    let mut store = state.peer_store.write().await;
    for peer in store.values_mut() {
        if peer.ip == target_ip {
            peer.is_blocked = false;
        }
    }

    let peer_list: Vec<PeerNode> = store.values().cloned().collect();
    let _ = app_handle.emit("peers-updated", peer_list);

    Ok(())
}

#[tauri::command]
pub async fn set_peer_bandwidth_limit(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    target_ip: String,
    limit_kbps: Option<u32>,
) -> Result<(), String> {
    let mut limits = state.bandwidth_limits.write().await;
    if let Some(kbps) = limit_kbps {
        limits.insert(target_ip.clone(), kbps);
    } else {
        limits.remove(&target_ip);
    }

    let mut store = state.peer_store.write().await;
    for peer in store.values_mut() {
        if peer.ip == target_ip {
            peer.bandwidth_limit_kbps = limit_kbps;
        }
    }

    let peer_list: Vec<PeerNode> = store.values().cloned().collect();
    let _ = app_handle.emit("peers-updated", peer_list);

    Ok(())
}

#[tauri::command]
pub async fn send_clipboard(
    state: State<'_, AppState>,
    target_ip: String,
    target_port: u16,
    text: String,
) -> Result<(), String> {
    let blocked = state.blocked_peers.read().await;
    if blocked.contains(&target_ip) {
        return Err(format!("Cannot send clipboard: Peer {} is blocked", target_ip));
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("http://{}:{}/clipboard", target_ip, target_port);

    let payload = serde_json::json!({
        "content": text,
        "sender_name": state.local_info.name
    });

    let res = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            format!(
                "Device {} is connected to Wi-Fi but is not running LAN Drop app: {}",
                target_ip, e
            )
        })?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(format!("Peer returned error status: {}", res.status()))
    }
}

#[tauri::command]
pub async fn send_files(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    target_ip: String,
    target_port: u16,
    file_paths: Vec<String>,
) -> Result<(), String> {
    let blocked = state.blocked_peers.read().await;
    if blocked.contains(&target_ip) {
        return Err(format!("Cannot send files: Peer {} is blocked", target_ip));
    }

    let limits = state.bandwidth_limits.read().await;
    let limit_kbps = limits.get(&target_ip).copied();
    drop(limits);
    drop(blocked);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3600))
        .connect_timeout(Duration::from_secs(3))
        .build()
        .map_err(|e| e.to_string())?;

    // 1. Pre-flight health check to verify LAN Drop is running on target
    let ping_url = format!("http://{}:{}/ping", target_ip, target_port);
    let ping_check = client
        .get(&ping_url)
        .timeout(Duration::from_millis(1000))
        .send()
        .await;

    if ping_check.is_err() || !ping_check.unwrap().status().is_success() {
        return Err(format!(
            "Device {} is connected to Wi-Fi but does NOT have LAN Drop running on port {}. Please open LAN Drop on {} to receive files.",
            target_ip, target_port, target_ip
        ));
    }

    let url = format!("http://{}:{}/upload", target_ip, target_port);
    let sender_name = state.local_info.name.clone();

    for path_str in file_paths {
        let path = std::path::Path::new(&path_str);
        if !path.exists() {
            return Err(format!("File does not exist: {}", path_str));
        }

        let file_name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "file".to_string());

        let file = File::open(path)
            .await
            .map_err(|e| format!("Failed opening file {}: {}", path_str, e))?;

        let metadata = file
            .metadata()
            .await
            .map_err(|e| format!("Failed getting metadata for {}: {}", path_str, e))?;

        let file_size = metadata.len();
        let transfer_id = uuid::Uuid::new_v4().to_string();

        let app_handle_clone = app_handle.clone();
        let file_name_clone = file_name.clone();
        let transfer_id_clone = transfer_id.clone();
        let target_ip_clone = target_ip.clone();

        let reader_stream = ReaderStream::new(file);

        let mut transferred: u64 = 0;
        let start_time = std::time::Instant::now();
        let mut last_emit = std::time::Instant::now();

        let progress_stream = reader_stream.map(move |item| {
            if let Ok(ref bytes) = item {
                let chunk_len = bytes.len();
                transferred += chunk_len as u64;

                if let Some(kbps) = limit_kbps {
                    if kbps > 0 {
                        let bytes_per_sec = (kbps as f64) * 1024.0;
                        let target_chunk_secs = chunk_len as f64 / bytes_per_sec;
                        std::thread::sleep(Duration::from_secs_f64(target_chunk_secs));
                    }
                }

                if last_emit.elapsed().as_millis() >= 100 || transferred == file_size {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let speed = if elapsed > 0.0 {
                        transferred as f64 / elapsed
                    } else {
                        0.0
                    };

                    let progress = TransferProgressPayload {
                        transfer_id: transfer_id_clone.clone(),
                        file_name: file_name_clone.clone(),
                        total_bytes: file_size,
                        transferred_bytes: transferred,
                        speed_bytes_per_sec: speed,
                        is_complete: transferred == file_size,
                        is_error: false,
                        error_message: None,
                        direction: "outgoing".to_string(),
                        peer_name: target_ip_clone.clone(),
                    };
                    let _ = app_handle_clone.emit("transfer-progress", progress);
                    last_emit = std::time::Instant::now();
                }
            }
            item
        });

        let body = reqwest::Body::wrap_stream(progress_stream);
        let file_part = Part::stream_with_length(body, file_size).file_name(file_name.clone());

        let form = Form::new()
            .text("sender_name", sender_name.clone())
            .part("file", file_part);

        let res = client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Failed streaming file to {}: {}", target_ip, e))?;

        if !res.status().is_success() {
            let err_msg = format!("HTTP error from peer: {}", res.status());
            let error_progress = TransferProgressPayload {
                transfer_id: transfer_id.clone(),
                file_name: file_name.clone(),
                total_bytes: file_size,
                transferred_bytes: 0,
                speed_bytes_per_sec: 0.0,
                is_complete: false,
                is_error: true,
                error_message: Some(err_msg.clone()),
                direction: "outgoing".to_string(),
                peer_name: target_ip.clone(),
            };
            let _ = app_handle.emit("transfer-progress", error_progress);
            return Err(err_msg);
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn select_files() -> Result<Vec<String>, String> {
    let files = rfd::AsyncFileDialog::new()
        .set_title("Select Files for LAN Drop")
        .pick_files()
        .await;

    if let Some(handles) = files {
        let paths = handles
            .into_iter()
            .map(|h| h.path().to_string_lossy().to_string())
            .collect();
        Ok(paths)
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
pub fn open_downloads_folder() -> Result<(), String> {
    let dir = crate::server::get_download_dir();
    open::that(dir).map_err(|e| format!("Failed to open downloads directory: {}", e))
}

#[tauri::command]
pub fn copy_to_clipboard(text: String) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new()
        .map_err(|e| format!("Failed accessing clipboard: {}", e))?;
    clipboard
        .set_text(text)
        .map_err(|e| format!("Failed setting clipboard text: {}", e))
}
