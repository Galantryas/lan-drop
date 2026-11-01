<script lang="ts">
  import type { PeerNode, NetworkDevice } from '../types';
  import PeerCard from './PeerCard.svelte';
  import { Radio, Wifi, ShieldAlert, WifiOff } from 'lucide-svelte';

  interface Props {
    peers: PeerNode[];
    networkDevices: NetworkDevice[];
    wifiCutIPs: string[];
    onSendFile: (peer: PeerNode) => void;
    onSendClipboard: (peer: PeerNode) => void;
    onDropFiles: (peer: PeerNode, files: FileList) => void;
    onToggleBlock: (peer: PeerNode) => void;
    onSetBandwidthLimit: (peer: PeerNode, limitKbps: number | null) => void;
    onCutWifi: (peer: PeerNode) => void;
    onRestoreWifi: (peer: PeerNode) => void;
  }

  let {
    peers,
    networkDevices,
    wifiCutIPs,
    onSendFile,
    onSendClipboard,
    onDropFiles,
    onToggleBlock,
    onSetBandwidthLimit,
    onCutWifi,
    onRestoreWifi
  }: Props = $props();

  let activeTab = $state<'peers' | 'devices' | 'cut'>('peers');

  let activePeers = $derived(peers.filter((p) => !p.is_blocked));
  let blockedPeers = $derived(peers.filter((p) => p.is_blocked));

  function deviceToPeer(dev: NetworkDevice): PeerNode {
    return {
      id: dev.ip,
      name: dev.name,
      ip: dev.ip,
      port: dev.port || 49872,
      os_type: dev.os_type || 'wifi',
      last_seen: Date.now(),
      is_app_user: dev.is_app_user,
      is_blocked: dev.is_blocked,
      bandwidth_limit_kbps: dev.bandwidth_limit_kbps,
    };
  }

  let cutDevicesList = $derived(
    networkDevices.filter((d) => wifiCutIPs.includes(d.ip))
  );
</script>

<div class="flex-1 p-6 overflow-y-auto flex flex-col">
  <!-- Header Tabs & Filters -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-6">
    <div>
      <h2 class="text-base font-bold text-white tracking-tight flex items-center gap-2">
        <span>Network Discovery</span>
      </h2>
      <p class="text-xs text-slate-400">Discover LAN Drop peers and devices connected to your Wi-Fi</p>
    </div>

    <!-- Filter Tabs -->
    <div class="flex items-center gap-1 bg-slate-900 p-1 rounded-xl border border-slate-800 text-xs font-medium self-start sm:self-auto">
      <button
        onclick={() => (activeTab = 'peers')}
        class={`px-3 py-1.5 rounded-lg transition-all flex items-center gap-1.5 ${
          activeTab === 'peers'
            ? 'bg-blue-600 text-white shadow-xs'
            : 'text-slate-400 hover:text-slate-200'
        }`}
      >
        <Radio class="w-3.5 h-3.5" />
        <span>LAN Drop Peers ({activePeers.length})</span>
      </button>

      <button
        onclick={() => (activeTab = 'devices')}
        class={`px-3 py-1.5 rounded-lg transition-all flex items-center gap-1.5 ${
          activeTab === 'devices'
            ? 'bg-blue-600 text-white shadow-xs'
            : 'text-slate-400 hover:text-slate-200'
        }`}
      >
        <Wifi class="w-3.5 h-3.5" />
        <span>Wi-Fi Subnet ({networkDevices.length})</span>
      </button>

      <button
        onclick={() => (activeTab = 'cut')}
        class={`px-3 py-1.5 rounded-lg transition-all flex items-center gap-1.5 ${
          activeTab === 'cut'
            ? 'bg-red-600 text-white shadow-xs'
            : 'text-slate-400 hover:text-slate-200'
        }`}
      >
        <WifiOff class="w-3.5 h-3.5" />
        <span>Wi-Fi Cut ({wifiCutIPs.length})</span>
      </button>
    </div>
  </div>

  <!-- Content Section -->
  <div class="flex-1">
    {#if activeTab === 'peers'}
      {#if activePeers.length === 0}
        <div class="h-80 rounded-3xl border border-slate-800/80 bg-slate-900/40 flex flex-col items-center justify-center p-8 text-center relative overflow-hidden shadow-inner">
          <div class="absolute w-72 h-72 rounded-full border border-blue-500/10 radar-pulse pointer-events-none"></div>
          <div class="absolute w-48 h-48 rounded-full border border-blue-500/20 radar-pulse pointer-events-none" style="animation-delay: 1s;"></div>
          <div class="absolute w-24 h-24 rounded-full border border-blue-500/30 radar-pulse pointer-events-none" style="animation-delay: 2s;"></div>

          <div class="relative w-16 h-16 rounded-2xl bg-slate-800 border border-slate-700 flex items-center justify-center text-blue-400 mb-4 shadow-lg">
            <Radio class="w-8 h-8 animate-pulse text-blue-400" />
          </div>

          <h3 class="text-sm font-semibold text-slate-200">Scanning for LAN Peers...</h3>
          <p class="text-xs text-slate-400 max-w-sm mt-1.5 leading-relaxed">
            Open LAN Drop on another device connected to the same Wi-Fi. Or click "Scan Network" above to list all Wi-Fi devices.
          </p>
        </div>
      {:else}
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-5">
          {#each activePeers as peer (peer.id)}
            <PeerCard
              {peer}
              isWifiCut={wifiCutIPs.includes(peer.ip)}
              {onSendFile}
              {onSendClipboard}
              {onDropFiles}
              {onToggleBlock}
              {onSetBandwidthLimit}
              {onCutWifi}
              {onRestoreWifi}
            />
          {/each}
        </div>
      {/if}

    {:else if activeTab === 'devices'}
      {#if networkDevices.length === 0}
        <div class="h-64 rounded-2xl border border-slate-800 bg-slate-900/40 flex flex-col items-center justify-center p-6 text-center text-slate-400 text-xs">
          <Wifi class="w-8 h-8 text-slate-600 mb-2" />
          <p>No Wi-Fi subnet scan results yet.</p>
          <p class="text-slate-500 mt-1">Click <strong>"Scan Network"</strong> in the top header to discover devices on your subnet.</p>
        </div>
      {:else}
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-5">
          {#each networkDevices as dev (dev.ip)}
            <PeerCard
              peer={deviceToPeer(dev)}
              isWifiCut={wifiCutIPs.includes(dev.ip)}
              {onSendFile}
              {onSendClipboard}
              {onDropFiles}
              {onToggleBlock}
              {onSetBandwidthLimit}
              {onCutWifi}
              {onRestoreWifi}
            />
          {/each}
        </div>
      {/if}

    {:else if activeTab === 'cut'}
      {#if wifiCutIPs.length === 0}
        <div class="h-64 rounded-2xl border border-slate-800 bg-slate-900/40 flex flex-col items-center justify-center p-6 text-center text-slate-400 text-xs">
          <WifiOff class="w-8 h-8 text-slate-600 mb-2" />
          <p>No devices currently cut from Wi-Fi.</p>
        </div>
      {:else}
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-5">
          {#each cutDevicesList as dev (dev.ip)}
            <PeerCard
              peer={deviceToPeer(dev)}
              isWifiCut={true}
              {onSendFile}
              {onSendClipboard}
              {onDropFiles}
              {onToggleBlock}
              {onSetBandwidthLimit}
              {onCutWifi}
              {onRestoreWifi}
            />
          {/each}
        </div>
      {/if}
    {/if}
  </div>
</div>
