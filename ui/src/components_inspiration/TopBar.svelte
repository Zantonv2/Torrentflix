<script>
  import { createEventDispatcher } from 'svelte';
  import { t } from '../i18n.js';

  const dispatch = createEventDispatcher();
  let searchValue = '';

  function handleSearch() {
    dispatch('search', searchValue);
  }

  function handleKeyPress(event) {
    if (event.key === 'Enter') {
      handleSearch();
    }
  }
</script>

<div class="bg-gradient-to-r from-netflix-black/95 to-netflix-card-bg/95 backdrop-blur-xl border-b border-white/10 h-[70px] flex items-center justify-between px-6 shadow-2xl">
  <!-- Logo - left aligned -->
  <div class="text-netflix-red text-2xl font-bold tracking-wider w-[200px]">
    {t('appName')}
  </div>

  <!-- Search Bar - centered -->
  <div class="flex-1 max-w-2xl mx-auto">
    <input
      type="text"
      placeholder={t('search')}
      bind:value={searchValue}
      on:keypress={handleKeyPress}
      class="w-full bg-white/5 backdrop-blur-md border border-white/10 rounded-lg px-4 py-2.5 text-white placeholder-white/40 focus:outline-none focus:border-netflix-red/50 focus:bg-white/10 transition-all duration-300"
    />
  </div>

  <!-- Right spacer for balance -->
  <div class="w-[200px]"></div>
</div>
