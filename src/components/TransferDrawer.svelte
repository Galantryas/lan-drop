<script lang="ts">
  import type { TransferProgressPayload, ReceivedFileRecord } from '../types';
  import { ArrowUpRight, ArrowDownLeft, CheckCircle2, AlertCircle, FileText, FolderOpen, Copy } from 'lucide-svelte';

  interface Props {
    transfers: Record<string, TransferProgressPayload>;
    history: ReceivedFileRecord[];
    onOpenDownloads: () => void;
    onCopyPath: (path: string) => void;
  }

  let { transfers, history, onOpenDownloads, onCopyPath }: Props = $props();

  let activeTab = $state<'active' | 'history'>('active');

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  function formatSpeed(bytesPerSec: number): string {
    return `${formatBytes(bytesPerSec)}/s`;
  }

  let activeTransfersList = $derived(Object.values(transfers));
</script>

<aside class="w-full lg:w-96 border-t lg:border-t-0 lg:border-l border-slate-800/80 bg-slate-900/60 backdrop-blur-md flex flex-col h-72 lg:h-auto">
  <!-- Drawer Header Tabs -->
  <div class="px-5 py-3.5 border-b border-slate-800/80 flex items-center justify-between bg-slate-900/80">
    <div class="flex items-center gap-1 bg-slate-800 p-1 rounded-xl border border-slate-700/60 text-xs font-medium">
      <button
        onclick={() => (activeTab = 'active')}
        class={`px-3 py-1.5 rounded-lg transition-all ${
          activeTab === 'active'
            ? 'bg-blue-600 text-white shadow-xs'
            : 'text-slate-400 hover:text-slate-200'
        }`}
      >
        Transfers ({activeTransfersList.length})
      </button>
      <button
        onclick={() => (activeTab = 'history')}
        class={`px-3 py-1.5 rounded-lg transition-all ${
          activeTab === 'history'
            ? 'bg-blue-600 text-white shadow-xs'
            : 'text-slate-400 hover:text-slate-200'
        }`}
      >
        Received History ({history.length})
      </button>
    </div>

    <button
      onclick={onOpenDownloads}
      class="text-xs text-blue-400 hover:text-blue-300 font-medium flex items-center gap-1 transition-colors"
      title="Open Download Folder"
    >
      <FolderOpen class="w-3.5 h-3.5" />
      <span>Folder</span>
    </button>
  </div>

  <!-- Content List Area -->
  <div class="flex-1 p-4 overflow-y-auto space-y-3">
    {#if activeTab === 'active'}
      {#if activeTransfersList.length === 0}
        <div class="h-48 flex flex-col items-center justify-center text-center text-slate-500 text-xs">
          <FileText class="w-8 h-8 text-slate-700 mb-2 stroke-1" />
          <span>No active transfers</span>
        </div>
      {:else}
        {#each activeTransfersList as item (item.transfer_id)}
          <div class="p-3.5 rounded-xl border border-slate-800 bg-slate-850/80 space-y-2.5 shadow-sm">
            <div class="flex items-start justify-between gap-2">
              <div class="flex items-center gap-2 overflow-hidden">
                {#if item.direction === 'outgoing'}
                  <div class="w-6 h-6 rounded-lg bg-blue-500/10 text-blue-400 flex items-center justify-center shrink-0">
                    <ArrowUpRight class="w-3.5 h-3.5" />
                  </div>
                {:else}
                  <div class="w-6 h-6 rounded-lg bg-emerald-500/10 text-emerald-400 flex items-center justify-center shrink-0">
                    <ArrowDownLeft class="w-3.5 h-3.5" />
                  </div>
                {/if}
                <div class="truncate">
                  <p class="text-xs font-semibold text-slate-200 truncate" title={item.file_name}>
                    {item.file_name}
                  </p>
                  <p class="text-[10px] text-slate-400 truncate">
                    {item.direction === 'outgoing' ? 'To:' : 'From:'} {item.peer_name}
                  </p>
                </div>
              </div>

              {#if item.is_complete}
                <span class="text-emerald-400 flex items-center gap-1 text-[10px] font-bold uppercase shrink-0">
                  <CheckCircle2 class="w-3.5 h-3.5" /> Done
                </span>
              {:else if item.is_error}
                <span class="text-rose-400 flex items-center gap-1 text-[10px] font-bold uppercase shrink-0">
                  <AlertCircle class="w-3.5 h-3.5" /> Error
                </span>
              {:else}
                <span class="text-blue-400 font-mono text-[11px] shrink-0 font-medium">
                  {formatSpeed(item.speed_bytes_per_sec)}
                </span>
              {/if}
            </div>

            <!-- Progress Bar -->
            <div class="space-y-1">
              <div class="w-full bg-slate-800 rounded-full h-1.5 overflow-hidden">
                <div
                  class={`h-full rounded-full transition-all duration-200 ${
                    item.is_error
                      ? 'bg-rose-500'
                      : item.is_complete
                      ? 'bg-emerald-500'
                      : 'bg-blue-500'
                  }`}
                  style={`width: ${
                    item.total_bytes > 0
                      ? Math.min(100, Math.round((item.transferred_bytes / item.total_bytes) * 100))
                      : 100
                  }%`}
                ></div>
              </div>
              <div class="flex justify-between text-[10px] text-slate-400 font-mono">
                <span>{formatBytes(item.transferred_bytes)}</span>
                <span>{item.total_bytes > 0 ? formatBytes(item.total_bytes) : 'Streaming...'}</span>
              </div>
            </div>
          </div>
        {/each}
      {/if}
    {:else}
      <!-- History Tab -->
      {#if history.length === 0}
        <div class="h-48 flex flex-col items-center justify-center text-center text-slate-500 text-xs">
          <FolderOpen class="w-8 h-8 text-slate-700 mb-2 stroke-1" />
          <span>No received files yet</span>
        </div>
      {:else}
        {#each history as rec (rec.id)}
          <div class="p-3.5 rounded-xl border border-slate-800 bg-slate-850/80 flex items-center justify-between gap-3 shadow-sm hover:border-slate-700 transition-colors">
            <div class="overflow-hidden">
              <p class="text-xs font-semibold text-slate-200 truncate" title={rec.file_name}>
                {rec.file_name}
              </p>
              <p class="text-[10px] text-slate-400 mt-0.5 flex items-center gap-2">
                <span>From: {rec.sender_name}</span>
                <span>•</span>
                <span>{formatBytes(rec.file_size)}</span>
              </p>
            </div>

            <button
              onclick={() => onCopyPath(rec.file_path)}
              class="p-2 rounded-lg bg-slate-800 hover:bg-slate-750 text-slate-300 hover:text-white border border-slate-700 shrink-0 transition-colors"
              title="Copy file path"
            >
              <Copy class="w-3.5 h-3.5" />
            </button>
          </div>
        {/each}
      {/if}
    {/if}
  </div>
</aside>
