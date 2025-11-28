<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let currentTab = 'feed';
  const dispatch = createEventDispatcher();

  $: tabs = [
    { id: 'history', label: '📋 History' },
    { id: 'feed', label: '🎬 Feed' },
    { id: 'bookmarks', label: '⭐ Bookmarks' },
  ];

  let bookmarkBadge = 0;

  function selectTab(tabId: string) {
    dispatch('tabChange', tabId);
  }
</script>

<div class="h-[80px] flex items-center justify-center px-6 border-b border-white/5 bg-gradient-to-b from-[#181818] to-transparent">
  <!-- Centered tabs with enhanced styling -->
  <div class="flex gap-1 p-1 bg-white/5 rounded-full backdrop-blur-sm">
    {#each tabs as tab}
      <button
        class="relative px-12 py-3 text-lg font-medium transition-all duration-300 rounded-full {currentTab === tab.id
          ? 'text-white bg-gradient-to-r from-netflix-red to-netflix-red-hover shadow-lg shadow-netflix-red/25 scale-105'
          : 'text-white/70 hover:text-white hover:bg-white/10'}"
        on:click={() => selectTab(tab.id)}
      >
        {tab.label}
        
        <!-- Badge for bookmarks tab -->
        {#if tab.id === 'bookmarks' && bookmarkBadge > 0}
          <div class="absolute -top-1 -right-1 bg-netflix-red text-white text-xs font-bold rounded-full w-5 h-5 flex items-center justify-center shadow-lg animate-pulse">
            {bookmarkBadge}
          </div>
        {/if}
        
        {#if currentTab === tab.id}
          <!-- Active indicator dot -->
          <div class="absolute -bottom-1 left-1/2 transform -translate-x-1/2 w-1 h-1 bg-netflix-red rounded-full"></div>
        {/if}
      </button>
    {/each}
  </div>
</div>

<style>
  :global(.from-netflix-red) {
    --tw-gradient-from: #e50914;
  }
  
  :global(.to-netflix-red-hover) {
    --tw-gradient-to: #f40612;
  }
  
  :global(.shadow-netflix-red\/25) {
    --tw-shadow-color: rgba(229, 9, 20, 0.25);
  }
  
  :global(.bg-netflix-red) {
    background-color: #e50914;
  }
</style>
