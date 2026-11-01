<script lang="ts">
  import type { PeerNode } from '../types';
  import {
    Send,
    Clipboard,
    HardDrive,
    Monitor,
    Laptop,
    Server,
    Upload,
    Ban,
    Gauge,
    ShieldAlert,
    WifiOff,
    CheckCircle2
  } from 'lucide-svelte';

  interface Props {
    peer: PeerNode;
    onSendFile: (peer: PeerNode) => void;
    onSendClipboard: (peer: PeerNode) => void;
    onDropFiles: (peer: PeerNode, files: FileList) => void;
    onToggleBlock: (peer: PeerNode) => void;
    onSetBandwidthLimit: (peer: PeerNode, limitKbps: number | null) => void;
    onCutWifi: (peer: PeerNode) => void;
    onRestoreWifi: (peer: PeerNode) => void;
    isWifiCut?: boolean;
  }

  let {
    peer,
    onSendFile,
    onSendClipboard,
    onDropFiles,
    onToggleBlock,
    onSetBandwidthLimit,
    onCutWifi,
    onRestoreWifi,
    isWifiCut = false
  }: Props = $props();

  let isDraggingOver = $state(false);

  function getOsIcon(os: string) {
    const lower = os.toLowerCase();
    if (lower.includes('win')) return HardDrive;
    if (lower.includes('darwin') || lower.includes('mac')) return Laptop;
    if (lower.includes('linux')) return Server;
    return Monitor;
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    if (peer.is_app_user && !peer.is_blocked) {
      isDraggingOver = true;
    }
  }

  function handleDragLeave(e: DragEvent) {
    e.preventDefault();
    isDraggingOver = false;
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDraggingOver = false;
    if (peer.is_app_user && !peer.is_blocked && e.dataTransfer && e.dataTransfer.files.length > 0) {
      onDropFiles(peer, e.dataTransfer.files);
    }
  }

  function handleLimitChange(e: Event) {
    const val = (e.target as HTMLSelectElement).value;
    if (val === 'unlimited') {
      onSetBandwidthLimit(peer, null);
    } else {
      onSetBandwidthLimit(peer, parseInt(val, 10));
    }
  }

  let OsComponent = $derived(getOsIcon(peer.os_type));
  let currentLimitVal = $derived(
    peer.bandwidth_limit_kbps ? peer.bandwidth_limit_kbps.toString() : 'unlimited'
  );
</script>

<div
  ondragover={handleDragOver}
  ondragleave={handleDragLeave}
  ondrop={handleDrop}
  class={`relative group rounded-2xl border p-5 transition-all duration-300 flex flex-col justify-between overflow-hidden shadow-lg ${
    isWifiCut
      ? 'border-red-600 bg-red-950/30 ring-2 ring-red-500/30'
      : peer.is_blocked
      ? 'border-rose-900/60 bg-rose-950/20 opacity-80'
      : isDraggingOver
      ? 'border-blue-500 bg-blue-500/10 scale-102 ring-4 ring-blue-500/20'
      : 'border-slate-800/80 bg-slate-900/60 hover:bg-slate-900/90 hover:border-slate-700/80'
  }`}
