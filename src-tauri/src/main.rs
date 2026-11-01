// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod discovery;
mod netcut;
mod scanner;
mod server;
mod types;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

use netcut::NetCutManager;
use types::{LocalNodeInfo, PeerStore};

pub struct AppState {
    pub peer_store: PeerStore,
    pub local_info: LocalNodeInfo,
    pub mdns_daemon: Arc<tokio::sync::Mutex<Option<mdns_sd::ServiceDaemon>>>,
    pub blocked_peers: Arc<RwLock<HashSet<String>>>,
    pub bandwidth_limits: Arc<RwLock<HashMap<String, u32>>>, // IP -> KB/s
    pub netcut_manager: Arc<NetCutManager>,
}

#[tokio::main]
async fn main() {
    let peer_store: PeerStore = Arc::new(RwLock::new(HashMap::new()));
    let blocked_peers = Arc::new(RwLock::new(HashSet::new()));
    let bandwidth_limits = Arc::new(RwLock::new(HashMap::new()));
    let netcut_manager = Arc::new(NetCutManager::new());

    let local_ip = local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string());

    let device_name = hostname::get()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|_| "LAN-Drop-Node".to_string());

    let (bound_port, listener_opt) = match server::bind_listener(&local_ip, 49872).await {
        Ok((port, listener)) => (port, Some(listener)),
        Err(e) => {
            eprintln!("Error binding HTTP listener: {}", e);
            (49872, None)
        }
    };

    let os_type = std::env::consts::OS.to_string();
    let download_dir = server::get_download_dir().to_string_lossy().to_string();

    let local_info = LocalNodeInfo {
        name: device_name.clone(),
        ip: local_ip.clone(),
        port: bound_port,
        os_type,
        download_dir,
    };

    let mdns_daemon = Arc::new(tokio::sync::Mutex::new(None));

    let state = AppState {
        peer_store: peer_store.clone(),
        local_info,
        mdns_daemon: mdns_daemon.clone(),
        blocked_peers: blocked_peers.clone(),
        bandwidth_limits: bandwidth_limits.clone(),
        netcut_manager,
    };

    let peer_store_clone = peer_store.clone();

    tauri::Builder::default()
        .manage(state)
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let local_ip_inner = local_ip.clone();
            let device_name_inner = device_name.clone();

            if let Some(listener) = listener_opt {
                server::serve_listener(app_handle.clone(), listener);
            }

            tauri::async_runtime::spawn(async move {
                if let Ok(daemon) = discovery::setup_mdns_discovery(
                    app_handle.clone(),
                    peer_store_clone,
                    device_name_inner,
                    local_ip_inner,
                    bound_port,
                ) {
                    let mut lock = mdns_daemon.lock().await;
                    *lock = Some(daemon);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_active_peers,
            commands::get_local_info,
            commands::send_clipboard,
            commands::send_files,
            commands::select_files,
            commands::open_downloads_folder,
            commands::copy_to_clipboard,
            commands::scan_subnet,
            commands::disconnect_peer,
            commands::reconnect_peer,
            commands::set_peer_bandwidth_limit,
            commands::cut_wifi_device,
            commands::restore_wifi_device,
        ])
        .run(tauri::generate_context!())
        .expect("error while running LAN Drop application");
}
