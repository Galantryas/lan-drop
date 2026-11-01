use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::types::{NetworkDevice, PeerStore};

pub async fn scan_local_subnet(
    local_ip_str: &str,
    peer_store: PeerStore,
    blocked_peers: HashSet<String>,
    bandwidth_limits: std::collections::HashMap<String, u32>,
) -> Vec<NetworkDevice> {
    let local_ip: Ipv4Addr = match local_ip_str.parse() {
        Ok(ip) => ip,
        Err(_) => return vec![],
    };

    let octets = local_ip.octets();
    let subnet_prefix = format!("{}.{}.{}", octets[0], octets[1], octets[2]);

    let known_peers = peer_store.read().await;
    let app_peers: std::collections::HashMap<String, (String, u16, String)> = known_peers
        .values()
        .map(|p| (p.ip.clone(), (p.name.clone(), p.port, p.os_type.clone())))
        .collect();
    drop(known_peers);

    let discovered_devices = Arc::new(Mutex::new(Vec::<NetworkDevice>::new()));
    let mut tasks = Vec::new();

    let common_ports = [49872, 80, 443, 8080, 22, 139, 445, 5353];

    for i in 1..=254 {
        let target_ip_str = format!("{}.{}", subnet_prefix, i);
        let peer_info = app_peers.get(&target_ip_str).cloned();
        let is_blocked = blocked_peers.contains(&target_ip_str);
        let bw_limit = bandwidth_limits.get(&target_ip_str).copied();
        let discovered_ref = discovered_devices.clone();

        tasks.push(tokio::spawn(async move {
            if let Some((name, port, os_type)) = peer_info {
                let dev = NetworkDevice {
                    ip: target_ip_str.clone(),
                    name,
                    port,
                    os_type,
                    is_app_user: true,
                    is_online: true,
                    is_blocked,
                    bandwidth_limit_kbps: bw_limit,
                };
                discovered_ref.lock().await.push(dev);
                return;
            }

            let target_ip: Ipv4Addr = match target_ip_str.parse() {
                Ok(ip) => ip,
                Err(_) => return,
            };

            let mut is_online = false;

            for &port in &common_ports {
                let sockaddr = SocketAddr::new(IpAddr::V4(target_ip), port);
                let connect_timeout = tokio::time::timeout(
                    Duration::from_millis(150),
                    TcpStream::connect(sockaddr),
                )
                .await;

                if let Ok(Ok(_)) = connect_timeout {
                    is_online = true;
                    break;
                }
            }

            if is_online || is_blocked {
                let name = if i == octets[3] {
                    format!("This Device ({})", target_ip_str)
                } else if i == 1 {
                    format!("Gateway / Router ({})", target_ip_str)
                } else {
                    format!("Wi-Fi Device ({})", target_ip_str)
                };

                let dev = NetworkDevice {
                    ip: target_ip_str,
                    name,
                    port: 49872,
                    os_type: "wifi".to_string(),
                    is_app_user: false,
                    is_online: true,
                    is_blocked,
                    bandwidth_limit_kbps: bw_limit,
                };
                discovered_ref.lock().await.push(dev);
            }
        }));
    }

    for task in tasks {
        let _ = task.await;
    }

    let mut result = discovered_devices.lock().await.clone();
    result.sort_by(|a, b| a.ip.cmp(&b.ip));
    result
}
