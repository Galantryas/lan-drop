<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke, isTauri } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import type {
    PeerNode,
    NetworkDevice,
    LocalNodeInfo,
    TransferProgressPayload,
    ReceivedFileRecord,
    ToastItem,
    ClipboardPayload
  } from './types';

  import Header from './components/Header.svelte';
  import PeerGrid from './components/PeerGrid.svelte';
  import TransferDrawer from './components/TransferDrawer.svelte';
  import ClipboardModal from './components/ClipboardModal.svelte';
  import ToastContainer from './components/ToastContainer.svelte';
  import { Info } from 'lucide-svelte';

  // Svelte 5 Runes State
  let localInfo = $state<LocalNodeInfo | null>(null);
  let peers = $state<PeerNode[]>([]);
  let networkDevices = $state<NetworkDevice[]>([]);
  let wifiCutIPs = $state<string[]>([]);
  let isScanningSubnet = $state<boolean>(false);
  let transfers = $state<Record<string, TransferProgressPayload>>({});
  let history = $state<ReceivedFileRecord[]>([]);
  let toasts = $state<ToastItem[]>([]);
  let inTauriEnv = $state<boolean>(true);

  let selectedPeerForClipboard = $state<PeerNode | null>(null);

  function addToast(item: Omit<ToastItem, 'id'>) {
    const id = Math.random().toString(36).substring(2, 9);
    const newToast: ToastItem = { ...item, id };
    toasts = [newToast, ...toasts];

    setTimeout(() => {
      dismissToast(id);
    }, item.duration || 6000);
  }

  function dismissToast(id: string) {
    toasts = toasts.filter((t) => t.id !== id);
  }

  async function loadInitialData() {
    inTauriEnv = isTauri();
    if (!inTauriEnv) {
      localInfo = {
        name: 'Browser Preview Node',
        ip: '127.0.0.1',
        port: 49872,
        os_type: 'web',
        download_dir: 'Downloads/LANDrop',
      };
      return;
    }

    try {
      localInfo = await invoke<LocalNodeInfo>('get_local_info');
      peers = await invoke<PeerNode[]>('get_active_peers');
    } catch (err) {
      console.error('Failed initializing state:', err);
    }
  }

  onMount(() => {
    loadInitialData();

    if (!isTauri()) return;

    const unlistenPeers = listen<PeerNode[]>('peers-updated', (event) => {
      peers = event.payload;
    });

    const unlistenProgress = listen<TransferProgressPayload>('transfer-progress', (event) => {
      const payload = event.payload;
      transfers = {
        ...transfers,
        [payload.transfer_id]: payload,
      };

      if (payload.is_error) {
        addToast({
          type: 'error',
          title: 'Transfer Failed',
          message: payload.error_message || `Failed sending ${payload.file_name}`,
        });
      }
    });

    const unlistenClipboard = listen<ClipboardPayload>('clipboard-received', (event) => {
      const payload = event.payload;
      addToast({
        type: 'clipboard',
        title: `Clipboard from ${payload.sender_name}`,
        message: 'Content updated in clipboard',
        clipboardContent: payload.content,
      });
    });

    const unlistenFileReceived = listen<ReceivedFileRecord>('file-received', (event) => {
      const record = event.payload;
      history = [record, ...history];

      addToast({
        type: 'success',
        title: 'File Received!',
        message: `${record.file_name} from ${record.sender_name} saved to Downloads/LANDrop`,
      });
    });

    return () => {
      unlistenPeers.then((fn) => fn());
      unlistenProgress.then((fn) => fn());
      unlistenClipboard.then((fn) => fn());
      unlistenFileReceived.then((fn) => fn());
    };
  });

  async function handleScanNetwork() {
    if (!inTauriEnv) {
      addToast({
        type: 'info',
        title: 'Browser Preview',
        message: 'Network scanner requires running via "npm run tauri dev"',
      });
      return;
    }

    isScanningSubnet = true;
    addToast({
      type: 'info',
      title: 'Scanning Wi-Fi Subnet...',
      message: 'Probing IP addresses on local Wi-Fi network.',
      duration: 3000,
    });

    try {
      const devices = await invoke<NetworkDevice[]>('scan_subnet');
      networkDevices = devices;
      addToast({
        type: 'success',
        title: 'Subnet Scan Complete',
        message: `Discovered ${devices.length} active device(s) on Wi-Fi subnet.`,
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: 'Scan Error',
        message: String(err),
      });
    } finally {
      isScanningSubnet = false;
    }
  }

  async function handleCutWifi(peer: PeerNode) {
    if (!inTauriEnv) return;

    try {
      await invoke('cut_wifi_device', { targetIp: peer.ip });
      if (!wifiCutIPs.includes(peer.ip)) {
        wifiCutIPs = [...wifiCutIPs, peer.ip];
      }
      addToast({
        type: 'warning',
        title: 'Wi-Fi Traffic Cut',
        message: `Actively cutting Wi-Fi network connection for target IP ${peer.ip}`,
        duration: 5000,
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: 'NetCut Error',
        message: String(err),
      });
    }
  }

  async function handleRestoreWifi(peer: PeerNode) {
    if (!inTauriEnv) return;

    try {
      await invoke('restore_wifi_device', { targetIp: peer.ip });
      wifiCutIPs = wifiCutIPs.filter((ip) => ip !== peer.ip);
      addToast({
        type: 'success',
        title: 'Wi-Fi Restored',
        message: `Restored Wi-Fi network access for IP ${peer.ip}`,
        duration: 4000,
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: 'Restore Error',
        message: String(err),
      });
    }
  }

  async function handleToggleBlockPeer(peer: PeerNode) {
    if (!inTauriEnv) return;
    try {
      if (peer.is_blocked) {
        await invoke('reconnect_peer', { targetIp: peer.ip });
        addToast({
          type: 'success',
          title: 'Peer Reconnected',
          message: `Restored connection with ${peer.name} (${peer.ip})`,
        });
      } else {
        await invoke('disconnect_peer', { targetIp: peer.ip });
        addToast({
          type: 'warning',
          title: 'Peer Blocked',
          message: `Disconnected and blocked transfers from ${peer.name} (${peer.ip})`,
        });
      }
    } catch (err) {
      addToast({
        type: 'error',
        title: 'Action Error',
        message: String(err),
      });
    }
  }

  async function handleSetBandwidthLimit(peer: PeerNode, limitKbps: number | null) {
    if (!inTauriEnv) return;
    try {
      await invoke('set_peer_bandwidth_limit', {
        targetIp: peer.ip,
        limitKbps,
      });

      const label = limitKbps ? `${limitKbps / 1024} MB/s` : 'Max Speed';
      addToast({
        type: 'info',
        title: 'Speed Limit Updated',
        message: `Set transfer limit for ${peer.name} to ${label}`,
        duration: 3000,
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: 'Limit Error',
        message: String(err),
      });
    }
  }

  async function handleSendFile(peer: PeerNode) {
    if (!inTauriEnv) {
      addToast({
        type: 'info',
        title: 'Browser Preview',
        message: 'File transfers require running via "npm run tauri dev"',
      });
      return;
    }

    if (!peer.is_app_user) {
      addToast({
        type: 'warning',
        title: 'Target Device Missing App',
        message: `Device ${peer.ip} is on Wi-Fi but is not running LAN Drop app.`,
      });
      return;
    }

    if (peer.is_blocked) {
      addToast({
        type: 'warning',
        title: 'Peer Blocked',
        message: 'Cannot send files to a blocked peer. Reconnect peer first.',
      });
      return;
    }

    try {
      const paths = await invoke<string[]>('select_files');
      if (paths && paths.length > 0) {
        await invoke('send_files', {
          targetIp: peer.ip,
          targetPort: peer.port,
          filePaths: paths,
        });

        addToast({
          type: 'info',
          title: 'Transfer Initiated',
          message: `Sending ${paths.length} file(s) to ${peer.name}...`,
        });
      }
    } catch (err) {
      addToast({
        type: 'error',
        title: 'File Transfer Diagnostic',
        message: String(err),
      });
    }
  }

  function handleOpenClipboardModal(peer: PeerNode) {
    if (!peer.is_app_user) {
      addToast({
        type: 'warning',
        title: 'Target Device Missing App',
        message: `Device ${peer.ip} is on Wi-Fi but is not running LAN Drop app.`,
      });
      return;
    }

    if (peer.is_blocked) {
      addToast({
        type: 'warning',
        title: 'Peer Blocked',
        message: 'Cannot sync clipboard with a blocked peer. Reconnect peer first.',
      });
      return;
    }
    selectedPeerForClipboard = peer;
  }

  async function handleSendClipboard(peer: PeerNode, text: string) {
    if (!inTauriEnv) return;
    try {
      await invoke('send_clipboard', {
        targetIp: peer.ip,
        targetPort: peer.port,
        text,
      });

      addToast({
        type: 'success',
        title: 'Clipboard Sent',
        message: `Synced text payload with ${peer.name}`,
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: 'Clipboard Send Error',
        message: String(err),
      });
    }
  }

  function handleDropFilesOnPeer(peer: PeerNode, fileList: FileList) {
    addToast({
      type: 'info',
      title: 'Files Dropped',
      message: `Use native 'Send File' dialog to confirm paths for ${fileList.length} dropped file(s).`,
    });
    handleSendFile(peer);
  }

  async function handleOpenDownloads() {
    if (!inTauriEnv) return;
    try {
      await invoke('open_downloads_folder');
    } catch (err) {
      addToast({
        type: 'error',
        title: 'Failed to open folder',
        message: String(err),
      });
    }
  }

  async function handleCopyText(text: string) {
    if (!inTauriEnv) {
      navigator.clipboard.writeText(text);
      return;
    }
    try {
      await invoke('copy_to_clipboard', { text });
      addToast({
        type: 'success',
        title: 'Copied!',
        message: 'Text copied to OS clipboard',
        duration: 2500,
      });
    } catch (err) {
      console.error('Failed copying text:', err);
    }
  }
