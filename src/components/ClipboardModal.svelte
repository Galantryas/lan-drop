<script lang="ts">
  import type { PeerNode } from '../types';
  import { X, Send, Clipboard, Sparkles } from 'lucide-svelte';

  interface Props {
    peer: PeerNode | null;
    onClose: () => void;
    onSend: (peer: PeerNode, text: string) => void;
  }

  let { peer, onClose, onSend }: Props = $props();

  let textContent = $state('');
  let isSending = $state(false);

  function handleSend() {
    if (!peer || !textContent.trim()) return;
    isSending = true;
    onSend(peer, textContent.trim());
    isSending = false;
    textContent = '';
    onClose();
  }

  async function pasteLocalClipboard() {
    try {
      const text = await navigator.clipboard.readText();
      if (text) textContent = text;
    } catch (err) {
      console.error('Failed reading clipboard:', err);
    }
  }
</script>

{#if peer}
  <div class="fixed inset-0 bg-slate-950/80 backdrop-blur-xs flex items-center justify-center p-4 z-50 animate-fade-in">
    <div class="w-full max-w-lg bg-slate-900 border border-slate-800 rounded-2xl shadow-2xl overflow-hidden flex flex-col">
      <!-- Modal Header -->
      <div class="px-5 py-4 border-b border-slate-800 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <div class="w-8 h-8 rounded-lg bg-indigo-500/10 text-indigo-400 flex items-center justify-center">
            <Clipboard class="w-4 h-4" />
          </div>
          <div>
            <h3 class="text-sm font-bold text-white">Send Clipboard Content</h3>
            <p class="text-xs text-slate-400">To: <span class="font-medium text-slate-200">{peer.name}</span> ({peer.ip})</p>
          </div>
        </div>

        <button
          onclick={onClose}
          class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Modal Body -->
      <div class="p-5 space-y-3">
        <div class="flex justify-between items-center text-xs">
          <label for="clip-input" class="text-slate-300 font-medium">Text or Link Payload</label>
          <button
            onclick={pasteLocalClipboard}
            class="text-indigo-400 hover:text-indigo-300 font-medium flex items-center gap-1 transition-colors"
          >
            <Sparkles class="w-3 h-3" /> Paste Current Clipboard
          </button>
        </div>

        <textarea
          id="clip-input"
          bind:value={textContent}
          rows="5"
          placeholder="Paste or type text to sync with peer's clipboard..."
          class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-slate-100 text-xs focus:outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 font-mono resize-none"
        ></textarea>
      </div>

      <!-- Modal Footer -->
      <div class="px-5 py-3.5 bg-slate-950 border-t border-slate-800/80 flex items-center justify-end gap-2">
        <button
          onclick={onClose}
          class="px-4 py-2 rounded-xl text-xs text-slate-400 hover:text-white hover:bg-slate-800 transition-colors font-medium"
        >
          Cancel
        </button>
        <button
          disabled={!textContent.trim() || isSending}
          onclick={handleSend}
          class="flex items-center gap-1.5 px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white text-xs font-semibold transition-all shadow-md active:scale-95"
        >
          <Send class="w-3.5 h-3.5" />
          <span>{isSending ? 'Sending...' : 'Sync Clipboard'}</span>
        </button>
      </div>
    </div>
  </div>
{/if}
