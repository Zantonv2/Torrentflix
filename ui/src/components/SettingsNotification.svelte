<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let message: string;
  export let type: 'success' | 'error' = 'success';

  const dispatch = createEventDispatcher();

  function close() {
    dispatch('close');
  }
</script>

<div class="mb-6 p-4 rounded-lg flex items-center justify-between {type === 'success' ? 'bg-green-900/30 border border-green-500/50' : 'bg-red-900/30 border border-red-500/50'}">
  <div class="flex items-center gap-3">
    <span class="text-xl">{type === 'success' ? '✅' : '❌'}</span>
    <p class={type === 'success' ? 'text-green-400' : 'text-red-400'}>{message}</p>
  </div>
  <button
    on:click={close}
    class="text-gray-400 hover:text-white transition-colors"
    title="Close"
  >
    ✕
  </button>
</div>

<style>
  :global(.bg-green-900\/30) {
    background-color: rgba(20, 83, 45, 0.3);
  }

  :global(.border-green-500\/50) {
    border-color: rgba(34, 197, 94, 0.5);
  }

  :global(.text-green-400) {
    color: #4ade80;
  }

  :global(.bg-red-900\/30) {
    background-color: rgba(127, 29, 29, 0.3);
  }

  :global(.border-red-500\/50) {
    border-color: rgba(239, 68, 68, 0.5);
  }

  :global(.text-red-400) {
    color: #f87171;
  }
</style>
