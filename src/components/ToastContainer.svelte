<script lang="ts">
  import type { ToastItem } from '../types';
  import { X, CheckCircle2, AlertCircle, Info, ClipboardCheck, Copy } from 'lucide-svelte';

  interface Props {
    toasts: ToastItem[];
    onDismiss: (id: string) => void;
    onCopyClipboard: (text: string) => void;
  }

  let { toasts, onDismiss, onCopyClipboard }: Props = $props();
</script>

<div class="fixed bottom-5 right-5 z-50 flex flex-col gap-3 max-w-sm w-full pointer-events-none">
  {#each toasts as toast (toast.id)}
    <div
      class="pointer-events-auto p-4 rounded-2xl border bg-slate-900/95 backdrop-blur-md shadow-2xl flex items-start gap-3 transition-all duration-300 animate-slide-in border-slate-800 text-slate-100"
    >
      {#if toast.type === 'clipboard'}
        <div class="w-8 h-8 rounded-xl bg-indigo-500/10 text-indigo-400 flex items-center justify-center shrink-0 mt-0.5">
          <ClipboardCheck class="w-4 h-4" />
        </div>
      {:else if toast.type === 'success'}
        <div class="w-8 h-8 rounded-xl bg-emerald-500/10 text-emerald-400 flex items-center justify-center shrink-0 mt-0.5">
          <CheckCircle2 class="w-4 h-4" />
        </div>
      {:else if toast.type === 'error'}
        <div class="w-8 h-8 rounded-xl bg-rose-500/10 text-rose-400 flex items-center justify-center shrink-0 mt-0.5">
          <AlertCircle class="w-4 h-4" />
        </div>
      {:else}
        <div class="w-8 h-8 rounded-xl bg-blue-500/10 text-blue-400 flex items-center justify-center shrink-0 mt-0.5">
          <Info class="w-4 h-4" />
        </div>
      {/if}

      <div class="flex-1 min-w-0">
        <h4 class="text-xs font-bold text-white">{toast.title}</h4>
        <p class="text-xs text-slate-300 mt-0.5 leading-relaxed break-words">{toast.message}</p>

        {#if toast.clipboardContent}
          <div class="mt-2.5 pt-2 border-t border-slate-800 flex items-center justify-between gap-2">
            <span class="text-[11px] font-mono text-indigo-300 truncate max-w-[200px]" title={toast.clipboardContent}>
              "{toast.clipboardContent}"
            </span>
            <button
              onclick={() => toast.clipboardContent && onCopyClipboard(toast.clipboardContent)}
              class="flex items-center gap-1 text-[11px] px-2.5 py-1 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white font-medium transition-colors shrink-0 shadow-xs"
            >
              <Copy class="w-3 h-3" />
              <span>Copy</span>
            </button>
          </div>
        {/if}
      </div>

      <button
        onclick={() => onDismiss(toast.id)}
        class="text-slate-400 hover:text-white p-1 rounded-lg hover:bg-slate-800 transition-colors shrink-0"
      >
        <X class="w-3.5 h-3.5" />
      </button>
    </div>
  {/each}
</div>
