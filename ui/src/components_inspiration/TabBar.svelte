<script>
  import { createEventDispatcher } from 'svelte';
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { t } from '../i18n.js';

  export let currentTab = 'feed';
  const dispatch = createEventDispatcher();

  $: tabs = [
    { id: 'history', label: '📋 История' },
    { id: 'feed', label: t('feed') },
    { id: 'bookmarks', label: t('bookmarks') },
  ];

  let bookmarkBadge = 0;
  let pollingInterval;

  async function checkBookmarkSources() {
    try {
      const result = await invoke('check_bookmark_sources');
      // Tauri already converts JSON to object, no need to parse
      const data = typeof result === 'string' ? JSON.parse(result) : result;
      bookmarkBadge = data.ready || 0;
      console.log('🔔 DEBUG: Bookmark sources checked:', data);
    } catch (error) {
      console.error('❌ DEBUG: Failed to check bookmark sources:', error);
      bookmarkBadge = 0;
    }
  }

  function selectTab(tabId) {
    dispatch('tabChange', tabId);
  }

  onMount(() => {
    // Check immediately
    checkBookmarkSources();
    
    // Poll every 30 seconds
    pollingInterval = setInterval(checkBookmarkSources, 30000);
  });

  onDestroy(() => {
    if (pollingInterval) {
      clearInterval(pollingInterval);
    }
  });
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