>
  <!-- Background Glow Effect -->
  <div
    class={`absolute -top-12 -right-12 w-28 h-28 rounded-full blur-2xl transition-all ${
      isWifiCut ? 'bg-red-500/20' : peer.is_blocked ? 'bg-rose-500/10' : 'bg-blue-500/5 group-hover:bg-blue-500/10'
    }`}
  ></div>

  <!-- Drag Overlay Indicator -->
  {#if isDraggingOver}
    <div class="absolute inset-0 bg-blue-600/20 backdrop-blur-xs flex flex-col items-center justify-center gap-2 text-white font-medium z-10 animate-fade-in">
      <Upload class="w-8 h-8 text-blue-400 animate-bounce" />
      <span class="text-sm">Drop files to send to {peer.name}</span>
    </div>
  {/if}

  <!-- Header Portion -->
  <div>
    <div class="flex items-start justify-between gap-3">
      <div class="flex items-center gap-3">
        <div
          class={`w-10 h-10 rounded-xl border flex items-center justify-center transition-colors shadow-inner ${
            isWifiCut
              ? 'bg-red-900/40 border-red-700 text-red-400'
              : peer.is_blocked
              ? 'bg-rose-900/40 border-rose-800 text-rose-400'
              : 'bg-slate-800 border-slate-700 text-blue-400 group-hover:text-blue-300 group-hover:bg-slate-750'
          }`}
        >
          {#if isWifiCut}
            <WifiOff class="w-5 h-5 text-red-400 animate-pulse" />
          {:else if peer.is_blocked}
            <ShieldAlert class="w-5 h-5 text-rose-400" />
          {:else}
            <OsComponent class="w-5 h-5" />
          {/if}
        </div>
        <div>
          <h3
            class="font-semibold text-slate-100 text-sm group-hover:text-white transition-colors truncate max-w-[150px]"
            title={peer.name}
          >
            {peer.name}
          </h3>
          <p class="text-xs text-slate-400 font-mono mt-0.5">
            {peer.ip}<span class="text-slate-600 font-normal">:{peer.port}</span>
          </p>
        </div>
      </div>

      <!-- Status Badge -->
      {#if isWifiCut}
        <span class="text-[10px] uppercase font-bold tracking-wider px-2 py-0.5 rounded-full bg-red-500/20 text-red-400 border border-red-500/40">
          Wi-Fi Cut
        </span>
      {:else if peer.is_blocked}
        <span class="text-[10px] uppercase font-bold tracking-wider px-2 py-0.5 rounded-full bg-rose-500/10 text-rose-400 border border-rose-500/30">
          App Blocked
        </span>
      {:else if peer.is_app_user}
        <span class="text-[10px] uppercase font-bold tracking-wider px-2 py-0.5 rounded-full bg-blue-500/10 text-blue-400 border border-blue-500/20">
          LAN Drop
        </span>
      {:else}
        <span class="text-[10px] font-medium px-2 py-0.5 rounded-full bg-slate-800 text-slate-400 border border-slate-700">
          Wi-Fi Host (No App)
        </span>
      {/if}
    </div>
  </div>

  <!-- Speed Limiter & NetCut Controls -->
  <div class="mt-4 pt-3 border-t border-slate-800/60 flex items-center justify-between gap-2 text-xs">
    <!-- Bandwidth Limiter Dropdown -->
    <div class="flex items-center gap-1.5 bg-slate-950 px-2.5 py-1 rounded-xl border border-slate-800 text-slate-300">
      <Gauge class="w-3.5 h-3.5 text-indigo-400 shrink-0" />
      <select
        value={currentLimitVal}
        onchange={handleLimitChange}
        disabled={peer.is_blocked || isWifiCut}
        class="bg-transparent text-[11px] text-slate-200 focus:outline-none cursor-pointer disabled:opacity-50"
      >
        <option value="unlimited" class="bg-slate-900 text-slate-200">Max Speed</option>
        <option value="1024" class="bg-slate-900 text-slate-200">1 MB/s</option>
        <option value="5120" class="bg-slate-900 text-slate-200">5 MB/s</option>
        <option value="10240" class="bg-slate-900 text-slate-200">10 MB/s</option>
      </select>
    </div>

    <!-- Cut Wi-Fi Access Action Button -->
    {#if isWifiCut}
      <button
        onclick={() => onRestoreWifi(peer)}
        class="flex items-center gap-1 text-[11px] px-2.5 py-1 rounded-xl border font-semibold bg-emerald-500/10 hover:bg-emerald-500/20 border-emerald-500/30 text-emerald-400 transition-all shadow-xs"
        title="Restore target device's Wi-Fi network connection"
      >
        <CheckCircle2 class="w-3 h-3" />
        <span>Restore Wi-Fi</span>
      </button>
    {:else}
      <button
        onclick={() => onCutWifi(peer)}
        class="flex items-center gap-1 text-[11px] px-2.5 py-1 rounded-xl border font-semibold bg-red-600/20 hover:bg-red-600/30 border-red-500/40 text-red-300 transition-all shadow-xs active:scale-95"
        title="Cut target device's Wi-Fi network connection (ARP spoofing)"
      >
        <WifiOff class="w-3 h-3" />
        <span>Cut Wi-Fi</span>
      </button>
    {/if}
  </div>

  <!-- Actions Footer -->
  <div class="mt-3 pt-3 border-t border-slate-800/60 grid grid-cols-2 gap-2">
    <button
      disabled={!peer.is_app_user || peer.is_blocked || isWifiCut}
      onclick={() => onSendFile(peer)}
      class="flex items-center justify-center gap-1.5 px-3 py-2 rounded-xl bg-blue-600 hover:bg-blue-500 disabled:opacity-40 text-white text-xs font-medium transition-all shadow-sm hover:shadow-blue-500/25 active:scale-95"
      title={peer.is_app_user ? 'Send file to peer' : 'This device is connected to Wi-Fi but does not have the LAN Drop app running.'}
    >
      <Send class="w-3.5 h-3.5" />
      <span>Send File</span>
    </button>

    <button
      disabled={!peer.is_app_user || peer.is_blocked || isWifiCut}
      onclick={() => onSendClipboard(peer)}
      class="flex items-center justify-center gap-1.5 px-3 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 disabled:opacity-40 text-slate-200 hover:text-white border border-slate-700 text-xs font-medium transition-all shadow-sm active:scale-95"
      title={peer.is_app_user ? 'Sync clipboard text' : 'This device is connected to Wi-Fi but does not have the LAN Drop app running.'}
    >
      <Clipboard class="w-3.5 h-3.5 text-indigo-400" />
      <span>Clipboard</span>
    </button>
  </div>
</div>