</script>

<div class="flex flex-col h-screen bg-slate-950 text-slate-100 select-none overflow-hidden">
  {#if !inTauriEnv}
    <!-- Browser Mode Warning Bar -->
    <div class="bg-indigo-600/90 text-white text-xs px-4 py-1.5 flex items-center justify-between font-medium shadow-sm">
      <div class="flex items-center gap-2">
        <Info class="w-4 h-4 shrink-0" />
        <span>Browser Preview Mode. Launch the desktop app with <code class="bg-indigo-900/60 px-1.5 py-0.5 rounded text-[11px] font-mono">npm run tauri dev</code> to access mDNS discovery & network IPC.</span>
      </div>
    </div>
  {/if}

  <!-- Top Navigation Header -->
  <Header
    {localInfo}
    peerCount={peers.length}
    wifiDeviceCount={networkDevices.length}
    {isScanningSubnet}
    onOpenDownloads={handleOpenDownloads}
    onScanNetwork={handleScanNetwork}
  />

  <!-- Main Split Layout -->
  <main class="flex-1 flex flex-col lg:flex-row overflow-hidden">
    <!-- Left / Center: Radar & Subnet Peer Grid -->
    <PeerGrid
      {peers}
      {networkDevices}
      {wifiCutIPs}
      onSendFile={handleSendFile}
      onSendClipboard={handleOpenClipboardModal}
      onDropFiles={handleDropFilesOnPeer}
      onToggleBlock={handleToggleBlockPeer}
      onSetBandwidthLimit={handleSetBandwidthLimit}
      onCutWifi={handleCutWifi}
      onRestoreWifi={handleRestoreWifi}
    />

    <!-- Right Drawer: Live Transfer Queue & History -->
    <TransferDrawer
      {transfers}
      {history}
      onOpenDownloads={handleOpenDownloads}
      onCopyPath={handleCopyText}
    />
  </main>

  <!-- Clipboard Modal -->
  {#if selectedPeerForClipboard}
    <ClipboardModal
      peer={selectedPeerForClipboard}
      onClose={() => (selectedPeerForClipboard = null)}
      onSend={handleSendClipboard}
    />
  {/if}

  <!-- Toast Notification Stack -->
  <ToastContainer
    {toasts}
    onDismiss={dismissToast}
    onCopyClipboard={handleCopyText}
  />
</div>
