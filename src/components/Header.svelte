<script lang="ts">
  import type { LocalNodeInfo } from '../types';
  import { Folder, Radio, Wifi, Monitor, Cpu, Search, Loader2 } from 'lucide-svelte';

  interface Props {
    localInfo: LocalNodeInfo | null;
    peerCount: number;
    wifiDeviceCount: number;
    isScanning: boolean;
    onOpenDownloads: () => void;
    onScanNetwork: () => void;
  }

  let {
    localInfo,
    peerCount,
    wifiDeviceCount,
    isScanning,
    onOpenDownloads,
    onScanNetwork
  }: Props = $props();
</script>

<header class="bg-slate-900/80 backdrop-blur-md border-b border-slate-800/80 px-6 py-4 flex items-center justify-between shadow-lg">
  <div class="flex items-center gap-4">
    <!-- Logo & Status -->
    <div class="relative flex items-center justify-center w-11 h-11 rounded-xl bg-gradient-to-tr from-blue-600 to-indigo-500 shadow-md shadow-blue-500/20">
      <Wifi class="w-6 h-6 text-white" />
      <span class="absolute -top-1 -right-1 flex h-3.5 w-3.5">
        <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
        <span class="relative inline-flex rounded-full h-3.5 w-3.5 bg-emerald-500"></span>
      </span>
    </div>

    <div>
      <div class="flex items-center gap-2">
        <h1 class="font-bold text-lg text-white tracking-tight">LAN Drop</h1>
        <span class="text-xs font-semibold px-2 py-0.5 rounded-full bg-blue-500/10 text-blue-400 border border-blue-500/20">v1.1</span>
      </div>
      <p class="text-xs text-slate-400 flex items-center gap-2 mt-0.5">
        <span>Subnet & Zero-Config Peer Utilities</span>
        <span class="text-slate-600">•</span>
        <span class="text-emerald-400 font-medium flex items-center gap-1">
          <Radio class="w-3 h-3 animate-pulse" /> Active Discovery
        </span>
      </p>
    </div>
  </div>

  <!-- Center Identity Pill -->
  {#if localInfo}
    <div class="hidden md:flex items-center gap-3 px-4 py-2 rounded-xl bg-slate-850 border border-slate-800 text-xs text-slate-300 shadow-inner">
      <div class="flex items-center gap-1.5 font-medium text-slate-200">
        <Monitor class="w-3.5 h-3.5 text-blue-400" />
        <span>{localInfo.name}</span>
      </div>
      <span class="text-slate-700">|</span>
      <div class="flex items-center gap-1.5 font-mono text-slate-400">
        <Cpu class="w-3.5 h-3.5 text-indigo-400" />
        <span>{localInfo.ip}:{localInfo.port}</span>
      </div>
      <span class="text-slate-700">|</span>
      <div class="flex items-center gap-1.5 text-slate-400">
        <span class="w-2 h-2 rounded-full bg-emerald-400"></span>
        <span>{peerCount} Peers</span>
        <span class="text-slate-600">/</span>
        <span>{wifiDeviceCount} Wi-Fi Hosts</span>
      </div>
    </div>
  {/if}

  <!-- Action Controls -->
  <div class="flex items-center gap-2.5">
    <button
      onclick={onScanNetwork}
      disabled={isScanning}
      class="flex items-center gap-1.5 px-3 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-60 text-white text-xs font-medium transition-all shadow-sm active:scale-95"
      title="Scan Wi-Fi Subnet for active network devices"
    >
      {#if isScanning}
        <Loader2 class="w-3.5 h-3.5 animate-spin text-white" />
        <span>Scanning...</span>
      {:else}
        <Search class="w-3.5 h-3.5" />
        <span>Scan Network</span>
      {/if}
    </button>

    <button
      onclick={onOpenDownloads}
      class="flex items-center gap-1.5 px-3 py-2 rounded-xl bg-slate-800 hover:bg-slate-750 text-slate-200 hover:text-white border border-slate-700/60 transition-all text-xs font-medium shadow-sm hover:shadow-md active:scale-95"
      title="Open LANDrop Downloads Directory"
    >
      <Folder class="w-4 h-4 text-blue-400" />
      <span>Folder</span>
    </button>
  </div>
</header>
