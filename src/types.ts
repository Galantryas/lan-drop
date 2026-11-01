export interface PeerNode {
  id: string;
  name: string;
  ip: string;
  port: number;
  os_type: string;
  last_seen: number;
  is_app_user: boolean;
  is_blocked: boolean;
  bandwidth_limit_kbps?: number | null;
}

export interface NetworkDevice {
  ip: string;
  name: string;
  port: number;
  os_type: string;
  is_app_user: boolean;
  is_online: boolean;
  is_blocked: boolean;
  bandwidth_limit_kbps?: number | null;
}

export interface LocalNodeInfo {
  name: string;
  ip: string;
  port: number;
  os_type: string;
  download_dir: string;
}

export interface ClipboardPayload {
  content: string;
  sender_name: string;
}

export interface TransferProgressPayload {
  transfer_id: string;
  file_name: string;
  total_bytes: number;
  transferred_bytes: number;
  speed_bytes_per_sec: number;
  is_complete: boolean;
  is_error: boolean;
  error_message?: string;
  direction: 'incoming' | 'outgoing';
  peer_name: string;
}

export interface ReceivedFileRecord {
  id: string;
  file_name: string;
  file_path: string;
  file_size: number;
  sender_name: string;
  timestamp: string;
}

export interface ToastItem {
  id: string;
  type: 'info' | 'success' | 'warning' | 'error' | 'clipboard';
  title: string;
  message: string;
  clipboardContent?: string;
  duration?: number;
}
