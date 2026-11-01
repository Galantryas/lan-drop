use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

use crate::types::{PeerNode, PeerStore};

const SERVICE_TYPE: &str = "_landrop._tcp.local.";

pub fn setup_mdns_discovery(
    app_handle: AppHandle,
    peer_store: PeerStore,
    local_name: String,
    local_ip: String,
    local_port: u16,
) -> Result<ServiceDaemon, String> {
    let mdns = ServiceDaemon::new().map_err(|e| format!("Failed to create mDNS daemon: {}", e))?;

    let host_name = format!("{}.local.", sanitize_hostname(&local_name));

    let mut txt_properties = HashMap::new();
    txt_properties.insert("port".to_string(), local_port.to_string());
    txt_properties.insert("platform".to_string(), std::env::consts::OS.to_string());
    txt_properties.insert("version".to_string(), "1.0".to_string());

    let service_info = ServiceInfo::new(
        SERVICE_TYPE,
        &local_name,
        &host_name,
        local_ip.as_str(),
        local_port,
        txt_properties,
    )
    .map_err(|e| format!("Failed to create ServiceInfo: {}", e))?;

    mdns.register(service_info)
        .map_err(|e| format!("Failed to register mDNS service: {}", e))?;

    let receiver = mdns
        .browse(SERVICE_TYPE)
        .map_err(|e| format!("Failed to browse mDNS: {}", e))?;

    let app_clone = app_handle.clone();
    let store_clone = peer_store.clone();
    let self_ip = local_ip.clone();
    let self_port = local_port;

    // Run continuous listener in background thread
    std::thread::spawn(move || {
        while let Ok(event) = receiver.recv() {
            match event {
                ServiceEvent::ServiceResolved(info) => {
                    let fullname = info.get_fullname().to_string();
                    let port = info.get_port();

                    let ips: Vec<String> = info
                        .get_addresses_v4()
                        .iter()
                        .map(|ip| ip.to_string())
                        .collect();

                    if ips.is_empty() {
                        continue;
                    }

                    let ip = ips[0].clone();

                    // Skip self-discovery
                    if ip == self_ip && port == self_port {
                        continue;
                    }

                    let instance_name = info
                        .get_fullname()
                        .split('.')
                        .next()
                        .unwrap_or("Unknown Device")
                        .to_string();

                    let os_type = info
                        .get_property_val_str("platform")
                        .unwrap_or("unknown")
                        .to_string();

                    let peer = PeerNode {
                        id: fullname.clone(),
                        name: instance_name,
                        ip,
                        port,
                        os_type,
                        last_seen: now_timestamp(),
                        is_app_user: true,
                        is_blocked: false,
                        bandwidth_limit_kbps: None,
                    };

                    let store = store_clone.clone();
                    let app = app_clone.clone();
                    tokio::spawn(async move {
                        let mut map = store.write().await;
                        // Preserve existing block/bandwidth settings if peer re-connects
                        if let Some(existing) = map.get(&fullname) {
                            let mut updated_peer = peer.clone();
                            updated_peer.is_blocked = existing.is_blocked;
                            updated_peer.bandwidth_limit_kbps = existing.bandwidth_limit_kbps;
                            map.insert(fullname, updated_peer);
                        } else {
                            map.insert(fullname, peer);
                        }
                        let peer_list: Vec<PeerNode> = map.values().cloned().collect();
                        let _ = app.emit("peers-updated", peer_list);
                    });
                }
                ServiceEvent::ServiceRemoved(_svc, fullname) => {
                    let store = store_clone.clone();
                    let app = app_clone.clone();
                    tokio::spawn(async move {
                        let mut map = store.write().await;
                        map.remove(&fullname);
                        let peer_list: Vec<PeerNode> = map.values().cloned().collect();
                        let _ = app.emit("peers-updated", peer_list);
                    });
                }
                _ => {}
            }
        }
    });

    Ok(mdns)
}

fn sanitize_hostname(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>()
        .trim_matches('-')
        .to_lowercase()
}

fn now_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
